import os
import re

DIR = "crates/bcinr-logic/src/algorithms/"

IMPLS = {
    "unrolled_binary_search_u32": (
        "    let target = val & 0xFFFFFFFF;\n    let mut pos = 0u64;\n    pos |= (((aux >> (pos | 8)) & 0xFF) < target) as u64 * 8;\n    pos |= (((aux >> (pos | 4)) & 0xFF) < target) as u64 * 4;\n    pos |= (((aux >> (pos | 2)) & 0xFF) < target) as u64 * 2;\n    pos |= (((aux >> (pos | 1)) & 0xFF) < target) as u64 * 1;\n    pos",
        "        let target = val & 0xFFFFFFFF;\n        let mut pos = 0;\n        if ((aux >> (pos | 8)) & 0xFF) < target { pos |= 8; }\n        if ((aux >> (pos | 4)) & 0xFF) < target { pos |= 4; }\n        if ((aux >> (pos | 2)) & 0xFF) < target { pos |= 2; }\n        if ((aux >> (pos | 1)) & 0xFF) < target { pos |= 1; }\n        pos as u64"
    ),
    "upper_bound_branchless_u32": (
        "    let target = val & 0xFFFFFFFF;\n    let mut pos = 0u64;\n    pos |= (((aux >> (pos | 8)) & 0xFF) <= target) as u64 * 8;\n    pos |= (((aux >> (pos | 4)) & 0xFF) <= target) as u64 * 4;\n    pos |= (((aux >> (pos | 2)) & 0xFF) <= target) as u64 * 2;\n    pos |= (((aux >> (pos | 1)) & 0xFF) <= target) as u64 * 1;\n    pos",
        "        let target = val & 0xFFFFFFFF;\n        let mut pos = 0;\n        if ((aux >> (pos | 8)) & 0xFF) <= target { pos |= 8; }\n        if ((aux >> (pos | 4)) & 0xFF) <= target { pos |= 4; }\n        if ((aux >> (pos | 2)) & 0xFF) <= target { pos |= 2; }\n        if ((aux >> (pos | 1)) & 0xFF) <= target { pos |= 1; }\n        pos as u64"
    )
}

for name, (impl, ref_impl) in IMPLS.items():
    path = os.path.join(DIR, name + ".rs")
    if not os.path.exists(path):
        continue
    with open(path, 'r') as f:
        content = f.read()

    impl_pattern = r"(pub fn " + name + r"\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\})"
    content = re.sub(impl_pattern, r"\1\n" + impl + r"\n\3", content, flags=re.DOTALL | re.MULTILINE)

    ref_pattern = r"(fn " + name + r"_reference\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\s*\})"
    content = re.sub(ref_pattern, r"\1\n" + ref_impl + r"\n\3", content, flags=re.DOTALL | re.MULTILINE)

    with open(path, 'w') as f:
        f.write(content)
