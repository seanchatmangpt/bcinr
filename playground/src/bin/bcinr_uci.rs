use std::{
    io::{self, BufRead},
    str::FromStr,
    time::Instant,
};

use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};

/// CPU-side NNUE accumulator: the L1 hidden activations for a board.
struct Accumulator {
    hidden: [i32; 16],
}

/// Compute the NNUE L1 accumulator for a board on the CPU.
///
/// Despite the legacy name, this runs entirely on the CPU; the score is always
/// white-relative (see `evaluate_board` for the side-to-move sign flip).
fn board_to_accumulator(b: &Board, nnue: &playground::nnue::BranchTorchNNUE) -> Accumulator {
    let mut hidden = nnue.l1_biases;
    let pieces = [
        chess::Piece::Pawn,
        chess::Piece::Knight,
        chess::Piece::Bishop,
        chess::Piece::Rook,
        chess::Piece::Queen,
        chess::Piece::King,
    ];
    let mut p_idx = 0;
    for &p in &pieces {
        let w_bb = *b.color_combined(Color::White) & *b.pieces(p);
        for sq in w_bb {
            let sq_idx = sq.to_index();
            for i in 0..16 {
                hidden[i] += nnue.l1_weights[i][p_idx * 64 + sq_idx];
            }
        }
        let b_bb = *b.color_combined(Color::Black) & *b.pieces(p);
        for sq in b_bb {
            let sq_idx = sq.to_index();
            for i in 0..16 {
                hidden[i] += nnue.l1_weights[i][(p_idx + 6) * 64 + sq_idx];
            }
        }
        p_idx += 1;
    }
    Accumulator { hidden }
}

// Alpha-Beta Search implementation
const MAX_DEPTH: usize = 100;

fn piece_value(p: chess::Piece) -> i32 {
    match p {
        chess::Piece::Pawn => 100,
        chess::Piece::Knight => 320,
        chess::Piece::Bishop => 330,
        chess::Piece::Rook => 500,
        chess::Piece::Queen => 900,
        chess::Piece::King => 20000,
    }
}

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

fn order_moves(board: &Board, moves: &mut Vec<ChessMove>) {
    moves.sort_by_cached_key(|m| -move_score(board, m));
}

fn evaluate_board(board: &Board, nnue: &playground::nnue::BranchTorchNNUE) -> f32 {
    let acc = board_to_accumulator(board, nnue);
    // Neuron 0 holds the signed white-relative material+PST eval. Neuron 1 is its
    // mirror (-neuron0), used only by the GPU's split-ReLU value head; summing all
    // 16 hidden units would cancel neuron0 against neuron1 and yield ~0. On the CPU
    // we read the signed eval directly from neuron 0.
    let score = acc.hidden[0] as f32;
    // Score is always relative to White in board_to_accumulator
    if board.side_to_move() == Color::Black {
        -score
    } else {
        score
    }
}

fn quiescence(
    mut alpha: f32,
    beta: f32,
    board: &Board,
    nnue: &playground::nnue::BranchTorchNNUE,
) -> f32 {
    let stand_pat = evaluate_board(board, nnue);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    moves.retain(|m| board.piece_on(m.get_dest()).is_some()); // Only captures
    order_moves(board, &mut moves);

    for m in moves {
        let child = board.make_move_new(m);
        let score = -quiescence(-beta, -alpha, &child, nnue);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

fn alphabeta(
    alpha: f32,
    beta: f32,
    depth: usize,
    board: &Board,
    nnue: &playground::nnue::BranchTorchNNUE,
    start_time: Instant,
    max_time_ms: u128,
    nodes: &mut u64,
) -> f32 {
    *nodes += 1;

    if *nodes % 2048 == 0 && start_time.elapsed().as_millis() >= max_time_ms {
        return 0.0; // Time out
    }

    if board.status() == BoardStatus::Checkmate {
        return -100000.0 + (100 - depth) as f32;
    }
    if board.status() == BoardStatus::Stalemate {
        return 0.0;
    }

    if depth == 0 {
        return quiescence(alpha, beta, board, nnue);
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    order_moves(board, &mut moves);

    let mut best_val = -1000000.0;
    let mut local_alpha = alpha;

    for m in moves {
        let child = board.make_move_new(m);
        let val = -alphabeta(
            -beta,
            -local_alpha,
            depth - 1,
            &child,
            nnue,
            start_time,
            max_time_ms,
            nodes,
        );

        if start_time.elapsed().as_millis() >= max_time_ms {
            return 0.0;
        }

        if val > best_val {
            best_val = val;
        }
        if val > local_alpha {
            local_alpha = val;
        }
        if local_alpha >= beta {
            break;
        }
    }
    best_val
}

/// Fixed-depth root search (no time limit) — used for latency measurement.
/// `depth==1` means a 1-ply search whose leaves are resolved by quiescence,
/// which is roughly Stockfish-depth-1 quality.
fn fixed_depth_best_move(
    board: &Board,
    depth: usize,
    nnue: &playground::nnue::BranchTorchNNUE,
) -> Option<ChessMove> {
    let start = Instant::now();
    let mut nodes = 0u64;
    let mut best = None;
    let mut best_val = -1.0e9;
    let mut alpha = -1.0e9;
    let beta = 1.0e9;
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    order_moves(board, &mut moves);
    for m in &moves {
        let child = board.make_move_new(*m);
        let val = -alphabeta(-beta, -alpha, depth - 1, &child, nnue, start, u128::MAX, &mut nodes);
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

/// Measure per-move latency for the current position at a fixed depth.
/// Reports min / median / p99 / max nanoseconds over `iters` warm iterations,
/// with the NNUE built ONCE (not per move).
fn latency_probe(board: &Board, depth: usize, iters: usize) {
    let nnue = playground::nnue::BranchTorchNNUE::new();
    // Warm up.
    for _ in 0..1000 {
        let _ = fixed_depth_best_move(board, depth, &nnue);
    }
    let mut samples: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        let mv = fixed_depth_best_move(board, depth, &nnue);
        let ns = t.elapsed().as_nanos();
        std::hint::black_box(mv);
        samples.push(ns);
    }
    samples.sort_unstable();
    let pick = |q: f64| samples[((iters as f64 * q) as usize).min(iters - 1)];
    let mv = fixed_depth_best_move(board, depth, &nnue);
    println!(
        "latency depth={depth} iters={iters}: min={}ns median={}ns p99={}ns max={}ns | move={}",
        samples[0],
        pick(0.50),
        pick(0.99),
        samples[iters - 1],
        mv.map(|m| m.to_string()).unwrap_or_else(|| "none".into())
    );
}

fn search_best_move(board: &Board, max_time_ms: u128) -> Option<ChessMove> {
    let start_time = Instant::now();
    let nnue_inst = playground::nnue::BranchTorchNNUE::new();
    let mut best_move_overall = None;
    let mut nodes = 0;

    for depth in 1..=MAX_DEPTH {
        let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
        order_moves(board, &mut moves);

        let mut best_val = -1000000.0;
        let mut best_move = None;
        let mut alpha = -1000000.0;
        let beta = 1000000.0;

        for m in &moves {
            let child = board.make_move_new(*m);
            let val = -alphabeta(
                -beta,
                -alpha,
                depth - 1,
                &child,
                &nnue_inst,
                start_time,
                max_time_ms,
                &mut nodes,
            );

            if start_time.elapsed().as_millis() >= max_time_ms {
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

        if start_time.elapsed().as_millis() >= max_time_ms {
            break; // Keep best_move_overall from previous completed depth
        }

        if let Some(m) = best_move {
            best_move_overall = Some(m);
        }

        let elapsed = start_time.elapsed().as_millis();
        println!(
            "info depth {} nodes {} time {} nps {}",
            depth,
            nodes,
            elapsed,
            (nodes as u128 * 1000) / elapsed.max(1)
        );
    }

    best_move_overall
}

fn main() {
    let mut board = Board::default();
    // Build the NNUE once so fixed-depth moves aren't charged the weight-init cost.
    let nnue = playground::nnue::BranchTorchNNUE::new();

    println!("id name BCINR AlphaBeta");
    println!("id author AG");
    println!("uciok");

    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        let line = line_result.unwrap();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name BCINR AlphaBeta");
                println!("id author AG");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "position" => {
                if tokens.len() > 1 && tokens[1] == "startpos" {
                    board = Board::default();
                    if tokens.len() > 2 && tokens[2] == "moves" {
                        for m_str in &tokens[3..] {
                            if let Ok(m) = ChessMove::from_str(m_str) {
                                board = board.make_move_new(m);
                            }
                        }
                    }
                } else if tokens.len() > 1 && tokens[1] == "fen" {
                    let mut fen = String::new();
                    let mut start = 2;
                    while start < tokens.len() && tokens[start] != "moves" {
                        fen.push_str(tokens[start]);
                        fen.push(' ');
                        start += 1;
                    }
                    if let Ok(b) = Board::from_str(fen.trim()) {
                        board = b;
                    }
                    if start < tokens.len() && tokens[start] == "moves" {
                        for m_str in &tokens[start + 1..] {
                            if let Ok(m) = ChessMove::from_str(m_str) {
                                board = board.make_move_new(m);
                            }
                        }
                    }
                }
            }
            "go" => {
                let mut max_time_ms = 1000;
                let mut fixed_depth: Option<usize> = None;
                for i in 1..tokens.len() {
                    if tokens[i] == "movetime" && i + 1 < tokens.len() {
                        max_time_ms = tokens[i + 1].parse::<u128>().unwrap_or(1000);
                    }
                    if tokens[i] == "depth" && i + 1 < tokens.len() {
                        fixed_depth = tokens[i + 1].parse::<usize>().ok();
                    }
                }

                // `go depth N` => microsecond-scale fixed-depth move (no iterative
                // deepening, NNUE prebuilt). This is the speed-parity mode used to
                // play at a per-move latency below Stockfish's depth-1 floor.
                let chosen = if let Some(d) = fixed_depth {
                    fixed_depth_best_move(&board, d.max(1), &nnue)
                } else {
                    search_best_move(&board, max_time_ms)
                };
                if let Some(m) = chosen {
                    println!("bestmove {}", m);
                } else {
                    let moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
                    if !moves.is_empty() {
                        println!("bestmove {}", moves[0]);
                    } else {
                        println!("bestmove 0000");
                    }
                }
            }
            "latency" => {
                let depth: usize = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                let iters: usize = tokens.get(2).and_then(|s| s.parse().ok()).unwrap_or(20000);
                latency_probe(&board, depth, iters);
            }
            "quit" => {
                break;
            }
            _ => {}
        }
    }
}
