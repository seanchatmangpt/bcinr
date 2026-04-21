use bcinr_core::logic::fix::add_sat;
use bcinr_core::logic::int::popcount_u64;
use bcinr_core::logic::mask::select_u32;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_mask(c: &mut Criterion) {
    c.bench_function("mask::select_u32", |b| {
        b.iter(|| select_u32(0xFFFFFFFF, 10, 20))
    });
}

fn bench_int(c: &mut Criterion) {
    c.bench_function("int::popcount_u64", |b| {
        b.iter(|| popcount_u64(0x123456789ABCDEF0))
    });
}

fn bench_fix(c: &mut Criterion) {
    c.bench_function("fix::add_sat", |b| b.iter(|| add_sat(200, 100)));
}

criterion_group!(benches, bench_mask, bench_int, bench_fix);
criterion_main!(benches);
