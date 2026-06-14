import maturity_auditor

files = [
    "quantize_u32.rs", "quaternion_mul_branchless.rs", "quotient_filter_add_u64.rs", "radix_sort_step_branchless.rs",
    "random_permutation_fixed_seed.rs", "rank_select_dictionary_rrr.rs", "rank_select_sort_u32.rs", "rank_u128.rs",
    "ray_sphere_intersect_branchless.rs", "ray_triangle_intersect_branchless.rs", "regex_nfa_simd_step.rs", "relu_u32.rs",
    "reservoir_sample_branchless.rs", "reservoir_sample_weighted_simd.rs", "reverse_bits_u128.rs", "reverse_slice_branchless.rs",
    "rolling_hash_buzhash.rs", "rolling_hash_gear.rs", "rolling_hash_rabin_karp.rs", "rotate_left_u64.rs", "rotate_right_u64.rs",
    "rotate_slice_branchless.rs", "round_down_u32.rs", "round_to_nearest_u32.rs", "round_up_u32.rs", "scatter_bits_u64.rs",
    "search_eytzinger_u32.rs", "search_van_emde_boas.rs", "select_u128.rs", "set_difference_branchless.rs", "set_intersection_branchless.rs"
]

for f in files:
    path = f"crates/bcinr-logic/src/algorithms/{f}"
    score, issues = maturity_auditor.audit_file(path)
    print(f"{f}: Score {score}, Issues: {issues}")
