//! Throughput Benchmark Suite
//!
//! Measures MB/s, ops/ns, and provides side-by-side SIMD vs scalar comparisons
//! for core bcinr-logic primitives using Criterion's Throughput API.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// ---------------------------------------------------------------------------
// Byte-search throughput (MB/s): SWAR scan vs naive iterator
// ---------------------------------------------------------------------------

fn bench_byte_search(c: &mut Criterion) {
    use bcinr_logic::scan::find_byte_mask;

    let mut group = c.benchmark_group("byte_search_throughput");

    for &size in &[64usize, 256, 1024, 4096, 65536] {
        let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));

        // Naive baseline: iterator position search
        group.bench_with_input(BenchmarkId::new("naive_find", size), &data, |b, data| {
            b.iter(|| data.iter().position(|&x| x == black_box(42u8)));
        });

        // SWAR scan: processes up to 64 bytes branchlessly via bitmask
        group.bench_with_input(BenchmarkId::new("swar_scan", size), &data, |b, data| {
            b.iter(|| find_byte_mask(black_box(data), black_box(42u8)));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Population count throughput (MB/s): bcinr vs std count_ones
// ---------------------------------------------------------------------------

fn bench_popcount(c: &mut Criterion) {
    use bcinr_logic::int::popcount_u64;

    let mut group = c.benchmark_group("popcount_throughput");

    for &size in &[1024usize, 65536, 1_048_576] {
        let data: Vec<u64> = (0..size / 8)
            .map(|i| (i as u64).wrapping_mul(0x9e3779b9_7f4a7c15))
            .collect();
        // Report in bytes: each u64 is 8 bytes
        group.throughput(Throughput::Bytes((data.len() * 8) as u64));

        group.bench_with_input(
            BenchmarkId::new("bcinr_popcount_u64", size),
            &data,
            |b, data| {
                b.iter(|| {
                    black_box(
                        data.iter()
                            .fold(0u64, |acc, &x| acc + popcount_u64(black_box(x))),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("std_count_ones", size),
            &data,
            |b, data| {
                b.iter(|| {
                    black_box(
                        data.iter()
                            .fold(0u64, |acc, &x| acc + x.count_ones() as u64),
                    )
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Hash throughput (MB/s): xxhash64, adler32, fnv1a, crc32c
// ---------------------------------------------------------------------------

fn bench_hash_throughput(c: &mut Criterion) {
    use bcinr_logic::algorithms::adler32_branchless::adler32_branchless;
    use bcinr_logic::algorithms::farmhash64::farmhash64;
    use bcinr_logic::algorithms::siphash_2_4_branchless::siphash_2_4_branchless;
    use bcinr_logic::algorithms::xxhash64::xxhash64;

    let mut group = c.benchmark_group("hash_throughput");

    for &size in &[16usize, 64, 256, 1024, 8192] {
        let data: Vec<u8> = (0..size).map(|i| i as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));

        // xxhash64: chain across the buffer in 8-byte chunks so throughput numbers
        // reflect processing the full buffer rather than a single 8-byte word.
        // Remainder bytes (size % 8 != 0) are ignored via chunks_exact.
        group.bench_with_input(BenchmarkId::new("xxhash64", size), &data, |b, data| {
            b.iter(|| {
                let mut h = 0u64;
                for chunk in data.chunks_exact(8) {
                    let word = u64::from_le_bytes(chunk.try_into().unwrap());
                    h = xxhash64(black_box(word), h);
                }
                black_box(h)
            });
        });

        // adler32: chain across the buffer in 8-byte chunks
        group.bench_with_input(BenchmarkId::new("adler32", size), &data, |b, data| {
            b.iter(|| {
                let mut h = 0u64;
                for chunk in data.chunks_exact(8) {
                    let word = u64::from_le_bytes(chunk.try_into().unwrap());
                    h = adler32_branchless(black_box(word), h);
                }
                black_box(h)
            });
        });

        // farmhash64: chain across the buffer in 8-byte chunks
        group.bench_with_input(BenchmarkId::new("farmhash64", size), &data, |b, data| {
            b.iter(|| {
                let mut h = 0u64;
                for chunk in data.chunks_exact(8) {
                    let word = u64::from_le_bytes(chunk.try_into().unwrap());
                    h = farmhash64(black_box(word), h);
                }
                black_box(h)
            });
        });

        // siphash_2_4: chain across the buffer in 8-byte chunks
        group.bench_with_input(BenchmarkId::new("siphash_2_4", size), &data, |b, data| {
            b.iter(|| {
                let mut h = 0u64;
                for chunk in data.chunks_exact(8) {
                    let word = u64::from_le_bytes(chunk.try_into().unwrap());
                    h = siphash_2_4_branchless(black_box(word), h);
                }
                black_box(h)
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Sorting network throughput (ops/ns): branchless vs std sort
// ---------------------------------------------------------------------------

fn bench_sorting_networks(c: &mut Criterion) {
    use bcinr_logic::algorithms::bit_parallel_sort8_u32::bit_parallel_sort8_u32;
    use bcinr_logic::algorithms::sort_pairs_u32x4::sort_pairs_u32x4;

    let mut group = c.benchmark_group("sorting_networks_throughput");

    // 4-element pair sort: packs two u16 pairs into a u64
    group.throughput(Throughput::Elements(4));
    group.bench_function("sort_4_branchless", |b| {
        b.iter(|| {
            sort_pairs_u32x4(
                black_box(0xDEAD_BEEF_CAFE_BABE_u64),
                black_box(0x1337_0000_FFFF_0001_u64),
            )
        });
    });

    // 8-element byte-lane sort via Batcher odd-even network
    group.throughput(Throughput::Elements(8));
    group.bench_function("sort_8_branchless", |b| {
        b.iter(|| {
            bit_parallel_sort8_u32(
                black_box(0xDEAD_BEEF_CAFE_BABE_u64),
                black_box(0x1337_0000_FFFF_0001_u64),
            )
        });
    });

    // std::slice::sort baseline for 8 u8 values
    group.throughput(Throughput::Elements(8));
    group.bench_function("sort_8_std", |b| {
        let mut data = [0xDE_u8, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        b.iter(|| {
            let mut d = black_box(data);
            d.sort_unstable();
            data = d;
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Mask select throughput (ops/ns): branchless select vs branch
// ---------------------------------------------------------------------------

fn bench_mask_ops(c: &mut Criterion) {
    use bcinr_logic::mask::select_u64;

    let mut group = c.benchmark_group("mask_select_throughput");
    const N: u64 = 1_000_000;
    group.throughput(Throughput::Elements(N));

    // Branchless select_u64 — mask is all-ones or all-zeros
    group.bench_function("select_u64_branchless", |b| {
        let mut sum = 0u64;
        b.iter(|| {
            for i in 0u64..N {
                let mask = 0u64.wrapping_sub(i & 1); // 0x000…0 or 0xFFF…F
                sum = sum.wrapping_add(select_u64(black_box(mask), i, i * 2));
            }
            black_box(sum)
        });
    });

    // Branch baseline: standard if/else
    group.bench_function("if_else_u64_branch", |b| {
        let mut sum = 0u64;
        b.iter(|| {
            for i in 0u64..N {
                sum = sum.wrapping_add(if black_box(i) & 1 == 0 { i } else { i * 2 });
            }
            black_box(sum)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Horizontal reduction throughput (ops/ns): bcinr OR vs std fold
// ---------------------------------------------------------------------------

fn bench_reductions(c: &mut Criterion) {
    use bcinr_logic::reduce::{horizontal_and_u32, horizontal_or_u32, horizontal_xor_u32};

    let mut group = c.benchmark_group("horizontal_reductions_throughput");

    for &size in &[64usize, 512, 4096] {
        let data: Vec<u32> = (0..size).map(|i| i as u32).collect();
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("horizontal_or_u32", size),
            &data,
            |b, data| {
                b.iter(|| horizontal_or_u32(black_box(data)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("horizontal_and_u32", size),
            &data,
            |b, data| {
                b.iter(|| horizontal_and_u32(black_box(data)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("horizontal_xor_u32", size),
            &data,
            |b, data| {
                b.iter(|| horizontal_xor_u32(black_box(data)));
            },
        );

        group.bench_with_input(BenchmarkId::new("std_fold_or", size), &data, |b, data| {
            b.iter(|| data.iter().fold(0u32, |acc, &x| acc | black_box(x)));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// ASCII validation throughput (MB/s): SWAR 8-byte-at-a-time vs std iterator
// ---------------------------------------------------------------------------

fn bench_ascii_classify(c: &mut Criterion) {
    use bcinr_logic::scan::is_ascii_u64_slice;

    let mut group = c.benchmark_group("ascii_classify_throughput");

    for &size in &[256usize, 4096, 65536] {
        // Printable ASCII range 32..=127
        let data: Vec<u8> = (0..size).map(|i| (32 + (i % 96)) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("swar_is_ascii", size), &data, |b, data| {
            b.iter(|| is_ascii_u64_slice(black_box(data)));
        });

        group.bench_with_input(BenchmarkId::new("std_is_ascii", size), &data, |b, data| {
            b.iter(|| data.iter().all(|c| black_box(*c).is_ascii()));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Fixed-point arithmetic throughput (ops/ns): bcinr add_sat vs saturating_add
// ---------------------------------------------------------------------------

fn bench_fixed_point(c: &mut Criterion) {
    use bcinr_logic::fix::{add_sat, clamp_u32};

    let mut group = c.benchmark_group("fixed_point_throughput");
    const N: usize = 1024;
    group.throughput(Throughput::Elements(N as u64));

    let values: Vec<u32> = (0..N as u32).map(|i| i.wrapping_mul(0x9e3779b9)).collect();

    // Branchless saturating add from fix module
    group.bench_function("add_sat_u32_branchless", |b| {
        b.iter(|| {
            black_box(
                values
                    .iter()
                    .fold(0u32, |acc, &x| add_sat(acc, black_box(x))),
            )
        });
    });

    // std saturating_add as baseline
    group.bench_function("std_saturating_add_u32", |b| {
        b.iter(|| {
            black_box(
                values
                    .iter()
                    .fold(0u32, |acc, &x| acc.saturating_add(black_box(x))),
            )
        });
    });

    // Branchless clamp vs std clamp
    group.bench_function("clamp_u32_branchless", |b| {
        b.iter(|| {
            black_box(values.iter().fold(0u32, |acc, &x| {
                acc.wrapping_add(clamp_u32(black_box(x), 100, 900_000))
            }))
        });
    });

    group.bench_function("std_clamp_u32", |b| {
        b.iter(|| {
            black_box(values.iter().fold(0u32, |acc, &x| {
                acc.wrapping_add(black_box(x).clamp(100, 900_000))
            }))
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Bitset operations throughput (MB/s): SWAR Hamming distance / Jaccard
// ---------------------------------------------------------------------------

fn bench_bitset_ops(c: &mut Criterion) {
    use bcinr_logic::bitset::{hamming_u64_slices, jaccard_u64_slices, union_u64_slices};

    let mut group = c.benchmark_group("bitset_ops_throughput");

    for &size in &[64usize, 512, 4096] {
        let a: Vec<u64> = (0..size)
            .map(|i| (i as u64).wrapping_mul(0x9e3779b9_7f4a7c15))
            .collect();
        let mut b: Vec<u64> = (0..size)
            .map(|i| (i as u64).wrapping_mul(0x6c62272e_07bb0142))
            .collect();
        // Report bytes: each u64 = 8 bytes
        group.throughput(Throughput::Bytes((size * 8) as u64));

        group.bench_with_input(
            BenchmarkId::new("hamming_u64_slices", size),
            &(&a, &b),
            |b_iter, (a, b)| {
                b_iter.iter(|| hamming_u64_slices(black_box(a), black_box(b)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("jaccard_u64_slices", size),
            &(&a, &b),
            |b_iter, (a, b)| {
                b_iter.iter(|| jaccard_u64_slices(black_box(a), black_box(b)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("union_u64_slices", size),
            &a,
            |b_iter, a| {
                b_iter.iter(|| {
                    let mut dst = a.clone();
                    union_u64_slices(black_box(&mut dst), black_box(&b));
                    black_box(dst);
                });
            },
        );

        // std iterator baseline for Hamming (popcount XOR)
        group.bench_with_input(
            BenchmarkId::new("std_hamming_fold", size),
            &(&a, &b),
            |b_iter, (a, b)| {
                b_iter.iter(|| {
                    black_box(
                        a.iter()
                            .zip(b.iter())
                            .fold(0usize, |acc, (&x, &y)| acc + (x ^ y).count_ones() as usize),
                    )
                });
            },
        );

        let _ = b.as_mut_slice(); // suppress unused-mut warning
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Delta encode/decode throughput (MB/s): SIMD-style vs naive
// ---------------------------------------------------------------------------

fn bench_delta_codec(c: &mut Criterion) {
    use bcinr_logic::algorithms::delta_decode_simd_u32::delta_decode_simd_u32;
    use bcinr_logic::algorithms::delta_encode_simd_u32::delta_encode_simd_u32;

    let mut group = c.benchmark_group("delta_codec_throughput");
    // Each call encodes/decodes a packed u64 (8 bytes conceptually)
    const ITERS: u64 = 8192;
    group.throughput(Throughput::Bytes(ITERS * 8));

    group.bench_function("delta_encode_simd_u32", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for i in 0..ITERS {
                acc = acc.wrapping_add(delta_encode_simd_u32(black_box(i), black_box(acc)));
            }
            black_box(acc)
        });
    });

    group.bench_function("delta_decode_simd_u32", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for i in 0..ITERS {
                acc = acc.wrapping_add(delta_decode_simd_u32(black_box(i), black_box(acc)));
            }
            black_box(acc)
        });
    });

    // Naive delta encode baseline
    group.bench_function("delta_encode_naive", |b| {
        b.iter(|| {
            let mut prev = 0u64;
            let mut acc = 0u64;
            for i in 0..ITERS {
                let delta = black_box(i).wrapping_sub(prev);
                prev = black_box(i);
                acc = acc.wrapping_add(delta);
            }
            black_box(acc)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Hamming distance throughput (ops/ns): SIMD-style u64 vs popcount XOR
// ---------------------------------------------------------------------------

fn bench_hamming_distance(c: &mut Criterion) {
    use bcinr_logic::algorithms::hamming_dist_simd::hamming_dist_simd;

    let mut group = c.benchmark_group("hamming_distance_throughput");
    const N: u64 = 100_000;
    group.throughput(Throughput::Elements(N));

    group.bench_function("hamming_dist_simd_100k", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for i in 0..N {
                acc = acc.wrapping_add(hamming_dist_simd(
                    black_box(i.wrapping_mul(0x9e3779b9_7f4a7c15)),
                    black_box(i.wrapping_mul(0x6c62272e_07bb0142)),
                ));
            }
            black_box(acc)
        });
    });

    group.bench_function("popcount_xor_100k", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for i in 0..N {
                let a = black_box(i.wrapping_mul(0x9e3779b9_7f4a7c15));
                let b_val = black_box(i.wrapping_mul(0x6c62272e_07bb0142));
                acc = acc.wrapping_add((a ^ b_val).count_ones() as u64);
            }
            black_box(acc)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// criterion_group and main
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_byte_search,
    bench_popcount,
    bench_hash_throughput,
    bench_sorting_networks,
    bench_mask_ops,
    bench_reductions,
    bench_ascii_classify,
    bench_fixed_point,
    bench_bitset_ops,
    bench_delta_codec,
    bench_hamming_distance,
);
criterion_main!(benches);
