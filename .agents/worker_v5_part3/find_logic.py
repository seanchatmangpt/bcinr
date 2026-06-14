import os
import re

algos = [
    "norm_u32", "fp_mul_u32_q16", "fp_div_u32_q16", "fp_sqrt_u32_q16", "fp_sin_u32_q16",
    "fp_cos_u32_q16", "fp_atan2_u32_q16", "log2_u64_fixed", "exp2_u64_fixed", "sigmoid_sat_u32",
    "relu_u32", "leaky_relu_u32", "softmax_u32x4", "fast_inverse_sqrt_u32", "gcd_u64_branchless",
    "lcm_u64_branchless", "modular_add_u64", "modular_sub_u64", "modular_mul_u64", "is_prime_u64_branchless",
    "factorial_sat_u32", "binom_sat_u32", "pow_sat_u64", "clamped_scaling_u64", "branchless_signum_i64",
    "copy_sign_i64", "is_finite_fp32_branchless", "is_nan_fp32_branchless", "round_to_nearest_u32", "round_up_u32",
    "round_down_u32"
]

root = "/Users/sac/bcinr"
py_files = [f for f in os.listdir(root) if f.endswith(".py")]

found_maps = {}

for algo in algos:
    found_maps[algo] = []
    # Try finding in inject files and implement files
    for pf in py_files:
        path = os.path.join(root, pf)
        with open(path, "r", errors="ignore") as f:
            content = f.read()
        
        # Look for keys in dictionary
        # like "algo_name" or "algo_name.rs"
        pattern_quote_rs = rf'"{algo}\.rs"\s*:'
        pattern_quote = rf'"{algo}"\s*:'
        pattern_single_quote_rs = rf"'{algo}\.rs'\s*:"
        pattern_single_quote = rf"'{algo}'\s*:"
        
        if (re.search(pattern_quote_rs, content) or 
            re.search(pattern_quote, content) or 
            re.search(pattern_single_quote_rs, content) or 
            re.search(pattern_single_quote, content)):
            found_maps[algo].append(pf)

for algo, files in found_maps.items():
    print(f"{algo}: {files}")
