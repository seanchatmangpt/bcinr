use bcinr_logic::algorithms::*;
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn algorithms_301_307(c: &mut Criterion) {
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::fixed_point_log2::fixed_point_log2;
    c.bench_function("fixed_point_log2_avg", |b| b.iter(|| fixed_point_log2(black_box(42), black_box(1337))));
    c.bench_function("fixed_point_log2_min", |b| b.iter(|| fixed_point_log2(black_box(0), black_box(0))));
    c.bench_function("fixed_point_log2_max", |b| b.iter(|| fixed_point_log2(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::branchless_vtable_lookup::branchless_vtable_lookup;
    c.bench_function("branchless_vtable_lookup_avg", |b| b.iter(|| branchless_vtable_lookup(black_box(42), black_box(1337))));
    c.bench_function("branchless_vtable_lookup_min", |b| b.iter(|| branchless_vtable_lookup(black_box(0), black_box(0))));
    c.bench_function("branchless_vtable_lookup_max", |b| b.iter(|| branchless_vtable_lookup(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::base64_decode_chunk4::base64_decode_chunk4;
    c.bench_function("base64_decode_chunk4_avg", |b| b.iter(|| base64_decode_chunk4(black_box(42), black_box(1337))));
    c.bench_function("base64_decode_chunk4_min", |b| b.iter(|| base64_decode_chunk4(black_box(0), black_box(0))));
    c.bench_function("base64_decode_chunk4_max", |b| b.iter(|| base64_decode_chunk4(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::unrolled_binary_search_u32::unrolled_binary_search_u32;
    c.bench_function("unrolled_binary_search_u32_avg", |b| b.iter(|| unrolled_binary_search_u32(black_box(42), black_box(1337))));
    c.bench_function("unrolled_binary_search_u32_min", |b| b.iter(|| unrolled_binary_search_u32(black_box(0), black_box(0))));
    c.bench_function("unrolled_binary_search_u32_max", |b| b.iter(|| unrolled_binary_search_u32(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::utf8_validate_chunk8::utf8_validate_chunk8;
    c.bench_function("utf8_validate_chunk8_avg", |b| b.iter(|| utf8_validate_chunk8(black_box(42), black_box(1337))));
    c.bench_function("utf8_validate_chunk8_min", |b| b.iter(|| utf8_validate_chunk8(black_box(0), black_box(0))));
    c.bench_function("utf8_validate_chunk8_max", |b| b.iter(|| utf8_validate_chunk8(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::hex_encode_chunk8::hex_encode_chunk8;
    c.bench_function("hex_encode_chunk8_avg", |b| b.iter(|| hex_encode_chunk8(black_box(42), black_box(1337))));
    c.bench_function("hex_encode_chunk8_min", |b| b.iter(|| hex_encode_chunk8(black_box(0), black_box(0))));
    c.bench_function("hex_encode_chunk8_max", |b| b.iter(|| hex_encode_chunk8(black_box(u64::MAX), black_box(u64::MAX))));
    // Import each module explicitly if needed or use the star import
    use bcinr_logic::algorithms::bit_parallel_sort8_u32::bit_parallel_sort8_u32;
    c.bench_function("bit_parallel_sort8_u32_avg", |b| b.iter(|| bit_parallel_sort8_u32(black_box(42), black_box(1337))));
    c.bench_function("bit_parallel_sort8_u32_min", |b| b.iter(|| bit_parallel_sort8_u32(black_box(0), black_box(0))));
    c.bench_function("bit_parallel_sort8_u32_max", |b| b.iter(|| bit_parallel_sort8_u32(black_box(u64::MAX), black_box(u64::MAX))));
}

criterion_group!(benches, algorithms_301_307);
criterion_main!(benches);
