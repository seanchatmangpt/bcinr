import json
import os
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

# Load already extracted json
with open("/Users/sac/bcinr/.agents/worker_v5_part6/all_part6_logic.json", "r", encoding="utf-8") as f:
    extracted = json.load(f)

# Also let's extract extra files manually or from refine_all_batches.py, etc.
# We'll read from refine_all_batches.py
refine_path = "/Users/sac/bcinr/refine_all_batches.py"
refine_algos = {}
if os.path.exists(refine_path):
    with open(refine_path, "r") as f:
        content = f.read()
    # Find the ALGOS dict
    import ast
    try:
        tree = ast.parse(content)
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if isinstance(target, ast.Name) and target.id == "ALGOS":
                        val = ast.literal_eval(node.value)
                        if isinstance(val, dict):
                            refine_algos = val
    except Exception as e:
         print(f"Error reading refine: {e}")

# We'll map each algorithm to its final implementation and reference
final_specs = {}

for name in PART6:
    impl = None
    ref = None
    
    # Check if we have extracted it in all_part6_logic.json
    if name in extracted:
        for entry in extracted[name]:
            source = entry["source"]
            # prioritize genuine ones (not implement_101_200.py)
            if source == "implement_101_200.py":
                continue
            
            t = entry["type"]
            val = entry.get("val")
            text = entry.get("text")
            
            if t == "dict":
                if isinstance(val, dict):
                    impl = val.get("branchless")
                    ref = val.get("branchful")
                elif isinstance(val, list) or isinstance(val, tuple):
                    impl = val[0]
                    ref = val[1]
            elif t == "tuple":
                impl = val[2]
                ref = val[3]
            elif t == "conditional_block":
                m_impl = re.search(r'impl_body\s*=\s*"""(.*?)"""', text, re.DOTALL)
                m_ref = re.search(r'ref_body\s*=\s*"""(.*?)"""', text, re.DOTALL)
                if m_impl:
                    impl = m_impl.group(1).strip()
                if m_ref:
                    ref = m_ref.group(1).strip()
                else:
                    m_ret = re.search(r'return\s*"""(.*?)"""', text, re.DOTALL)
                    if m_ret:
                        impl = m_ret.group(1).strip()
                        ref = impl
            
            if impl and ref:
                break
                
    # Check refine_algos
    if (not impl or not ref) and name in refine_algos:
        entry = refine_algos[name]
        impl = entry.get("branchless")
        ref = entry.get("reference")
        
    final_specs[name] = {"impl": impl, "ref": ref}

# Manual overrides and fixes
final_specs["heavy_keepers_add"] = {
    "impl": "    let idx = (aux & 0xF) * 4;\n    let count = (val >> idx) & 0xF;\n    let is_not_max = (count < 15) as u64;\n    let new_count = count + is_not_max;\n    (val & !(0xF << idx)) | (new_count << idx)",
    "ref": "    let idx = (aux & 0xF) * 4;\n    let count = (val >> idx) & 0xF;\n    let new_count = if count < 15 { count + 1 } else { 15 };\n    (val & !(0xF << idx)) | (new_count << idx)"
}

final_specs["hashing_trick_u64"] = {
    "impl": "    let mut x = val;\n    x = x.wrapping_mul(0xbf58476d1ce4e5b9);\n    x = (x ^ (x >> 30)).wrapping_mul(0x94d049bb133111eb);\n    x = (x ^ (x >> 27)).wrapping_mul(0xff51afd7ed558ccd);\n    x = x ^ (x >> 33);\n    x % (aux | 1)",
    "ref": "    let mut x = val;\n    x = x.wrapping_mul(0xbf58476d1ce4e5b9);\n    x = (x ^ (x >> 30)).wrapping_mul(0x94d049bb133111eb);\n    x = (x ^ (x >> 27)).wrapping_mul(0xff51afd7ed558ccd);\n    x = x ^ (x >> 33);\n    x % (aux | 1)"
}

# Ensure all nulls or category-specific bluffs are replaced with correct reference logic
# Let's inspect other algos
# knuth_hash_u64 is v.wrapping_mul(...)
final_specs["knuth_hash_u64"] = {
    "impl": "    val.wrapping_mul(11400714819323198485) ^ aux",
    "ref": "    val.wrapping_mul(11400714819323198485) ^ aux"
}

# fibonacci_hash_u64
final_specs["fibonacci_hash_u64"] = {
    "impl": "    val.wrapping_mul(11400714819323198485)",
    "ref": "    val.wrapping_mul(11400714819323198485)"
}

# zobrist_hash_64: Zobrist hashing is XORing array elements. Here it's a step.
# val is the current hash, aux is the piece/square key.
final_specs["zobrist_hash_64"] = {
    "impl": "    val ^ aux",
    "ref": "    val ^ aux"
}

# hyperloglog_add_u64
final_specs["hyperloglog_add_u64"] = {
    "impl": "    let r = (val.wrapping_shr(aux as u32)).leading_zeros() + 1; r as u64",
    "ref": "    let r = (val.wrapping_shr(aux as u32)).leading_zeros() + 1; r as u64"
}

# hyperloglog_merge
final_specs["hyperloglog_merge"] = {
    "impl": "    val.max(aux)",
    "ref": "    if val > aux { val } else { aux }"
}

# count_min_sketch_add
final_specs["count_min_sketch_add"] = {
    "impl": "    val.wrapping_add((aux.wrapping_mul(0x9E3779B97F4A7C15u64) >> 48) | (aux.wrapping_mul(0x85EBCA6B00000000u64) & 0xFFFF000000000000u64))",
    "ref": "    val.wrapping_add((aux.wrapping_mul(0x9E3779B97F4A7C15u64) >> 48) | (aux.wrapping_mul(0x85EBCA6B00000000u64) & 0xFFFF000000000000u64))"
}

# count_min_sketch_query
final_specs["count_min_sketch_query"] = {
    "impl": "    let c1 = val & 0xFFFF; let c2 = (val >> 16) & 0xFFFF; let c3 = (val >> 32) & 0xFFFF; let c4 = (val >> 48) & 0xFFFF; (c1.min(c2)).min(c3.min(c4))",
    "ref": "    let c1 = val & 0xFFFF; let c2 = (val >> 16) & 0xFFFF; let c3 = (val >> 32) & 0xFFFF; let c4 = (val >> 48) & 0xFFFF; (c1.min(c2)).min(c3.min(c4))"
}

# cuckoo_filter_add_u64
final_specs["cuckoo_filter_add_u64"] = {
    "impl": "    val ^ (aux.wrapping_mul(0x9E3779B97F4A7C15u64))",
    "ref": "    val ^ (aux.wrapping_mul(0x9E3779B97F4A7C15u64))"
}

# t_digest_add_u32
final_specs["t_digest_add_u32"] = {
    "impl": "    let w1 = val & 0xFFFFFFFF; let c1 = val >> 32;\n    let w2 = aux & 0xFFFFFFFF; let c2 = aux >> 32;\n    let w_sum = w1.wrapping_add(w2);\n    let c_sum = c1.wrapping_add(c2);\n    w_sum | (c_sum << 32)",
    "ref": "    let w1 = val & 0xFFFFFFFF; let c1 = val >> 32;\n    let w2 = aux & 0xFFFFFFFF; let c2 = aux >> 32;\n    (w1.wrapping_add(w2)) | (c1.wrapping_add(c2) << 32)"
}

with open("/Users/sac/bcinr/.agents/worker_v5_part6/final_part6_logic.json", "w", encoding="utf-8") as f:
    json.dump(final_specs, f, indent=2)

print(json.dumps(final_specs, indent=2))
