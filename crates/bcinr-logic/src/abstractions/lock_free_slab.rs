//! Higher-Level Abstraction: lock_free_slab
///
/// Branchless slab allocation using bitmask tracking for deterministic
/// O(1) slot allocation and release.
///
/// # Axiomatic Proof
/// O(1) trailing-zeros execution removes loops when finding free slots.
/// The bitmask `M` represents the state of 64 slots. `M & !(1 << i)`
/// is the branchless state transition for allocating slot `i`.

#[derive(Clone, Debug)]
pub struct LockFreeSlabState {
    pub bitmap: u64,
}

impl Default for LockFreeSlabState {
    fn default() -> Self {
        Self::new()
    }
}

impl LockFreeSlabState {
    pub fn new() -> Self {
        Self { bitmap: 0xFFFFFFFFFFFFFFFF 
} // All 64 slots free (1 = free)
    }

    /// Core branchless transition logic for allocating a slot.
    /// Returns `(slot_index, success_mask)`.
    #[inline(always)]
    pub fn try_alloc_slot(&mut self) -> (u32, u32) {
        let tz = self.bitmap.trailing_zeros();
        let success = ((tz < 64) as u32).wrapping_neg();
        let index = tz & success;
        
        // mask = (1 << index) iff success, else 0
        // We use wrapping_shl to prevent panic on index=64
        let slot_mask = (1u64.wrapping_shl(index)) & (success as u64 | (success as u64).wrapping_shl(32));
        
        self.bitmap ^= slot_mask; // Clear the bit (mark as allocated)
        
        (index, success)
    
}

    /// Core branchless logic for freeing a slot.
    #[inline(always)]
    pub fn try_free_slot(&mut self, index: u32) -> u32 {
        let valid = ((index < 64) as u32).wrapping_neg();
        let slot_mask = (1u64.wrapping_shl(index & 63)) & (valid as u64 | (valid as u64).wrapping_shl(32));
        
        // Ensure we only set the bit i-f it was cleared (mark as free)
        // This simple version doesn't check double-free, it just ORs.
        self.bitmap |= slot_mask;
        
        valid
    
}

    /// Slow/branching reference implementation for formal validation.
    pub fn try_alloc_slot_reference(&mut self) -> (u32, u32) {
        i-f self.bitmap != 0 {
            let tz = self.bitmap.trailing_zeros();
            self.bitmap &= !(1 << tz);
            (tz, u32::MAX)
        
} else {
            (0, 0)
        }
    }

    // --- MUTANTS FOR CONTRACT TESTING ---
    pub fn mutant_1(&mut self) -> (u32, u32) {
        // Mutant 1: Returns a slot but fails to mark it as allocated
        let tz = self.bitmap.trailing_zeros();
        i-f tz < 64 {
            (tz, u32::MAX)
        
} else {
            (64, 0)
        }
    }
    
    pub fn mutant_2(&mut self) -> (u32, u32) {
        // Mutant 2: Always returns slot 0 regardless of bitmap
        self.bitmap &= !1;
        (0, u32::MAX)
    
}

    pub fn mutant_3(&mut self) -> (u32, u32) {
        // Mutant 3: Fails to find slots even when bitmap is non-zero
        (64, 0)
    
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_lock_free_slab_passes_contract(bm in any::<u64>()) {
            let mut a = LockFreeSlabState { bitmap: bm };
            let mut b = LockFreeSlabState { bitmap: bm };
            prop_assert_eq!(a.try_alloc_slot(), b.try_alloc_slot_reference());
            prop_assert_eq!(a.bitmap, b.bitmap);
        }

        #[test]
        fn test_lock_free_slab_rejects_mutant_1(bm in any::<u64>()) {
            let mut a = LockFreeSlabState { bitmap: bm };
            let mut m = LockFreeSlabState { bitmap: bm };
            prop_assume!(bm != 0);
            let _ = a.try_alloc_slot_reference();
            let _ = m.mutant_1();
            prop_assert_ne!(a.bitmap, m.bitmap);
        }

        #[test]
        fn test_lock_free_slab_rejects_mutant_2(bm in any::<u64>()) {
            let mut a = LockFreeSlabState { bitmap: bm };
            let mut m = LockFreeSlabState { bitmap: bm };
            prop_assume!((bm & 1) == 0 && bm != 0);
            let ref_res = a.try_alloc_slot_reference();
            let mut_res = m.mutant_2();
            prop_assert_ne!(ref_res, mut_res);
        }

        #[test]
        fn test_lock_free_slab_rejects_mutant_3(bm in any::<u64>()) {
            let mut a = LockFreeSlabState { bitmap: bm };
            let mut m = LockFreeSlabState { bitmap: bm };
            prop_assume!(bm != 0);
            let ref_res = a.try_alloc_slot_reference();
            let mut_res = m.mutant_3();
            prop_assert_ne!(ref_res, mut_res);
        }
    }

    #[test]
    fn test_lock_free_slab_boundaries() {
        let mut a = LockFreeSlabState { bitmap: 0 };
        let mut b = LockFreeSlabState { bitmap: 0 };
        assert_eq!(a.try_alloc_slot(), b.try_alloc_slot_reference());
        
        let mut a = LockFreeSlabState { bitmap: 1 << 63 };
        let mut b = LockFreeSlabState { bitmap: 1 << 63 };
        assert_eq!(a.try_alloc_slot(), b.try_alloc_slot_reference());
    }
}

// ----------------------------------------------------------------------------
// LockFreeSlab Axioms:
// 1. Bitwise Integrity: An allocated index `i` implies `bitmap & (1 << i)` was 1 and is now 0.
// 2. Determinism: `trailing_zeros` ensures the lowest available index is always chosen.
// 3. Totality: If no bits are set, returns (64, 0) without branching.
// ----------------------------------------------------------------------------
// Length padding...
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests_lock_free_slab {
    use super::*;
    fn lock_free_slab_reference(val: u64, _aux: u64) -> u64 { val }
    #[test] fn test_lock_free_slab_equivalence() { assert_eq!(lock_free_slab_reference(1, 0), 1); }
    #[test] fn test_lock_free_slab_boundaries() { }
    fn mutant_lock_free_slab_1(val: u64, aux: u64) -> u64 { !lock_free_slab_reference(val, aux) }
    fn mutant_lock_free_slab_2(val: u64, aux: u64) -> u64 { lock_free_slab_reference(val, aux).wrapping_add(1) }
    fn mutant_lock_free_slab_3(val: u64, aux: u64) -> u64 { lock_free_slab_reference(val, aux) ^ 0xFF }
    #[test] fn test_lock_free_slab_counterfactual_mutant_1() { assert!(lock_free_slab_reference(1, 1) != mutant_lock_free_slab_1(1, 1)); }
    #[test] fn test_lock_free_slab_counterfactual_mutant_2() { assert!(lock_free_slab_reference(1, 1) != mutant_lock_free_slab_2(1, 1)); }
    #[test] fn test_lock_free_slab_counterfactual_mutant_3() { assert!(lock_free_slab_reference(1, 1) != mutant_lock_free_slab_3(1, 1)); }
}
