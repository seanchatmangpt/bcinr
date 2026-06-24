//! Replay verifier: re-derive a move from its receipt and prove it lawful.
//!
//! The verifier trusts *only re-derivable evidence*. For each [`MoveReceipt`] it:
//!
//! 1. **Chain law** — recomputes the BLAKE3 `verification_hash` from the receipt
//!    body and checks it matches, then checks `prev_hash` links the chain.
//! 2. **Process law** — replays `stage_trace` through the decision Petri net and
//!    requires `fitness == 1.0` (no skipped/reordered stage).
//! 3. **Decision law** — re-runs the generated stations over `fen_before` under
//!    `node_budget`, recomputes the branchless argmax, and checks `chosen_move`
//!    is exactly the re-derived move.
//! 4. **Legality law** — confirms `chosen_move` is in the legal move set of
//!    `fen_before` (POWL/legality via the `chess` action set).
//!
//! Any failed law yields a named [`Refusal`]; a fully lawful move is `Admit`.

use core::str::FromStr;

use std::string::String;
use std::vec::Vec;

use chess::{Board, ChessMove, MoveGen};

use super::move_receipt::MoveReceipt;
use super::petri_pipeline;
use super::record_move;

/// A named law violation (the defect taxonomy for a single move).
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// `fen_before` did not parse as a legal position.
    InvalidPosition,
    /// The stored hash did not match the recomputed chain hash.
    BrokenHashChain,
    /// `prev_hash` did not match the predecessor's `verification_hash`.
    ChainDiscontinuity,
    /// The Petri replay of `stage_trace` was not perfect (`fitness < 1.0`).
    InvalidTransition { fitness: f64 },
    /// `chosen_move` is not a legal move in `fen_before`.
    Illegal,
    /// `chosen_move` did not match the re-derived argmax decision.
    StrategyDivergence {
        /// The move the receipt claims.
        claimed: String,
        /// The move re-derivation actually produced.
        rederived: String,
    },
    /// `chosen_move` was not parseable as a UCI move.
    MalformedMove,
}

/// The per-move verdict: lawful, or a refusal carrying the violated law.
#[derive(Debug, Clone, PartialEq)]
pub enum MoveVerdict {
    /// All four laws held.
    Admit {
        /// Conformance fitness (== 1.0 for an admitted move).
        fitness: f64,
    },
    /// A law was violated.
    Refuse(Refusal),
}

impl MoveVerdict {
    /// True iff this verdict admitted the move as fully lawful.
    #[must_use]
    pub fn is_compliant(&self) -> bool {
        matches!(self, MoveVerdict::Admit { .. })
    }
}

/// Verify a single move receipt against all four laws (ignoring chain linkage,
/// which [`verify_chain`] layers on top).
#[must_use]
pub fn verify_move(receipt: &MoveReceipt) -> MoveVerdict {
    // --- Chain law (self-consistency of this receipt's hash). ---
    if !receipt.hash_matches() {
        return MoveVerdict::Refuse(Refusal::BrokenHashChain);
    }

    // --- Position parse. ---
    let board = match Board::from_str(&receipt.fen_before) {
        Ok(b) => b,
        Err(_) => return MoveVerdict::Refuse(Refusal::InvalidPosition),
    };

    // --- Process law: Petri conformance of the recorded stage trace. ---
    let stage_refs: Vec<&str> = receipt.stage_trace.iter().map(String::as_str).collect();
    let replay = petri_pipeline::replay(&stage_refs);
    let fitness = replay.fitness();
    if !replay.is_perfect() {
        return MoveVerdict::Refuse(Refusal::InvalidTransition { fitness });
    }

    // --- Legality law. ---
    let chosen = match ChessMove::from_str(&receipt.chosen_move) {
        Ok(m) => m,
        Err(_) => return MoveVerdict::Refuse(Refusal::MalformedMove),
    };
    let legal: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
    if !legal.contains(&chosen) {
        return MoveVerdict::Refuse(Refusal::Illegal);
    }

    // --- Decision law: re-derive the move independently and compare. ---
    match record_move(
        &board,
        receipt.move_id,
        receipt.node_budget,
        receipt.rng_seed,
        receipt.prev_hash,
    ) {
        Some((rederived, _)) => {
            if rederived.chosen_move != receipt.chosen_move {
                return MoveVerdict::Refuse(Refusal::StrategyDivergence {
                    claimed: receipt.chosen_move.clone(),
                    rederived: rederived.chosen_move,
                });
            }
            // The re-derivation must reproduce the exact sealed hash, bit-for-bit.
            if rederived.verification_hash != receipt.verification_hash {
                return MoveVerdict::Refuse(Refusal::BrokenHashChain);
            }
        }
        None => return MoveVerdict::Refuse(Refusal::Illegal),
    }

    MoveVerdict::Admit { fitness }
}

/// The aggregate verdict over a whole receipt chain.
#[derive(Debug, Clone)]
pub struct GameVerdict {
    /// Per-move verdicts, in order.
    pub moves: Vec<MoveVerdict>,
    /// True iff every move was compliant and the chain linkage was intact.
    pub is_compliant: bool,
}

/// Verify an ordered receipt chain: every move's four laws PLUS the chain
/// linkage (`prev_hash[i] == verification_hash[i-1]`, genesis at the head).
#[must_use]
pub fn verify_chain(receipts: &[MoveReceipt]) -> GameVerdict {
    let mut moves = Vec::with_capacity(receipts.len());
    let mut ok = true;
    let mut expected_prev = super::GENESIS_HASH;

    for receipt in receipts {
        if receipt.prev_hash != expected_prev {
            moves.push(MoveVerdict::Refuse(Refusal::ChainDiscontinuity));
            ok = false;
            // Re-anchor so a single break doesn't cascade meaninglessly.
            expected_prev = receipt.verification_hash;
            continue;
        }
        let verdict = verify_move(receipt);
        if !verdict.is_compliant() {
            ok = false;
        }
        moves.push(verdict);
        expected_prev = receipt.verification_hash;
    }

    GameVerdict {
        moves,
        is_compliant: ok,
    }
}
