use criterion::{criterion_group, criterion_main, Criterion};
use bcinr::logic::mask::select_u32;
use bcinr::logic::int::popcount_u64;
use bcinr::logic::fix::add_sat_u8;

fn bench_mask(c: &mut Criterion) {
    c.bench_function("mask::select_u32", |b| b.iter(|| select_u32(0xFFFFFFFF, 10, 20)));
}

fn bench_int(c: &mut Criterion) {
    c.bench_function("int::popcount_u64", |b| b.iter(|| popcount_u64(0x123456789ABCDEF0)));
}

fn bench_fix(c: &mut Criterion) {
    c.bench_function("fix::add_sat_u8", |b| b.iter(|| add_sat_u8(200, 100)));
}

criterion_group!(benches, bench_mask, bench_int, bench_fix);
criterion_main!(benches);
