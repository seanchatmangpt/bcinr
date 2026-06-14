import maturity_auditor

files = [
    "trim_whitespace_branchless.rs", "tzmsk_u64.rs", "unique_branchless_u32.rs", "unrolled_binary_search_u32.rs",
    "upper_bound_branchless_u32.rs", "url_decode_branchless.rs", "url_encode_branchless.rs", "utf16_to_utf8_simd.rs",
    "utf8_to_utf16_simd.rs", "utf8_to_utf32_simd.rs", "utf8_validate_chunk8.rs", "varint_decode_simd.rs",
    "varint_encode_simd.rs", "vector_cross_product_f32.rs", "vector_dot_product_simd_f32.rs", "waitfree_queue_push.rs",
    "wavelet_tree_access_branchless.rs", "weight_u64.rs", "weighted_avg_u32.rs", "weighted_reservoir_sample.rs",
    "wildcard_match_branchless.rs", "xoroshiro128_plus.rs", "xxh3_64.rs", "xxhash64.rs",
    "z_order_curve_2d_u32.rs", "zigzag_decode_i64.rs", "zigzag_encode_i64.rs", "zobrist_hash_64.rs"
]

for f in files:
    path = f"crates/bcinr-logic/src/algorithms/{f}"
    try:
        score, issues = maturity_auditor.audit_file(path)
        if score < 100:
            print(f"{f}: Score {score}, Issues: {issues}")
    except Exception as e:
        print(f"Error auditing {f}: {e}")
