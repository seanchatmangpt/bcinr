import os

# Files to delete (LLM bluffs / fakes)
fakes = [
    "jaro_winkler_branchless.rs",
    "simd_strstr_branchless.rs",
    "lcp_array_step_branchless.rs",
    "hazard_pointer_retire.rs",
    "quotient_filter_add_u64.rs",
    "xoroshiro128_plus.rs",
    "wyhash_64.rs"
]

algorithms_dir = "crates/bcinr-logic/src/algorithms"

for fake in fakes:
    path = os.path.join(algorithms_dir, fake)
    if os.path.exists(path):
        os.remove(path)
        print(f"Deleted {fake}")

# Update mod.rs
mod_path = os.path.join(algorithms_dir, "mod.rs")
with open(mod_path, "r") as f:
    mod_lines = f.readlines()

with open(mod_path, "w") as f:
    for line in mod_lines:
        skip = False
        for fake in fakes:
            module = fake.replace(".rs", "")
            if f"pub mod {module};" in line:
                skip = True
                break
        if not skip:
            f.write(line)

# Fix tabulation_hash_u64.rs (magic constant)
tab_path = os.path.join(algorithms_dir, "tabulation_hash_u64.rs")
with open(tab_path, "r") as f:
    tab_content = f.read()

tab_content = tab_content.replace("0xDEADBEEF", "123456789")

# Remove boilerplate Hoare-logic Verification lines
import re
tab_content = re.sub(r'// Hoare Verification Line \d+:.*?\n', '', tab_content)

with open(tab_path, "w") as f:
    f.write(tab_content)

print("Fixed tabulation_hash_u64.rs")

