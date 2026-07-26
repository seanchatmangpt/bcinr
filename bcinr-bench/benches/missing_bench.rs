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

// ---------------------------------------------------------------------------
// Hashing family: cm_hash, fnv1a_64_hash, murmur3_32_hash, polynomial_hash_u64,
// wyhash_64, wymix, tabulation_hash_{init_tables,u64}, pearson_hash_16,
// crc32c_branchless
// ---------------------------------------------------------------------------

fn bench_cm_hash(c: &mut Criterion) {
    use bcinr_logic::algorithms::count_min_sketch_update::cm_hash;
    c.bench_function("cm_hash", |b| {
        b.iter(|| cm_hash(black_box(0x1234_5678_9ABC_DEF0), black_box(7)))
    });
}

fn bench_fnv1a_64_hash(c: &mut Criterion) {
    use bcinr_logic::algorithms::fnv1a_64_hash::fnv1a_64_hash;
    let data = *b"the quick brown fox jumps over the lazy dog";
    c.bench_function("fnv1a_64_hash", |b| {
        b.iter(|| fnv1a_64_hash(black_box(&data)))
    });
}

fn bench_murmur3_32_hash(c: &mut Criterion) {
    use bcinr_logic::algorithms::murmur3_32_hash::murmur3_32_hash;
    let data = *b"the quick brown fox jumps over the lazy dog";
    c.bench_function("murmur3_32_hash", |b| {
        b.iter(|| murmur3_32_hash(black_box(&data), black_box(0x9747_b28c)))
    });
}

fn bench_polynomial_hash_u64(c: &mut Criterion) {
    use bcinr_logic::algorithms::polynomial_hash_u64::polynomial_hash_u64;
    let data = *b"the quick brown fox jumps over the lazy dog";
    c.bench_function("polynomial_hash_u64", |b| {
        b.iter(|| polynomial_hash_u64(black_box(&data), black_box(31), black_box(1_000_000_007)))
    });
}

fn bench_wyhash_64(c: &mut Criterion) {
    use bcinr_logic::algorithms::wyhash_64::wyhash_64;
    let data = *b"the quick brown fox jumps over the lazy dog";
    c.bench_function("wyhash_64", |b| {
        b.iter(|| wyhash_64(black_box(&data), black_box(0xAAAA_BBBB_CCCC_DDDD)))
    });
}

fn bench_wymix(c: &mut Criterion) {
    use bcinr_logic::algorithms::wyhash_64::wymix;
    c.bench_function("wymix", |b| {
        b.iter(|| {
            wymix(
                black_box(0x1111_2222_3333_4444),
                black_box(0x5555_6666_7777_8888),
            )
        })
    });
}

fn bench_tabulation_hash(c: &mut Criterion) {
    use bcinr_logic::algorithms::tabulation_hash_u64::{
        tabulation_hash_init_tables, tabulation_hash_u64,
    };

    c.bench_function("tabulation_hash_init_tables", |b| {
        b.iter(|| {
            let mut tables = [[0u64; 256]; 4];
            tabulation_hash_init_tables(black_box(0x0BAD_C0DE), &mut tables);
            black_box(tables)
        })
    });

    let mut tables = [[0u64; 256]; 4];
    tabulation_hash_init_tables(0x0BAD_C0DE, &mut tables);
    c.bench_function("tabulation_hash_u64", |b| {
        b.iter(|| tabulation_hash_u64(black_box(0xDEAD_BEEF), black_box(&tables)))
    });
}

fn bench_pearson_hash_16(c: &mut Criterion) {
    use bcinr_logic::algorithms::pearson_hash_16::pearson_hash_16;
    let data = *b"the quick brown fox jumps over the lazy dog";
    c.bench_function("pearson_hash_16", |b| {
        b.iter(|| pearson_hash_16(black_box(&data)))
    });
}

fn bench_crc32c_branchless(c: &mut Criterion) {
    use bcinr_logic::algorithms::crc32c_branchless::crc32c_branchless;
    let data = *b"the quick brown fox jumps over the lazy dog";
    c.bench_function("crc32c_branchless", |b| {
        b.iter(|| crc32c_branchless(black_box(&data), black_box(0)))
    });
}

// ---------------------------------------------------------------------------
// Sketching / cardinality family: count_min_sketch_update, linear_counting_*,
// hyperloglog_add_u64_registers, heavy_hitter_update, simhash_*,
// xor_filter_*, reservoir_sample_*
// ---------------------------------------------------------------------------

fn bench_count_min_sketch_update(c: &mut Criterion) {
    use bcinr_logic::algorithms::count_min_sketch_update::count_min_sketch_update;
    let mut sketch = vec![0u32; 4 * 256];
    c.bench_function("count_min_sketch_update", |b| {
        b.iter(|| {
            count_min_sketch_update(
                black_box(&mut sketch),
                black_box(4),
                black_box(256),
                black_box(0x1234_5678_9ABC_DEF0),
                black_box(1),
            )
        })
    });
}

fn bench_linear_counting(c: &mut Criterion) {
    use bcinr_logic::algorithms::cardinality_linear_counting::{
        linear_counting_add, linear_counting_estimate,
    };
    c.bench_function("linear_counting_add", |b| {
        let mut bitmap = [0u64; 64];
        b.iter(|| linear_counting_add(black_box(&mut bitmap), black_box(0xAAAA_BBBB_CCCC_DDDD)))
    });

    let mut bitmap = [0u64; 64];
    for i in 0..200u64 {
        linear_counting_add(&mut bitmap, i.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    }
    c.bench_function("linear_counting_estimate", |b| {
        b.iter(|| linear_counting_estimate(black_box(&bitmap)))
    });
}

fn bench_hyperloglog_add_u64_registers(c: &mut Criterion) {
    use bcinr_logic::algorithms::hyperloglog_add_u64_registers::hyperloglog_add_u64_registers;
    let mut registers = [0u8; 1 << 10];
    c.bench_function("hyperloglog_add_u64_registers", |b| {
        b.iter(|| {
            hyperloglog_add_u64_registers(
                black_box(&mut registers),
                black_box(0x1234_5678_9ABC_DEF0),
                black_box(10),
            )
        })
    });
}

fn bench_heavy_hitter_update(c: &mut Criterion) {
    use bcinr_logic::algorithms::heavy_hitter_update::heavy_hitter_update;
    let mut table = vec![(0u64, 0u64); 64];
    c.bench_function("heavy_hitter_update", |b| {
        b.iter(|| heavy_hitter_update(black_box(&mut table), black_box(0x1234_5678_9ABC_DEF0)))
    });
}

fn bench_simhash(c: &mut Criterion) {
    use bcinr_logic::algorithms::simhash_cosine_u64::{
        simhash_cosine_u64, simhash_hamming_distance,
    };
    let features = [
        0x1111_2222_3333_4444u64,
        0x5555_6666_7777_8888,
        0x9999_AAAA_BBBB_CCCC,
        0xDDDD_EEEE_FFFF_0000,
    ];
    c.bench_function("simhash_cosine_u64", |b| {
        b.iter(|| simhash_cosine_u64(black_box(&features)))
    });

    let a = simhash_cosine_u64(&features);
    let b_features = [
        0x1111_2222_3333_4445u64,
        0x5555_6666_7777_8889,
        0x9999_AAAA_BBBB_CCCD,
        0xDDDD_EEEE_FFFF_0001,
    ];
    let b_hash = simhash_cosine_u64(&b_features);
    c.bench_function("simhash_hamming_distance", |b| {
        b.iter(|| simhash_hamming_distance(black_box(a), black_box(b_hash)))
    });
}

fn bench_xor_filter(c: &mut Criterion) {
    use bcinr_logic::algorithms::xor_filter_lookup::{
        xor_filter_fingerprint, xor_filter_hash, xor_filter_lookup,
    };
    c.bench_function("xor_filter_fingerprint", |b| {
        b.iter(|| xor_filter_fingerprint(black_box(0x1234_5678_9ABC_DEF0), black_box(7)))
    });
    c.bench_function("xor_filter_hash", |b| {
        b.iter(|| xor_filter_hash(black_box(0x1234_5678_9ABC_DEF0), black_box(7), black_box(3)))
    });
    let table = vec![0u8; 1024];
    c.bench_function("xor_filter_lookup", |b| {
        b.iter(|| {
            xor_filter_lookup(
                black_box(0x1234_5678_9ABC_DEF0),
                black_box(&table),
                black_box(7),
            )
        })
    });
}

fn bench_reservoir_sample(c: &mut Criterion) {
    use bcinr_logic::algorithms::reservoir_sample_simd::{
        reservoir_sample_batch, reservoir_sample_step,
    };
    c.bench_function("reservoir_sample_step", |b| {
        b.iter(|| {
            reservoir_sample_step(
                black_box(0),
                black_box(42),
                black_box(1),
                black_box(0xAAAA_BBBB),
            )
        })
    });

    let stream: Vec<u64> = (0..64u64).collect();
    c.bench_function("reservoir_sample_batch", |b| {
        b.iter(|| {
            let mut rng = 0xAAAA_BBBB_u64;
            let lcg = |r: &mut u64| {
                *r = r
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                *r
            };
            reservoir_sample_batch(
                black_box(stream[0]),
                black_box(&stream[1..]),
                black_box(2),
                &mut rng,
                lcg,
            )
        })
    });
}

// ---------------------------------------------------------------------------
// Sorting / ranking family: optimal_sort_{5,6,7,8}_u32, merge_sorted_u32x8,
// sort_stable_key_value_u32x8, sorting_network_verify_u32, rank_u32x8
// ---------------------------------------------------------------------------

fn bench_optimal_sorts(c: &mut Criterion) {
    use bcinr_logic::algorithms::optimal_sort_5_u32::optimal_sort_5_u32;
    use bcinr_logic::algorithms::optimal_sort_6_u32::optimal_sort_6_u32;
    use bcinr_logic::algorithms::optimal_sort_7_u32::optimal_sort_7_u32;
    use bcinr_logic::algorithms::optimal_sort_8_u32::optimal_sort_8_u32;

    c.bench_function("optimal_sort_5_u32", |b| {
        b.iter(|| optimal_sort_5_u32(black_box([5, 4, 3, 2, 1])))
    });
    c.bench_function("optimal_sort_6_u32", |b| {
        b.iter(|| optimal_sort_6_u32(black_box([6, 5, 4, 3, 2, 1])))
    });
    c.bench_function("optimal_sort_7_u32", |b| {
        b.iter(|| optimal_sort_7_u32(black_box([7, 6, 5, 4, 3, 2, 1])))
    });
    c.bench_function("optimal_sort_8_u32", |b| {
        b.iter(|| optimal_sort_8_u32(black_box([8, 7, 6, 5, 4, 3, 2, 1])))
    });
}

fn bench_merge_sorted_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::merge_sorted_u32x8::merge_sorted_u32x8;
    c.bench_function("merge_sorted_u32x8", |b| {
        b.iter(|| merge_sorted_u32x8(black_box([1, 3, 5, 7]), black_box([2, 4, 6, 8])))
    });
}

fn bench_sort_stable_key_value_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::sort_stable_key_value_u32x8::sort_stable_key_value_u32x8;
    let pairs: [(u32, u32); 8] = [
        (8, 0),
        (7, 1),
        (6, 2),
        (5, 3),
        (4, 4),
        (3, 5),
        (2, 6),
        (1, 7),
    ];
    c.bench_function("sort_stable_key_value_u32x8", |b| {
        b.iter(|| sort_stable_key_value_u32x8(black_box(pairs)))
    });
}

fn bench_sorting_network_verify_u32(c: &mut Criterion) {
    use bcinr_logic::algorithms::sorting_network_verify_u32::sorting_network_verify_u32;
    let sorted: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    c.bench_function("sorting_network_verify_u32", |b| {
        b.iter(|| sorting_network_verify_u32(black_box(&sorted)))
    });
}

fn bench_rank_u32x8(c: &mut Criterion) {
    use bcinr_logic::algorithms::rank_u32x8::rank_u32x8;
    c.bench_function("rank_u32x8", |b| {
        b.iter(|| rank_u32x8(black_box([40, 10, 30, 80, 20, 70, 60, 50])))
    });
}

#[cfg(feature = "alloc")]
criterion_group!(
    benches,
    bench_bitonic_sort_16u32,
    bench_kernel_integrity_check,
    bench_cm_hash,
    bench_fnv1a_64_hash,
    bench_murmur3_32_hash,
    bench_polynomial_hash_u64,
    bench_wyhash_64,
    bench_wymix,
    bench_tabulation_hash,
    bench_pearson_hash_16,
    bench_crc32c_branchless,
    bench_count_min_sketch_update,
    bench_linear_counting,
    bench_hyperloglog_add_u64_registers,
    bench_heavy_hitter_update,
    bench_simhash,
    bench_xor_filter,
    bench_reservoir_sample,
    bench_optimal_sorts,
    bench_merge_sorted_u32x8,
    bench_sort_stable_key_value_u32x8,
    bench_sorting_network_verify_u32,
    bench_rank_u32x8,
);

#[cfg(not(feature = "alloc"))]
criterion_group!(
    benches,
    bench_bitonic_sort_16u32,
    bench_cm_hash,
    bench_fnv1a_64_hash,
    bench_murmur3_32_hash,
    bench_polynomial_hash_u64,
    bench_wyhash_64,
    bench_wymix,
    bench_tabulation_hash,
    bench_pearson_hash_16,
    bench_crc32c_branchless,
    bench_count_min_sketch_update,
    bench_linear_counting,
    bench_hyperloglog_add_u64_registers,
    bench_heavy_hitter_update,
    bench_simhash,
    bench_xor_filter,
    bench_reservoir_sample,
    bench_optimal_sorts,
    bench_merge_sorted_u32x8,
    bench_sort_stable_key_value_u32x8,
    bench_sorting_network_verify_u32,
    bench_rank_u32x8,
);

criterion_main!(benches);
