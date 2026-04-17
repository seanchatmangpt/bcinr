use bcinr_core::logic::{bitset, dfa, fix, int, mask, network, parse, reduce, scan, simd, sketch, utf8};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_bitset(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitset");
    let x = 0xAAAAAAAAAAAAAAAAu64;
    group.bench_function("rank_u64", |b| {
        b.iter(|| bitset::rank_u64(black_box(x), black_box(32)))
    });
    group.bench_function("select_bit_u64", |b| {
        b.iter(|| bitset::select_bit_u64(black_box(x), black_box(16)))
    });
    group.bench_function("set_bit_u64", |b| {
        b.iter(|| bitset::set_bit_u64(black_box(x), black_box(31)))
    });
    group.finish();
}

#[allow(clippy::identity_op, clippy::erasing_op)]
fn bench_dfa(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfa");
    let mut table = vec![0; 2 * 256];
    for c in b'a'..=b'z' {
        table[0 * 256 + c as usize] = 1;
        table[1 * 256 + c as usize] = 1;
    }
    let input = vec![b'a'; 1024];
    group.bench_function("run_1k", |b| {
        b.iter(|| {
            dfa::dfa_run(
                black_box(&input),
                black_box(&table),
                black_box(256),
                black_box(0),
            )
        })
    });
    let accept_states = [1, 5, 10];
    group.bench_function("is_accepting", |b| {
        b.iter(|| dfa::dfa_is_accepting(black_box(1), black_box(&accept_states)))
    });
    group.finish();
}

fn bench_fix(c: &mut Criterion) {
    let mut group = c.benchmark_group("fix");
    group.bench_function("add_sat_u8", |b| {
        b.iter(|| fix::add_sat_u8(black_box(200), black_box(100)))
    });
    group.bench_function("sub_sat_u8", |b| {
        b.iter(|| fix::sub_sat_u8(black_box(50), black_box(100)))
    });
    group.bench_function("clamp_u32", |b| {
        b.iter(|| fix::clamp_u32(black_box(150), black_box(100), black_box(200)))
    });
    group.finish();
}

fn bench_int(c: &mut Criterion) {
    let mut group = c.benchmark_group("int");
    group.bench_function("popcount_u32", |b| {
        b.iter(|| int::popcount_u32(black_box(0xAAAAAAAA)))
    });
    group.bench_function("next_power_of_two_u32", |b| {
        b.iter(|| int::next_power_of_two_u32(black_box(1025)))
    });
    group.bench_function("leading_zeros_u32", |b| {
        b.iter(|| int::leading_zeros_u32(black_box(0x0000FFFF)))
    });
    group.finish();
}

fn bench_mask(c: &mut Criterion) {
    let mut group = c.benchmark_group("mask");
    group.bench_function("select_u32", |b| {
        b.iter(|| {
            mask::select_u32(
                black_box(0xAAAAAAAA),
                black_box(0x12345678),
                black_box(0x87654321),
            )
        });
    });
    group.finish();
}

fn bench_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("network");
    let data = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
    group.bench_function("bitonic_sort_16u32", |b| {
        b.iter(|| {
            let mut temp = data;
            network::bitonic_sort_16u32(black_box(&mut temp));
        })
    });
    group.finish();
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    let input = b"18446744073709551615";
    group.bench_function("parse_decimal_u64", |b| {
        b.iter(|| parse::parse_decimal_u64(black_box(input)))
    });
    let ws_input = b"    \t\n  hello";
    group.bench_function("skip_whitespace", |b| {
        b.iter(|| parse::skip_whitespace(black_box(ws_input)))
    });
    group.finish();
}

fn bench_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduce");
    let arr = [1, 2, 3, 4, 5, 6, 7, 8];
    group.bench_function("horizontal_sum_u8x8", |b| {
        b.iter(|| reduce::horizontal_sum_u8x8(black_box(&arr)))
    });
    group.finish();
}

fn bench_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    let data = vec![b'a'; 1024];
    group.bench_function("ascii_1k", |b| {
        b.iter(|| scan::is_ascii_u64_slice(black_box(&data)));
    });
    group.finish();
}

fn bench_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd");
    let a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let b_arr = [
        16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    ];
    let mask = [0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15];
    group.bench_function("shuffle_u8x16", |b| {
        b.iter(|| simd::shuffle_u8x16(black_box(a), black_box(b_arr), black_box(mask)))
    });
    group.bench_function("movemask_u8x16", |b| {
        b.iter(|| simd::movemask_u8x16(black_box(a)))
    });
    group.finish();
}

fn bench_sketch(c: &mut Criterion) {
    let mut group = c.benchmark_group("sketch");
    let input = b"hello world this is a test string for hashing";
    group.bench_function("xxhash32", |b| {
        b.iter(|| sketch::xxhash32(black_box(input), black_box(0)))
    });
    group.bench_function("murmur3_32", |b| {
        b.iter(|| sketch::murmur3_32(black_box(input), black_box(0)))
    });
    group.finish();
}

fn bench_utf8(c: &mut Criterion) {
    let mut group = c.benchmark_group("utf8");
    let input = "hello world 🦀 hello world 🦀 hello world 🦀 hello world 🦀".as_bytes();
    group.bench_function("count_codepoints", |b| {
        b.iter(|| utf8::count_codepoints(black_box(input)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_bitset,
    bench_dfa,
    bench_fix,
    bench_int,
    bench_mask,
    bench_network,
    bench_parse,
    bench_reduce,
    bench_scan,
    bench_simd,
    bench_sketch,
    bench_utf8
);
criterion_main!(benches);
