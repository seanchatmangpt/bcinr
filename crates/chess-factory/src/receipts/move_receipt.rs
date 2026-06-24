//! Per-move receipt: the self-certifying record of one lawful decision.
//!
//! A [`MoveReceipt`] captures everything required to *re-derive a move
//! deterministically* and prove it lawful — the position, the per-station
//! evidence, the selection path, the recorded Petri stage trace, and the
//! BLAKE3 hash chain linking this move to the previous one. The verifier
//! ([`super::verifier`]) consumes a receipt, recomputes the move from
//! `fen_before` under `node_budget`, replays the stage trace, and recomputes the
//! chain hash; any divergence is a defect.
//!
//! Determinism contract: seeded RNG (`rng_seed`), node-bounded search
//! (`node_budget` — a deterministic node count, never wall-clock), a fixed
//! station order (`feature_set` / `stations`), and byte-stable JSON. The
//! resulting decision is bit-exact across replays.

use std::string::String;
use std::sync::{Mutex, OnceLock};
use std::vec::Vec;

use serde::{Deserialize, Serialize};

/// One station's contribution to a move decision, as recorded in the receipt.
///
/// Mirrors [`crate::station::Evidence`] in a serde-stable shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationRecord {
    /// Stable ordinal of the emitting station (fixed station order).
    pub station_id: u16,
    /// Human-readable station name (e.g. `"material"`).
    pub name: String,
    /// Bitmask of squares/features that fired.
    pub fired_mask: u64,
    /// Raw, pre-weight centipawn contribution (white-relative).
    pub raw_cp: i32,
    /// Q8.8 fixed-point weight applied during aggregation.
    pub weight_q8: i32,
}

/// One candidate move considered during selection, with its aggregated score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionStep {
    /// Candidate move in UCI long-algebraic form (e.g. `"e2e4"`).
    pub mv: String,
    /// Side-to-move-relative aggregated score in centipawns.
    pub score_cp: i32,
}

/// A complete, hash-chained receipt for a single move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveReceipt {
    /// Monotone move ordinal within the game (0-based).
    pub move_id: u32,
    /// FEN of the position *before* the move was made.
    pub fen_before: String,
    /// Per-station evidence, in fixed station order.
    pub stations: Vec<StationRecord>,
    /// Deterministic node budget for the search (node count, NOT wall-clock).
    pub node_budget: u32,
    /// Seed for the replay RNG (recorded so replay is bit-exact).
    pub rng_seed: u64,
    /// Names of the active feature stations, in evaluation order.
    pub feature_set: Vec<String>,
    /// The full candidate selection path (every legal move + its score).
    pub selection_path: Vec<SelectionStep>,
    /// The chosen move (UCI long-algebraic), the argmax of `selection_path`.
    pub chosen_move: String,
    /// The ordered Petri stage trace recorded for this decision.
    pub stage_trace: Vec<String>,
    /// Hash of the previous receipt in the chain (genesis = all-zero).
    pub prev_hash: [u8; 32],
    /// BLAKE3 hash binding this receipt to `prev_hash` (the chain link).
    pub verification_hash: [u8; 32],
}

impl MoveReceipt {
    /// Canonical preimage bytes for the chain hash.
    ///
    /// Folds the lawful, replay-relevant fields into a byte-stable preimage:
    /// `prev_hash`, position, budget/seed, the ordered station evidence, the
    /// selection path, the chosen move, the stage trace, and the active
    /// `feature_set` (so the claimed contributing stations are chain-bound). The
    /// `verification_hash` itself is excluded (it is the output).
    fn preimage(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.prev_hash);
        buf.extend_from_slice(&self.move_id.to_le_bytes());
        buf.extend_from_slice(self.fen_before.as_bytes());
        buf.extend_from_slice(&self.node_budget.to_le_bytes());
        buf.extend_from_slice(&self.rng_seed.to_le_bytes());
        for s in &self.stations {
            buf.extend_from_slice(&s.station_id.to_le_bytes());
            buf.extend_from_slice(s.name.as_bytes());
            buf.extend_from_slice(&s.fired_mask.to_le_bytes());
            buf.extend_from_slice(&s.raw_cp.to_le_bytes());
            buf.extend_from_slice(&s.weight_q8.to_le_bytes());
        }
        for step in &self.selection_path {
            buf.extend_from_slice(step.mv.as_bytes());
            buf.extend_from_slice(&step.score_cp.to_le_bytes());
        }
        buf.extend_from_slice(self.chosen_move.as_bytes());
        for stage in &self.stage_trace {
            buf.extend_from_slice(stage.as_bytes());
        }
        // Bind the claimed contributing stations into the chain hash so the
        // decision provenance (which stations fired) cannot be forged without
        // breaking the receipt.
        for name in &self.feature_set {
            buf.extend_from_slice(name.as_bytes());
        }
        buf
    }

    /// Recompute the chain hash this receipt *should* carry, given its
    /// `prev_hash` and lawful fields.
    #[must_use]
    pub fn recompute_hash(&self) -> [u8; 32] {
        *blake3::hash(&self.preimage()).as_bytes()
    }

    /// Stamp `verification_hash` from the current fields. Call after all lawful
    /// fields (including `prev_hash`) are set.
    pub fn seal(&mut self) {
        self.verification_hash = self.recompute_hash();
    }

    /// True iff the stored `verification_hash` matches the recomputed hash.
    #[must_use]
    pub fn hash_matches(&self) -> bool {
        self.recompute_hash() == self.verification_hash
    }
}

// ---------------------------------------------------------------------------
// Ring-buffer emission API (used by search_best_move_us)
// ---------------------------------------------------------------------------

/// Global ring buffer of the last 1024 move receipts.
fn receipt_buffer() -> &'static Mutex<Vec<MoveReceipt>> {
    static BUF: OnceLock<Mutex<Vec<MoveReceipt>>> = OnceLock::new();
    BUF.get_or_init(|| Mutex::new(Vec::with_capacity(1024)))
}

/// Emit a receipt into the ring buffer. Non-blocking: drops silently if mutex is contended.
pub fn emit(receipt: MoveReceipt) {
    if let Ok(mut buf) = receipt_buffer().try_lock() {
        if buf.len() >= 1024 {
            buf.remove(0);
        }
        buf.push(receipt);
    }
}

/// Drain all buffered receipts (clears the buffer). Used by tests and Manufacturing Graph feedback.
pub fn drain() -> Vec<MoveReceipt> {
    receipt_buffer()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .drain(..)
        .collect()
}

/// Peek at the last `n` receipts without draining.
#[must_use]
pub fn last(n: usize) -> Vec<MoveReceipt> {
    let buf = receipt_buffer().lock().unwrap_or_else(|e| e.into_inner());
    let start = buf.len().saturating_sub(n);
    buf[start..].to_vec()
}
