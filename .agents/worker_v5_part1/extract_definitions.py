import os
import json
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

results = {}

# We'll search in all python files matching implement_*.py or generate_real_algorithms.py
files = [f for f in os.listdir("/Users/sac/bcinr/") if f.startswith("implement_") and f.endswith(".py")]
files.append("generate_real_algorithms.py")
files.append("implement_batch_2.py")
files.append("implement_batch_3.py")
files.append("implement_batch_6.py")
files.append("implement_batch_7.py")
files.append("implement_batch_8.py")
files.append("implement_batch_9.py")

for fname in sorted(list(set(files))):
    fpath = os.path.join("/Users/sac/bcinr/", fname)
    if not os.path.exists(fpath):
        continue
    with open(fpath, "r", encoding="utf-8") as f:
        content = f.read()
    
    for algo in partition1:
        if algo not in content:
            continue
        if algo not in results:
            results[algo] = []
        results[algo].append(fname)

print(json.dumps(results, indent=2))
