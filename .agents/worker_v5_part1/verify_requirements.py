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

all_ok = True

for algo in partition1:
    path = os.path.join(base_dir, f"{algo}.rs")
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    content = "".join(lines)
    
    line_count = len(lines)
    has_contract = "Branchless Contract" in content
    
    # Check if mutant functions are distinct from reference.
    # Find mutant definitions
    mutants = re.findall(r"fn mutant_" + algo + r"_\d\(.*?\)\s*->\s*u64\s*\{(.*?)\}", content, re.DOTALL)
    
    print(f"{algo}.rs: lines={line_count}, contract={has_contract}, mutants_count={len(mutants)}")
    
    if line_count < 100:
        print(f"  --> Line count too low: {line_count}")
        all_ok = False
    if not has_contract:
        print("  --> Lacks 'Branchless Contract'!")
        all_ok = False

if all_ok:
    print("All file metrics (lines and contract headers) are satisfied!")
else:
    print("Some file metrics are not satisfied.")
