import os
import re

partition1 = [
    "parallel_bits_deposit_u64", "parallel_bits_extract_u64", "blsr_u64", "blsi_u64", "blsmsk_u64",
    "t1mskc_u64", "tzmsk_u64", "bext_u64", "bset_u64", "bclr_u64", "btst_u64", "popcount_u128",
    "reverse_bits_u128", "clmul_u64", "morton_encode_2d_u32", "morton_decode_2d_u32", "morton_encode_3d_u32",
    "gray_encode_u64", "gray_decode_u64", "parity_check_u128", "next_lexicographic_permutation_u64",
    "count_consecutive_set_bits_u64", "find_nth_set_bit_u128", "mask_range_u64", "rotate_left_u64",
    "rotate_right_u64", "funnel_shift_left_u64", "funnel_shift_right_u64", "bit_swap_u64",
    "gather_bits_u64", "scatter_bits_u64"
]

base_dir = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms/"

for algo in partition1:
    path = os.path.join(base_dir, f"{algo}.rs")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Locate pub fn {algo}
    pat = r"pub fn " + algo + r"\((.*?)\) -> u64 \{(.*?)\}"
    m = re.search(pat, content, re.DOTALL)
    if m:
        params, body = m.group(1).strip(), m.group(2).strip()
        print(f"File: {algo}.rs")
        print(f"  Params: {params}")
        # Print first few lines of body
        body_lines = body.split("\n")
        print(f"  Body (first 3 lines):")
        for line in body_lines[:3]:
            print(f"    {line}")
    else:
        # If match failed, let's see why
        print(f"File: {algo}.rs - Match failed!")
        # Let's print the actual function declaration
        idx = content.find(f"pub fn {algo}")
        if idx != -1:
            print("  Decl: " + content[idx:idx+150].replace("\n", " "))
    print("-" * 30)
