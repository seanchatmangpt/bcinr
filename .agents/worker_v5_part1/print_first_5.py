import json

with open("/Users/sac/bcinr/.agents/worker_v5_part1/replacements_final.json", "r") as f:
    data = json.load(f)

first_5 = [
    "parallel_bits_deposit_u64", "parallel_bits_extract_u64", "blsr_u64", "blsi_u64", "blsmsk_u64"
]

for name in first_5:
    entry = data[name]
    print(f"=== {name} ===")
    print("FILE:", entry["file"])
    print("--- TARGET ---")
    print(entry["target"])
    print("--- REPLACEMENT ---")
    print(entry["replacement"])
    if entry["doc_target"]:
        print("--- DOC TARGET ---")
        print(entry["doc_target"])
        print("--- DOC REPLACEMENT ---")
        print(entry["doc_replacement"])
    print("=" * 60)
