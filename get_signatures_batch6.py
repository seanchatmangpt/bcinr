import os
import re

files = [
    "linear_congruential_generator_u64.rs",
    "linear_search_simd_u8.rs",
    "locality_sensitive_hash_cosine.rs",
    "locality_sensitive_hash_euclidean.rs",
    "lockfree_skip_list_step.rs",
    "log2_u64_fixed.rs",
    "lower_bound_branchless_u32.rs",
    "manhattan_dist_u32x2.rs",
    "mask_from_bool_slice.rs",
    "mask_range_u64.rs",
    "mask_xor_reduce_u64.rs",
    "matrix_mul_simd_f32.rs",
    "matrix_transpose_simd_f32.rs",
    "max_element_branchless_u32.rs",
    "max_flow_edmonds_karp_step.rs",
    "median3_u32.rs",
    "median5_u32.rs",
    "median9_u32.rs",
    "merge_u32_slices_branchless.rs",
    "mersenne_twister_step_simd.rs",
    "metaphone_encode_branchless.rs",
    "metrohash64.rs",
    "min_element_branchless_u32.rs",
    "minhash_u64_k.rs",
    "minimum_spanning_tree_prim_step.rs",
    "minmax_element_branchless_u32.rs",
    "mismatch_branchless_u8.rs",
    "misra_gries_add.rs",
    "modular_add_u64.rs",
    "modular_mul_u64.rs",
    "modular_sub_u64.rs"
]

for f in files:
    path = f"crates/bcinr-logic/src/algorithms/{f}"
    if not os.path.exists(path): continue
    content = open(path).read()
    m = re.search(r"pub fn ([a-zA-Z0-9_]+)\s*\((.*?)\)\s*->\s*(.*?) \{", content)
    if m:
        print(f'"{m.group(1)}": {{ "args": "{m.group(2)}", "ret": "{m.group(3)}" }},')
