import json

with open("/Users/sac/bcinr/.agents/worker_v5_part1/replacements_final.json", "r") as f:
    data = json.load(f)

for name in sorted(data.keys()):
    entry = data[name]
    file_path = entry["file"]
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    
    target = entry["target"]
    replacement = entry["replacement"]
    
    if entry["doc_target"] and entry["doc_target"] in content:
        doc_idx = content.find(entry["doc_target"])
        target_idx = content.find(target)
        if doc_idx != -1 and target_idx != -1 and doc_idx < target_idx:
            full_target = content[doc_idx : target_idx + len(target)]
            doc_part = content[doc_idx:target_idx].replace(entry["doc_target"], entry["doc_replacement"])
            full_replacement = doc_part + replacement
            target = full_target
            replacement = full_replacement
            
    start_char_idx = content.find(target)
    if start_char_idx == -1:
        print(f"// ERROR: Target not found in {name}")
        continue
    end_char_idx = start_char_idx + len(target)
    start_line = content[:start_char_idx].count("\n") + 1
    end_line = content[:end_char_idx].count("\n") + 1
    
    print(f"File: {name}")
    print(f"TargetFile: {file_path}")
    print(f"StartLine: {start_line}")
    print(f"EndLine: {end_line}")
    print(f"TargetContent: \"\"\"{target}\"\"\"")
    print(f"ReplacementContent: \"\"\"{replacement}\"\"\"")
    print("=" * 60)
