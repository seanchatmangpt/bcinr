import os
import json
import glob

CRITERION_DIR = "/Users/sac/bcinr/target/criterion"

FAMILIES = {
    "Bit Manipulation": [
        "bclr_u64", "bext_u64", "blsi_u64", "blsmsk_u64", "blsr_u64", "bset_u64", "btst_u64",
        "abs_diff_u64", "add_sat_i32", "avg_u64", "avg_ceil_u64", "binom_sat_u32", "bit_swap_u64",
        "clmul_u64", "compress_bits_u64", "expand_bits_u64", "count_consecutive_set_bits_u64",
        "bzhi_u64", "popcount_u64", "bit_reverse_u64", "parallel_bits_deposit_u64", "parallel_bits_extract_u64"
    ],
    "Sorting Networks": [
        "benes_network_u64", "bit_parallel_sort8_u32", "bitonic_merge_u64x8", "bitonic_sort_64u32",
        "counting_sort_branchless_u8", "odd_even_merge_sort_16u32", "green_sorting_network_16",
        "shear_sort_bitonic_2d", "rank_select_sort_u32"
    ],
    "Hashing & Sketching": [
        "adler32_branchless", "cityhash64", "clhash", "count_min_sketch_add", "count_min_sketch_query",
        "cuckoo_filter_add_u64", "farmhash64", "fibonacci_hash_u64", "fletcher32_branchless",
        "murmur3_x64_128", "xxhash64", "xxh3_64", "spookyhash_v2_128", "metrohash64",
        "siphash_2_4_branchless", "highwayhash_64"
    ]
}

def get_estimate(bench_name):
    path = os.path.join(CRITERION_DIR, bench_name, "new", "estimates.json")
    if not os.path.exists(path):
        return None
    try:
        with open(path, 'r') as f:
            data = json.load(f)
            return data["mean"]["point_estimate"]
    except Exception:
        return None

results = []

for family, primitives in FAMILIES.items():
    family_results = []
    for primitive in primitives:
        # Check _avg first
        bench_name = f"{primitive}_avg"
        val = get_estimate(bench_name)
        if val is not None:
            family_results.append((primitive, val))
        else:
            # Try without _avg if it's not found (some might not have it)
            val = get_estimate(primitive)
            if val is not None:
                family_results.append((primitive, val))
    
    results.append((family, family_results))

# Print family summaries
for family, family_results in results:
    print(f"## {family}")
    if not family_results:
        print("No results found.")
    else:
        family_results.sort(key=lambda x: x[1])
        for prim, val in family_results:
            # Convert to ns or ps
            if val < 1:
                print(f"- {prim}: {val * 1000:.2f} ps")
            else:
                print(f"- {prim}: {val:.2f} ns")
    print()

# Top 10 fastest overall
all_flat = []
for family, family_results in results:
    all_flat.extend(family_results)

all_flat.sort(key=lambda x: x[1])
print("## Top 10 Fastest Primitives")
for prim, val in all_flat[:10]:
    if val < 1:
        print(f"| {prim} | {val * 1000:.2f} ps |")
    else:
        print(f"| {prim} | {val:.2f} ns |")
