import os
import json
import re

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

extracted = {}

for pf in python_files:
    path = os.path.join(root, pf)
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Let's search for dictionaries: IMPLS, LOGIC, LOGIC_MAP, etc.
    # We can also parse the python file content dynamically using exec or ast.
    # Let's do ast parsing to extract dictionaries!
    import ast
    try:
        tree = ast.parse(content)
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id in ["IMPLS", "LOGIC", "LOGIC_MAP", "LOGIC_MAP6", "LOGIC_MAP7", "LOGIC_MAP8", "LOGIC_MAP9", "LOGIC_MAP10"]:
                        # We found a dictionary. Let's convert it to a python dict if possible
                        try:
                            val = ast.literal_eval(node.value)
                            if isinstance(val, dict):
                                for k, v in val.items():
                                    if k in PART6:
                                        if k not in extracted:
                                            extracted[k] = []
                                        extracted[k].append({
                                            "source": pf,
                                            "dict_name": target.id,
                                            "val": v
                                        })
                        except Exception as e:
                            # If literal_eval fails, it might contain complex expressions
                            pass
    except Exception as e:
        print(f"Error parsing {pf}: {e}")

# Let's write the results
with open("/Users/sac/bcinr/.agents/worker_v5_part6/part6_extracted_logic.json", "w", encoding="utf-8") as f:
    json.dump(extracted, f, indent=2)

print(f"Extracted {len(extracted)} entries.")
