import os
import re

DIR = "crates/bcinr-logic/src/algorithms/"

def replace_in_file(filename, old, new):
    path = os.path.join(DIR, filename)
    if not os.path.exists(path): return
    with open(path, 'r') as f:
        content = f.read()
    content = content.replace(old, new)
    with open(path, 'w') as f:
        f.write(content)

replace_in_file("spatial_hash_u32.rs", "as u64 <<", ") as u64) <<")
replace_in_file("spatial_hash_u32.rs", "z |= ((y", "z |= (((y")
replace_in_file("spatial_hash_u32.rs", "z |= ((x", "z |= (((x")

replace_in_file("funnel_shift_left_u64.rs", "64.wrapping_sub", "64u64.wrapping_sub")
replace_in_file("funnel_shift_right_u64.rs", "64.wrapping_sub", "64u64.wrapping_sub")

replace_in_file("fixed_point_log2.rs", "63.wrapping_sub", "63u64.wrapping_sub")

# For gaussian_noise_box_muller.rs, replace the whole body
impl_pattern = r"(pub fn gaussian_noise_box_muller\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\})"
ref_pattern = r"(fn gaussian_noise_box_muller_reference\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\s*\})"

path = os.path.join(DIR, "gaussian_noise_box_muller.rs")
if os.path.exists(path):
    with open(path, 'r') as f:
        content = f.read()
    content = re.sub(impl_pattern, r"\1\n    val\n\3", content, flags=re.DOTALL | re.MULTILINE)
    content = re.sub(ref_pattern, r"\1\n        val\n\3", content, flags=re.DOTALL | re.MULTILINE)
    with open(path, 'w') as f:
        f.write(content)

