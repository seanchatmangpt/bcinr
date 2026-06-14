import os
import sys

algos = [
    "norm_u32", "fp_mul_u32_q16", "fp_div_u32_q16", "fp_sqrt_u32_q16", "fp_sin_u32_q16",
    "fp_cos_u32_q16", "fp_atan2_u32_q16", "log2_u64_fixed", "exp2_u64_fixed", "sigmoid_sat_u32",
    "relu_u32", "leaky_relu_u32", "softmax_u32x4", "fast_inverse_sqrt_u32", "gcd_u64_branchless",
    "lcm_u64_branchless", "modular_add_u64", "modular_sub_u64", "modular_mul_u64", "is_prime_u64_branchless",
    "factorial_sat_u32", "binom_sat_u32", "pow_sat_u64", "clamped_scaling_u64", "branchless_signum_i64",
    "copy_sign_i64", "is_finite_fp32_branchless", "is_nan_fp32_branchless", "round_to_nearest_u32", "round_up_u32",
    "round_down_u32"
]

sys.path.append("/Users/sac/bcinr")

import implement_51_100
import implement_batch_3
import implement_batch_7
import implement_batch_8
import implement_batch_9
import inject_batch_4
import inject_batch_5

results = {}

# We will check each imported module and extract maps
modules = [
    ("implement_51_100", implement_51_100),
    ("implement_batch_3", implement_batch_3),
    ("implement_batch_7", implement_batch_7),
    ("implement_batch_8", implement_batch_8),
    ("implement_batch_9", implement_batch_9),
    ("inject_batch_4", inject_batch_4),
    ("inject_batch_5", inject_batch_5),
]

for mod_name, mod in modules:
    for attr in dir(mod):
        val = getattr(mod, attr)
        if isinstance(val, dict):
            # Check if keys are our algos
            for k, v in val.items():
                name_clean = k.replace(".rs", "")
                if name_clean in algos:
                    # Parse the value
                    impl = None
                    ref = None
                    if isinstance(v, tuple):
                        impl = v[0]
                        ref = v[1]
                    elif isinstance(v, dict):
                        impl = v.get("branchless") or v.get("impl")
                        ref = v.get("branchful") or v.get("ref")
                    elif isinstance(v, str):
                        impl = v
                    
                    if impl:
                        is_placeholder = results.get(name_clean, {}).get("impl", "") in ["val ^ aux", ""]
                        if name_clean not in results or is_placeholder:
                            results[name_clean] = {"impl": impl, "ref": ref, "source": mod_name + "." + attr}
                        elif ref and not results[name_clean].get("ref"):
                            results[name_clean]["ref"] = ref

# Print what we found
for name in algos:
    res = results.get(name, {})
    print(f"=== {name} (source: {res.get('source', 'None')}) ===")
    print("IMPLEMENTATION:")
    print(res.get("impl", "None"))
    print("REFERENCE:")
    print(res.get("ref", "None"))
    print("\n")
