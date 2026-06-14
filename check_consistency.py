import os
import re

algorithms_dir = "crates/bcinr-logic/src/algorithms"
mod_file = os.path.join(algorithms_dir, "mod.rs")

files = [f[:-3] for f in os.listdir(algorithms_dir) if f.endswith(".rs") and f != "mod.rs"]

with open(mod_file, "r") as f:
    mod_content = f.read()

mods = re.findall(r"pub mod (.*?);", mod_content)

orphaned_files = set(files) - set(mods)
missing_mods = set(mods) - set(files)

print(f"Orphaned files: {orphaned_files}")
print(f"Missing mods: {missing_mods}")

if not orphaned_files and not missing_mods:
    print("Consistency check passed: All files have corresponding modules and vice versa.")
else:
    print("Consistency check failed.")
