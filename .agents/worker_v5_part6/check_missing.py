import json
import os

PART6 = [
    "spookyhash_v2_128", "metrohash64", "siphash_2_4_branchless", "highwayhash_64", "clhash",
    "pearson_hash_u8", "knuth_hash_u64", "fibonacci_hash_u64", "zobrist_hash_64", "perfect_hash_lookup_u32",
    "minhash_u64_k", "hyperloglog_add_u64", "hyperloglog_merge", "count_min_sketch_add", "count_min_sketch_query",
    "bloom_filter_add_u64", "bloom_filter_query_u64", "cuckoo_filter_add_u64", "quotient_filter_add_u64", "t_digest_add_u32",
    "heavy_keepers_add", "space_saving_add", "misra_gries_add", "reservoir_sample_branchless", "weighted_reservoir_sample",
    "consistent_hash_jump_u64", "consistent_hash_maglev", "bloom_filter_intersect", "bloom_filter_union", "hashing_trick_u64",
    "locality_sensitive_hash_euclidean"
]

with open("/Users/sac/bcinr/.agents/worker_v5_part6/all_part6_logic.json", "r", encoding="utf-8") as f:
    data = json.load(f)

missing = [name for name in PART6 if name not in data]
print(f"Missing {len(missing)} algorithms: {missing}")

# Let's search the entire directory for python files containing these missing names
for name in missing:
    print(f"Searching for {name}...")
    for root, dirs, files in os.walk("/Users/sac/bcinr"):
        # skip .git, target, .agents
        if any(p in root for p in [".git", "target", ".agents"]):
            continue
        for file in files:
            if file.endswith(".py") or file.endswith(".rs"):
                path = os.path.join(root, file)
                try:
                    with open(path, "r", encoding="utf-8") as f:
                        content = f.read()
                    if name in content:
                        print(f"  Found {name} in {path}")
                except Exception:
                    pass
