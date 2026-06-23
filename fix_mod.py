import os
import glob

algorithms_dir = "crates/bcinr-logic/src/algorithms"
files = [os.path.basename(f) for f in glob.glob(os.path.join(algorithms_dir, "*.rs"))]
files.remove("mod.rs")

mod_content = "// Academic-grade branchless algorithm library: mod\n\n"
for f in sorted(files):
    mod_content += f"pub mod {f[:-3]};\n"

with open(os.path.join(algorithms_dir, "mod.rs"), "w") as f:
    f.write(mod_content)
