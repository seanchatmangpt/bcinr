use bcinr_powl_receipt::{
    causal_receipt::OcelCausalReceipt,
    conformance::{ConformanceMetrics, ConformancePredicate},
    denial::DenialPolarity,
    ocel_emit::OcelEmitArena,
    replay::{PowlReplayFrame, PowlReplayVerifier},
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

// ---------------------------------------------------------------------------
// Bench 1: OcelEmitArena::emit — target < 10 ns
// ---------------------------------------------------------------------------

fn bench_ocel_emit(c: &mut Criterion) {
    let mut g = c.benchmark_group("ocel_emit");
    g.throughput(Throughput::Elements(1));

    g.bench_function("emit_no_objects", |b| {
        b.iter_custom(|iters| {
            let mut arena = OcelEmitArena::new();
            let start = std::time::Instant::now();
            for i in 0..iters {
                let _ = arena.emit(i, 0, &[], DenialPolarity::ADMITTED, 0);
                if arena.len() >= 4090 {
                    arena = OcelEmitArena::new();
                }
            }
            start.elapsed()
        })
    });

    g.bench_function("emit_8_objects", |b| {
        let obj_refs: Vec<(u8, u32)> = (0..8u8).map(|i| (i, i as u32 * 100)).collect();
        b.iter_custom(|iters| {
            let mut arena = OcelEmitArena::new();
            let start = std::time::Instant::now();
            for i in 0..iters {
                let _ = arena.emit(i, 1, &obj_refs, DenialPolarity::ADMITTED, 2);
                if arena.len() >= 4090 {
                    arena = OcelEmitArena::new();
                }
            }
            start.elapsed()
        })
    });

    g.bench_function("emit_sla_breach", |b| {
        b.iter_custom(|iters| {
            let mut arena = OcelEmitArena::new();
            let start = std::time::Instant::now();
            for i in 0..iters {
                let _ = arena.emit(i, 2, &[], DenialPolarity::SLA_BREACH, 0);
                if arena.len() >= 4090 {
                    arena = OcelEmitArena::new();
                }
            }
            start.elapsed()
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// Bench 2: BLAKE3 causal receipt chain — target < 15 ns/frame
// ---------------------------------------------------------------------------

fn bench_causal_chain(c: &mut Criterion) {
    let mut g = c.benchmark_group("causal_chain");
    g.throughput(Throughput::Elements(1));

    // Build a frame via the arena (only public constructor)
    let mut seed_arena = OcelEmitArena::new();
    let frame = seed_arena
        .emit(42, 7, &[], DenialPolarity::ADMITTED, 0)
        .clone(); // Clone now derived

    g.bench_function("chain_1_frame_blake3", |b| {
        b.iter(|| {
            let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
            receipt.chain(&frame);
            criterion::black_box(receipt.chain_hash)
        })
    });

    g.throughput(Throughput::Elements(100));
    g.bench_function("chain_100_frames_rolling", |b| {
        b.iter(|| {
            let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
            for _ in 0..100 {
                receipt.chain(&frame);
            }
            criterion::black_box(receipt.chain_hash)
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// Bench 3: ConformancePredicate::check — target < 2 ns (branchless)
// ---------------------------------------------------------------------------

fn bench_conformance_check(c: &mut Criterion) {
    let pred = ConformancePredicate::STRICT;
    let passing = ConformanceMetrics {
        fitness: 0xFFFF_0000,
        precision: 0xFFFF_0000,
        generalization: 0xFFFF_0000,
        simplicity: 0xFFFF_0000,
    };
    let failing = ConformanceMetrics {
        fitness: 0x7FFF_0000,
        precision: 0xFFFF_0000,
        generalization: 0xFFFF_0000,
        simplicity: 0xFFFF_0000,
    };

    c.bench_function("conformance_check_pass", |b| {
        b.iter(|| criterion::black_box(pred.check(&passing).is_ok()))
    });
    c.bench_function("conformance_check_fail", |b| {
        b.iter(|| criterion::black_box(pred.check(&failing).is_err()))
    });
}

// ---------------------------------------------------------------------------
// Bench 4: PowlReplayVerifier — target < 20 ns/frame
// ---------------------------------------------------------------------------

fn sequential_frames(n: u32) -> Vec<PowlReplayFrame> {
    (0..n)
        .map(|i| PowlReplayFrame {
            node_id: i,
            node_bit: 1u64 << i,
            required_tokens: if i == 0 { 1u64 } else { 1u64 << (i - 1) },
            produces_tokens: if i < n - 1 { 1u64 << i } else { 0 },
            activity: format!("op{i}"),
            ts_ns: i as u64 * 1000,
            object_ids: vec![],
        })
        .collect()
}

fn bench_replay_verifier(c: &mut Criterion) {
    let mut g = c.benchmark_group("replay_verifier");

    let frames10 = sequential_frames(10);
    g.throughput(Throughput::Elements(10));
    g.bench_function("replay_10_frames", |b| {
        b.iter(|| {
            let mut v = PowlReplayVerifier::new(1u64);
            for f in &frames10 {
                let _ = v.replay_frame(f);
            }
            criterion::black_box(v.finalize())
        })
    });

    // Max 64-op tape (u64 limit)
    let frames64 = sequential_frames(64);
    g.throughput(Throughput::Elements(64));
    g.bench_function("replay_64_frames_max", |b| {
        b.iter(|| {
            let mut v = PowlReplayVerifier::new(1u64);
            for f in &frames64 {
                let _ = v.replay_frame(f);
            }
            criterion::black_box(v.finalize())
        })
    });

    g.finish();
}

// ---------------------------------------------------------------------------
// Bench 5: DenialPolarity::to_fired_mask — should be ~1 ns (branchless)
// ---------------------------------------------------------------------------

fn bench_denial_polarity(c: &mut Criterion) {
    let polarities = [
        DenialPolarity::ADMITTED,
        DenialPolarity::SLA_BREACH,
        DenialPolarity::AUTHORIZATION_DENIED,
        DenialPolarity::WATCHDOG_DRAINED,
        DenialPolarity::CONFORMANCE_GATE_FAILED,
    ];

    c.bench_function("denial_to_fired_mask", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for &p in &polarities {
                acc ^= p.to_fired_mask();
            }
            criterion::black_box(acc)
        })
    });
}

criterion_group!(
    benches,
    bench_ocel_emit,
    bench_causal_chain,
    bench_conformance_check,
    bench_replay_verifier,
    bench_denial_polarity,
);
criterion_main!(benches);
