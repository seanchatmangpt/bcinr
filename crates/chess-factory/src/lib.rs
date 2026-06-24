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
        // White up a queen
        println!("white +queen:      {}", score(&["e2e4","d7d5","e4d5","d8d5","b1c3","d5a2","c1d2","a2a1"]));
    }
}
