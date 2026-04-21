//! Integration tests for Refined Aggregate Patterns.

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
        let events = [1, 2, 3];
        let mask = fiber.execute_budget_fixed(&events);
        let _ = mask;
        assert_eq!(fiber.instruction_pointer, 3);
    }

    #[test]
    fn test_register_sql() {
        let mut data = [8, 7, 6, 5, 4, 3, 2, 1];
        let mask = RegisterEngine::sort_and_filter(&mut data, 5);
        assert_eq!(mask, 0x0F);
    }

    // Advanced Abstraction Tests
    #[test]
    fn test_matrix_lru() {
        let mut lru = MatrixLru::<4>::new();
        lru.access(0);
        lru.access(1);
        assert_eq!(lru.find_lru(), 2);
    }

    #[test]
    fn test_chacha_sponge() {
        let mut sponge = ChaChaSponge::new([0; 4]);
        sponge.absorb(0x1234);
        let h1 = sponge.squeeze();
        assert_ne!(h1, 0);
    }

    #[test]
    fn test_swar_quotient() {
        let mut q = SwarQuotientFilter::<4>::new();
        assert!(q.insert(0, 0xAB));
        assert!(q.contains(0, 0xAB));
    }

    #[test]
    fn test_bitonic_pq() {
        let mut pq = BitonicPriorityQueue8::new();
        pq.push(10);
        pq.push(5);
        let (v, _) = pq.pop();
        assert_eq!(v, 5);
    }

    #[test]
    fn test_hazard_shield() {
        let shield = HazardShield::<4>::new();
        shield.protect(0, 0xDEAD);
        assert_ne!(shield.is_shielded(0xDEAD), 0);
    }

    #[test]
    fn test_radix_trie() {
        let mut node = RadixTrieNode::<8>::new();
        // b'a' is 97. Index into bitmap[1]
        node.bitmap[1] |= 1u64.wrapping_shl(97 - 64);
        node.children[0] = 42;
        let (idx, _) = node.lookup(b'a');
        assert_eq!(idx, 42);
    }

    #[test]
    fn test_consensus_bft() {
        let mut bft = FixedConsensus::<2>::new();
        bft.vote(0);
        assert_eq!(bft.is_reached(), 0);
        bft.vote(1);
        assert_ne!(bft.is_reached(), 0);
    }

    #[test]
    fn test_time_wheel() {
        let mut wheel = TimeWheel::<4>::new();
        wheel.schedule(1, 0);
        assert_eq!(wheel.tick(), 0);
        assert_eq!(wheel.tick(), 1);
    }

    #[test]
    fn test_universe64() {
        use crate::patterns::universe64::*;
        let mut block = UniverseBlock::new();
        let coord = UCoord::new(1, 2, 3);
        
        let cell_idx = coord.word_index();
        block.set_cell(cell_idx, 42);
        assert_eq!(block.get_cell(cell_idx), 42);
        
        let (next_marking, success) = cell_try_fire(42, 2, 8);
        assert_eq!(success, !0); // 42 & 2 == 2, so success = !0
        assert_eq!(next_marking, 40); // (42 & !2) | 8 = 40 | 8 = 40
        
        let receipt = receipt_mix_transition(0, coord, success);
        assert_ne!(receipt.value(), 0);
    }

    // ---- Universe1 integration tests ----

    #[test]
    fn test_u1_cell_fire() {
        let c = U1_64::new(0b0011);
        let (c2, fired) = c.fire(0b0011, 0b1100);
        assert_eq!(fired, !0u64);
        assert_eq!(c2.get(), 0b1100);

        let (c3, fired2) = c.fire(0b1000, 0b0001);
        assert_eq!(fired2, 0);
        assert_eq!(c3.get(), 0b0011);
    }

    #[test]
    fn test_u1_block_conformance() {
        let a = U1_512::new([0xFF, 0xF0, 0, 0, 0, 0, 0, 0]);
        let b = U1_512::new([0x0F, 0xF0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.conformance_distance(&b), 4);
        assert_eq!(b.conformance_distance(&b), 0);
    }

    #[test]
    fn test_u1_domain_delta() {
        let a = U1_4096::new([0xFFu64; 64]);
        let b = U1_4096::new([0x0Fu64; 64]);
        let d = a.delta(&b);
        assert_eq!(d[0], 0xF0);
        assert_eq!(d[63], 0xF0);
        assert_eq!(a.conformance_distance(&b), 64 * 4);
    }

    #[test]
    fn test_u1_receipt_mix() {
        let r0 = new_u1_receipt();
        let c = U1Coord::new_const(1, 2, 3);
        let r1 = receipt_mix_u1_transition(r0, c, 0, !0u64, 0xDEAD);
        assert_ne!(r0, r1);

        // Deterministic: same inputs give same output.
        let r1b = receipt_mix_u1_transition(r0, c, 0, !0u64, 0xDEAD);
        assert_eq!(r1, r1b);
    }

    #[test]
    fn test_u1_coord_roundtrip() {
        for d in 0u8..8 {
            for c in 0u8..8 {
                for p in 0u8..8 {
                    let coord = U1Coord::new_const(d, c, p);
                    assert_eq!(coord.domain(), d);
                    assert_eq!(coord.cell(), c);
                    assert_eq!(coord.place(), p);
                    assert_eq!(coord.word_index(), (d as usize) * 8 + (c as usize));
                }
            }
        }
        assert!(U1Coord::try_new(8, 0, 0).is_none());
    }

    #[test]
    fn test_u1_cell_law_violation() {
        let c = U1_64::new(0b1111);
        assert!(!c.is_admissible(0b1010));
        assert_eq!(c.law_violation(0b1010), 0b0101);
        assert!(U1_64::new(0b1010).is_admissible(0b1111));
    }

    #[test]
    fn test_u1_cross_tier_alignment() {
        use crate::patterns::universe1::constants::*;
        // 64 U1_4096 blocks (64 × 512 B = 32 KiB) ≡ one U64 UniverseBlock.
        assert_eq!(U1_4096_BYTES, 512);
        assert_eq!(U1_4096_BYTES * 64, 32768);
        // One U1_4096 == one Universe64 "row" (64 words).
        assert_eq!(U1_4096_WORDS, 64);
        // U1_512 is exactly one L1 cache line.
        assert_eq!(U1_512_BYTES, 64);
        // U1_64 is exactly one u64 register.
        assert_eq!(U1_64_BYTES, 8);
    }
}
