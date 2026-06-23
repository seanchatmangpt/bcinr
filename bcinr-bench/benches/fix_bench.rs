use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Import fix algorithms and saturating arithmetic
use bcinr_logic::algorithms::add_sat_i32::add_sat_i32;
use bcinr_logic::algorithms::sub_sat_i32::sub_sat_i32;
use bcinr_logic::algorithms::mul_sat_i32::mul_sat_i32;
use bcinr_logic::algorithms::mul_sat_u64::mul_sat_u64;
use bcinr_logic::algorithms::div_sat_u64::div_sat_u64;

fn bench_add_sat_i32(c: &mut Criterion) {
    c.bench_function("add_sat_i32_avg", |b| {
        b.iter(|| add_sat_i32(black_box(42u64), black_box(1337u64)))
    });
    c.bench_function("add_sat_i32_min", |b| {
        b.iter(|| add_sat_i32(black_box(0u64), black_box(0u64)))
    });
    c.bench_function("add_sat_i32_max", |b| {
        b.iter(|| add_sat_i32(black_box(i32::MAX as u64), black_box(i32::MAX as u64)))
    });
}

fn bench_sub_sat_i32(c: &mut Criterion) {
    c.bench_function("sub_sat_i32_avg", |b| {
        b.iter(|| sub_sat_i32(black_box(1337u64), black_box(42u64)))
    });
    c.bench_function("sub_sat_i32_min", |b| {
        b.iter(|| sub_sat_i32(black_box(0u64), black_box(0u64)))
    });
    c.bench_function("sub_sat_i32_max", |b| {
        b.iter(|| sub_sat_i32(black_box(i32::MAX as u64), black_box(i32::MIN as u64)))
    });
}

fn bench_mul_sat_i32(c: &mut Criterion) {
    c.bench_function("mul_sat_i32_avg", |b| {
        b.iter(|| mul_sat_i32(black_box(42u64), black_box(1337u64)))
    });
    c.bench_function("mul_sat_i32_min", |b| {
        b.iter(|| mul_sat_i32(black_box(0u64), black_box(0u64)))
    });
    c.bench_function("mul_sat_i32_max", |b| {
        b.iter(|| mul_sat_i32(black_box(i32::MAX as u64), black_box(i32::MAX as u64)))
    });
}

fn bench_mul_sat_u64(c: &mut Criterion) {
    c.bench_function("mul_sat_u64_avg", |b| {
        b.iter(|| mul_sat_u64(black_box(42u64), black_box(1337u64)))
    });
    c.bench_function("mul_sat_u64_min", |b| {
        b.iter(|| mul_sat_u64(black_box(0u64), black_box(0u64)))
    });
    c.bench_function("mul_sat_u64_max", |b| {
        b.iter(|| mul_sat_u64(black_box(u64::MAX), black_box(u64::MAX)))
    });
}

fn bench_div_sat_u64(c: &mut Criterion) {
    c.bench_function("div_sat_u64_avg", |b| {
        b.iter(|| div_sat_u64(black_box(1337u64), black_box(42u64)))
    });
    c.bench_function("div_sat_u64_min", |b| {
        b.iter(|| div_sat_u64(black_box(0u64), black_box(1u64)))
    });
    c.bench_function("div_sat_u64_max", |b| {
        b.iter(|| div_sat_u64(black_box(u64::MAX), black_box(u64::MAX)))
    });
}

criterion_group!(
    benches,
    bench_add_sat_i32,
    bench_sub_sat_i32,
    bench_mul_sat_i32,
    bench_mul_sat_u64,
    bench_div_sat_u64
);
criterion_main!(benches);
