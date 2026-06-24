//! GGEN Chess Factory.
//!
//! A branchless chess decision system manufactured from semantic law
//! (TTL -> SPARQL -> GGEN -> Tera -> Rust). The runtime stations are
//! `no_std`, `#![forbid(unsafe_code)]`, and each public station kernel
//! is held to cyclomatic complexity 1 by the contract gate.
#![no_std]
#![forbid(unsafe_code)]

pub mod station;
pub mod position;
pub mod rays;
pub mod aggregator;
pub mod select;
pub mod stations;
pub mod motifs;
pub mod weights;
pub mod defects;
pub mod evidence;

/// Evidence cells: per-move receipts, Petri conformance, and the replay verifier.
///
/// This module is `std`-backed: it serializes receipts (serde/serde_json),
/// hash-chains them (blake3), and re-derives moves through the `chess` crate
/// boundary during verification.
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod receipts;

/// Opening book: sorted (hash, UCI-move) table, binary-searched at probe time.
/// Computed numerics — not GGEN-manufactured.
#[cfg(feature = "std")]
pub mod opening_book;

/// Manufactured search wrapper (alpha-beta + quiescence + MVV-LVA) over the
/// generated aggregator. `std`-only; the hand-authored search boundary.
#[cfg(feature = "std")]
pub mod search;

/// NNUE (Efficiently Updatable Neural Network) evaluation.
/// HalfKP features -> L1 accumulator -> clipped-ReLU -> L2 -> output (centipawns).
/// Until trained weights are loaded, degrades gracefully to material evaluation.
#[cfg(feature = "std")]
pub mod nnue;

/// Stockfish NNUE file loader (HalfKAv2, COMPRESSED_LEB128).
/// Loads the actual SF18 weight file and runs the full 3-layer forward pass.
#[cfg(feature = "std")]
pub mod nnue_sf;

/// Phase classification and O(1) topology selection.
/// Implements the Chatman Equation admission layer: O* → μ → topology.
/// The topology table is promoted by the Manufacturing Graph (offline benchmarks).
#[cfg(feature = "std")]
pub mod phase;

/// POWL v2 type-state runtime scheduler.
/// Branchless SWAR evaluation of the search DAG; one op array per topology.
#[cfg(feature = "std")]
pub mod powl_runner;

/// Manufacturing graph: topology benchmarking and offline eval.
#[cfg(feature = "std")]
pub mod manufacturing_graph;

#[cfg(test)]
mod eval_debug {
    extern crate std;
    use chess::{Board, ChessMove};
    use std::str::FromStr;
    use crate::aggregator::aggregate;
    use crate::position::PositionView;

    fn score(fen_moves: &[&str]) -> i32 {
        let mut b = Board::default();
        for m in fen_moves { b = b.make_move_new(ChessMove::from_str(m).unwrap()); }
        let v = PositionView::from_board(&b);
        aggregate(&v)
    }

    #[test]
    fn eval_sanity() {
        extern crate std;
        use std::println;
        println!("startpos:          {}", score(&[]));
        println!("after 1.e4:        {}", score(&["e2e4"]));
        println!("after 1.a3:        {}", score(&["a2a3"]));
        println!("after 1.e4 e5:     {}", score(&["e2e4","e7e5"]));
        println!("after 1.e4 a6:     {}", score(&["e2e4","a7a6"]));
        // Move 3 candidates after 1.e4 e5 2.Nf3 d6
        println!("1.e4 e5 Nf3 d6 d4:   {}", score(&["e2e4","e7e5","g1f3","d7d6","d2d4"]));
        println!("1.e4 e5 Nf3 d6 a3:   {}", score(&["e2e4","e7e5","g1f3","d7d6","a2a3"]));
        println!("1.e4 e5 Nf3 d6 Nc3:  {}", score(&["e2e4","e7e5","g1f3","d7d6","b1c3"]));
        println!("1.e4 e5 Nf3 d6 Bc4:  {}", score(&["e2e4","e7e5","g1f3","d7d6","f1c4"]));
        // White up a queen
        println!("white +queen:      {}", score(&["e2e4","d7d5","e4d5","d8d5","b1c3","d5a2","c1d2","a2a1"]));
    }

    #[test]
    #[cfg(feature = "std")]
    fn search_prefers_development() {
        extern crate std;
        use std::str::FromStr;
        use chess::{Board, ChessMove};
        use crate::search::{fixed_depth_best_move};
        let mut b = Board::default();
        for m in &["e2e4","e7e5","g1f3","d7d6"] {
            b = b.make_move_new(ChessMove::from_str(m).unwrap());
        }
        let best = fixed_depth_best_move(&b, 1).unwrap();
        std::println!("Depth-1 best: {}", best);
        // Bc4, Nc3, or d4 — not a passive pawn move
        use std::string::ToString;
        let bad_moves = ["a2a3","b2b3","h2h3","a2a4","g2g3"];
        assert!(!bad_moves.contains(&best.to_string().as_str()),
            "Factory should develop, not play {}", best);
    }
}
