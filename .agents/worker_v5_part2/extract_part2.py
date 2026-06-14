import os
import re

part2_algos = [
    "is_contiguous_mask_u64", "get_mask_boundary_low_u64", "get_mask_boundary_high_u64",
    "bit_matrix_transpose_8x8", "bit_matrix_transpose_64x64", "rank_u128", "select_u128",
    "weight_u64", "delta_swap_u64", "benes_network_u64", "bit_permute_step_u64",
    "compress_bits_u64", "expand_bits_u64", "crossbar_permute_u8x16", "mask_from_bool_slice",
    "bool_slice_from_mask", "bit_permute_identity_64", "is_subset_mask_u64",
    "mask_xor_reduce_u64", "mul_sat_u64", "div_sat_u64", "add_sat_i32", "sub_sat_i32",
    "mul_sat_i32", "abs_diff_u64", "abs_diff_i64", "avg_u64", "avg_ceil_u64",
    "clamp_i64", "lerp_sat_u8", "lerp_sat_u32"
]

# We look through all implement*.py files
files_to_scan = [f for f in os.listdir("/Users/sac/bcinr") if f.endswith(".py") and ("implement" in f or "generate" in f or "dump" in f or "stabilize" in f)]
files_to_scan = sorted(files_to_scan)

for fname in files_to_scan:
    path = os.path.join("/Users/sac/bcinr", fname)
    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
    
    for algo in part2_algos:
        if algo in content:
            idx = content.find(f'"{algo}"')
            if idx == -1:
                idx = content.find(f"'{algo}'")
            if idx != -1:
                print(f"--- Found {algo} in {fname} ---")
                start = max(0, idx - 50)
                end = min(len(content), idx + 2000)
                print(content[start:end])
                print("==================================\n")
