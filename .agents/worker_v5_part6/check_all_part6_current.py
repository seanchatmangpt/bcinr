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

root_dir = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"
status = {}

for name in PART6:
    path = os.path.join(root_dir, f"{name}.rs")
    if not os.path.exists(path):
        status[name] = "missing"
        continue
    
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    lines = content.split("\n")
    has_contract = "Branchless Contract" in content
    
    # Check reference function body for dummy hashes
    # Let's locate the reference function
    ref_body_lines = []
    in_ref = False
    brace_count = 0
    for line in lines:
        if f"fn {name}_reference" in line or f"fn reference_impl" in line:
            in_ref = True
            brace_count = line.count("{") - line.count("}")
            ref_body_lines.append(line)
            continue
        if in_ref:
            ref_body_lines.append(line)
            brace_count += line.count("{") - line.count("}")
            if brace_count <= 0:
                in_ref = False
                
    ref_body = "\n".join(ref_body_lines)
    is_dummy = "val ^ aux" in ref_body or "val == aux" in ref_body or ref_body.strip() == ""
    
    status[name] = {
        "lines": len(lines),
        "has_contract": has_contract,
        "is_dummy": is_dummy
    }

print(json.dumps(status, indent=2))
