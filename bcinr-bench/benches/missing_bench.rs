use bcinr_logic::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_bitonic_sort_16u32(c: &mut Criterion) {
    c.bench_function("bitonic_sort_16u32", |b| {
        b.iter(|| {
            let mut arg0 = [0u32; 16];
            network::bitonic_sort_16u32(black_box(&mut arg0))
        })
    });
}

#[cfg(feature = "alloc")]
fn bench_kernel_integrity_check(c: &mut Criterion) {
    c.bench_function("kernel_integrity_check", |b| {
        b.iter(|| autonomic::kernel::kernel_integrity_check(black_box(0u64)))
    });
}

#[cfg(feature = "alloc")]
criterion_group!(
    benches,
    bench_bitonic_sort_16u32,
    bench_kernel_integrity_check,
);

#[cfg(not(feature = "alloc"))]
criterion_group!(benches, bench_bitonic_sort_16u32,);

criterion_main!(benches);
