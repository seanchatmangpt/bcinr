import os

def fix_spatial():
    path = "crates/bcinr-logic/src/algorithms/spatial_hash_u32.rs"
    with open(path, "r") as f:
        content = f.read()
    content = content.replace("((x >> i) & 1) as u64 << (2*i)", "(((x >> i) & 1) as u64) << (2*i)")
    content = content.replace("((y >> i) & 1) as u64 << (2*i + 1)", "(((y >> i) & 1) as u64) << (2*i + 1)")
    with open(path, "w") as f:
        f.write(content)

def fix_unrolled():
    path = "crates/bcinr-logic/src/algorithms/unrolled_binary_search_u32.rs"
    with open(path, "r") as f:
        content = f.read()
    content = content.replace("< target", "< target.into()")
    content = content.replace("< (target as u32)", "< target") # revert if needed
    with open(path, "w") as f:
        f.write(content)

fix_spatial()
fix_unrolled()
