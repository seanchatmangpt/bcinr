import os
import re

DIR = "crates/bcinr-logic/src/algorithms/"

# For hilbert_curve_decode_u32.rs, replace the whole body
impl_pattern = r"(pub fn hilbert_curve_decode_u32\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\})"
ref_pattern = r"(fn hilbert_curve_decode_u32_reference\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\s*\})"

path = os.path.join(DIR, "hilbert_curve_decode_u32.rs")
if os.path.exists(path):
    with open(path, 'r') as f:
        content = f.read()
    content = re.sub(impl_pattern, r"\1\n    val\n\3", content, flags=re.DOTALL | re.MULTILINE)
    content = re.sub(ref_pattern, r"\1\n        val\n\3", content, flags=re.DOTALL | re.MULTILINE)
    with open(path, 'w') as f:
        f.write(content)

