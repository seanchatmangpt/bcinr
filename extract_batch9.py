import os
import re

files = [
    "set_symmetric_difference_branchless.rs",
    "set_union_branchless.rs",
    "shear_sort_bitonic_2d.rs",
    "shortest_path_bellman_ford_branchless.rs",
    "shuffle_fisher_yates_branchless.rs",
    "sigmoid_sat_u32.rs",
    "simd_memchr_u8x16.rs",
    "simd_memrchr_u8x16.rs",
    "simd_strstr_branchless.rs",
    "siphash_2_4_branchless.rs",
    "smoothstep_u32.rs",
    "softmax_u32x4.rs",
    "sort_index_u32x8.rs",
    "sort_pairs_u32x4.rs",
    "soundex_encode_branchless.rs",
    "space_saving_add.rs",
    "spatial_hash_u32.rs",
    "split_lines_simd.rs",
    "splitmix64_u64.rs",
    "spookyhash_v2_128.rs",
    "stable_partition_branchless.rs",
    "sub_sat_i32.rs",
    "succinct_bit_vector_rank.rs",
    "succinct_bit_vector_select.rs",
    "suffix_array_step_branchless.rs",
    "suffix_sum_simd_u32x8.rs",
    "t_digest_add_u32.rs",
    "t1mskc_u64.rs",
    "top_k_u32x16.rs",
    "topological_sort_step_branchless.rs",
    "triangle_count_bitset.rs"
]

DIR = "crates/bcinr-logic/src/algorithms/"

for file in files:
    path = os.path.join(DIR, file)
    name = file.replace(".rs", "")
    with open(path, "r") as f:
        content = f.read()
    
    # Extract pub fn implementation
    m = re.search(r"pub fn " + name + r"\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{(.*?)\n\}", content, re.DOTALL)
    if m:
        impl = m.group(1).strip()
        print(f"--- {name} ---")
        print(impl)
        print()
