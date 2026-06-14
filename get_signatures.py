import os
import re

files = [
    "find_last_of_branchless.rs", "find_nth_set_bit_u128.rs", "fixed_point_log2.rs", "fletcher32_branchless.rs",
    "fp_atan2_u32_q16.rs", "fp_cos_u32_q16.rs", "fp_div_u32_q16.rs", "fp_mul_u32_q16.rs", "fp_sin_u32_q16.rs",
    "fp_sqrt_u32_q16.rs", "frustum_culling_branchless.rs", "funnel_shift_left_u64.rs", "funnel_shift_right_u64.rs",
    "gather_bits_u64.rs", "gaussian_noise_box_muller.rs", "gcd_u64_branchless.rs", "get_mask_boundary_high_u64.rs",
    "get_mask_boundary_low_u64.rs", "graph_bfs_simd_step.rs", "graph_dfs_bit_parallel.rs", "gray_decode_u64.rs",
    "gray_encode_u64.rs", "green_sorting_network_16.rs", "halton_sampler_simd.rs", "halton_sequence_u32.rs",
    "hamming_dist_simd.rs", "hashing_trick_u64.rs", "hazard_pointer_retire.rs", "heavy_keepers_add.rs",
    "hex_decode_simd.rs"
]

base_path = "crates/bcinr-logic/src/algorithms/"

for f in files:
    path = os.path.join(base_path, f)
    with open(path, 'r') as fd:
        content = fd.read()
        match = re.search(r'pub fn (\w+)\((.*?)\) -> (.*?) \{', content)
        if match:
            print(f"{f}: {match.group(1)}({match.group(2)}) -> {match.group(3)}")
        else:
            print(f"{f}: NOT FOUND")
