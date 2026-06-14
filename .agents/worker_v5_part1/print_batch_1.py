import json

with open("/Users/sac/bcinr/.agents/worker_v5_part1/replacements_final.json", "r") as f:
    data = json.load(f)

batch = [
    "parallel_bits_deposit_u64", "parallel_bits_extract_u64", "blsr_u64", "blsi_u64", "blsmsk_u64",
    "t1mskc_u64", "tzmsk_u64", "bext_u64", "bset_u64", "bclr_u64"
]

for name in batch:
    entry = data[name]
    # If the file has a doc_target and we want to include it, let's read the file content
    # and find the contiguous range from doc_target start to target end.
    file_path = entry["file"]
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    
    target = entry["target"]
    replacement = entry["replacement"]
    
    if entry["doc_target"] and entry["doc_target"] in content:
        doc_idx = content.find(entry["doc_target"])
        target_idx = content.find(target)
        if doc_idx != -1 and target_idx != -1 and doc_idx < target_idx:
            # We can replace the contiguous range from doc_idx to target_idx + len(target)
            full_target = content[doc_idx : target_idx + len(target)]
            # Construct the full replacement:
            # First, the doc comment part:
            doc_part = content[doc_idx:target_idx].replace(entry["doc_target"], entry["doc_replacement"])
            full_replacement = doc_part + replacement
            target = full_target
            replacement = full_replacement
    
    # Let's count line numbers
    lines = content.splitlines(keepends=True)
    
    # Find start and end line of the target
    # We find start index in content
    start_char_idx = content.find(target)
    if start_char_idx == -1:
        print(f"ERROR: Target not found in {name}")
        continue
    
    end_char_idx = start_char_idx + len(target)
    
    # Convert char indices to 1-based line numbers
    start_line = content[:start_char_idx].count("\n") + 1
    end_line = content[:end_char_idx].count("\n") + 1
    
    print(f"// --- {name} ---")
    print(f"TargetFile: {file_path}")
    print(f"StartLine: {start_line}")
    print(f"EndLine: {end_line}")
    print("TargetContent:")
    print(repr(target))
    print("ReplacementContent:")
    print(repr(replacement))
    print()
