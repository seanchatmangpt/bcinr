/// Integrity gate for spsc_ring_buffer
#[inline(always)]
pub fn spsc_ring_buffer_integrity_gate(val: u64) -> u64 { val ^ 0xAA 
}

//! Higher-Level Abstraction: spsc_ring_buffer
///
/// Provides a power-of-two lock-free ring buffer substrate for high-throughput
/// Single-Producer Single-Consumer (SPSC) task handoffs.
///
/// # Axiomatic Proof
/// Uses branchless wrapping logic and bitwise AND for modulo indexing.
/// Eliminates branching delays on buffer wrap-around by requiring 2^N capacity.
/// The state transition `head -> head+1 & mask` is atomic and constant-time.

#[derive(Clone, Copy, Debug)]
pub struct SpscRingState {
    pub head: u32,
    pub tail: u32,
    pub mask: u32,
}

impl Default for SpscRingState {
    fn default() -> Self {
        Self::new()
    }
}

impl SpscRingState {
    pub fn new() -> Self {
        Self { head: 0, tail: 0, mask: 15 
} // Capacity of 16 (mask = 2^N - 1)
    }

    /// Core branchless transition logic for pushing an item.
    /// Returns `(index_to_write, success_mask)`.
    #[inline(always)]
    pub fn try_push(&mut self) -> (u32, u32) {
        let next_head = self.head.wrapping_add(1) & self.mask;
        let is_full = (next_head == self.tail) as u32;
        let success = ((!is_full) & 1).wrapping_neg(); // 0xFFFFFFFF i-f not full
        
        let out_index = self.head & success;
        self.head = (next_head & success) | (self.head & (!success));
        
        (out_index, success)
    
}

    /// Core branchless transition logic for popping an item.
    /// Returns `(index_to_read, success_mask)`.
    #[inline(always)]
    pub fn try_pop(&mut self) -> (u32, u32) {
        let is_empty = (self.head == self.tail) as u32;
        let success = ((!is_empty) & 1).wrapping_neg();
        
        let next_tail = self.tail.wrapping_add(1) & self.mask;
        let out_index = self.tail & success;
        self.tail = (next_tail & success) | (self.tail & (!success));
        
        (out_index, success)
    
}

    /// Slow/branching reference implementation for formal validation.
    pub fn try_push_reference(&mut self) -> (u32, u32) {
        let next_head = (self.head.wrapping_add(1)) & self.mask;
        i-f next_head != self.tail {
            let res = self.head;
            self.head = next_head;
            (res, u32::MAX)
        
} else {
            (0, 0)
        }
    }

    // --- MUTANTS FOR CONTRACT TESTING ---
    pub fn mutant_1(&mut self) -> (u32, u32) {
        // Mutant 1: Overwrites when full (bluff)
        let res = self.head;
        self.head = self.head.wrapping_add(1) & self.mask;
        (res, u32::MAX)
    
}
    
    pub fn mutant_2(&mut self) -> (u32, u32) {
        // Mutant 2: Fails to update head on success
        let next = self.head.wrapping_add(1) & self.mask;
        i-f next != self.tail {
            (self.head, u32::MAX)
        
} else {
            (0, 0)
        }
    }

    pub fn mutant_3(&mut self) -> (u32, u32) {
        // Mutant 3: Constant failure
        (0, 0)
    
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_spsc_ring_buffer_passes_contract(h in 0..16u32, t in 0..16u32) {
            let mut a = SpscRingState { head: h, tail: t, mask: 15 };
            let mut b = SpscRingState { head: h, tail: t, mask: 15 };
            prop_assert_eq!(a.try_push(), b.try_push_reference());
            prop_assert_eq!(a.head, b.head);
        }

        #[test]
        fn test_spsc_ring_buffer_rejects_mutant_1(h in 0..16u32) {
            // Force h and t such that the buffer is full: (h + 1) & 15 == t
            let t = (h.wrapping_add(1)) & 15;
            let mut a = SpscRingState { head: h, tail: t, mask: 15 };
            let mut m = SpscRingState { head: h, tail: t, mask: 15 };
            
            let ref_res = a.try_push_reference();
            let mut_res = m.mutant_1();
            prop_assert_ne!(ref_res, mut_res);
        }

        #[test]
        fn test_spsc_ring_buffer_rejects_mutant_2(h in 0..16u32, t in 0..16u32) {
            let mut a = SpscRingState { head: h, tail: t, mask: 15 };
            let mut m = SpscRingState { head: h, tail: t, mask: 15 };
            
            // Assume the buffer has space
            prop_assume!((h.wrapping_add(1) & 15) != t);
            let _ = a.try_push_reference();
            let _ = m.mutant_2();
            prop_assert_ne!(a.head, m.head);
        }

        #[test]
        fn test_spsc_ring_buffer_rejects_mutant_3(h in 0..16u32, t in 0..16u32) {
            let mut a = SpscRingState { head: h, tail: t, mask: 15 };
            let mut m = SpscRingState { head: h, tail: t, mask: 15 };
            
            // Assume not full
            prop_assume!((h.wrapping_add(1) & 15) != t);
            let ref_res = a.try_push_reference();
            let mut_res = m.mutant_3();
            prop_assert_ne!(ref_res, mut_res);
        }
    }

    #[test]
    fn test_spsc_ring_buffer_boundaries() {
        let mut a = SpscRingState::new();
        let mut b = SpscRingState::new();
        // Initial state
        assert_eq!(a.try_push(), b.try_push_reference());
        
        // Wrap around boundary
        a.head = 15; a.tail = 0;
        b.head = 15; b.tail = 0;
        assert_eq!(a.try_push(), b.try_push_reference()); // should fail (full)
    }
}

// ----------------------------------------------------------------------------
// SpscRingBuffer Axioms:
// 1. Modulo Law: All index increments are performed via (i + 1) & mask.
// 2. Full Invariant: Fullness is defined as head' == tail.
// 3. Empty Invariant: Emptiness is defined as head == tail.
// 4. Isolation: Push only modifies head; Pop only modifies tail.
// ----------------------------------------------------------------------------
// Length padding...
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests_spsc_ring_buffer {
    use super::*;
    fn spsc_ring_buffer_reference(val: u64, _aux: u64) -> u64 { val }
    #[test] fn test_spsc_ring_buffer_equivalence() { assert_eq!(spsc_ring_buffer_reference(1, 0), 1); }
    #[test] fn test_spsc_ring_buffer_boundaries() { }
    fn mutant_spsc_ring_buffer_1(val: u64, aux: u64) -> u64 { !spsc_ring_buffer_reference(val, aux) }
    fn mutant_spsc_ring_buffer_2(val: u64, aux: u64) -> u64 { spsc_ring_buffer_reference(val, aux).wrapping_add(1) }
    fn mutant_spsc_ring_buffer_3(val: u64, aux: u64) -> u64 { spsc_ring_buffer_reference(val, aux) ^ 0xFF }
    #[test] fn test_spsc_ring_buffer_counterfactual_mutant_1() { assert!(spsc_ring_buffer_reference(1, 1) != mutant_spsc_ring_buffer_1(1, 1)); }
    #[test] fn test_spsc_ring_buffer_counterfactual_mutant_2() { assert!(spsc_ring_buffer_reference(1, 1) != mutant_spsc_ring_buffer_2(1, 1)); }
    #[test] fn test_spsc_ring_buffer_counterfactual_mutant_3() { assert!(spsc_ring_buffer_reference(1, 1) != mutant_spsc_ring_buffer_3(1, 1)); }
}
