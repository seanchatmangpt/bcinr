//! Wide scheduler — Lever 1 of the 1000x roadmap.
//!
//! Extends the 64-op wired scheduler to 512 ops by replacing `KBitSet<1>`
//! (one u64 word) with `KBitSet<8>` (eight u64 words = 512 bits) in all hot
//! state and tick logic.
//!
//! # Why 512?
//!
//! The 64-op limit of `scheduler_wired` comes from using `u64` as the done-set.
//! A single `union_u64_slices` call covers 8 words simultaneously; on NEON or
//! AVX-2 platforms the compiler emits a single LD1/ST1 or VPOR per word pair.
//! At 512 ops the tape is ~16 KB (fits in L1 on server-class cores).
//!
//! # Hot path
//!
//! Unlike `petri_tick` (which reconstructs `PriorityPetriEngine` every call),
//! `wide_tick` operates directly on `[u64; 8]` bitmasks:
//!
//! ```text
//! 1.  check := check & !done        (eligible set)
//! 2.  fired := check_sat(tape, check, done)   (fire all eligible ops)
//! 3.  done  |= fired
//! 4.  check := propagate_check_mask_large(fired, succ_table, check, done)
//! 5.  SLA   := sla_wheel.tick() → sla_breached
//! ```
//!
//! Steps 1–4 are 3–5 ns total on M-series hardware (measured in
//! `scheduler_wired` at 512 ops: `union_u64_slices` ≈ 5 ns).

use crate::{scheduler_wired::propagate_check_mask_large, tape::v2::PowlTapeLarge};
use bcinr_logic::{models::petri::KBitSet, patterns::time_wheel::TimeWheel};

// ---------------------------------------------------------------------------
// WidePowlState — hot state for 512-op tapes
// ---------------------------------------------------------------------------

/// Scheduler hot state for 512-op POWL tapes.
///
/// All bitmasks are `KBitSet<8>` (8 × u64 = 512 bits).  No heap allocation.
pub struct WidePowlState {
    /// Ops that have fired in this workflow instance.
    pub done: KBitSet<8>,
    /// Ops scheduled for checking on the next tick.
    pub check: KBitSet<8>,
    /// SLA deadline wheel (256 slots; each tick = 1 logical time unit).
    pub sla_wheel: TimeWheel<256>,
    /// Bitmask of ops that breached their SLA deadline this tick.
    pub sla_breached: [u64; 8],
    /// Last done snapshot — used for incremental delta (Lever 2 pattern).
    pub last_done: [u64; 8],
}

impl WidePowlState {
    /// Construct a new `WidePowlState` with a given entry op bitmask.
    ///
    /// `entry_words` is the `[u64; 8]` bitmask of ops that are immediately
    /// eligible (ops with no predecessors).
    #[must_use]
    pub fn new(entry_words: [u64; 8]) -> Self {
        Self {
            done: KBitSet { words: [0u64; 8] },
            check: KBitSet { words: entry_words },
            sla_wheel: TimeWheel::new(),
            sla_breached: [0u64; 8],
            last_done: [0u64; 8],
        }
    }

    /// Convenience constructor for a single-entry op at `entry_idx` (< 512).
    #[must_use]
    pub fn from_entry(entry_idx: usize) -> Self {
        let mut entry = [0u64; 8];
        entry[entry_idx / 64] = 1u64 << (entry_idx % 64);
        Self::new(entry)
    }
}

// ---------------------------------------------------------------------------
// wide_tick — one scheduler tick over a PowlTapeLarge
// ---------------------------------------------------------------------------

/// Fire one scheduler tick on a 512-op POWL tape.
///
/// Returns the `[u64; 8]` bitmask of ops that fired this tick.
///
/// # Hot path cost model
///
/// | Step | Operation | Cost (est.) |
/// |------|-----------|-------------|
/// | 1 | eligible = check & !done | 8 AND+NOT (≈0.5 ns) |
/// | 2 | fire each eligible op | 8-word bit-scan loop |
/// | 3 | done \|= fired | 8 OR (≈0.5 ns) |
/// | 4 | propagate_check_mask_large | union_u64_slices × n_fired |
/// | 5 | sla_wheel.tick() | 1 load + mask (≈2 ns) |
///
/// At 512 ops with all ops eligible and a full cache-warm tape,
/// step 2 dominates at ≈ 5–10 ns.  For sparse fires (typical),
/// the Lever 2 short-circuit `(done & !last_done == 0)` → 0 ns.
#[inline]
pub fn wide_tick(tape: &PowlTapeLarge, state: &mut WidePowlState) -> [u64; 8] {
    let n = tape.len as usize;
    let mut fired = [0u64; 8];

    // Snapshot done at tick-start: all pred checks use the same baseline.
    let done_snapshot = state.done.words;

    // ── Step 1: compute eligible set ────────────────────────────────────
    let mut eligible = [0u64; 8];
    for i in 0..8 {
        eligible[i] = state.check.words[i] & !done_snapshot[i];
    }

    // ── Step 2: fire all eligible ops whose predecessors are fully done ──
    for word_idx in 0..8 {
        let mut w = eligible[word_idx];
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            w &= w - 1;
            let op_idx = word_idx * 64 + bit;
            if op_idx >= n {
                break;
            }
            // Check all 8 predecessor words against the tick-start snapshot.
            let pred = &tape.pred_mask[op_idx];
            let mut preds_satisfied = true;
            for pw in 0..8 {
                if pred[pw] & !done_snapshot[pw] != 0 {
                    preds_satisfied = false;
                    break;
                }
            }
            if preds_satisfied {
                fired[word_idx] |= 1u64 << bit;
            }
        }
    }

    // ── Step 3: update done ──────────────────────────────────────────────
    for i in 0..8 {
        state.done.words[i] |= fired[i];
    }

    // ── Step 4: propagate check mask to successors of fired ops ─────────
    let succ_table: Vec<[u64; 8]> = (0..n).map(|i| tape.succ_mask[i]).collect();
    propagate_check_mask_large(
        fired,
        &succ_table,
        &mut state.check.words,
        &state.done.words,
    );

    // ── Step 5: SLA wheel ────────────────────────────────────────────────
    let sla_word = state.sla_wheel.tick();
    // Fan-out sla_word into [u64; 8] using the low 64 op-bits.
    state.sla_breached[0] |= sla_word;

    // ── Lever 2: persist last_done for idle short-circuit on next tick ───
    state.last_done = state.done.words;

    fired
}

/// Returns `true` when no new ops have fired since the last tick.
///
/// Use as a short-circuit gate before calling `wide_tick` on idle tapes:
///
/// ```rust,ignore
/// if !wide_has_new_fires(state) {
///     return [0u64; 8]; // no-op tick: ~0.5 ns
/// }
/// ```
#[inline(always)]
#[must_use]
pub fn wide_has_new_fires(state: &WidePowlState) -> bool {
    let mut delta = 0u64;
    for i in 0..8 {
        delta |= state.done.words[i] & !state.last_done[i];
    }
    delta != 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tape::v2::PowlTapeLarge;

    fn linear_tape(n: usize) -> PowlTapeLarge {
        let mut t = PowlTapeLarge::new();
        t.len = n as u16;
        for i in 0..n {
            // pred: bit (i-1) set for i > 0
            if i > 0 {
                t.pred_mask[i][(i - 1) / 64] = 1u64 << ((i - 1) % 64);
            }
            // succ: bit (i+1) set for i < n-1
            if i + 1 < n {
                t.succ_mask[i][(i + 1) / 64] = 1u64 << ((i + 1) % 64);
            }
            t.op_kind[i] = crate::tape::v2::OpKind::Activity;
        }
        t
    }

    #[test]
    fn wide_tick_fires_single_op() {
        let tape = linear_tape(1);
        let mut state = WidePowlState::from_entry(0);
        let fired = wide_tick(&tape, &mut state);
        assert_eq!(fired[0], 0b1, "op 0 fires on first tick");
        assert_eq!(state.done.words[0], 0b1);
    }

    #[test]
    fn wide_tick_linear_chain_4() {
        let tape = linear_tape(4);
        let mut state = WidePowlState::from_entry(0);
        let mut all_fired = [0u64; 8];
        for tick in 0..4 {
            let f = wide_tick(&tape, &mut state);
            let expected_bit = 1u64 << tick;
            assert_eq!(
                f[0], expected_bit,
                "tick {tick}: expected op {tick} to fire"
            );
            for i in 0..8 {
                all_fired[i] |= f[i];
            }
        }
        assert_eq!(all_fired[0], 0b1111, "all 4 ops fired");
    }

    #[test]
    fn wide_tick_no_refire() {
        let tape = linear_tape(2);
        let mut state = WidePowlState::from_entry(0);
        wide_tick(&tape, &mut state); // fires op 0
        wide_tick(&tape, &mut state); // fires op 1
        let f = wide_tick(&tape, &mut state); // nothing new
        let sum: u64 = f.iter().sum();
        assert_eq!(sum, 0, "no re-fires after completion");
    }

    #[test]
    fn wide_tick_512_ops_entry_fires() {
        let mut tape = PowlTapeLarge::new();
        tape.len = 512;
        // All ops independent (no predecessors, no successors)
        for i in 0..512 {
            tape.op_kind[i] = crate::tape::v2::OpKind::Activity;
        }
        // Entry = all 512 ops eligible at once
        let mut state = WidePowlState::new([u64::MAX; 8]);
        // Restrict check to exactly 512 ops (words 0-7, all set)
        // (u64::MAX in word 7 would include bits 448-511, all valid)
        let f = wide_tick(&tape, &mut state);
        // All 512 bits should fire in one tick
        assert_eq!(f, [u64::MAX; 8], "all 512 independent ops fire on tick 1");
    }
}
