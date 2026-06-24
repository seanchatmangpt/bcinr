//! Evidence cells: receipts, Petri conformance, and the replay verifier.
//!
//! The factory does not merely make a fast move — it makes a *provably lawful*
//! one. Each decision is recorded as a [`MoveReceipt`] (hash-chained, byte-stable
//! JSON); the [`verifier`] re-derives the move deterministically from the receipt
//! and confirms three laws hold: (1) the BLAKE3 chain is intact, (2) the recorded
//! decision-pipeline stage trace replays through the [`petri_pipeline`] Petri net
//! with `fitness == 1.0`, and (3) the chosen move is legal and is the argmax the
//! generated stations actually produce. Any divergence is a first-class defect.
//!
//! The recorder ([`record_move`]) is the shared engine-side surface used by both
//! `bin/bcinr_factory` (game play) and the replay-completeness tests.

pub mod move_receipt;
pub mod petri_pipeline;
pub mod verifier;

pub use move_receipt::{MoveReceipt, SelectionStep, StationRecord, emit, drain, last};
pub use petri_pipeline::{ReplayResult, STAGE_NAMES};
pub use verifier::{verify_chain, verify_move, MoveVerdict, Refusal};

use std::string::{String, ToString};
use std::vec::Vec;

use chess::{Board, ChessMove, Color, MoveGen};

use crate::aggregator::aggregate;
use crate::position::PositionView;
use crate::select::argmax_i32;
use crate::stations::STATION_REGISTRY;

/// The genesis (all-zero) previous-hash for the first receipt in a chain.
pub const GENESIS_HASH: [u8; 32] = [0u8; 32];

/// Score a board from White's perspective via the generated aggregator.
#[must_use]
pub fn white_cp(board: &Board) -> i32 {
    aggregate(&PositionView::from_board(board))
}

/// Deterministically decide and record one move for `board`.
///
/// Re-derivable contract: enumerates the legal action set, scores each child
/// position with the generated stations (side-to-move-relative), selects the
/// branchless argmax, and emits a sealed [`MoveReceipt`] chained onto `prev_hash`.
/// The `node_budget` and `rng_seed` are recorded for replay; the 1-ply station
/// scan is itself deterministic. Returns `None` only at game end (no legal move).
#[must_use]
pub fn record_move(
    board: &Board,
    move_id: u32,
    node_budget: u32,
    rng_seed: u64,
    prev_hash: [u8; 32],
) -> Option<(MoveReceipt, ChessMove)> {
    let moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if moves.is_empty() {
        return None;
    }

    let stm = board.side_to_move();
    let sign: i32 = if stm == Color::White { 1 } else { -1 };

    // Selection path: every legal candidate + its stm-relative aggregated score.
    let scores: Vec<i32> = moves
        .iter()
        .map(|m| sign.wrapping_mul(white_cp(&board.make_move_new(*m))))
        .collect();
    let selection_path: Vec<SelectionStep> = moves
        .iter()
        .zip(scores.iter())
        .map(|(m, &score_cp)| SelectionStep {
            mv: m.to_string(),
            score_cp,
        })
        .collect();

    let best_idx = argmax_i32(&scores) as usize;
    let chosen = moves[best_idx];

    // Per-station evidence of the perceived (pre-move) position, fixed order.
    let view = PositionView::from_board(board);
    let stations: Vec<StationRecord> = STATION_REGISTRY
        .iter()
        .map(|spec| {
            let r = (spec.evaluate)(&view);
            StationRecord {
                station_id: spec.id,
                name: spec.name.to_string(),
                fired_mask: r.evidence.fired_mask,
                raw_cp: r.evidence.raw_cp,
                weight_q8: spec.weight_q8,
            }
        })
        .collect();
    let feature_set: Vec<String> = STATION_REGISTRY.iter().map(|s| s.name.to_string()).collect();

    let mut receipt = MoveReceipt {
        move_id,
        fen_before: board.to_string(),
        stations,
        node_budget,
        rng_seed,
        feature_set,
        selection_path,
        chosen_move: chosen.to_string(),
        stage_trace: STAGE_NAMES.iter().map(|s| (*s).to_string()).collect(),
        prev_hash,
        verification_hash: [0u8; 32],
    };
    receipt.seal();
    Some((receipt, chosen))
}

/// Record a full chain of up to `max_moves` moves from `start`, returning the
/// sealed receipts in order.
#[must_use]
pub fn record_game(
    start: &Board,
    max_moves: u32,
    node_budget: u32,
    rng_seed: u64,
) -> Vec<MoveReceipt> {
    let mut board = *start;
    let mut prev = GENESIS_HASH;
    let mut out = Vec::new();
    let mut id = 0u32;
    while id < max_moves {
        match record_move(&board, id, node_budget, rng_seed, prev) {
            Some((receipt, mv)) => {
                prev = receipt.verification_hash;
                out.push(receipt);
                board = board.make_move_new(mv);
                id += 1;
            }
            None => break,
        }
    }
    out
}
