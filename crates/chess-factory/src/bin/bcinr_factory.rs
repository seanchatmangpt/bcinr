//! `bcinr_factory` — the manufactured chess decision engine (UCI).
//!
//! A persistent UCI engine whose evaluation is the GGEN-manufactured feature
//! stations (`aggregator::aggregate`) and whose search is the hand-authored
//! `search` wrapper (alpha-beta + quiescence + MVV-LVA), porting
//! `bcinr_uci`'s control surface:
//!
//!   uci / isready / ucinewgame / position [startpos|fen ...] [moves ...]
//!   go [movetime <ms> | depth <N>]      -> bestmove <uci>
//!   latency [depth] [iters]             -> min/median/p99/max ns per move
//!   quit
//!
//! One-shot mode is preserved for scripts: `bcinr_factory startpos` or
//! `bcinr_factory "<FEN>"` prints a single `bestmove` for a fixed-depth search.

use std::io::{self, BufRead};
use std::str::FromStr;
use std::time::Instant;

use chess::{Board, ChessMove, MoveGen};

use chess_factory::search::{fixed_depth_best_move, search_best_move, search_best_move_us};

/// Default fixed depth for `go depth` / one-shot mode.
const DEFAULT_DEPTH: usize = 4;

/// Measure per-move latency at a fixed depth: min/median/p99/max ns.
fn latency_probe(board: &Board, depth: usize, iters: usize) {
    for _ in 0..1000 {
        let _ = fixed_depth_best_move(board, depth);
    }
    let mut samples: Vec<u128> = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        let mv = fixed_depth_best_move(board, depth);
        let ns = t.elapsed().as_nanos();
        std::hint::black_box(mv);
        samples.push(ns);
    }
    samples.sort_unstable();
    let pick = |q: f64| samples[((iters as f64 * q) as usize).min(iters - 1)];
    let mv = fixed_depth_best_move(board, depth);
    println!(
        "latency depth={depth} iters={iters}: min={:.2}µs median={:.2}µs p99={:.2}µs max={:.2}µs | move={}",
        samples[0] as f64 / 1000.0,
        pick(0.50) as f64 / 1000.0,
        pick(0.99) as f64 / 1000.0,
        samples[iters - 1] as f64 / 1000.0,
        mv.map(|m| m.to_string()).unwrap_or_else(|| "none".into())
    );
}

/// Apply `moves ...` UCI tokens to a board.
fn apply_moves(board: &mut Board, tokens: &[&str]) {
    for m_str in tokens {
        if let Ok(m) = ChessMove::from_str(m_str) {
            if board.legal(m) {
                *board = board.make_move_new(m);
            }
        }
    }
}

/// One-shot mode: print a single bestmove for a FEN/startpos and exit.
fn one_shot(arg: &str) {
    let board = if arg == "startpos" {
        Board::default()
    } else {
        match Board::from_str(arg) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("invalid FEN: {e}");
                std::process::exit(2);
            }
        }
    };
    match fixed_depth_best_move(&board, DEFAULT_DEPTH) {
        Some(m) => println!("bestmove {m}"),
        None => println!("bestmove 0000"),
    }
}

fn uci_loop() {
    let mut board = Board::default();
    println!("id name BCINR Factory");
    println!("id author GGEN Chess Factory");
    println!("uciok");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "uci" => {
                println!("id name BCINR Factory");
                println!("id author GGEN Chess Factory");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => board = Board::default(),
            "position" => {
                if tokens.len() > 1 && tokens[1] == "startpos" {
                    board = Board::default();
                    if tokens.len() > 2 && tokens[2] == "moves" {
                        apply_moves(&mut board, &tokens[3..]);
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
                        apply_moves(&mut board, &tokens[start + 1..]);
                    }
                }
            }
            "go" => {
                let mut max_time_ms: u128 = 1000;
                let mut max_time_us: Option<u128> = None;
                let mut fixed_depth: Option<usize> = None;
                let mut i = 1;
                while i < tokens.len() {
                    match tokens[i] {
                        "movetime" => {
                            if let Some(v) = tokens.get(i + 1) {
                                max_time_ms = v.parse().unwrap_or(1000);
                            }
                        }
                        // Sub-millisecond budget in microseconds: `go movetime_us 100`
                        "movetime_us" => {
                            if let Some(v) = tokens.get(i + 1) {
                                max_time_us = v.parse().ok();
                            }
                        }
                        "depth" => {
                            fixed_depth = tokens.get(i + 1).and_then(|s| s.parse().ok());
                        }
                        _ => {}
                    }
                    i += 1;
                }
                let chosen = match fixed_depth {
                    Some(d) => fixed_depth_best_move(&board, d.max(1)),
                    None => {
                        if let Some(us) = max_time_us {
                            search_best_move_us(&board, us)
                        } else {
                            search_best_move(&board, max_time_ms)
                        }
                    }
                };
                match chosen.or_else(|| MoveGen::new_legal(&board).next()) {
                    Some(m) => println!("bestmove {m}"),
                    None => println!("bestmove 0000"),
                }
            }
            "latency" => {
                // Default depth=1 to measure the 1-ply branchless constant.
                let depth: usize = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                let iters: usize = tokens.get(2).and_then(|s| s.parse().ok()).unwrap_or(1000);
                latency_probe(&board, depth, iters);
            }
            "quit" => break,
            _ => {}
        }
    }
}

fn main() {
    match std::env::args().nth(1) {
        Some(arg) => one_shot(&arg),
        None => uci_loop(),
    }
}
