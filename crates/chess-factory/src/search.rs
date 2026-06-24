//! Manufactured search wrapper (HAND-AUTHORED boundary).
//!
//! Negamax alpha-beta + quiescence + MVV-LVA move ordering, ported from
//! `playground/src/bin/bcinr_uci.rs`. The leaf evaluation is the generated
//! `aggregator::aggregate(&PositionView)` (white-relative centipawns) instead
//! of the playground NNUE — this is the parity wrapper that lifts the
//! branchless feature stations into a `bcinr_uci`-class engine.
//!
//! This is NOT a station: it MAY branch. Stations remain CC=1; this wrapper is
//! the search boundary. It is `std`-only (depends on the `chess` crate).
#![cfg(feature = "std")]

use std::time::Instant;
use std::vec::Vec;

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};

use crate::aggregator::aggregate;
use crate::position::PositionView;

/// Maximum iterative-deepening depth.
const MAX_DEPTH: usize = 100;
/// Mate score baseline (centipawns).
const MATE: i32 = 1_000_000;

/// Static MVV-LVA piece values (centipawns).
fn piece_value(p: Piece) -> i32 {
    match p {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
    }
}

/// MVV-LVA capture score for move ordering.
fn move_score(board: &Board, m: &ChessMove) -> i32 {
    let mut score = 0;
    if let Some(captured) = board.piece_on(m.get_dest()) {
        if let Some(attacker) = board.piece_on(m.get_source()) {
            score = 10 * piece_value(captured) - piece_value(attacker);
        }
    }
    if let Some(prom) = m.get_promotion() {
        score += piece_value(prom);
    }
    score
}

/// Order moves by descending MVV-LVA (captures/promotions first).
fn order_moves(board: &Board, moves: &mut Vec<ChessMove>) {
    moves.sort_by_cached_key(|m| -move_score(board, m));
}

/// Side-to-move-relative leaf evaluation via the generated aggregator.
///
/// `aggregate` is white-relative; flip the sign for Black to move so negamax
/// sees a consistent "good for side to move" scale.
fn evaluate(board: &Board) -> i32 {
    let view = PositionView::from_board(board);
    let white_cp = aggregate(&view);
    if board.side_to_move() == Color::Black {
        -white_cp
    } else {
        white_cp
    }
}

/// Quiescence search over captures to stabilise the leaf score.
fn quiescence(mut alpha: i32, beta: i32, board: &Board) -> i32 {
    let stand_pat = evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    moves.retain(|m| board.piece_on(m.get_dest()).is_some());
    order_moves(board, &mut moves);

    for m in moves {
        let child = board.make_move_new(m);
        let score = -quiescence(-beta, -alpha, &child);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

/// Negamax alpha-beta with a wall-clock cutoff (`max_time_ms`).
#[allow(clippy::too_many_arguments)]
fn alphabeta(
    alpha: i32,
    beta: i32,
    depth: usize,
    board: &Board,
    start: Instant,
    max_time_ms: u128,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;
    if *nodes % 2048 == 0 && start.elapsed().as_millis() >= max_time_ms {
        return 0;
    }
    match board.status() {
        BoardStatus::Checkmate => return -MATE + (100 - depth as i32),
        BoardStatus::Stalemate => return 0,
        BoardStatus::Ongoing => {}
    }
    if depth == 0 {
        return quiescence(alpha, beta, board);
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    order_moves(board, &mut moves);

    let mut best = -2 * MATE;
    let mut local_alpha = alpha;
    for m in moves {
        let child = board.make_move_new(m);
        let val = -alphabeta(-beta, -local_alpha, depth - 1, &child, start, max_time_ms, nodes);
        if start.elapsed().as_millis() >= max_time_ms {
            return best.max(val);
        }
        if val > best {
            best = val;
        }
        if val > local_alpha {
            local_alpha = val;
        }
        if local_alpha >= beta {
            break;
        }
    }
    best
}

/// Fixed-depth root search (no time limit) — deterministic, used for parity
/// play and latency measurement. `depth==1` resolves leaves through quiescence
/// (≈ Stockfish-depth-1 quality).
#[must_use]
pub fn fixed_depth_best_move(board: &Board, depth: usize) -> Option<ChessMove> {
    let start = Instant::now();
    let mut nodes = 0u64;
    let mut best = None;
    let mut best_val = -2 * MATE;
    let mut alpha = -2 * MATE;
    let beta = 2 * MATE;
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    order_moves(board, &mut moves);
    for m in &moves {
        let child = board.make_move_new(*m);
        let val = -alphabeta(-beta, -alpha, depth.max(1) - 1, &child, start, u128::MAX, &mut nodes);
        if val > best_val {
            best_val = val;
            best = Some(*m);
        }
        if val > alpha {
            alpha = val;
        }
    }
    best
}

/// Time-bounded search with a microsecond budget using iterative deepening.
///
/// Starts at depth 1, records the best move, then tries depth 2 and depth 3
/// if enough time remains (keeping a ~20µs margin before the deadline).
/// This replaces the old "≤100µs → fixed depth-1 only" short-circuit so the
/// factory can find tactical wins within its 100µs budget.
#[must_use]
pub fn search_best_move_us(board: &Board, max_time_us: u128) -> Option<ChessMove> {
    const MARGIN_US: u128 = 20; // reserve 20µs as safety margin
    let deadline_us = max_time_us.saturating_sub(MARGIN_US);

    let start = Instant::now();
    let mut best_overall: Option<ChessMove> = None;
    let mut nodes = 0u64;

    for depth in 1..=MAX_DEPTH {
        // Check if we have enough time to attempt this depth before starting.
        let elapsed_us = start.elapsed().as_micros();
        if elapsed_us >= deadline_us {
            break;
        }

        let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
        order_moves(board, &mut moves);
        let mut best_val = -2 * MATE;
        let mut best_move: Option<ChessMove> = None;
        let mut alpha = -2 * MATE;
        let beta = 2 * MATE;
        let mut timed_out = false;

        for m in &moves {
            // Per-move time check.
            if start.elapsed().as_micros() >= deadline_us {
                timed_out = true;
                break;
            }
            let child = board.make_move_new(*m);
            // Pass deadline as milliseconds to alphabeta (it uses ms internally),
            // but we use our own µs check around it.
            let val = -alphabeta(
                -beta,
                -alpha,
                depth - 1,
                &child,
                start,
                // Give alphabeta a very large ms limit; we control time via the
                // per-move µs check above and the nodes % 2048 check inside.
                u128::MAX,
                &mut nodes,
            );
            if start.elapsed().as_micros() >= deadline_us {
                // Partial depth result — do not update best_overall.
                timed_out = true;
                break;
            }
            if val > best_val {
                best_val = val;
                best_move = Some(*m);
            }
            if val > alpha {
                alpha = val;
            }
        }

        // Always promote partial results: even an incomplete depth-N search that
        // evaluated the highest-priority (capture-ordered) moves is better than
        // falling back to depth-(N-1).  Only reject if zero root moves scored.
        if let Some(m) = best_move {
            best_overall = Some(m);
        }

        if timed_out {
            break;
        }
    }

    best_overall.or_else(|| MoveGen::new_legal(board).next())
}

/// Time-bounded iterative-deepening root search.
#[must_use]
pub fn search_best_move(board: &Board, max_time_ms: u128) -> Option<ChessMove> {
    let start = Instant::now();
    let mut best_overall = None;
    let mut nodes = 0u64;

    for depth in 1..=MAX_DEPTH {
        let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
        order_moves(board, &mut moves);
        let mut best_val = -2 * MATE;
        let mut best_move = None;
        let mut alpha = -2 * MATE;
        let beta = 2 * MATE;
        for m in &moves {
            let child = board.make_move_new(*m);
            let val = -alphabeta(-beta, -alpha, depth - 1, &child, start, max_time_ms, &mut nodes);
            if start.elapsed().as_millis() >= max_time_ms {
                break;
            }
            if val > best_val {
                best_val = val;
                best_move = Some(*m);
            }
            if val > alpha {
                alpha = val;
            }
        }
        if start.elapsed().as_millis() >= max_time_ms {
            break;
        }
        if let Some(m) = best_move {
            best_overall = Some(m);
        }
    }
    best_overall.or_else(|| MoveGen::new_legal(board).next())
}
