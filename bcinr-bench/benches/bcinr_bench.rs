use bcinr_core::logic::fix::add_sat;
use bcinr_core::logic::int::popcount_u64;
use bcinr_core::logic::mask::select_u32;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_mask(c: &mut Criterion) {
    c.bench_function("mask::select_u32", |b| {
        b.iter(|| black_box(select_u32(black_box(0xFFFFFFFF), black_box(10), black_box(20))))
    });
}

fn bench_int(c: &mut Criterion) {
    c.bench_function("int::popcount_u64", |b| {
        b.iter(|| black_box(popcount_u64(black_box(0x123456789ABCDEF0))))
    });
}

fn bench_fix(c: &mut Criterion) {
    c.bench_function("fix::add_sat", |b| {
        b.iter(|| black_box(add_sat(black_box(200), black_box(100))))
    });
}

criterion_group!(benches, bench_mask, bench_int, bench_fix);
criterion_main!(benches);
