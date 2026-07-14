use bcinr_logic::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_bitonic_sort_16u32(c: &mut Criterion) {
    // Reverse-sorted: worst case for most sorting networks
    let input: [u32; 16] = [
        0xFFFF_0000,
        0xEEEE_1111,
        0xDDDD_2222,
        0xCCCC_3333,
        0xBBBB_4444,
        0xAAAA_5555,
        0x9999_6666,
        0x8888_7777,
        0x7777_8888,
        0x6666_9999,
        0x5555_AAAA,
        0x4444_BBBB,
        0x3333_CCCC,
        0x2222_DDDD,
        0x1111_EEEE,
        0x0000_FFFF,
    ];
    c.bench_function("bitonic_sort_16u32", |b| {
        b.iter(|| {
            let mut arr = black_box(input);
            network::bitonic_sort_16u32(&mut arr);
            black_box(arr)
        });
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
