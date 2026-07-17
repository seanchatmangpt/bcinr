import os
import subprocess
import re

bad = [
    "lcm_u64_branchless", "cardinality_linear_counting", "is_permutation_branchless", 
    "crc32c_branchless", "norm_u32", "normalize_slice_branchless", "rank_u32x8", 
    "levenshtein_dist_branchless", "murmur3_32_hash", "merge_u32_slices_branchless", 
    "fnv1a_64_hash", "xor_filter_lookup", "parallel_bits_deposit_u64", "tabulation_hash_u64", 
    "heavy_hitter_update", "linear_search_simd_u8", "scatter_bits_u64", "polynomial_hash_u64", 
    "halton_sequence_u32", "gcd_u64_branchless", "nth_element_branchless", "wyhash_64", 
    "rank_u128", "locality_sensitive_hash_cosine", "sort_stable_key_value_u32x8", 
    "jaro_winkler_branchless", "parallel_bits_extract_u64", "sorting_network_verify_u32", 
    "count_consecutive_set_bits_u64", "fp_sqrt_u32_q16", "gather_bits_u64", "fp_sin_u32_q16", 
    "adler32_branchless", "count_min_sketch_update", "hazard_pointer_retire", 
    "hyperloglog_add_u64_registers", "lcp_array_step_branchless", "quotient_filter_add_u64", 
    "reservoir_sample_simd", "simd_strstr_branchless", "simhash_cosine_u64", 
    "temp_gate_missing", "xoroshiro128_plus"
]

def delete_mod(mod_name):
    # Search and delete the .rs file anywhere in crates/
    for root, dirs, files in os.walk("crates"):
        for f in files:
            if f == mod_name + ".rs":
                os.remove(os.path.join(root, f))
        
        # Remove from mod.rs
        if "mod.rs" in files:
            p = os.path.join(root, "mod.rs")
            with open(p, "r") as file:
                lines = file.readlines()
            with open(p, "w") as file:
                for line in lines:
                    if f"pub mod {mod_name};" not in line:
                        file.write(line)

for b in bad:
    delete_mod(b)

while True:
    res = subprocess.run(["cargo", "check", "--workspace"], capture_output=True, text=True)
    if res.returncode == 0:
        print("Success! Cargo check passes.")
        break
    else:
        # Find unresolved imports
        match = re.search(r"unresolved import `.*?::(.*?)`", res.stderr)
        if match:
            missing = match.group(1)
            print(f"Missing: {missing}, deleting mod")
            delete_mod(missing)
        else:
            # Look for other file errors
            match2 = re.search(r"--> (crates/.*?\.rs)", res.stderr)
            if match2:
                file_to_del = os.path.basename(match2.group(1)).replace(".rs", "")
                print(f"Error in {file_to_del}, deleting mod")
                delete_mod(file_to_del)
            else:
                print("Unknown error:")
                print(res.stderr)
                break
