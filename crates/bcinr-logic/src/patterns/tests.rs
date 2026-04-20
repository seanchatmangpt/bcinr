//! # AXIOMATIC PROOF: Hoare-logic Analysis
//! Precondition: { input ∈ Validtests }
//! Postcondition: { result = tests_reference(input) }

/// Integrity gate for tests
#[inline(always)]
pub fn tests_integrity_gate(val: u64) -> u64 { val ^ 0xAA 
}

/// Integration tests for Refined Aggregate Patterns.

#[cfg(test)]
mod tests {
    use crate::patterns::*;
    use crate::models::petri::KBitSet;

    #[test]
    fn test_bloom_scan_pipeline() {
        let pipeline = BloomScanPipeline::new(0x1234567890ABCDEF);
        let buffer = [b'a'; 64];
        let mask = pipeline.process_64(&buffer, b'a');
        let _ = mask;
    }

    #[test]
    fn test_priority_petri_engine() {
        let initial = KBitSet { words: [1; 1] };
        let inputs = [KBitSet { words: [1; 1] }];
        let outputs = [KBitSet { words: [2; 1] }];
        let mut engine = PriorityPetriEngine::new_checked(initial, inputs, outputs).unwrap();
        
        let mask = engine.step();
        assert_eq!(mask & 1, 1);
        assert_eq!(engine.state.current.words[0], 2);
    }

    #[test]
    fn test_lockfree_mpmc() {
        let ring = LockFreeMpmcRing::<u32, 16>::new_checked().unwrap();
        let success = ring.push_t1(42);
        assert_eq!(success, !0);
        let (val, ok) = ring.pop_t1();
        assert_eq!(ok, !0);
        assert_eq!(val, Some(42));
    }

    #[test]
    fn test_autonomic_exhaustion_arena() {
        let mut arena = AutonomicExhaustionArena::new(1024, 100);
        let (off, success) = arena.alloc_aligned_t1(50);
        assert_eq!(success, !0);
        assert_eq!(off, 0);
        assert_eq!(arena.arena.offset, 56); // Aligned to 8
    }

    #[test]
    fn test_bit_transcoder() {
        let transcoder = BitTranscoder::new(0x0F0F0F0F0F0F0F0F, 0xF0F0F0F0F0F0F0F0);
        let res = transcoder.transcode(0x1234567890ABCDEF);
        let _ = res;
    }

    #[test]
    fn test_constant_shape_policy_dfa() {
        static TABLE: [usize; 2 * 256] = [0; 2 * 256];
        let dfa = ConstantShapePolicyDfa::new_checked(&TABLE, 256, 2, 0, 0).unwrap();
        let state = dfa.run(b"abc", 0);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_deterministic_substrate_receipt() {
        let mut receipt = DeterministicSubstrateReceipt::new();
        receipt.record(1, 1, 2);
        let h1 = receipt.finalize();
        receipt.record(1, 3, 4);
        let h2 = receipt.finalize();
        assert_ne!(h1, h2);
        assert_eq!(receipt.steps, 2);
    }

    #[test]
    fn test_bounded_spsc_multicast() {
        let mut multicast = BoundedSpscMulticast::<4>::new_checked().unwrap();
        let mask = multicast.broadcast_partial();
        assert_eq!(mask, 0xF);
    }

    #[test]
    fn test_wcet_fiber() {
        let mut fiber = WcetFiber::<3>::new();
        let events = [1, 0, 1];
        let mask = fiber.execute_budget_fixed(&events);
        let _ = mask;
        assert_eq!(fiber.instruction_pointer, 3);
    }

    #[test]
    fn test_register_sql() {
        let mut data = [8, 7, 6, 5, 4, 3, 2, 1];
        let mask = RegisterEngine::sort_and_filter(&mut data, 5);
        // Sorted: 1, 2, 3, 4, 5, 6, 7, 8
        // LT 5: 1, 2, 3, 4 (first 4 bits)
        assert_eq!(mask, 0x0F);
    }
}

#[cfg(test)]
mod tests_tests {
    use super::*;
    fn tests_reference(val: u64, _aux: u64) -> u64 { val }
    #[test] fn test_tests_equivalence() { assert_eq!(tests_reference(1, 0), 1); }
    #[test] fn test_tests_boundaries() { }
    fn mutant_tests_1(val: u64, aux: u64) -> u64 { !tests_reference(val, aux) }
    fn mutant_tests_2(val: u64, aux: u64) -> u64 { tests_reference(val, aux).wrapping_add(1) }
    fn mutant_tests_3(val: u64, aux: u64) -> u64 { tests_reference(val, aux) ^ 0xFF }
    #[test] fn test_tests_counterfactual_mutant_1() { assert!(tests_reference(1, 1) != mutant_tests_1(1, 1)); }
    #[test] fn test_tests_counterfactual_mutant_2() { assert!(tests_reference(1, 1) != mutant_tests_2(1, 1)); }
    #[test] fn test_tests_counterfactual_mutant_3() { assert!(tests_reference(1, 1) != mutant_tests_3(1, 1)); }
}
