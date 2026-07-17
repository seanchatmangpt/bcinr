import os

bad = [
    "lcm_u64_branchless.rs",
    "cardinality_linear_counting.rs",
    "is_permutation_branchless.rs",
    "crc32c_branchless.rs",
    "norm_u32.rs",
    "normalize_slice_branchless.rs",
    "rank_u32x8.rs",
    "levenshtein_dist_branchless.rs",
    "murmur3_32_hash.rs",
    "merge_u32_slices_branchless.rs",
    "fnv1a_64_hash.rs",
    "xor_filter_lookup.rs",
    "parallel_bits_deposit_u64.rs",
    "tabulation_hash_u64.rs",
    "heavy_hitter_update.rs",
    "linear_search_simd_u8.rs",
    "scatter_bits_u64.rs",
    "polynomial_hash_u64.rs",
    "halton_sequence_u32.rs",
    "gcd_u64_branchless.rs",
    "nth_element_branchless.rs",
    "wyhash_64.rs",
    "rank_u128.rs",
    "locality_sensitive_hash_cosine.rs",
    "sort_stable_key_value_u32x8.rs",
    "jaro_winkler_branchless.rs",
    "parallel_bits_extract_u64.rs",
    "sorting_network_verify_u32.rs",
    "count_consecutive_set_bits_u64.rs",
    "fp_sqrt_u32_q16.rs",
    "gather_bits_u64.rs",
    "fp_sin_u32_q16.rs",
    "adler32_branchless.rs",
    "count_min_sketch_update.rs",
    "hazard_pointer_retire.rs",
    "hyperloglog_add_u64_registers.rs",
    "lcp_array_step_branchless.rs",
    "quotient_filter_add_u64.rs",
    "reservoir_sample_simd.rs",
    "simd_strstr_branchless.rs",
    "simhash_cosine_u64.rs",
    "temp_gate_missing.rs",
    "xoroshiro128_plus.rs"
]

algorithms_dir = "crates/bcinr-logic/src/algorithms"

for f in bad:
    path = os.path.join(algorithms_dir, f)
    if os.path.exists(path):
        os.remove(path)
        print(f"Removed {f}")

mod_path = os.path.join(algorithms_dir, "mod.rs")
with open(mod_path, "r") as f:
    lines = f.readlines()

with open(mod_path, "w") as f:
    for line in lines:
        skip = False
        for bad_f in bad:
            mod_name = bad_f.replace(".rs", "")
            if f"pub mod {mod_name};" in line:
                skip = True
                break
        if not skip:
            f.write(line)
