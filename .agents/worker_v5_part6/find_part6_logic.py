import os
import json

PART6 = [
    "spookyhash_v2_128", "metrohash64", "siphash_2_4_branchless", "highwayhash_64", "clhash",
    "pearson_hash_u8", "knuth_hash_u64", "fibonacci_hash_u64", "zobrist_hash_64", "perfect_hash_lookup_u32",
    "minhash_u64_k", "hyperloglog_add_u64", "hyperloglog_merge", "count_min_sketch_add", "count_min_sketch_query",
    "bloom_filter_add_u64", "bloom_filter_query_u64", "cuckoo_filter_add_u64", "quotient_filter_add_u64", "t_digest_add_u32",
    "heavy_keepers_add", "space_saving_add", "misra_gries_add", "reservoir_sample_branchless", "weighted_reservoir_sample",
    "consistent_hash_jump_u64", "consistent_hash_maglev", "bloom_filter_intersect", "bloom_filter_union", "hashing_trick_u64",
    "locality_sensitive_hash_euclidean"
]

root = "/Users/sac/bcinr"
python_files = [f for f in os.listdir(root) if f.startswith("implement_") and f.endswith(".py")]

results = {}

for name in PART6:
    results[name] = []
    for pf in python_files:
        path = os.path.join(root, pf)
        with open(path, "r", encoding="utf-8") as f:
            content = f.read()
        if name in content:
            results[name].append(pf)

print(json.dumps(results, indent=2))
