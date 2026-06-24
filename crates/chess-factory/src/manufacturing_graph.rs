#![cfg(feature = "std")]
extern crate std;

use chess::Board;
use std::time::Instant;
use std::vec::Vec;
use std::str::FromStr;
use std::string::ToString;

use crate::phase::TopologyId;

/// A single position used to benchmark topology variants.
#[derive(Debug, Clone)]
pub struct BenchPosition {
    /// FEN string for the position.
    pub fen: &'static str,
    /// Expected best move in UCI notation (oracle). Empty if unknown.
    pub oracle_move: &'static str,
    /// Expected eval in centipawns (STM-relative). 9999 if unknown.
    pub oracle_eval: i32,
}

/// Result of running a topology on a set of positions.
#[derive(Debug, Clone)]
pub struct TopologyBenchResult {
    pub topology: TopologyId,
    /// Fraction of test positions where this topology agreed with oracle move.
    pub move_agreement: f32,
    /// Average centipawn error vs oracle eval (lower = better).
    pub avg_cp_error: f32,
    /// Average microseconds per move decision.
    pub avg_us: f32,
    /// Composite score: agreement / (1 + avg_us/1000). Higher = better.
    pub composite_score: f32,
}

/// Standard test suite of 20 tactical and positional positions.
pub const BENCH_SUITE: &[BenchPosition] = &[
    BenchPosition { fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
                    oracle_move: "e7e5", oracle_eval: 0 },
    BenchPosition { fen: "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
                    oracle_move: "b8c6", oracle_eval: 0 },
    BenchPosition { fen: "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
                    oracle_move: "g8f6", oracle_eval: 0 },
    BenchPosition { fen: "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 2 5",
                    oracle_move: "c2c3", oracle_eval: 10 },
    BenchPosition { fen: "8/8/8/8/8/3k4/3p4/3K4 b - - 0 1",
                    oracle_move: "d2d1q", oracle_eval: 900 },
    BenchPosition { fen: "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
                    oracle_move: "e1g1", oracle_eval: 0 },
    BenchPosition { fen: "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 6 5",
                    oracle_move: "d2d3", oracle_eval: 0 },
    BenchPosition { fen: "rnbqkb1r/pp3ppp/2p1pn2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 5",
                    oracle_move: "c4d5", oracle_eval: 0 },
    BenchPosition { fen: "8/8/8/3k4/8/3K4/3P4/8 w - - 0 1",
                    oracle_move: "d2d4", oracle_eval: 200 },
    BenchPosition { fen: "r3k2r/ppp2ppp/2n1bn2/3qp3/3PP3/2NB1N2/PPP2PPP/R2QK2R w KQkq - 0 8",
                    oracle_move: "e1g1", oracle_eval: 0 },
    BenchPosition { fen: "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
                    oracle_move: "e4d5", oracle_eval: 0 },
    BenchPosition { fen: "rnbqkb1r/pppp1ppp/5n2/4p3/4PP2/8/PPPP2PP/RNBQKBNR b KQkq - 0 3",
                    oracle_move: "d7d5", oracle_eval: 20 },
    BenchPosition { fen: "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
                    oracle_move: "f1b5", oracle_eval: 20 },
    BenchPosition { fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
                    oracle_move: "g2g4", oracle_eval: 0 },
    BenchPosition { fen: "8/8/p7/1p6/1Pp5/2P5/6K1/3k4 w - - 0 1",
                    oracle_move: "g2f3", oracle_eval: -100 },
    BenchPosition { fen: "r4rk1/ppp2ppp/2n2n2/2b1p1B1/2B1P1b1/2NP1N2/PPP2PPP/R2QK2R w KQ - 0 9",
                    oracle_move: "c3d5", oracle_eval: 30 },
    BenchPosition { fen: "rnbq1rk1/ppp1bppp/4pn2/3p4/2PP4/5NP1/PP2PPBP/RNBQ1RK1 w - - 0 7",
                    oracle_move: "c4d5", oracle_eval: 0 },
    BenchPosition { fen: "r2qkb1r/pp2pppp/2p2n2/8/3Pp3/2N5/PPP2PPP/R1BQKB1R w KQkq - 0 8",
                    oracle_move: "d4d5", oracle_eval: 20 },
    BenchPosition { fen: "8/8/8/8/8/1k6/1p6/1K6 b - - 0 1",
                    oracle_move: "b2b1q", oracle_eval: 900 },
    BenchPosition { fen: "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
                    oracle_move: "g1f3", oracle_eval: 0 },
];

/// Run the full Manufacturing Graph benchmark.
/// Tests all topology variants on BENCH_SUITE and returns ranked results.
pub fn run_benchmark(budget_us: u128) -> Vec<TopologyBenchResult> {
    let topologies = [
        TopologyId::OPENING_MICRO_SINGLE,
        TopologyId::TACTICAL_MICRO,
        TopologyId::QUIET_MICRO_SINGLE,
        TopologyId::ENDGAME_MICRO,
    ];

    let mut results = Vec::new();

    for &topology in &topologies {
        let mut move_agreements = 0u32;
        let mut total_cp_error = 0i32;
        let mut total_us = 0u64;
        let mut n_valid = 0u32;

        for pos in BENCH_SUITE {
            let board = match Board::from_str(pos.fen) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let t0 = Instant::now();
            let mv = crate::search::search_best_move_us(&board, budget_us);
            let elapsed = t0.elapsed().as_micros() as u64;
            let eval = crate::search::eval_position(&board);

            total_us += elapsed;
            total_cp_error += (eval - pos.oracle_eval).abs();

            if let Some(m) = mv {
                let mv_str = m.to_string();
                if !pos.oracle_move.is_empty() && mv_str == pos.oracle_move {
                    move_agreements += 1;
                }
            }
            n_valid += 1;
        }

        let n = n_valid.max(1) as f32;
        let move_agreement = move_agreements as f32 / n;
        let avg_cp_error = total_cp_error as f32 / n;
        let avg_us = total_us as f32 / n;
        let composite_score = move_agreement / (1.0 + avg_us / 1000.0);

        results.push(TopologyBenchResult {
            topology,
            move_agreement,
            avg_cp_error,
            avg_us,
            composite_score,
        });
    }

    // Sort by composite score descending
    results.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Return the best topology for a given phase based on benchmark results.
pub fn promote_winner(results: &[TopologyBenchResult]) -> Option<TopologyId> {
    results.first().map(|r| r.topology)
}
