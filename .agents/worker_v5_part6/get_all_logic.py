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
python_files = [f for f in os.listdir(root) if (f.startswith("implement_") or f.startswith("refine_") or f.startswith("fix_")) and f.endswith(".py")]

extracted = {}

# We'll scan each file
for pf in python_files:
    path = os.path.join(root, pf)
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Let's extract dictionaries first
    import ast
    try:
        tree = ast.parse(content)
        for node in ast.walk(tree):
            # 1. Dictionary mapping
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id in ["IMPLS", "LOGIC", "LOGIC_MAP", "LOGIC_MAP6", "LOGIC_MAP7", "LOGIC_MAP8", "LOGIC_MAP9", "LOGIC_MAP10", "impls", "algos", "ALGO_DATA", "files_impls", "files_to_fix"]:
                        try:
                            val = ast.literal_eval(node.value)
                            if isinstance(val, dict):
                                for k, v in val.items():
                                    clean_k = k.replace(".rs", "")
                                    if clean_k in PART6:
                                        if clean_k not in extracted:
                                            extracted[clean_k] = []
                                        extracted[clean_k].append({
                                            "source": pf,
                                            "type": "dict",
                                            "val": v
                                        })
                        except Exception:
                            pass
            
            # 2. List of tuples (e.g. in implement_batch_2.py or similar)
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id in ["ALGORITHMS", "algos", "algorithms"]:
                        try:
                            val = ast.literal_eval(node.value)
                            if isinstance(val, list):
                                for item in val:
                                    if isinstance(item, tuple) and len(item) >= 3:
                                        name = item[0]
                                        if name in PART6:
                                            if name not in extracted:
                                                extracted[name] = []
                                            extracted[name].append({
                                                "source": pf,
                                                "type": "tuple",
                                                "val": item
                                            })
                        except Exception:
                            pass
    except Exception as e:
        print(f"AST Error in {pf}: {e}")

    # Now let's scan for if/elif blocks using regex or line parsing
    lines = content.split("\n")
    for idx, line in enumerate(lines):
        m = re.search(r'(?:if|elif)\s+(?:algo|algo_name)\s*==\s*["\']([^"\']+)["\']', line)
        if m:
            name = m.group(1)
            if name in PART6:
                # We found a block! Let's collect lines until next if/elif or end of function/dedent
                block_lines = []
                j = idx + 1
                while j < len(lines):
                    next_line = lines[j]
                    if re.match(r'^\s*(?:if|elif|def|class)\s+', next_line) and not next_line.startswith(" " * (len(line) - len(line.lstrip()) + 4)):
                        # Dedent or sibling if/elif
                        if not ("impl_body" in next_line or "ref_body" in next_line or "return" in next_line):
                            break
                    block_lines.append(next_line)
                    j += 1
                
                block_text = "\n".join(block_lines)
                if name not in extracted:
                    extracted[name] = []
                extracted[name].append({
                    "source": pf,
                    "type": "conditional_block",
                    "text": block_text
                })

# Write the final results
with open("/Users/sac/bcinr/.agents/worker_v5_part6/all_part6_logic.json", "w", encoding="utf-8") as f:
    json.dump(extracted, f, indent=2)

print(f"Done. Extracted {len(extracted)} algorithms logic.")
