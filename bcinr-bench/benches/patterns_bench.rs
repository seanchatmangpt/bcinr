use bcinr_logic::patterns::*;
use bcinr_logic::models::petri::KBitSet;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_bloom_scan_pipeline(c: &mut Criterion) {
    let pipeline = BloomScanPipeline::new(0x1234567890ABCDEF);
    let buffer = [b'a'; 64];
    c.bench_function("BloomScanPipeline", |b| b.iter(|| {
        pipeline.process_64(black_box(&buffer), black_box(b'a'))
    }));
}

fn bench_priority_petri_engine(c: &mut Criterion) {
    let initial = KBitSet { words: [1; 1] };
    let inputs = [KBitSet { words: [1; 1] }; 16];
    let outputs = [KBitSet { words: [2; 1] }; 16];
    let mut engine = PriorityPetriEngine::new_checked(initial, inputs, outputs).unwrap();
    c.bench_function("PriorityPetriEngine", |b| b.iter(|| {
        engine.step()
    }));
}

fn bench_lockfree_mpmc(c: &mut Criterion) {
    let ring = LockFreeMpmcRing::<u32, 16>::new_checked().unwrap();
    c.bench_function("LockFreeMpmcRing_Push", |b| b.iter(|| {
        ring.push_t1(black_box(42))
    }));
    c.bench_function("LockFreeMpmcRing_Pop", |b| b.iter(|| {
        ring.pop_t1()
    }));
}

fn bench_autonomic_exhaustion_arena(c: &mut Criterion) {
    let mut arena = AutonomicExhaustionArena::new(1024, 100);
    c.bench_function("AutonomicExhaustionArena", |b| b.iter(|| {
        arena.alloc_aligned_t1(black_box(50))
    }));
}

fn bench_bit_transcoder(c: &mut Criterion) {
    let transcoder = BitTranscoder::new(0x0F0F0F0F0F0F0F0F, 0xF0F0F0F0F0F0F0F0);
    c.bench_function("BitTranscoder", |b| b.iter(|| {
        transcoder.transcode(black_box(0x1234567890ABCDEF))
    }));
}

fn bench_constant_shape_policy_dfa(c: &mut Criterion) {
    static TABLE: [usize; 2 * 256] = [0; 2 * 256];
    let dfa = ConstantShapePolicyDfa::new_checked(&TABLE, 256, 2, 0, 0).unwrap();
    c.bench_function("ConstantShapePolicyDfa", |b| b.iter(|| {
        dfa.step(black_box(0), black_box(b'a'))
    }));
}

fn bench_deterministic_substrate_receipt(c: &mut Criterion) {
    let mut receipt = DeterministicSubstrateReceipt::new();
    c.bench_function("DeterministicSubstrateReceipt", |b| b.iter(|| {
        receipt.record(black_box(1), black_box(1), black_box(2));
        receipt.finalize()
    }));
}

fn bench_bounded_spsc_multicast(c: &mut Criterion) {
    let mut multicast = BoundedSpscMulticast::<4>::new_checked().unwrap();
    c.bench_function("BoundedSpscMulticast", |b| b.iter(|| {
        multicast.broadcast_partial()
    }));
}

fn bench_wcet_fiber(c: &mut Criterion) {
    let mut fiber = WcetFiber::<3>::new();
    let events = [1, 2, 3];
    c.bench_function("WcetFiber", |b| b.iter(|| {
        fiber.execute_budget_fixed(black_box(&events))
    }));
}

fn bench_register_sql(c: &mut Criterion) {
    let mut data = [8, 7, 6, 5, 4, 3, 2, 1];
    c.bench_function("RegisterEngine", |b| b.iter(|| {
        RegisterEngine::sort_and_filter(black_box(&mut data), black_box(5))
    }));
}

fn bench_matrix_lru(c: &mut Criterion) {
    let mut lru = MatrixLru::<4>::new();
    c.bench_function("MatrixLru", |b| b.iter(|| {
        lru.access(black_box(0));
        lru.find_lru()
    }));
}

fn bench_chacha_sponge(c: &mut Criterion) {
    let mut sponge = ChaChaSponge::new([0; 4]);
    c.bench_function("ChaChaSponge", |b| b.iter(|| {
        sponge.absorb(black_box(0x1234));
        sponge.squeeze()
    }));
}

fn bench_swar_quotient(c: &mut Criterion) {
    let mut q = SwarQuotientFilter::<4>::new();
    c.bench_function("SwarQuotientFilter", |b| b.iter(|| {
        q.insert(black_box(0), black_box(0xAB));
        q.contains(black_box(0), black_box(0xAB))
    }));
}

fn bench_bitonic_pq(c: &mut Criterion) {
    let mut pq = BitonicPriorityQueue8::new();
    c.bench_function("BitonicPriorityQueue8", |b| b.iter(|| {
        pq.push(black_box(10));
        pq.pop()
    }));
}

fn bench_hazard_shield(c: &mut Criterion) {
    let shield = HazardShield::<4>::new();
    c.bench_function("HazardShield", |b| b.iter(|| {
        shield.protect(black_box(0), black_box(0xDEAD));
        shield.is_shielded(black_box(0xDEAD))
    }));
}

fn bench_radix_trie(c: &mut Criterion) {
    let mut node = RadixTrieNode::<8>::new();
    node.bitmap[1] |= 1u64.wrapping_shl(97 - 64);
    node.children[0] = 42;
    c.bench_function("RadixTrieNode", |b| b.iter(|| {
        node.lookup(black_box(b'a'))
    }));
}

fn bench_consensus_bft(c: &mut Criterion) {
    let mut bft = FixedConsensus::<2>::new();
    c.bench_function("FixedConsensus", |b| b.iter(|| {
        bft.vote(black_box(0));
        bft.is_reached()
    }));
}

fn bench_time_wheel(c: &mut Criterion) {
    let mut wheel = TimeWheel::<4>::new();
    c.bench_function("TimeWheel", |b| b.iter(|| {
        wheel.schedule(black_box(1), black_box(0));
        wheel.tick()
    }));
}

criterion_group!(benches,
    bench_bloom_scan_pipeline,
    bench_priority_petri_engine,
    bench_lockfree_mpmc,
    bench_autonomic_exhaustion_arena,
    bench_bit_transcoder,
    bench_constant_shape_policy_dfa,
    bench_deterministic_substrate_receipt,
    bench_bounded_spsc_multicast,
    bench_wcet_fiber,
    bench_register_sql,
    bench_matrix_lru,
    bench_chacha_sponge,
    bench_swar_quotient,
    bench_bitonic_pq,
    bench_hazard_shield,
    bench_radix_trie,
    bench_consensus_bft,
    bench_time_wheel
);

criterion_main!(benches);
