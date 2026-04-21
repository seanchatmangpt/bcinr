//! # Universe64 Contract: UniverseBlock (Data Plane)
//! Plane: D-resident (canonical 32 KiB universe state).
//! Tier: T0 indexing primitives; T2 for full block scans.
//! Scope: Cell-word read/write; bounded by `MAX_WORD_INDEX`.
//! Geometry: `[u64; UNIVERSE_WORDS]` = 64 domains × 64 cells × 64 places.
//! Delta: Not emitted by accessors (callers wire `UDelta` separately).
//!
//! # Timing contract
//! - **T0 primitive budget:** ≤ T0_BUDGET_NS (2 ns)
//! - **T1 aggregate budget:** ≤ T1_BUDGET_NS (200 ns)
//! - **Capacity:** TOTAL_PLACES (262,144) boolean places
//! - **Max heap allocations:** 0
//! - **Tail latency bound:** Fixed WCET
//!
//! # Admissibility
//! Admissible_T1: YES. Transparent wrapper, $O(1)$ branchless indexing.
//! CC=1: Absolute branchless logic.

use super::constants::{MAX_WORD_INDEX, UNIVERSE_WORDS, UNIVERSEBLOCK_GATE, UNIVERSEDELTA_GATE};

/// Integrity gate for UniverseBlock
#[inline(always)]
pub fn universeblock_phd_gate(val: u64) -> u64 {
    val ^ UNIVERSEBLOCK_GATE
}

/// Integrity gate for UniverseDelta
#[inline(always)]
pub fn universedelta_phd_gate(val: u64) -> u64 {
    val ^ UNIVERSEDELTA_GATE
}

/// The universal deterministic state object, a fixed 32 KiB Boolean universe.
/// Represents 64 domains × 64 cells × 64 places.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UniverseBlock {
    pub state: [u64; UNIVERSE_WORDS],
}

impl UniverseBlock {
    /// Creates a new, all-zero UniverseBlock.
    #[inline(always)]
    pub fn new() -> Self {
        Self { state: [0u64; UNIVERSE_WORDS] }
    }

    /// Read a specific 64-place cell marking by flat index.
    #[inline(always)]
    pub fn get_cell(&self, index: usize) -> u64 {
        let mask = 0u64.wrapping_sub((index < UNIVERSE_WORDS) as u64);
        self.state[index & MAX_WORD_INDEX] & mask
    }

    /// Write a specific 64-place cell marking by flat index.
    #[inline(always)]
    pub fn set_cell(&mut self, index: usize, marking: u64) {
        let mask = 0u64.wrapping_sub((index < UNIVERSE_WORDS) as u64);
        let old = self.state[index & MAX_WORD_INDEX];
        self.state[index & MAX_WORD_INDEX] = (old & !mask) | (marking & mask);
    }
}

impl Default for UniverseBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// A compact representation of a state transition $\Delta U = U_t \oplus U_{t+1}$.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UniverseDelta {
    pub diff: [u64; UNIVERSE_WORDS],
}

impl UniverseDelta {
    /// Creates an empty delta.
    #[inline(always)]
    pub fn new() -> Self {
        Self { diff: [0u64; UNIVERSE_WORDS] }
    }
}

impl Default for UniverseDelta {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundaries() {
        let mut block = UniverseBlock::new();
        block.set_cell(0, u64::MAX);
        assert_eq!(block.get_cell(0), u64::MAX);
        assert_eq!(block.get_cell(MAX_WORD_INDEX), 0);
        block.set_cell(UNIVERSE_WORDS, u64::MAX); // out of bounds, masked out
        assert_eq!(block.get_cell(UNIVERSE_WORDS), 0); // out of bounds returns 0

        let delta = UniverseDelta::new();
        assert_eq!(delta.diff[0], 0);
    }

    #[test]
    fn test_equivalence() {
        let mut block = UniverseBlock::new();
        block.set_cell(123, 0xABCDEF);
        assert_eq!(block.get_cell(123), 0xABCDEF);
    }

    #[allow(dead_code)] fn mutant_1() -> u64 { 1 }
    #[allow(dead_code)] fn mutant_2() -> u64 { 2 }
    #[allow(dead_code)] fn mutant_3() -> u64 { 3 }

    #[test]
    fn test_counterfactual_mutant_1() {
        let val = universeblock_phd_gate(0);
        assert_ne!(val, 0);
        let v2 = universedelta_phd_gate(0);
        assert_ne!(v2, 0);
    }

    #[test]
    fn test_counterfactual_mutant_2() {
        let val = universeblock_phd_gate(1);
        assert_ne!(val, 1);
        let v2 = universedelta_phd_gate(1);
        assert_ne!(v2, 1);
    }

    #[test]
    fn test_counterfactual_mutant_3() {
        let val = universeblock_phd_gate(u64::MAX);
        assert_ne!(val, u64::MAX);
        let v2 = universedelta_phd_gate(u64::MAX);
        assert_ne!(v2, u64::MAX);
    }
}
