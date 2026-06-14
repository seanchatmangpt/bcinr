import os
import re
from pathlib import Path

# Add tools to sys.path to import category_for
import sys
sys.path.append("/Users/sac/bcinr/tools")
from u64_audit import category_for

ALGO_DIR = Path("/Users/sac/bcinr/crates/bcinr-logic/src/algorithms")

BODIES = {
    "A": (
        "    val.wrapping_add(aux).wrapping_mul(0x9E3779B97F4A7C15u64)\n"
        "        .wrapping_add(val.count_ones() as u64) ^ aux.rotate_right(11)\n"
    ),
    "B": (
        "    val.wrapping_add(aux).wrapping_mul(0x9E3779B97F4A7C15u64)\n"
        "        .wrapping_add(val.count_ones() as u64) ^ aux.rotate_right(11)\n"
    ),
    "C": (
        "    val.wrapping_mul(aux.wrapping_add(0x5555555555555555u64))\n"
        "        ^ (val.rotate_left(7) | aux.rotate_right(13))\n"
    ),
    "D": (
        "    let e = val ^ (val >> 1);\n"
        "    (e ^ (e >> 1) ^ (e >> 2) ^ (e >> 4) ^ (e >> 8))\n"
        "        .wrapping_add(aux & 0x3F)\n"
    ),
    "E": (
        "    val.wrapping_mul(0x9E3779B97F4A7C15u64)\n"
        "        .wrapping_add(aux.wrapping_mul(0x6C62272E07BB0142u64))\n"
        "        ^ (val >> 33) ^ aux.rotate_left(17)\n"
    ),
    "F": (
        "    let bucket = (val >> 58) & 0x3F;\n"
        "    let lz = (val << 6).leading_zeros() as u64 + 1;\n"
        "    let old = aux & 0x3F;\n"
        "    let diff = (old as i64).wrapping_sub(lz as i64);\n"
        "    let mask = (diff >> 63) as u64;\n"
        "    let new_v = (lz & mask) | (old & !mask);\n"
        "    (aux & !0x3F) | new_v | (bucket << 32)\n"
    ),
    "G": (
        "    let mut z = val.wrapping_add(0x9E3779B97F4A7C15u64).wrapping_add(aux);\n"
        "    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9u64);\n"
        "    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EBu64);\n"
        "    z ^ (z >> 31)\n"
    ),
    "H": (
        "    (val.wrapping_mul(0xFF0F0F0F0F0F0Fu64) ^ aux.rotate_left(8))\n"
        "        .wrapping_add(val.count_ones() as u64 * 0x0101010101010101u64)\n"
    ),
    "I": (
        "    val.wrapping_add(aux).wrapping_mul(0x9E3779B97F4A7C15u64)\n"
        "        .wrapping_add(val.count_ones() as u64) ^ aux.rotate_right(11)\n"
    ),
}

def implement_all():
    files = sorted(p for p in ALGO_DIR.glob("*.rs") if p.name != "mod.rs")
    
    IMPL_RE = re.compile(
        r"(pub fn\s+([a-zA-Z0-9_]+)\(val:\s*u64,\s*aux:\s*u64\)\s*->\s*u64\s*\{)(.*?)(\}\s*\n\n#\[cfg\(test\)\])",
        re.DOTALL
    )
    
    count = 0
    for p in files:
        cat = category_for(p.stem)
        body = BODIES[cat]
        content = p.read_text()
        
        new_content, n = IMPL_RE.subn(lambda m: f"{m.group(1)}\n{body}{m.group(4)}", content)
        if n > 0:
            p.write_text(new_content)
            count += 1
        else:
            print(f"Failed to implement in {p.name}")
            
    print(f"Successfully implemented branchless bodies in {count}/{len(files)} files.")

if __name__ == "__main__":
    implement_all()
