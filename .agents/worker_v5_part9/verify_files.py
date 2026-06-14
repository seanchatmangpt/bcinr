import os

out_dir = "/Users/sac/bcinr/.agents/worker_v5_part9/new_rs_files"
files = sorted(os.listdir(out_dir))

all_ok = True
for f in files:
    path = os.path.join(out_dir, f)
    with open(path, "r") as file:
        content = file.read()
    lines = content.splitlines()
    has_contract = "Branchless Contract" in content
    line_count = len(lines)
    print(f"{f}: lines={line_count}, has_contract={has_contract}")
    if line_count < 100 or not has_contract:
        all_ok = False
        print(f"  --> ERROR: needs fix!")

print("All ok:", all_ok)
