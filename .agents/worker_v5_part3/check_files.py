import os

algos = [
    "norm_u32", "fp_mul_u32_q16", "fp_div_u32_q16", "fp_sqrt_u32_q16", "fp_sin_u32_q16",
    "fp_cos_u32_q16", "fp_atan2_u32_q16", "log2_u64_fixed", "exp2_u64_fixed", "sigmoid_sat_u32",
    "relu_u32", "leaky_relu_u32", "softmax_u32x4", "fast_inverse_sqrt_u32", "gcd_u64_branchless",
    "lcm_u64_branchless", "modular_add_u64", "modular_sub_u64", "modular_mul_u64", "is_prime_u64_branchless",
    "factorial_sat_u32", "binom_sat_u32", "pow_sat_u64", "clamped_scaling_u64", "branchless_signum_i64",
    "copy_sign_i64", "is_finite_fp32_branchless", "is_nan_fp32_branchless", "round_to_nearest_u32", "round_up_u32",
    "round_down_u32"
]

algo_dir = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"

for algo in algos:
    path = os.path.join(algo_dir, f"{algo}.rs")
    if not os.path.exists(path):
        print(f"{algo}: MISSING FILE")
        continue
    
    with open(path, "r") as f:
        lines = f.readlines()
    
    content = "".join(lines)
    has_contract = "Branchless Contract" in content
    line_count = len(lines)
    
    # Check if implementation body is dummy or placeholder
    # For example, checking if the function body contains '0x9E3779B97F4A7C15' or '^' / 'wrapping_add'
    is_dummy = "0x9E3779B97F4A7C15" in content or "val ^ aux" in content
    
    print(f"{algo:30} | lines: {line_count:4} | contract: {str(has_contract):5} | dummy: {str(is_dummy):5}")
