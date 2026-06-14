import os
path = "crates/bcinr-logic/src/algorithms/hilbert_curve_decode_u32.rs"
with open(path, "r") as f:
    content = f.read()
content = content.replace("let s = 1 << i;", "let s = 1i32 << i;")
content = content.replace("let s = 1u64 << i;", "let s: u64 = 1 << i;") # Just in case it was that
with open(path, "w") as f:
    f.write(content)
