import json
import sys

batch_num = int(sys.argv[1])
with open("/Users/sac/bcinr/.agents/worker_v5_part9/replace_payloads.json", "r") as f:
    payloads = json.load(f)

sorted_keys = sorted(payloads.keys())

# Split into 4 batches of size 8, 8, 8, 7
batches = [
    sorted_keys[0:8],
    sorted_keys[8:16],
    sorted_keys[16:24],
    sorted_keys[24:31]
]

selected_keys = batches[batch_num - 1]

out_path = f"/Users/sac/bcinr/.agents/worker_v5_part9/batch{batch_num}_calls.txt"
with open(out_path, "w") as out:
    for name in selected_keys:
        p = payloads[name]
        call_str = f"""TargetFile: {p['TargetFile']}
StartLine: {p['StartLine']}
EndLine: {p['EndLine']}
TargetContent:
{p['TargetContent']}
ReplacementContent:
{p['ReplacementContent']}
"""
        out.write(f"=== {name} ===\n")
        out.write(call_str)
        out.write("\n" + "="*80 + "\n")

print(f"Batch {batch_num} tool calls written to {out_path}")
