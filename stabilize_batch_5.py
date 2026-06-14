import os
import re
import subprocess

files = [
    "hilbert_curve_encode_u32.rs", "huffman_decode_table_step.rs", "hyperloglog_add_u64.rs",
    "hyperloglog_merge.rs", "insertion_sort_branchless_fixed.rs", "internet_checksum_u16.rs",
    "inverse_permute_u32x8.rs", "is_alphanumeric_simd_u8x16.rs", "is_contiguous_mask_u64.rs",
    "is_digit_simd_u8x16.rs", "is_finite_fp32_branchless.rs", "is_nan_fp32_branchless.rs",
    "is_permutation_branchless.rs", "is_prime_u64_branchless.rs", "is_sorted_branchless_u32.rs",
    "is_space_simd_u8x16.rs", "is_subset_mask_u64.rs", "jaro_winkler_branchless.rs",
    "json_find_string_escapes_simd.rs", "json_find_structural_simd.rs", "k_independent_hash_gen.rs",
    "knuth_hash_u64.rs", "lcm_u64_branchless.rs", "lcp_array_step_branchless.rs",
    "leaky_relu_u32.rs", "leb128_decode_u64.rs", "leb128_encode_u64.rs", "lerp_sat_u32.rs",
    "lerp_sat_u8.rs", "levenshtein_dist_branchless.rs", "lex_compare_u8_slices_branchless.rs"
]

base_dir = "crates/bcinr-logic/src/algorithms"

for f in files:
    path = os.path.join(base_dir, f)
    with open(path, "r") as fp:
        content = fp.read()
    
    # 2. Replace standard operators (+, -, *) with wrapping equivalents.
    # Note: they might already be wrapping equivalents in some files, but we'll apply regex.
    # To be safe and not mess up strings or comments, we could parse but simple regex for typical rust binary ops:
    # A bit risky: a + b -> a.wrapping_add(b)
    # We will specifically target val + aux or similar in the logic.
    # Actually, we can look for basic occurrences of operators.
    content = re.sub(r'(\b\w+)\s*\+\s*(\b\w+)', r'\1.wrapping_add(\2)', content)
    content = re.sub(r'(\b\w+)\s*\-\s*(\b\w+)', r'\1.wrapping_sub(\2)', content)
    content = re.sub(r'(\b\w+)\s*\*\s*(\b\w+)', r'\1.wrapping_mul(\2)', content)
    
    # 5. Eliminate any JCC (while, for, if).
    # Specifically the if in mutants:
    # if val != aux && val != 0 && aux != 0 {
    #     prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
    # }
    pattern = r'if\s+(.*?)\s*\{\s*prop_assert!\((.*?),\s*("[^"]+")\);\s*\}'
    replacement = r'prop_assert!(!(\1) || (\2), \3);'
    content = re.sub(pattern, replacement, content)
    
    with open(path, "w") as fp:
        fp.write(content)

# 4. Run 'cargo test' for each to verify.
for f in files:
    mod_name = f.replace(".rs", "")
    print(f"Testing {mod_name}...")
    res = subprocess.run(["cargo", "test", "-p", "bcinr-logic", "--lib", f"algorithms::{mod_name}::", "--", "--quiet"], capture_output=True)
    if res.returncode != 0:
        print(f"{mod_name} failed!")
        print(res.stdout.decode())
    else:
        print(f"{mod_name} passed!")
