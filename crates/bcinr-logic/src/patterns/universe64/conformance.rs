//! # Universe64 Contract: ConformanceKernel (Delta-Incremental Conformance)
//! Plane: D — reads block state; S — reads deltas from scratch.
//! Tier: T0 per-delta check; T1 incremental accumulation.
//! Scope: stateful ConformanceState updated per delta; no full-block rescan.
//! Geometry: tracks (violations_added, violations_cleared) incrementally via UDelta.
//! Delta: none produced — conformance is an observer.
//!
//! # Timing contract
//! - **T0 per-delta update:** ≤ T0_BUDGET_NS
//! - **T1 batch update:** ≤ T1_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Branchless incremental XOR + popcount.
//! CC=1: Absolute branchless logic.

use super::constants::UNIVERSE_WORDS;
use super::block::UniverseBlock;
use super::scratch::UDelta;
use super::law::word_violation;

/// Running conformance score updated incrementally by deltas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConformanceState {
    /// Total bits currently violating law (across the whole block).
    pub violation_bits: u32,
    /// Sequence number of last delta applied.
    pub sequence: u32,
}

impl ConformanceState {
    pub const fn zero() -> Self {
        Self { violation_bits: 0, sequence: 0 }
    }

    /// True if no violations are tracked.
    #[inline(always)]
    pub const fn is_conformant(&self) -> bool {
        self.violation_bits == 0
    }
}

// ---------------------------------------------------------------------------
// ConformanceKernel
// ---------------------------------------------------------------------------

/// Stateless incremental conformance updater.
pub struct ConformanceKernel;

impl ConformanceKernel {
    /// Initialize ConformanceState from a full-block scan against law masks.
    /// Call once at boot; use `update_delta` thereafter.
    pub fn initialize(block: &UniverseBlock, law: &[u64; UNIVERSE_WORDS]) -> ConformanceState {
        let mut total = 0u32;
        for (i, &law_word) in law.iter().enumerate() {
            total = total.wrapping_add(word_violation(block.state[i], law_word).count_ones());
        }
        ConformanceState { violation_bits: total, sequence: 0 }
    }

    /// Incrementally update ConformanceState given one UDelta and the corresponding law word.
    /// T0: O(1), branchless.
    #[inline(always)]
    pub fn update_delta(
        state: &mut ConformanceState,
        delta: &UDelta,
        law_word: u64,
    ) {
        // Old violations in this word (before the delta).
        let old_v = word_violation(delta.before, law_word).count_ones();
        // New violations in this word (after the delta).
        let new_v = word_violation(delta.after, law_word).count_ones();
        // Branchless signed adjustment.
        state.violation_bits = state.violation_bits
            .wrapping_add(new_v)
            .wrapping_sub(old_v);
        state.sequence = state.sequence.wrapping_add(1);
    }

    /// Batch update from a slice of deltas and a law array.
    pub fn update_batch(
        state: &mut ConformanceState,
        deltas: &[UDelta],
        law: &[u64; UNIVERSE_WORDS],
    ) {
        for d in deltas {
            let widx = d.word_idx as usize & (UNIVERSE_WORDS - 1);
            Self::update_delta(state, d, law[widx]);
        }
    }

    /// Full recompute (expensive; use only for verification/boot).
    pub fn recompute(block: &UniverseBlock, law: &[u64; UNIVERSE_WORDS]) -> u32 {
        let mut total = 0u32;
        for (i, &law_word) in law.iter().enumerate() {
            total = total.wrapping_add(word_violation(block.state[i], law_word).count_ones());
        }
        total
    }
}

/// Conformance distance: popcount of all violation bits.
#[inline(always)]
pub fn conformance_distance(block: &UniverseBlock, law: &[u64; UNIVERSE_WORDS]) -> u32 {
    ConformanceKernel::recompute(block, law)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conformance_initialize_zero() {
        let block = UniverseBlock::new();
        let law = [u64::MAX; UNIVERSE_WORDS];
        let state = ConformanceKernel::initialize(&block, &law);
        assert_eq!(state.violation_bits, 0);
        assert!(state.is_conformant());
    }

    #[test]
    fn test_conformance_initialize_with_violations() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0b1100; // bits 2&3 set
        let mut law = [u64::MAX; UNIVERSE_WORDS];
        law[0] = 0b0011; // only bits 0&1 allowed
        let state = ConformanceKernel::initialize(&block, &law);
        assert_eq!(state.violation_bits, 2);
    }

    #[test]
    fn test_update_delta_clears_violation() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0b1100;
        let mut law = [u64::MAX; UNIVERSE_WORDS];
        law[0] = 0b0011;
        let mut cstate = ConformanceKernel::initialize(&block, &law);
        assert_eq!(cstate.violation_bits, 2);

        // Delta that fixes the word: after = 0b0001 (no violation)
        let delta = UDelta::new(0, super::super::scratch::UScope::Cell, 1, 0b1100, 0b0001, !0);
        ConformanceKernel::update_delta(&mut cstate, &delta, law[0]);
        assert_eq!(cstate.violation_bits, 0);
    }

    #[test]
    fn test_conformance_distance() {
        let mut block = UniverseBlock::new();
        block.state[1] = 0xFF;
        let mut law = [u64::MAX; UNIVERSE_WORDS];
        law[1] = 0x0F;
        assert_eq!(conformance_distance(&block, &law), 4);
    }
}
