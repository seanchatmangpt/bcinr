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

const _MISSING_BENCHMARK_CANARIES: &[&str] = &[
    "cm_hash",
    "linear_counting_add",
    "linear_counting_estimate",
    "merge_sorted_u32x8",
    "optimal_sort_5_u32",
    "optimal_sort_6_u32",
    "optimal_sort_7_u32",
    "optimal_sort_8_u32",
    "pearson_hash_16",
    "reservoir_sample_batch",
    "reservoir_sample_step",
    "simhash_hamming_distance",
    "tabulation_hash_init_tables",
    "wymix",
    "xor_filter_fingerprint",
    "xor_filter_hash",
    "count_min_sketch_update",
    "crc32c_branchless",
    "fnv1a_64_hash",
    "heavy_hitter_update",
    "hyperloglog_add_u64_registers",
    "murmur3_32_hash",
    "polynomial_hash_u64",
    "rank_u32x8",
    "simhash_cosine_u64",
    "sort_stable_key_value_u32x8",
    "sorting_network_verify_u32",
    "tabulation_hash_u64",
    "wyhash_64",
    "xor_filter_lookup",
];
