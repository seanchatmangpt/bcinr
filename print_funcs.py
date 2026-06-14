import os
import sys
import re

files = [
    "hilbert_curve_encode_u32.rs", "huffman_decode_table_step.rs", "hyperloglog_add_u64.rs",
    "hyperloglog_merge.rs", "insertion_sort_branchless_fixed.rs", "internet_checksum_u16.rs",
    "inverse_permute_u32x8.rs", "is_alphanumeric_simd_u8x16.rs", "is_contiguous_mask_u64.rs",
    "is_digit_simd_u8x16.rs", "is_finite_fp32_branchless.rs", "is_nan_fp32_branchless.rs",
    "is_permutation_branchless.rs", "is_prime_u64_branchless.rs", "is_sorted_branchless_u32.rs",
    "is_space_simd_u8x16.rs", "is_subset_mask_u64.rs", "jaro_winkler_branchless.rs",
    "json_find_string_escapes_simd.rs", "json_find_structural_simd.rs", "k_independent_hash_gen.rs",
    "knuth_hash_u64.rs", "lcm_u64_branchless.rs", "lcp_array_step_branchless.rs",
    "leaky_relu_u32.rs", "leb128_decode_u64.rs", "leb128_encode_u64.rs", "lerp_sat_u32.rs",
    "lerp_sat_u8.rs", "levenshtein_dist_branchless.rs", "lex_compare_u8_slices_branchless.rs"
]

base_dir = "crates/bcinr-logic/src/algorithms"
for f in files:
    with open(os.path.join(base_dir, f)) as fp:
        c = fp.read()
    m = re.search(r'pub fn \w+\(.*?\) -> \w+ \{([\s\S]*?)\n\}\n\n#\[cfg\(test\)\]', c)
    if m:
        print(f"--- {f} ---")
        print(m.group(1).strip())
