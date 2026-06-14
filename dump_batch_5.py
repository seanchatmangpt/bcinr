import os
import re

files = [
    "hilbert_curve_encode_u32.rs", "huffman_decode_table_step.rs", "hyperloglog_add_u64.rs", "hyperloglog_merge.rs", 
    "insertion_sort_branchless_fixed.rs", "internet_checksum_u16.rs", "inverse_permute_u32x8.rs", 
    "is_alphanumeric_simd_u8x16.rs", "is_contiguous_mask_u64.rs", "is_digit_simd_u8x16.rs", 
    "is_finite_fp32_branchless.rs", "is_nan_fp32_branchless.rs", "is_permutation_branchless.rs", 
    "is_prime_u64_branchless.rs", "is_sorted_branchless_u32.rs", "is_space_simd_u8x16.rs", 
    "is_subset_mask_u64.rs", "jaro_winkler_branchless.rs", "json_find_string_escapes_simd.rs", 
    "json_find_structural_simd.rs", "k_independent_hash_gen.rs", "knuth_hash_u64.rs", "lcm_u64_branchless.rs", 
    "lcp_array_step_branchless.rs", "leaky_relu_u32.rs", "leb128_decode_u64.rs", "leb128_encode_u64.rs", 
    "lerp_sat_u32.rs", "lerp_sat_u8.rs", "levenshtein_dist_branchless.rs", "lex_compare_u8_slices_branchless.rs"
]

def get_fn_body(content, start_pos):
    brace_start = content.find('{', start_pos)
    if brace_start == -1: return None, start_pos
    count = 0
    for i in range(brace_start, len(content)):
        if content[i] == '{': count += 1
        elif content[i] == '}':
            count -= 1
            if count == 0: return content[brace_start+1:i], i + 1
    return None, len(content)

for f in files:
    path = os.path.join("crates/bcinr-logic/src/algorithms", f)
    if not os.path.exists(path): continue
    with open(path, 'r') as fp: content = fp.read()
    
    # get signature
    algo = f[:-3]
    match = re.search(r'pub fn ' + algo + r'\((.*?)\)\s*->\s*(.*?)\s*\{', content)
    if match:
        args = match.group(1)
        ret = match.group(2)
        print(f"--- {algo} ---")
        print(f"SIG: {args} -> {ret}")
        
        # get ref body
        ref_match = re.search(r'fn ' + algo + r'_reference\s*\((.*?)\)\s*->\s*(.*?)\s*\{', content)
        if ref_match:
            body, _ = get_fn_body(content, ref_match.start())
            print(f"REF: {body.strip()}")
        else:
            print("REF: NOT FOUND")

