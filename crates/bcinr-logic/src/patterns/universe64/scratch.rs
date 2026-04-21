//! # Universe64 Contract: UniverseScratch (Scratch Plane)
//! Plane: S — bounded 32 KiB motion workspace, NOT canonical state.
//! Tier: T0 register-class staging; T1 active-word kernels; T2 reductions.
//! Scope: bounded by SCRATCH_BYTES; no heap, no allocation.
//! Geometry: parallel `[u64; UNIVERSE_WORDS]` for delta staging, bounded ActiveWordSet.
//! Delta: `UDelta` lives in this plane, not the data plane.
//!
//! # Timing contract
//! - **T0 primitive budget:** ≤ T0_BUDGET_NS (operand stage)
//! - **T1 aggregate budget:** ≤ T1_BUDGET_NS (active-word commit)
//! - **Capacity:** SCRATCH_BYTES = 32 KiB; ACTIVE_WORD_CAPACITY = 64 indexes
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Bounded scratch reuse, branchless updates.
//! CC=1: Absolute branchless logic.

use super::constants::{ACTIVE_WORD_CAPACITY, MAX_WORD_INDEX, UNIVERSE_WORDS, USCRATCH_GATE};

/// Integrity gate for UniverseScratch
#[inline(always)]
pub fn uscratch_phd_gate(val: u64) -> u64 {
    val ^ USCRATCH_GATE
}

/// Scope of a transition operation against the data plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UScope {
    /// One word (cell-level T0/T1).
    Cell = 0,
    /// Bounded active-word subset (T1 sparse).
    Sparse = 1,
    /// All cells in one domain (T1 domain SWAR).
    Domain = 2,
    /// All UNIVERSE_WORDS (T2 full block).
    Full = 3,
}

/// Sparse motion record. Drives projections, ready-mask, and receipts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct UDelta {
    /// Word index in [0, UNIVERSE_WORDS).
    pub word_idx: u16,
    /// Scope under which the change occurred.
    pub scope: UScope,
    /// Cause: instruction id (caller-defined opaque).
    pub instr_id: u32,
    /// Pre-image word.
    pub before: u64,
    /// Post-image word.
    pub after: u64,
    /// Branchless success mask (`!0` if fired, `0` if rejected).
    pub fired_mask: u64,
}

impl UDelta {
    /// Construct a delta record. Word index is masked to MAX_WORD_INDEX.
    #[inline(always)]
    pub const fn new(
        word_idx: usize,
        scope: UScope,
        instr_id: u32,
        before: u64,
        after: u64,
        fired_mask: u64,
    ) -> Self {
        Self {
            word_idx: (word_idx & MAX_WORD_INDEX) as u16,
            scope,
            instr_id,
            before,
            after,
            fired_mask,
        }
    }

    /// Bits that actually changed (`before XOR after`).
    #[inline(always)]
    pub const fn changed_mask(&self) -> u64 {
        self.before ^ self.after
    }

    /// True if no bits changed (no-op delta).
    #[inline(always)]
    pub const fn is_noop(&self) -> bool {
        (self.before ^ self.after) == 0
    }
}

/// Bounded set of word indexes touched by a scoped operation.
/// Capacity: ACTIVE_WORD_CAPACITY (64 indexes for T1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ActiveWordSet {
    /// Number of valid entries (≤ ACTIVE_WORD_CAPACITY).
    pub len: u8,
    /// Word indexes; entries beyond `len` are unspecified.
    pub indexes: [u16; ACTIVE_WORD_CAPACITY],
}

impl ActiveWordSet {
    /// Empty set.
    #[inline(always)]
    pub const fn new() -> Self {
        Self { len: 0, indexes: [0u16; ACTIVE_WORD_CAPACITY] }
    }

    /// Push a word index. Branchless saturation: full sets are no-ops.
    #[inline(always)]
    pub fn push(&mut self, word_idx: usize) {
        let admit = (self.len as usize) < ACTIVE_WORD_CAPACITY;
        let mask = 0u32.wrapping_sub(admit as u32);
        let slot = (self.len as usize) & (ACTIVE_WORD_CAPACITY - 1);
        let bits = ((word_idx & MAX_WORD_INDEX) as u32) & mask;
        self.indexes[slot] = bits as u16;
        self.len = self.len.wrapping_add(admit as u8);
    }

    /// Number of valid indexes.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// True when no indexes are present.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear all entries (sets `len = 0`; data left in place for branchless reuse).
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Default for ActiveWordSet {
    fn default() -> Self {
        Self::new()
    }
}

/// The Scratch Plane: bounded 32 KiB workspace.
/// Holds parallel staging arrays for active-word transition motion.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct UniverseScratch {
    /// Pre-image staging (parallel to UniverseBlock.state).
    pub before: [u64; UNIVERSE_WORDS],
    /// Post-image staging.
    pub after: [u64; UNIVERSE_WORDS],
    /// Active words touched in current cycle.
    pub active: ActiveWordSet,
}

impl UniverseScratch {
    /// All-zero scratch plane.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            before: [0u64; UNIVERSE_WORDS],
            after: [0u64; UNIVERSE_WORDS],
            active: ActiveWordSet::new(),
        }
    }

    /// Reset cycle state without zeroing arrays (branchless, T0-budget).
    #[inline(always)]
    pub fn reset_cycle(&mut self) {
        self.active.clear();
    }
}

impl Default for UniverseScratch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_noop_and_changed() {
        let d = UDelta::new(10, UScope::Cell, 7, 0xF0, 0xFF, !0);
        assert_eq!(d.changed_mask(), 0x0F);
        assert!(!d.is_noop());

        let n = UDelta::new(10, UScope::Cell, 7, 0xF0, 0xF0, 0);
        assert!(n.is_noop());
    }

    #[test]
    fn test_active_word_set_bounds() {
        let mut a = ActiveWordSet::new();
        for i in 0..ACTIVE_WORD_CAPACITY {
            a.push(i);
        }
        assert_eq!(a.len(), ACTIVE_WORD_CAPACITY);
        // Saturation: pushing more does not grow.
        a.push(9999);
        assert_eq!(a.len(), ACTIVE_WORD_CAPACITY);
    }

    #[test]
    fn test_scratch_lifecycle() {
        let mut s = UniverseScratch::new();
        s.active.push(42);
        assert_eq!(s.active.len(), 1);
        s.reset_cycle();
        assert!(s.active.is_empty());
    }

    #[test]
    fn test_uscratch_phd_gate() {
        assert_ne!(uscratch_phd_gate(0), 0);
        assert_ne!(uscratch_phd_gate(1), 1);
        assert_ne!(uscratch_phd_gate(u64::MAX), u64::MAX);
    }
}
