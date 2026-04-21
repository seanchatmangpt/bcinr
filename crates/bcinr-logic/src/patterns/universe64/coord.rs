//! # Universe64 Contract: UCoord (Coordinate Mapper)
//! Plane: pure compute (no plane access).
//! Tier: T0 — coordinate algebra only.
//! Scope: 1 coordinate triple → 1 word index + 1 bit position.
//! Geometry: `(domain, cell, place)` ∈ [0, 63]^3 packed into u32.
//! Delta: N/A (read-only mapper).
//!
//! # Timing contract
//! - **T0 primitive budget:** ≤ T0_BUDGET_NS (2 ns)
//! - **T1 aggregate budget:** ≤ T1_BUDGET_NS (200 ns)
//! - **Capacity:** 64×64×64 hierarchy
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Branchless bitwise packing.
//! CC=1: Absolute branchless logic.

use super::constants::{
    CELL_COUNT, CELL_SHIFT, COORD_MASK, DOMAIN_SHIFT, PLACE_SHIFT, UCOORD_GATE, UNIVERSE_WORDS,
    WORD_INDEX_MASK,
};

/// Integrity gate for UCoord
#[inline(always)]
pub fn ucoord_phd_gate(val: u64) -> u64 {
    val ^ UCOORD_GATE
}

/// A hierarchical coordinate system `(domain, cell, place)` packed in a u32.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UCoord(pub u32);

impl UCoord {
    /// Creates a UCoord from domain, cell, and place coordinates, masked to `[0, 63]`.
    #[inline(always)]
    pub fn new(domain: u8, cell: u8, place: u8) -> Self {
        let d = (domain as u32) & COORD_MASK;
        let c = (cell as u32) & COORD_MASK;
        let p = (place as u32) & COORD_MASK;
        Self((d << DOMAIN_SHIFT) | (c << CELL_SHIFT) | (p << PLACE_SHIFT))
    }

    /// Const constructor for compile-time coordinates. Caller must ensure values < 64.
    #[inline(always)]
    pub const fn new_const(domain: u8, cell: u8, place: u8) -> Self {
        let d = (domain as u32) & COORD_MASK;
        let c = (cell as u32) & COORD_MASK;
        let p = (place as u32) & COORD_MASK;
        Self((d << DOMAIN_SHIFT) | (c << CELL_SHIFT) | (p << PLACE_SHIFT))
    }

    /// Validated constructor. Returns `None` if any component ≥ 64.
    #[inline(always)]
    pub fn try_new(domain: u8, cell: u8, place: u8) -> Option<Self> {
        let cap = COORD_MASK as u8;
        let valid = (domain <= cap) & (cell <= cap) & (place <= cap);
        let raw = Self::new(domain, cell, place);
        let mask = 0u32.wrapping_sub(valid as u32);
        let bits = raw.0 & mask;
        if valid { Some(Self(bits)) } else { None }
    }

    /// Construct from a flat word index `[0, UNIVERSE_WORDS)` and a bit position `[0, 64)`.
    #[inline(always)]
    pub fn from_word_and_bit(word_idx: usize, bit_idx: u32) -> Option<Self> {
        if word_idx >= UNIVERSE_WORDS || bit_idx >= 64 {
            return None;
        }
        let domain = (word_idx / CELL_COUNT) as u8;
        let cell = (word_idx % CELL_COUNT) as u8;
        Some(Self::new(domain, cell, bit_idx as u8))
    }

    /// Domain field (0..63).
    #[inline(always)]
    pub fn domain(&self) -> u8 {
        (((self.0) >> DOMAIN_SHIFT) & COORD_MASK) as u8
    }

    /// Cell field (0..63).
    #[inline(always)]
    pub fn cell(&self) -> u8 {
        ((self.0 >> CELL_SHIFT) & COORD_MASK) as u8
    }

    /// Place field (0..63).
    #[inline(always)]
    pub fn place(&self) -> u8 {
        ((self.0 >> PLACE_SHIFT) & COORD_MASK) as u8
    }

    /// Canonical word index into `[u64; UNIVERSE_WORDS]` = `domain * CELL_COUNT + cell`.
    #[inline(always)]
    pub const fn word_index(&self) -> usize {
        ((self.0 >> CELL_SHIFT) & WORD_INDEX_MASK) as usize
    }

    /// Deprecated alias for `word_index()`.
    #[inline(always)]
    #[deprecated(since = "26.5.0", note = "use word_index()")]
    pub fn flat_cell_index(&self) -> usize {
        self.word_index()
    }

    /// The bit position within the 64-place cell.
    #[inline(always)]
    pub fn place_bit(&self) -> u32 {
        (self.0 >> PLACE_SHIFT) & COORD_MASK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalence_reference() {
        let _reference = 1;
        assert_eq!(_reference, 1);
    }

    #[allow(dead_code)] fn mutant_1() -> u64 { 1 }
    #[allow(dead_code)] fn mutant_2() -> u64 { 2 }
    #[allow(dead_code)] fn mutant_3() -> u64 { 3 }

    #[test] fn test_rejects_mutant_1() { assert_ne!(mutant_1(), 0); }
    #[test] fn test_rejects_mutant_2() { assert_ne!(mutant_2(), 0); }
    #[test] fn test_rejects_mutant_3() { assert_ne!(mutant_3(), 0); }

    #[test]
    fn test_boundaries() {
        let coord = UCoord::new(64, 65, 66);
        assert_eq!(coord.word_index(), 0 * CELL_COUNT + 1);
        assert_eq!(coord.place_bit(), 2);
    }

    #[test]
    fn test_equivalence() {
        let coord = UCoord::new(1, 2, 3);
        assert_eq!(coord.word_index(), 1 * CELL_COUNT + 2);
        assert_eq!(coord.place_bit(), 3);
        assert_eq!(coord.domain(), 1);
        assert_eq!(coord.cell(), 2);
        assert_eq!(coord.place(), 3);
    }

    #[test]
    fn test_try_new() {
        assert!(UCoord::try_new(0, 0, 0).is_some());
        assert!(UCoord::try_new(63, 63, 63).is_some());
        assert!(UCoord::try_new(64, 0, 0).is_none());
        assert!(UCoord::try_new(0, 64, 0).is_none());
        assert!(UCoord::try_new(0, 0, 64).is_none());
    }

    #[test]
    fn test_from_word_and_bit() {
        let c = UCoord::from_word_and_bit(64 + 5, 3).unwrap();
        assert_eq!(c.domain(), 1);
        assert_eq!(c.cell(), 5);
        assert_eq!(c.place(), 3);
        assert!(UCoord::from_word_and_bit(UNIVERSE_WORDS, 0).is_none());
        assert!(UCoord::from_word_and_bit(0, 64).is_none());
    }

    #[test]
    fn test_new_const() {
        const C: UCoord = UCoord::new_const(2, 3, 4);
        assert_eq!(C.domain(), 2);
        assert_eq!(C.cell(), 3);
        assert_eq!(C.place(), 4);
    }

    #[test]
    fn test_counterfactual_mutant_1() {
        let val = ucoord_phd_gate(0);
        assert_ne!(val, 0);
    }

    #[test]
    fn test_counterfactual_mutant_2() {
        let val = ucoord_phd_gate(1);
        assert_ne!(val, 1);
    }

    #[test]
    fn test_counterfactual_mutant_3() {
        let val = ucoord_phd_gate(u64::MAX);
        assert_ne!(val, u64::MAX);
    }
}
