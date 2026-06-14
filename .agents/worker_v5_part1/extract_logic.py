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

mapping = {}

# We'll read generate_real_algorithms.py first
if os.path.exists("generate_real_algorithms.py"):
    with open("generate_real_algorithms.py", "r", encoding="utf-8") as f:
        content = f.read()
    # Find LOGIC_MAP = { ... }
    # Let's extract LOGIC_MAP by matching keys
    for algo in partition1:
        # Match "algo": ( "impl", "ref" )
        pattern = rf'"{algo}":\s*\(\s*"(.*?)"\s*,\s*"(.*?)"\s*\)'
        m = re.search(pattern, content)
        if m:
            impl, ref = m.group(1), m.group(2)
            # Unescape strings
            impl = impl.encode().decode('unicode-escape')
            ref = ref.encode().decode('unicode-escape')
            if "val ^ aux" not in impl:
                mapping[algo] = (impl, ref, "generate_real_algorithms.py")

# Look in other files:
files = [f for f in os.listdir(".") if f.startswith("implement_") and f.endswith(".py")]

for fname in sorted(files):
    fpath = os.path.join(".", fname)
    with open(fpath, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Let's check if the file is one of implement_1_30.py style (which has algos_data = [ (name, impl, ref), ... ])
    # We can match: ("name", """impl""", """ref""") or ("name", "impl", "ref")
    # Let's do a search for the algorithm name
    for algo in partition1:
        if algo not in content:
            continue
        
        # Let's write a parser using regex
        # Look for: ( "algo" or 'algo', ... )
        # E.g. ( "algo", """impl""", """ref""" )
        pattern = rf'\(\s*["\']{algo}["\']\s*,\s*["\']{{3}}(.*?)["\']{{3}}\s*,\s*["\']{{3}}(.*?)["\']{{3}}\s*\)'
        m = re.search(pattern, content, re.DOTALL)
        if m:
            impl, ref = m.group(1), m.group(2)
            if "val ^ aux" not in impl:
                mapping[algo] = (impl, ref, fname)
            continue
        
        # Try without triple quotes
        pattern_single = rf'\(\s*["\']{algo}["\']\s*,\s*["\'](.*?)["\']\s*,\s*["\'](.*?)["\']\s*\)'
        m = re.search(pattern_single, content)
        if m:
            impl, ref = m.group(1), m.group(2)
            if "val ^ aux" not in impl:
                mapping[algo] = (impl, ref, fname)
            continue

        # In implement_batch_X.py style:
        # elif algo == "name":
        #     impl = """..."""
        #     ref = """..."""
        # or similar. Let's search for:
        # elif algo == "name":
        #    ...
        #    impl = ...
        #    ref = ...
        if f'algo == "{algo}"' in content or f"algo == '{algo}'" in content:
            # Let's extract the block
            block_pattern = rf'elif\s+algo\s*==\s*["\']{algo}["\'].*?(?=elif|def|if\s+__name__|$)'
            block_m = re.search(block_pattern, content, re.DOTALL)
            if block_m:
                block = block_m.group(0)
                impl_m = re.search(r'impl\s*=\s*(?:"""(.*?)"""|"(.*?)")', block, re.DOTALL)
                ref_m = re.search(r'ref\s*=\s*(?:"""(.*?)"""|"(.*?)")', block, re.DOTALL)
                if impl_m and ref_m:
                    impl = impl_m.group(1) or impl_m.group(2)
                    ref = ref_m.group(1) or ref_m.group(2)
                    if "val ^ aux" not in impl:
                        mapping[algo] = (impl, ref, fname)

# Print what we found
print(f"Found {len(mapping)} algorithms in map.")
for algo in partition1:
    if algo in mapping:
        print(f"=== {algo} ({mapping[algo][2]}) ===")
        print("--- IMPL ---")
        print(mapping[algo][0].strip())
        print("--- REF ---")
        print(mapping[algo][1].strip())
        print()
    else:
        print(f"!!! {algo} NOT FOUND !!!")
