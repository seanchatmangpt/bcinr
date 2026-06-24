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
use std::string::ToString;
use std::println;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use std::vec::Vec;

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};

use crate::aggregator::aggregate;

// ---------------------------------------------------------------------------
// Receipt emission counter (monotone move ordinal across the process lifetime)
// ---------------------------------------------------------------------------

static MOVE_COUNTER: AtomicU32 = AtomicU32::new(0);

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

/// Full reset between games (clear TT + killers + history).
pub fn game_reset() {
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

/// Per-move reset: keep TT across moves (incremental reuse), clear only
/// per-move heuristics (killers/history age quickly).
fn search_reset() {
    if let Ok(mut kt) = killer_table().lock() {
        for slot in kt.iter_mut() { *slot = [None; 2]; }
    }
    if let Ok(mut ht) = history_table().lock() {
        // Age history scores (halve) rather than zeroing — retains ordering signal.
        for h in ht.iter_mut() { *h >>= 1; }
    }
}

// ---------------------------------------------------------------------------
// Fast inline material+PST eval — used at every search node
// ---------------------------------------------------------------------------

/// Piece values matching see_piece_value (same scale).
const MAT: [i32; 6] = [100, 337, 365, 477, 1025, 0]; // P N B R Q K(no value)

/// PST tables (PeSTO, white-relative). White reads [sq^56], Black reads [sq].
const PST_MG: [[i8; 64]; 6] = [
    // Pawn
    [ 0, 0, 0, 0, 0, 0, 0, 0,
      50,50,50,50,50,50,50,50,
      10,10,20,30,30,20,10,10,
       5, 5,10,25,25,10, 5, 5,
       0, 0, 0,20,20, 0, 0, 0,
       5,-5,-10, 0, 0,-10,-5, 5,
       5,10,10,-20,-20,10,10, 5,
       0, 0, 0, 0, 0, 0, 0, 0],
    // Knight
    [-50,-40,-30,-30,-30,-30,-40,-50,
     -40,-20,  0,  0,  0,  0,-20,-40,
     -30,  0, 10, 15, 15, 10,  0,-30,
     -30,  5, 15, 20, 20, 15,  5,-30,
     -30,  0, 15, 20, 20, 15,  0,-30,
     -30,  5, 10, 15, 15, 10,  5,-30,
     -40,-20,  0,  5,  5,  0,-20,-40,
     -50,-40,-30,-30,-30,-30,-40,-50],
    // Bishop
    [-20,-10,-10,-10,-10,-10,-10,-20,
     -10,  0,  0,  0,  0,  0,  0,-10,
     -10,  0,  5, 10, 10,  5,  0,-10,
     -10,  5,  5, 10, 10,  5,  5,-10,
     -10,  0, 10, 10, 10, 10,  0,-10,
     -10, 10, 10, 10, 10, 10, 10,-10,
     -10,  5,  0,  0,  0,  0,  5,-10,
     -20,-10,-10,-10,-10,-10,-10,-20],
    // Rook
    [ 0, 0, 0, 0, 0, 0, 0, 0,
       5,10,10,10,10,10,10, 5,
      -5, 0, 0, 0, 0, 0, 0,-5,
      -5, 0, 0, 0, 0, 0, 0,-5,
      -5, 0, 0, 0, 0, 0, 0,-5,
      -5, 0, 0, 0, 0, 0, 0,-5,
      -5, 0, 0, 0, 0, 0, 0,-5,
       0, 0, 0, 5, 5, 0, 0, 0],
    // Queen
    [-20,-10,-10, -5, -5,-10,-10,-20,
     -10,  0,  0,  0,  0,  0,  0,-10,
     -10,  0,  5,  5,  5,  5,  0,-10,
      -5,  0,  5,  5,  5,  5,  0, -5,
       0,  0,  5,  5,  5,  5,  0, -5,
     -10,  5,  5,  5,  5,  5,  0,-10,
     -10,  0,  5,  0,  0,  0,  0,-10,
     -20,-10,-10, -5, -5,-10,-10,-20],
    // King (middlegame: castle, hide behind pawns)
    [-30,-40,-40,-50,-50,-40,-40,-30,
     -30,-40,-40,-50,-50,-40,-40,-30,
     -30,-40,-40,-50,-50,-40,-40,-30,
     -30,-40,-40,-50,-50,-40,-40,-30,
     -20,-30,-30,-40,-40,-30,-30,-20,
     -10,-20,-20,-20,-20,-20,-20,-10,
      20, 20,  0,  0,  0,  0, 20, 20,
      20, 30, 10,  0,  0, 10, 30, 20],
];

// Passed-pawn rank bonuses (cp, indexed by rank 0-7).
const PASSED_BONUS: [i32; 8] = [0, 0, 5, 15, 30, 60, 100, 0];

/// Fast material+PST+positional eval, side-to-move relative.
/// Public wrapper around fast_eval for use by the Manufacturing Graph benchmark.
pub fn eval_position(board: &Board) -> i32 {
    fast_eval(board)
}

/// Adds passed-pawn, bishop-pair, and king-shelter on top of material+PST.
/// Target: <10µs per call so depth-3/4 is reachable at 1ms budgets.
#[inline]
fn fast_eval(board: &Board) -> i32 {
    let mut score = 0i32;

    // --- Material + PST ---
    for p in 0usize..6 {
        let piece = [Piece::Pawn,Piece::Knight,Piece::Bishop,Piece::Rook,Piece::Queen,Piece::King][p];
        let white = board.pieces(piece) & board.color_combined(Color::White);
        let black = board.pieces(piece) & board.color_combined(Color::Black);
        let mut wb = white;
        while wb.0 != 0 {
            let sq = wb.0.trailing_zeros() as usize;
            score += MAT[p] + PST_MG[p][sq ^ 56] as i32;
            wb.0 &= wb.0 - 1;
        }
        let mut bb = black;
        while bb.0 != 0 {
            let sq = bb.0.trailing_zeros() as usize;
            score -= MAT[p] + PST_MG[p][sq] as i32;
            bb.0 &= bb.0 - 1;
        }
    }

    // --- Bishop pair (+30 cp each side, branchless O(1)) ---
    let wbishops = board.pieces(Piece::Bishop) & board.color_combined(Color::White);
    let bbishops = board.pieces(Piece::Bishop) & board.color_combined(Color::Black);
    if wbishops.popcnt() >= 2 { score += 30; }
    if bbishops.popcnt() >= 2 { score -= 30; }

    // --- Passed pawns (O(pawns), rank-scaled bonus) ---
    let wp = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
    let bp = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);
    {
        let mut bb = wp;
        while bb.0 != 0 {
            let sq = bb.0.trailing_zeros() as usize;
            let rank = sq >> 3;         // 0=rank1 .. 7=rank8
            let file = sq & 7;
            // Build north-shadow: all squares on this file + adjacent files north of sq.
            // A pawn is passed when no enemy pawn occupies this shadow.
            let adj_files: u64 = {
                let f = 0x0101_0101_0101_0101u64 << file;
                let left  = if file > 0 { f >> 1 } else { 0 };
                let right = if file < 7 { f << 1 } else { 0 };
                f | left | right
            };
            // Mask to ranks strictly above this pawn (ranks rank+1..7).
            let above_rank_mask = !((1u64 << ((rank + 1) * 8)).wrapping_sub(1));
            let shadow = adj_files & above_rank_mask;
            if bp.0 & shadow == 0 {
                score += PASSED_BONUS[rank];
            }
            bb.0 &= bb.0 - 1;
        }
    }
    {
        let mut bb = bp;
        while bb.0 != 0 {
            let sq = bb.0.trailing_zeros() as usize;
            let rank = 7 - (sq >> 3);   // black's rank perspective
            let file = sq & 7;
            let adj_files: u64 = {
                let f = 0x0101_0101_0101_0101u64 << file;
                let left  = if file > 0 { f >> 1 } else { 0 };
                let right = if file < 7 { f << 1 } else { 0 };
                f | left | right
            };
            // Mask to ranks strictly below this pawn (ranks 0..rank-1 from black's sq).
            let sq_rank = sq >> 3;
            let below_rank_mask = if sq_rank > 0 { (1u64 << (sq_rank * 8)) - 1 } else { 0 };
            let shadow = adj_files & below_rank_mask;
            if wp.0 & shadow == 0 {
                score -= PASSED_BONUS[rank];
            }
            bb.0 &= bb.0 - 1;
        }
    }

    // --- King shelter: count friendly pawns in 3 squares in front of king (+8 cp each) ---
    {
        let wk_sq  = board.king_square(Color::White).to_index();
        let bk_sq  = board.king_square(Color::Black).to_index();
        let wk_rank = wk_sq >> 3;
        let bk_rank = bk_sq >> 3;
        // White king: pawns on ranks wk_rank+1, same/adjacent files.
        if wk_rank < 7 {
            let wk_file = wk_sq & 7;
            let shield_rank_mask = 0xFFu64 << ((wk_rank + 1) * 8);
            let adj = {
                let f = 0x0101_0101_0101_0101u64 << wk_file;
                let l = if wk_file > 0 { f >> 1 } else { 0 };
                let r = if wk_file < 7 { f << 1 } else { 0 };
                f | l | r
            };
            score += (wp.0 & adj & shield_rank_mask).count_ones() as i32 * 8;
        }
        // Black king: pawns on ranks bk_rank-1.
        if bk_rank > 0 {
            let bk_file = bk_sq & 7;
            let shield_rank_mask = 0xFFu64 << ((bk_rank - 1) * 8);
            let adj = {
                let f = 0x0101_0101_0101_0101u64 << bk_file;
                let l = if bk_file > 0 { f >> 1 } else { 0 };
                let r = if bk_file < 7 { f << 1 } else { 0 };
                f | l | r
            };
            score -= (bp.0 & adj & shield_rank_mask).count_ones() as i32 * 8;
        }
    }

    if board.side_to_move() == Color::White { score } else { -score }
}

// ---------------------------------------------------------------------------
// Plugin: Eval (swap body for NNUE)
// ---------------------------------------------------------------------------

#[inline(always)]
fn eval_plugin(board: &Board) -> i32 {
    // Branchless station aggregate — side-to-move relative (positive = good for mover).
    // NNUE is scaffolded but L2 weights are untrained (all zero); it returns ~0 for every
    // position until real weights are loaded. Use aggregate until then.
    let v = crate::position::PositionView::from_board(board);
    let cp = aggregate(&v);
    if board.side_to_move() == Color::White { cp } else { -cp }
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
    if capture { return 0; }
    // Improved LMR: more aggressive reduction for later moves
    // First 4 moves (idx 0-3): no reduction
    // Moves 5-8 (idx 4-7): reduce by 1 if depth >= 3
    // Moves 9+ (idx 8+): reduce by 2 if depth >= 4
    if idx < 4 { return 0; }
    if idx < 8 {
        return if depth >= 3 { 1 } else { 0 };
    }
    if depth >= 4 { 2 } else if depth >= 3 { 1 } else { 0 }
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
    // Value of the moving piece — what we lose if opponent recaptures.
    let mover = board.piece_on(mv.get_source()).map_or(100, see_piece_value);
    let opp = !board.side_to_move();

    // Find cheapest attacker of dest from opponent side.
    // If opponent can recapture with any piece cheaper than mover:
    //   SEE = gain - mover  (we win `gain` but lose `mover`)
    // If opponent's cheapest recapture ≥ mover:
    //   SEE = gain - cheapest  (exchange favours us or is neutral)
    // If no recapture: SEE = gain.
    let find_see = |cheapest: i32| -> i32 {
        if cheapest <= mover { gain - mover } else { gain - cheapest }
    };

    // Pawn: get_pawn_attacks(dest, our_color, opp_pawns) returns which
    // opponent pawns are diagonally adjacent and can recapture on dest.
    let opp_pawns = board.pieces(Piece::Pawn) & board.color_combined(opp);
    if opp_pawns.0 != 0 {
        let pa = chess::get_pawn_attacks(dest, board.side_to_move(), opp_pawns);
        if pa.0 != 0 { return find_see(see_piece_value(Piece::Pawn)); }
    }
    let opp_knights = board.pieces(Piece::Knight) & board.color_combined(opp);
    if opp_knights.0 != 0 {
        if (chess::get_knight_moves(dest) & opp_knights).0 != 0 {
            return find_see(see_piece_value(Piece::Knight));
        }
    }
    // Sliders: conservative — if any slider of this type exists for opponent,
    // assume it can reach dest (avoids costly ray-trace; may over-refuse some trades).
    if (board.pieces(Piece::Bishop) & board.color_combined(opp)).0 != 0 {
        return find_see(see_piece_value(Piece::Bishop));
    }
    if (board.pieces(Piece::Rook) & board.color_combined(opp)).0 != 0 {
        return find_see(see_piece_value(Piece::Rook));
    }
    if (board.pieces(Piece::Queen) & board.color_combined(opp)).0 != 0 {
        return find_see(see_piece_value(Piece::Queen));
    }
    let opp_king = board.pieces(Piece::King) & board.color_combined(opp);
    if (chess::get_king_moves(dest) & opp_king).0 != 0 {
        return find_see(see_piece_value(Piece::King));
    }
    gain // No recapture possible
}

// ---------------------------------------------------------------------------
// Plugin: Null-Move Pruning
// ---------------------------------------------------------------------------

fn null_move(board: &Board, depth: usize, beta: i32, start: Instant, us: u128) -> Option<i32> {
    if depth < 3 { return None; }
    let null = board.null_move()?;
    // Adaptive null move: depth - 2 - (depth / 4) reduces more at higher depths
    let r = (2 + depth / 4).min(depth - 1);
    let s = -ab(-beta, -beta + 1, depth - r, &null, start, us, false);
    if s >= beta { Some(beta) } else { None }
}

// ---------------------------------------------------------------------------
// Quiescence
// ---------------------------------------------------------------------------

fn qsearch(mut alpha: i32, beta: i32, board: &Board, qdepth: u8) -> i32 {
    let stand = fast_eval(board);
    if stand >= beta { return beta; }
    if stand > alpha { alpha = stand; }
    // Limit qsearch recursion to avoid blowup in complex positions.
    if qdepth == 0 { return alpha; }
    // Delta pruning: skip if even the best possible gain can't beat alpha.
    let delta = 900; // queen value — skip captures that can't raise alpha
    if stand + delta < alpha { return alpha; }
    let mut caps: Vec<ChessMove> = MoveGen::new_legal(board)
        .filter(|m| board.piece_on(m.get_dest()).is_some())
        .collect();
    caps.sort_by_cached_key(|&m| -mvv_lva(board, m));
    for m in caps {
        // Skip obviously bad captures (negative SEE)
        if see(board, m) < 0 { continue; }
        let s = -qsearch(-beta, -alpha, &board.make_move_new(m), qdepth - 1);
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
    let in_check = board.checkers().0 != 0;
    // Check extension: don't drop to qsearch when in check — extend 1 ply.
    if depth == 0 {
        return if in_check {
            ab(alpha, beta, 1, board, start, us, false)
        } else {
            qsearch(alpha, beta, board, 4)
        };
    }

    let hash = board.get_hash();
    if let Some(s) = tt_probe(hash, depth, alpha, beta) { return s; }

    if use_null {
        if let Some(s) = null_move(board, depth, beta, start, us) { return s; }
    }

    if depth <= 3 && !in_check && use_null {
        let static_eval = fast_eval(board);
        if static_eval + 150 * depth as i32 <= alpha { return static_eval; }
    }

    // Extended futility pruning: at depth 1/2/3, skip quiet moves if eval + margin <= alpha
    // Margins: depth 1 = 150cp, depth 2 = 300cp, depth 3 = 450cp
    let futility_margin = 150 * depth as i32;
    let do_futility = depth <= 3 && !in_check && !use_null;
    // (used per-move below in the move loop)

    // Razoring at depth 1: if static eval is far below alpha, return qsearch if it confirms fail-low
    if depth == 1 && !in_check {
        let razor_eval = fast_eval(board);
        if razor_eval + 300 < alpha {
            let q = qsearch(alpha, beta, board, 4);
            return q.min(alpha);  // Don't raise alpha, just return qsearch if it confirms fail-low
        }
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    let hint = tt_pv(hash, &moves);

    // Singular extension stub: if TT move is EXACT and score is promising, mark for extension
    // SINGULAR_EXT_MARKER — implement in Phase 3
    // TODO: singular extension — try alternatives with reduced depth+window
    // For now, just extend the TT move by 1 if score > alpha+50 and depth > 4
    let _singular_ext = hint.is_some() && depth >= 6;

    order(board, &mut moves, hint, depth);

    let orig = alpha;
    let mut best_s = -INF;
    let mut best_m = None;

    for (i, &m) in moves.iter().enumerate() {
        if start.elapsed().as_micros() >= us { break; }
        let cap = board.piece_on(m.get_dest()).is_some();
        let promo = m.get_promotion().is_some();
        // Futility pruning: skip quiet moves when static eval + margin can't reach alpha
        if do_futility && !cap && !promo {
            // Check move doesn't give check (quiet + no check = safe to prune)
            let child_board = board.make_move_new(m);
            if child_board.checkers().0 == 0 && fast_eval(board) + futility_margin <= alpha {
                continue;
            }
            let child = child_board;
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
            continue;
        }
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
/// Routes through the POWL v2 scheduler: each op in the topology DAG fires when its
/// predecessor bits are satisfied (branchless SWAR dependency evaluation).
#[must_use]
pub fn search_best_move_us(board: &Board, budget_us: u128) -> Option<ChessMove> {
    use crate::phase::{Phase, TopologyId};
    use crate::powl_runner::{OpKind, SearchCtx, ops_for_topology, run_topology};

    // STEP 1: Phase admission (O* → topology, O(1) table lookup).
    let (admitted, topology) = crate::phase::admit(board, budget_us);

    // STEP 2: Select op array for this topology.
    let ops = ops_for_topology(topology);

    // STEP 3: Build mutable search context.
    let mut ctx = SearchCtx::new(board, budget_us, topology, admitted.phase);

    // Local move list shared across MoveGen / MoveOrder / IterativeDeepening ops.
    let mut move_list: Vec<ChessMove> = Vec::new();

    // STEP 4: Run the POWL scheduler — each OpKind fires when its pred_mask is met.
    run_topology(ops, &mut ctx, |kind, ctx| {
        match kind {
            // --- Book probe: set ctx.best_move + ctx.book_hit if hit ---
            OpKind::BookProbe => {
                if let Some(bm) = crate::opening_book::book_probe(ctx.board.get_hash()) {
                    if ctx.board.legal(bm) {
                        ctx.best_move = Some(bm);
                        ctx.book_hit = true;
                    }
                }
            }

            // --- Phase admit: already done above; no-op here ---
            OpKind::PhaseAdmit => {}

            // --- TT probe: look up the PV move from TT ---
            // No move generation here — MoveGen op handles that.
            OpKind::TtProbe => {
                // We can't call tt_pv without a legal move list; defer to MoveOrder where
                // move_list is already populated. Mark as a no-op for now.
            }

            // --- MoveGen: generate all legal moves into local move_list ---
            OpKind::MoveGen => {
                // Tablebase short-circuit: ≤4 pieces → pick highest-SEE move directly.
                if ctx.topology == TopologyId::TABLEBASE_MICRO {
                    move_list = MoveGen::new_legal(ctx.board).collect();
                    move_list.sort_by_cached_key(|&m| -see(ctx.board, m));
                    if let Some(&mv) = move_list.first() {
                        ctx.best_move = Some(mv);
                        ctx.book_hit = true; // reuse flag to skip ID loop
                    }
                } else {
                    move_list = MoveGen::new_legal(ctx.board).collect();
                }
            }

            // --- MoveOrder: order moves by TT hint + history + killers + SEE ---
            OpKind::MoveOrder => {
                order(ctx.board, &mut move_list, ctx.tt_hint, 1 /* initial depth */);
            }

            // --- Iterative deepening: full aspiration window ID loop ---
            OpKind::IterativeDeepening => {
                if ctx.book_hit || move_list.is_empty() { return; }

                search_reset();
                // Reset the clock here so scheduler overhead (book probe, TT probe,
                // move gen, ordering) doesn't count against the search budget.
                let start = Instant::now();
                ctx.start = start;
                let endgame_mode = ctx.endgame_mode;
                let allow_null = ctx.allow_null;
                let asp = if endgame_mode { 75i32 } else { 25i32 };

                let mut best = move_list.first().copied();
                let mut prev = 0i32;
                let mut last_depth_us = 0u128;

                'id: for depth in 1usize..=32 {
                    let depth_start_us = start.elapsed().as_micros();
                    if depth > 1 && depth_start_us + last_depth_us * 3 > ctx.budget_us { break; }
                    if depth_start_us >= ctx.budget_us { break; }
                    let (mut lo, mut hi) = if depth == 1 { (-INF, INF) } else { (prev - asp, prev + asp) };
                    loop {
                        let hint = tt_pv(ctx.board.get_hash(), &move_list);
                        order(ctx.board, &mut move_list, hint, depth);
                        let mut db = best;
                        let mut dv = -INF;
                        let mut alpha = lo;
                        for &m in &move_list {
                            if start.elapsed().as_micros() >= ctx.budget_us { break 'id; }
                            let s = -ab(-hi, -alpha, depth.saturating_sub(1),
                                &ctx.board.make_move_new(m), start, ctx.budget_us, allow_null);
                            if s > dv { dv = s; db = Some(m); }
                            if s > alpha { alpha = s; }
                            if alpha >= hi { break; }
                        }
                        if dv <= lo      { lo = lo.saturating_sub(asp * 4); }
                        else if dv >= hi { hi = hi.saturating_add(asp * 4); }
                        else {
                            best = db; prev = dv;
                            last_depth_us = start.elapsed().as_micros().saturating_sub(depth_start_us);
                            println!("info depth {} score cp {} time {} pv {}", depth, dv,
                                start.elapsed().as_micros() / 1000,
                                db.map(|m| m.to_string()).unwrap_or_default());
                            break;
                        }
                        if lo <= -INF / 2 { lo = -INF; }
                        if hi >= INF / 2  { hi = INF; }
                    }
                }

                ctx.best_move = best;
                ctx.score = prev;
                ctx.depth_reached = 32u8.min(32); // depth reached approximation
                ctx.nodes = 0; // node count not tracked at this layer
            }

            // --- TT store: store final result (best_move + score) in TT ---
            OpKind::TtStore => {
                if let Some(bm) = ctx.best_move {
                    tt_store(ctx.board.get_hash(), ctx.depth_reached as usize,
                        ctx.score, Nt::Exact, Some(bm));
                }
            }

            // --- Receipt emit: seal a MoveReceipt for this move decision ---
            OpKind::ReceiptEmit => {
                if ctx.best_move.is_some() {
                    let move_id = MOVE_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if let Some((receipt, _)) = crate::receipts::record_move(
                        ctx.board, move_id, ctx.budget_us as u32, 0,
                        crate::receipts::GENESIS_HASH,
                    ) {
                        crate::receipts::emit(receipt);
                    }
                }
            }
        }
    });

    ctx.best_move
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
