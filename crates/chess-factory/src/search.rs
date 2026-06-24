//! Search wrapper — Stockfish techniques as swappable plugins.
//!
//! Plugin slots (swap the function body to upgrade):
//!   `eval_plugin`   — leaf eval (currently branchless stations; swap for NNUE)
//!   `tt_probe/store`— transposition table
//!   `null_move_prune` — null-move pruning
//!   `lmr_reduction` — late-move reduction table
//!
//! This file MAY branch. Stations remain CC=1.
#![cfg(feature = "std")]

extern crate std;
use std::boxed::Box;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use std::vec::Vec;

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};

use crate::aggregator::aggregate;

const MATE: i32 = 1_000_000;
const INF: i32 = 2 * MATE;
const TT_SIZE: usize = 1 << 17;
const MAX_DEPTH: usize = 64;
const HISTORY_SIZE: usize = 64 * 64;

// ---------------------------------------------------------------------------
// Plugin: Transposition Table
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nt { Exact, Lower, Upper }

#[derive(Clone, Copy)]
struct TtEntry { hash: u64, depth: u8, score: i32, nt: Nt, best: u16 }

impl TtEntry {
    const EMPTY: Self = Self { hash: 0, depth: 0, score: 0, nt: Nt::Exact, best: 0xFFFF };
}

fn tt_table() -> &'static Mutex<Vec<TtEntry>> {
    static TT: OnceLock<Mutex<Vec<TtEntry>>> = OnceLock::new();
    TT.get_or_init(|| { let mut v = Vec::new(); v.resize(TT_SIZE, TtEntry::EMPTY); Mutex::new(v) })
}

fn tt_probe(hash: u64, depth: usize, alpha: i32, beta: i32) -> Option<i32> {
    let t = tt_table().lock().ok()?;
    let e = t[(hash as usize) & (TT_SIZE - 1)];
    if e.hash != hash || (e.depth as usize) < depth { return None; }
    match e.nt {
        Nt::Exact => Some(e.score),
        Nt::Lower => if e.score >= beta  { Some(e.score) } else { None },
        Nt::Upper => if e.score <= alpha { Some(e.score) } else { None },
    }
}

fn tt_store(hash: u64, depth: usize, score: i32, nt: Nt, best: Option<ChessMove>) {
    if let Ok(mut t) = tt_table().lock() {
        let idx = (hash as usize) & (TT_SIZE - 1);
        if t[idx].hash == hash && (t[idx].depth as usize) > depth { return; }
        let packed = best.map(|m| m.get_source().to_index() as u16 * 64
            + m.get_dest().to_index() as u16).unwrap_or(0xFFFF);
        t[idx] = TtEntry { hash, depth: depth as u8, score, nt, best: packed };
    }
}

fn tt_pv(hash: u64, legal: &[ChessMove]) -> Option<ChessMove> {
    let t = tt_table().lock().ok()?;
    let e = t[(hash as usize) & (TT_SIZE - 1)];
    if e.hash != hash || e.best == 0xFFFF { return None; }
    let src = (e.best / 64) as usize;
    let dst = (e.best % 64) as usize;
    legal.iter().copied().find(|m| m.get_source().to_index() == src && m.get_dest().to_index() == dst)
}

// ---------------------------------------------------------------------------
// Plugin: Killer Table
// ---------------------------------------------------------------------------

fn killer_table() -> &'static Mutex<Vec<[Option<u16>; 2]>> {
    static KT: OnceLock<Mutex<Vec<[Option<u16>; 2]>>> = OnceLock::new();
    KT.get_or_init(|| { let mut v = Vec::new(); v.resize(MAX_DEPTH, [None; 2]); Mutex::new(v) })
}

fn killer_store(depth: usize, mv: ChessMove) {
    let packed = mv.get_source().to_index() as u16 * 64 + mv.get_dest().to_index() as u16;
    if let Ok(mut kt) = killer_table().lock() {
        let d = depth.min(MAX_DEPTH - 1);
        if kt[d][0] != Some(packed) {
            kt[d][1] = kt[d][0];
            kt[d][0] = Some(packed);
        }
    }
}

fn killer_score(depth: usize, mv: ChessMove) -> i32 {
    let packed = mv.get_source().to_index() as u16 * 64 + mv.get_dest().to_index() as u16;
    if let Ok(kt) = killer_table().lock() {
        let d = depth.min(MAX_DEPTH - 1);
        if kt[d][0] == Some(packed) || kt[d][1] == Some(packed) {
            return 900_000;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// Plugin: History Table
// ---------------------------------------------------------------------------

fn history_table() -> &'static Mutex<Vec<i32>> {
    static HT: OnceLock<Mutex<Vec<i32>>> = OnceLock::new();
    HT.get_or_init(|| { let mut v = Vec::new(); v.resize(HISTORY_SIZE, 0); Mutex::new(v) })
}

fn history_score(mv: ChessMove) -> i32 {
    let idx = mv.get_source().to_index() * 64 + mv.get_dest().to_index();
    if let Ok(ht) = history_table().lock() {
        return ht[idx];
    }
    0
}

fn history_update(mv: ChessMove, depth: usize, good: bool) {
    let idx = mv.get_source().to_index() * 64 + mv.get_dest().to_index();
    if let Ok(mut ht) = history_table().lock() {
        let delta = (depth * depth) as i32;
        if good {
            ht[idx] += delta;
        } else {
            ht[idx] -= delta / 4;
        }
        ht[idx] = ht[idx].clamp(-10_000_000, 10_000_000);
    }
}

// ---------------------------------------------------------------------------
// Combined reset
// ---------------------------------------------------------------------------

fn search_reset() {
    if let Ok(mut t) = tt_table().lock() {
        for e in t.iter_mut() { *e = TtEntry::EMPTY; }
    }
    if let Ok(mut kt) = killer_table().lock() {
        for slot in kt.iter_mut() { *slot = [None; 2]; }
    }
    if let Ok(mut ht) = history_table().lock() {
        for h in ht.iter_mut() { *h = 0; }
    }
}

// ---------------------------------------------------------------------------
// Plugin: Eval (swap body for NNUE)
// ---------------------------------------------------------------------------

#[inline(always)]
fn eval_plugin(board: &Board) -> i32 {
    // Try NNUE first; fall back to branchless aggregate if lock is poisoned.
    if let Ok(weights) = crate::nnue::nnue_weights().lock() {
        let mut acc = crate::nnue::NnueAccumulator::zeroed();
        crate::nnue::nnue_refresh(board, &weights, &mut acc);
        crate::nnue::nnue_forward(&acc, board.side_to_move(), &weights)
    } else {
        let v = crate::position::PositionView::from_board(board);
        let cp = aggregate(&v);
        if board.side_to_move() == Color::White { cp } else { -cp }
    }
}

// ---------------------------------------------------------------------------
// Plugin: Move ordering (MVV-LVA + PV-first + killers + history)
// ---------------------------------------------------------------------------

fn pv(p: Piece) -> i32 {
    match p {
        Piece::Pawn=>100, Piece::Knight=>337, Piece::Bishop=>365,
        Piece::Rook=>477, Piece::Queen=>1025, Piece::King=>20_000,
    }
}

fn mvv_lva(board: &Board, m: ChessMove) -> i32 {
    let victim    = board.piece_on(m.get_dest()).map_or(0, pv);
    let aggressor = board.piece_on(m.get_source()).map_or(0, pv);
    let promo     = m.get_promotion().map_or(0, pv);
    10 * victim - aggressor + 5 * promo
}

fn order(board: &Board, moves: &mut Vec<ChessMove>, hint: Option<ChessMove>, depth: usize) {
    moves.sort_by_cached_key(|&m| {
        let cap_val = board.piece_on(m.get_dest()).map_or(0, pv);
        let pv_bonus = if Some(m) == hint { 1_000_000 } else { 0 };
        let score = if cap_val > 0 {
            let see_val = see(board, m);
            if see_val >= 0 { 500_000 + see_val } else { -100_000 + see_val }
        } else {
            killer_score(depth, m) + history_score(m)
        };
        -(score + pv_bonus)
    });
}

// ---------------------------------------------------------------------------
// Plugin: Late-Move Reduction (precomputed table)
// ---------------------------------------------------------------------------

fn lmr_table() -> &'static [[u8; 64]; 64] {
    static LMR: OnceLock<Box<[[u8; 64]; 64]>> = OnceLock::new();
    LMR.get_or_init(|| {
        let mut t = Box::new([[0u8; 64]; 64]);
        for d in 2..64usize {
            for i in 2..64usize {
                let r = ((d as f64).ln() * (i as f64).ln() / 2.0) as u8;
                t[d][i] = r.max(1);
            }
        }
        t
    })
}

fn lmr(depth: usize, idx: usize, capture: bool) -> usize {
    if capture || depth < 3 || idx < 3 { return 0; }
    lmr_table()[depth.min(63)][idx.min(63)] as usize
}

// ---------------------------------------------------------------------------
// Plugin: SEE (Static Exchange Evaluation)
// ---------------------------------------------------------------------------

fn see_piece_value(p: Piece) -> i32 {
    match p {
        Piece::Pawn=>100, Piece::Knight=>337, Piece::Bishop=>365,
        Piece::Rook=>477, Piece::Queen=>1025, Piece::King=>20_000,
    }
}

fn see(board: &Board, mv: ChessMove) -> i32 {
    let dest = mv.get_dest();
    let gain = match board.piece_on(dest) {
        None => return 0,
        Some(p) => see_piece_value(p),
    };
    let opp = !board.side_to_move();
    // Find cheapest attacker of dest from opponent side
    // Check in order: Pawn, Knight, Bishop, Rook, Queen, King
    let opp_pawns = board.pieces(Piece::Pawn) & board.color_combined(opp);
    if opp_pawns.0 != 0 {
        let pawn_attacks = chess::get_pawn_attacks(dest, board.side_to_move(), opp_pawns);
        if pawn_attacks.0 != 0 {
            return gain - see_piece_value(Piece::Pawn).max(0);
        }
    }
    let opp_knights = board.pieces(Piece::Knight) & board.color_combined(opp);
    if opp_knights.0 != 0 {
        let knight_attacks = chess::get_knight_moves(dest) & opp_knights;
        if knight_attacks.0 != 0 {
            return gain - see_piece_value(Piece::Knight);
        }
    }
    let opp_bishops = board.pieces(Piece::Bishop) & board.color_combined(opp);
    if opp_bishops.0 != 0 {
        return gain - see_piece_value(Piece::Bishop);
    }
    let opp_rooks = board.pieces(Piece::Rook) & board.color_combined(opp);
    if opp_rooks.0 != 0 {
        return gain - see_piece_value(Piece::Rook);
    }
    let opp_queens = board.pieces(Piece::Queen) & board.color_combined(opp);
    if opp_queens.0 != 0 {
        return gain - see_piece_value(Piece::Queen);
    }
    let opp_kings = board.pieces(Piece::King) & board.color_combined(opp);
    if opp_kings.0 != 0 {
        let king_attacks = chess::get_king_moves(dest) & opp_kings;
        if king_attacks.0 != 0 {
            return gain - see_piece_value(Piece::King);
        }
    }
    gain
}

// ---------------------------------------------------------------------------
// Plugin: Null-Move Pruning
// ---------------------------------------------------------------------------

fn null_move(board: &Board, depth: usize, beta: i32, start: Instant, us: u128) -> Option<i32> {
    if depth < 3 { return None; }
    let null = board.null_move()?;
    let r = 3.min(depth - 1);
    let s = -ab(-beta, -beta + 1, depth - r, &null, start, us, false);
    if s >= beta { Some(beta) } else { None }
}

// ---------------------------------------------------------------------------
// Quiescence
// ---------------------------------------------------------------------------

fn qsearch(mut alpha: i32, beta: i32, board: &Board) -> i32 {
    let stand = eval_plugin(board);
    if stand >= beta { return beta; }
    if stand > alpha { alpha = stand; }
    let mut caps: Vec<ChessMove> = MoveGen::new_legal(board)
        .filter(|m| board.piece_on(m.get_dest()).is_some())
        .collect();
    caps.sort_by_cached_key(|&m| -mvv_lva(board, m));
    for m in caps {
        let s = -qsearch(-beta, -alpha, &board.make_move_new(m));
        if s >= beta { return beta; }
        if s > alpha { alpha = s; }
    }
    alpha
}

// ---------------------------------------------------------------------------
// Alpha-beta
// ---------------------------------------------------------------------------

fn ab(mut alpha: i32, beta: i32, depth: usize, board: &Board,
      start: Instant, us: u128, use_null: bool) -> i32
{
    if start.elapsed().as_micros() >= us { return 0; }
    match board.status() {
        BoardStatus::Checkmate => return -MATE,
        BoardStatus::Stalemate => return 0,
        BoardStatus::Ongoing   => {}
    }
    if depth == 0 { return qsearch(alpha, beta, board); }

    let hash = board.get_hash();
    if let Some(s) = tt_probe(hash, depth, alpha, beta) { return s; }

    if use_null {
        if let Some(s) = null_move(board, depth, beta, start, us) { return s; }
    }

    let in_check = board.checkers().0 != 0;
    if depth <= 3 && !in_check && use_null {
        let static_eval = eval_plugin(board);
        if static_eval + 150 * depth as i32 <= alpha { return static_eval; }
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    let hint = tt_pv(hash, &moves);
    order(board, &mut moves, hint, depth);

    let orig = alpha;
    let mut best_s = -INF;
    let mut best_m = None;

    for (i, &m) in moves.iter().enumerate() {
        if start.elapsed().as_micros() >= us { break; }
        let cap = board.piece_on(m.get_dest()).is_some();
        let child = board.make_move_new(m);
        let r = lmr(depth, i, cap);
        let s = if r > 0 {
            let zw = -ab(-alpha - 1, -alpha, depth - 1 - r, &child, start, us, true);
            if zw > alpha { -ab(-beta, -alpha, depth - 1, &child, start, us, true) } else { zw }
        } else {
            -ab(-beta, -alpha, depth - 1, &child, start, us, true)
        };
        if s > best_s { best_s = s; best_m = Some(m); }
        if s > alpha  { alpha = s; }
        if alpha >= beta {
            if let Some(bm) = best_m {
                let is_quiet = board.piece_on(bm.get_dest()).is_none();
                if is_quiet {
                    killer_store(depth, bm);
                    history_update(bm, depth, true);
                }
            }
            tt_store(hash, depth, best_s, Nt::Lower, best_m);
            return best_s;
        }
    }

    if best_s == -INF { return if board.checkers().0 != 0 { -MATE } else { 0 }; }
    let nt = if best_s <= orig { Nt::Upper } else { Nt::Exact };
    tt_store(hash, depth, best_s, nt, best_m);
    best_s
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Fixed-depth search (deterministic).
#[must_use]
pub fn fixed_depth_best_move(board: &Board, depth: usize) -> Option<ChessMove> {
    search_reset();
    let start = Instant::now();
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if moves.is_empty() { return None; }
    let hint = tt_pv(board.get_hash(), &moves);
    order(board, &mut moves, hint, depth);
    let mut best = moves.first().copied();
    let mut best_v = -INF;
    for &m in &moves {
        let s = -ab(-INF, INF, depth.saturating_sub(1), &board.make_move_new(m), start, u128::MAX, true);
        if s > best_v { best_v = s; best = Some(m); }
    }
    best
}

/// Time-bounded iterative deepening with aspiration windows.
/// Admits O* = (board, budget, hardware, phase) → selects compiled topology → executes.
#[must_use]
pub fn search_best_move_us(board: &Board, budget_us: u128) -> Option<ChessMove> {
    // Admission layer: O* → topology (O(1) table lookup, no search).
    let (_admitted, _topology) = crate::phase::admit(board, budget_us);
    // TODO Phase 2: branch search graph based on topology.
    // Currently all topologies route to the same iterative deepening graph.

    if let Some(bm) = crate::opening_book::book_probe(board.get_hash()) {
        if board.legal(bm) { return Some(bm); }
    }
    search_reset();
    let start = Instant::now();
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if moves.is_empty() { return None; }

    let mut best = moves.first().copied();
    let mut prev = 0i32;
    let asp = 25i32;

    'id: for depth in 1usize..=32 {
        if start.elapsed().as_micros() >= budget_us / 2 { break; }
        // Depth-1 uses full window; aspiration only kicks in once we have a reliable prev score.
        let (mut lo, mut hi) = if depth == 1 { (-INF, INF) } else { (prev - asp, prev + asp) };
        loop {
            let hint = tt_pv(board.get_hash(), &moves);
            order(board, &mut moves, hint, depth);
            let mut db = best;
            let mut dv = -INF;
            let mut alpha = lo;
            for &m in &moves {
                if start.elapsed().as_micros() >= budget_us { break 'id; }
                let s = -ab(-hi, -alpha, depth.saturating_sub(1), &board.make_move_new(m), start, budget_us, true);
                if s > dv { dv = s; db = Some(m); }
                if s > alpha { alpha = s; }
                if alpha >= hi { break; }
            }
            if dv <= lo      { lo = lo.saturating_sub(asp * 4); }
            else if dv >= hi { hi = hi.saturating_add(asp * 4); }
            else             { best = db; prev = dv; break; }
            if lo <= -INF / 2 { lo = -INF; }
            if hi >= INF / 2  { hi = INF; }
        }
    }
    best
}

/// Measure eval latency. For the `latency` UCI command.
pub fn measure_latency_us(board: &Board, iters: u32) -> (u64, u64, u64) {
    let mut t: Vec<u64> = (0..iters).map(|_| {
        let t0 = Instant::now();
        let _ = eval_plugin(board);
        t0.elapsed().as_micros() as u64
    }).collect();
    t.sort_unstable();
    (t[0], t[t.len()/2], t[(t.len()*99/100).min(t.len()-1)])
}
