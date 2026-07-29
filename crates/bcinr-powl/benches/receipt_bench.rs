//! Receipt-subsystem benchmarks (Divan).
//!
//! Ported from Criterion when `bcinr-powl-receipt` was folded into this crate:
//! this was the workspace's last Criterion bench, and `powl_quick_bench.rs`
//! records why this crate standardised on Divan (Criterion's per-benchmark
//! overhead is what broke the suite's wall-clock budget).
//!
//! Stated targets, carried over from the Criterion version:
//!
//! - `OcelEmitArena::emit` — < 10 ns
//! - BLAKE3 causal receipt chain — < 15 ns/frame
//! - `ConformancePredicate::check` — < 2 ns (branchless)
//! - `PowlReplayVerifier` — < 20 ns/frame
//! - `DenialPolarity::to_fired_mask` — ~1 ns (branchless)

use bcinr_powl::receipt::{
    causal_receipt::OcelCausalReceipt,
    conformance::{ConformanceMetrics, ConformancePredicate},
    denial::DenialPolarity,
    ocel_emit::OcelEmitArena,
    replay::{PowlReplayFrame, PowlReplayVerifier},
};
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Bench 1: OcelEmitArena::emit — target < 10 ns
// ---------------------------------------------------------------------------

/// The arena is fixed-capacity (4096) and zero-initialised, so constructing a
/// fresh one per iteration makes `emit` pay first-touch page-fault cost that
/// the steady-state operation does not — that reads ~19 ns and is measuring
/// allocation, not `emit`.
///
/// These mirror the Criterion original's `iter_custom` instead: one arena is
/// reused across iterations and recycled only on reaching capacity, so the
/// construction cost amortises to nothing and what is timed is a warm `emit`.
#[divan::bench(counters = [divan::counter::ItemsCount::new(1usize)])]
fn emit_no_objects(bencher: Bencher) {
    let mut arena = OcelEmitArena::new();
    bencher.bench_local(move || {
        if arena.len() >= 4090 {
            arena = OcelEmitArena::new();
        }
        let _ = arena.emit(0, 0, &[], DenialPolarity::ADMITTED, 0);
        black_box(arena.len())
    });
}

#[divan::bench(counters = [divan::counter::ItemsCount::new(1usize)])]
fn emit_8_objects(bencher: Bencher) {
    let obj_refs: Vec<(u8, u32)> = (0..8u8).map(|i| (i, i as u32 * 100)).collect();
    let mut arena = OcelEmitArena::new();
    bencher.bench_local(move || {
        if arena.len() >= 4090 {
            arena = OcelEmitArena::new();
        }
        let _ = arena.emit(0, 1, &obj_refs, DenialPolarity::ADMITTED, 2);
        black_box(arena.len())
    });
}

#[divan::bench(counters = [divan::counter::ItemsCount::new(1usize)])]
fn emit_sla_breach(bencher: Bencher) {
    let mut arena = OcelEmitArena::new();
    bencher.bench_local(move || {
        if arena.len() >= 4090 {
            arena = OcelEmitArena::new();
        }
        let _ = arena.emit(0, 2, &[], DenialPolarity::SLA_BREACH, 0);
        black_box(arena.len())
    });
}

// ---------------------------------------------------------------------------
// Bench 2: BLAKE3 causal receipt chain — target < 15 ns/frame
// ---------------------------------------------------------------------------

#[divan::bench(counters = [divan::counter::ItemsCount::new(1usize)])]
fn chain_1_frame_blake3(bencher: Bencher) {
    let mut seed_arena = OcelEmitArena::new();
    let frame = seed_arena
        .emit(42, 7, &[], DenialPolarity::ADMITTED, 0)
        .clone();

    bencher.bench_local(|| {
        let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
        receipt.chain(&frame);
        black_box(receipt.chain_hash)
    });
}

#[divan::bench(counters = [divan::counter::ItemsCount::new(100usize)])]
fn chain_100_frames_rolling(bencher: Bencher) {
    let mut seed_arena = OcelEmitArena::new();
    let frame = seed_arena
        .emit(42, 7, &[], DenialPolarity::ADMITTED, 0)
        .clone();

    bencher.bench_local(|| {
        let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
        for _ in 0..100 {
            receipt.chain(&frame);
        }
        black_box(receipt.chain_hash)
    });
}

// ---------------------------------------------------------------------------
// Bench 3: ConformancePredicate::check — target < 2 ns (branchless)
// ---------------------------------------------------------------------------

const PASSING: ConformanceMetrics = ConformanceMetrics {
    fitness: 0xFFFF_0000,
    precision: 0xFFFF_0000,
    generalization: 0xFFFF_0000,
    simplicity: 0xFFFF_0000,
};

const FAILING: ConformanceMetrics = ConformanceMetrics {
    fitness: 0x7FFF_0000,
    precision: 0xFFFF_0000,
    generalization: 0xFFFF_0000,
    simplicity: 0xFFFF_0000,
};

#[divan::bench]
fn conformance_check_pass() -> bool {
    black_box(
        ConformancePredicate::STRICT
            .check(black_box(&PASSING))
            .is_ok(),
    )
}

#[divan::bench]
fn conformance_check_fail() -> bool {
    black_box(
        ConformancePredicate::STRICT
            .check(black_box(&FAILING))
            .is_err(),
    )
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

#[divan::bench(counters = [divan::counter::ItemsCount::new(10usize)])]
fn replay_10_frames(bencher: Bencher) {
    let frames = sequential_frames(10);
    bencher.bench_local(|| {
        let mut v = PowlReplayVerifier::new(1u64);
        for f in &frames {
            let _ = v.replay_frame(f);
        }
        black_box(v.finalize())
    });
}

/// 64 is the hard ceiling — the token mask is a `u64`.
#[divan::bench(counters = [divan::counter::ItemsCount::new(64usize)])]
fn replay_64_frames_max(bencher: Bencher) {
    let frames = sequential_frames(64);
    bencher.bench_local(|| {
        let mut v = PowlReplayVerifier::new(1u64);
        for f in &frames {
            let _ = v.replay_frame(f);
        }
        black_box(v.finalize())
    });
}

// ---------------------------------------------------------------------------
// Bench 5: DenialPolarity::to_fired_mask — should be ~1 ns (branchless)
// ---------------------------------------------------------------------------

#[divan::bench]
fn denial_to_fired_mask() -> u64 {
    let polarities = [
        DenialPolarity::ADMITTED,
        DenialPolarity::SLA_BREACH,
        DenialPolarity::AUTHORIZATION_DENIED,
        DenialPolarity::WATCHDOG_DRAINED,
        DenialPolarity::CONFORMANCE_GATE_FAILED,
    ];
    let mut acc = 0u64;
    for &p in black_box(&polarities) {
        acc ^= p.to_fired_mask();
    }
    black_box(acc)
}
