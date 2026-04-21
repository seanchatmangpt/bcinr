use bcinr_logic::*;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_bitonic_sort_16u32(c: &mut Criterion) {
    c.bench_function("bitonic_sort_16u32", |b| {
        b.iter(|| {
        let mut arg0 = [0u32; 16];
            network::bitonic_sort_16u32(black_box(&mut arg0))
        })
    });
}

fn bench_kernel_integrity_check(c: &mut Criterion) {
    c.bench_function("kernel_integrity_check", |b| {
        b.iter(|| {
            autonomic::kernel::kernel_integrity_check(black_box(0u64))
        })
    });
}

criterion_group!(benches,
    bench_bitonic_sort_16u32,
    bench_kernel_integrity_check,
);
criterion_main!(benches);
