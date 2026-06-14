import os

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

for name in part2_algos:
    path = f"/Users/sac/bcinr/crates/bcinr-logic/src/algorithms/{name}.rs"
    if not os.path.exists(path):
        print(f"FILE MISSING: {name}")
        continue
    
    with open(path, "r") as f:
        lines = f.readlines()
    
    # Extract pub fn {name} body
    impl_body = []
    in_impl = False
    brace_count = 0
    for line in lines:
        if f"pub fn {name}" in line:
            in_impl = True
            brace_count = line.count("{") - line.count("}")
            impl_body.append(line)
            continue
        if in_impl:
            impl_body.append(line)
            brace_count += line.count("{") - line.count("}")
            if brace_count == 0:
                in_impl = False
    
    # Extract fn {name}_reference body
    ref_body = []
    in_ref = False
    for line in lines:
        if f"fn {name}_reference" in line:
            in_ref = True
            brace_count = line.count("{") - line.count("}")
            ref_body.append(line)
            continue
        if in_ref:
            ref_body.append(line)
            brace_count += line.count("{") - line.count("}")
            if brace_count == 0:
                in_ref = False

    print(f"ALGORITHM: {name}")
    print("IMPLEMENTATION:")
    print("".join(impl_body).strip())
    print("REFERENCE:")
    print("".join(ref_body).strip())
    print("=" * 60)
