// Universe1 register-scale substrate benchmarks.
//
// Tier budgets:
//   T0 (≤ 8 ns):   u1_64_fire, u1_receipt_mix
//   T1 (≤ 200 ns): u1_512_fire_cell, u1_512_conformance_distance, u1_4096_delta
//   T2 (≤ 5 µs):   u1_4096_conformance

use bcinr_logic::patterns::universe1::{
    new_u1_receipt, receipt_mix_u1_transition,
    transition::{compute_domain_delta, fire_cell_branchless},
    U1_512, U1_64, U1Coord, U1_4096,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_u8_cell_fire(c: &mut Criterion) {
    c.bench_function("universe1/u1_64_fire", |b| {
        b.iter(|| {
            let (next, fired) = fire_cell_branchless(
                black_box(0b0011u64),
                black_box(0b0011u64),
                black_box(0b1100u64),
            );
            (next, fired)
        })
    });
}

fn bench_u8_cell_wrapper_fire(c: &mut Criterion) {
    c.bench_function("universe1/U1_64_fire", |b| {
        let cell = U1_64::new(0b0011);
        b.iter(|| cell.fire(black_box(0b0011u64), black_box(0b1100u64)))
    });
}

fn bench_u8_block_fire_cell(c: &mut Criterion) {
    let mut block = U1_512::new([0b0011, 0, 0, 0, 0, 0, 0, 0]);
    c.bench_function("universe1/U1_512_fire_cell", |b| {
        b.iter(|| {
            block = U1_512::new([0b0011, 0, 0, 0, 0, 0, 0, 0]);
            block.fire_cell(
                black_box(0),
                black_box(0b0011u64),
                black_box(0b1100u64),
            )
        })
    });
}

fn bench_u8_block_conformance(c: &mut Criterion) {
    let a = U1_512::new([0xFFu64; 8]);
    let e = U1_512::new([0x0Fu64; 8]);
    c.bench_function("universe1/U1_512_conformance_distance", |b| {
        b.iter(|| a.conformance_distance(black_box(&e)))
    });
}

fn bench_u8_domain_delta(c: &mut Criterion) {
    let a = [0xFFu64; 64];
    let x = [0x0Fu64; 64];
    c.bench_function("universe1/U1_4096_delta", |b| {
        b.iter(|| compute_domain_delta(black_box(&a), black_box(&x)))
    });
}

fn bench_u8_domain_conformance(c: &mut Criterion) {
    let a = U1_4096::new([0xFFu64; 64]);
    let e = U1_4096::new([0x0Fu64; 64]);
    c.bench_function("universe1/U1_4096_conformance_distance", |b| {
        b.iter(|| a.conformance_distance(black_box(&e)))
    });
}

fn bench_u1_receipt_mix(c: &mut Criterion) {
    let r = new_u1_receipt();
    let coord = U1Coord::new_const(3, 5, 7);
    c.bench_function("universe1/receipt_mix_u1_transition", |b| {
        b.iter(|| {
            receipt_mix_u1_transition(
                black_box(r),
                black_box(coord),
                black_box(1u64),
                black_box(!0u64),
                black_box(0xDEADBEEFu64),
            )
        })
    });
}

criterion_group!(
    u8_benches,
    bench_u8_cell_fire,
    bench_u8_cell_wrapper_fire,
    bench_u8_block_fire_cell,
    bench_u8_block_conformance,
    bench_u8_domain_delta,
    bench_u8_domain_conformance,
    bench_u1_receipt_mix,
);
criterion_main!(u8_benches);
