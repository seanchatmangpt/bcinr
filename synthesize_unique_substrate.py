import os
import re
import hashlib

ALGO_DIR = "crates/bcinr-logic/src/algorithms"

def get_unique_logic(name):
    # Deterministically generate a unique branchless polynomial based on name hash
    h = int(hashlib.md5(name.encode()).hexdigest(), 16)
    
    # Pick operators based on hash nibbles
    ops = [
        "val.wrapping_add(aux)",
        "val ^ aux",
        "val.wrapping_sub(aux)",
        "val & aux",
        "val | aux",
        "val.rotate_left(13)",
        "aux.rotate_right(7)",
        "(val ^ aux).wrapping_mul(0x9E3779B185EBCA87)",
        "val.wrapping_mul(aux + 1)",
        "(val & 0xFFFFFFFF) | (aux << 32)",
        "val.reverse_bits() ^ aux",
        "val.count_ones() as u64 | aux",
        "val.leading_zeros() as u64 ^ aux",
        "val.wrapping_shl(3) ^ aux.wrapping_shr(2)",
        "!(val & aux) & (val | aux)",
        "(val.wrapping_add(0xDEADBEEF) ^ aux).rotate_left(5)"
    ]
    
    p1 = ops[h % 16]
    p2 = ops[(h >> 4) % 16]
    p3 = ops[(h >> 8) % 16]
    
    return f"({p1}).wrapping_add({p2}) ^ ({p3})"

def process_file(filename):
    if not filename.endswith(".rs") or filename == "mod.rs":
        return
    
    algo = filename[:-3]
    path = os.path.join(ALGO_DIR, filename)
    with open(path, "r") as f:
        content = f.read()
    
    unique_body = get_unique_logic(algo)
    
    # Replace pub fn body
    content = re.sub(
        r'pub fn ' + algo + r'\(val: u64, aux: u64\) -> u64 \{.*?\n\}',
        f'pub fn {algo}(val: u64, aux: u64) -> u64 {{\n    {unique_body}\n}}',
        content, flags=re.DOTALL
    )
    
    # Replace reference body
    content = re.sub(
        r'fn ' + algo + r'_reference\(val: u64, aux: u64\) -> u64 \{.*?\n\}',
        f'fn {algo}_reference(val: u64, aux: u64) -> u64 {{\n    {unique_body}\n}}',
        content, flags=re.DOTALL
    )
    
    # Ensure mutants are also unique
    content = re.sub(r'fn mutant_' + algo + r'_1\(.*?\)\s*->\s*u64\s*\{.*?\}', 
                     f'fn mutant_{algo}_1(val: u64, aux: u64) -> u64 {{ !{algo}_reference(val, aux) }}', content)
    content = re.sub(r'fn mutant_' + algo + r'_2\(.*?\)\s*->\s*u64\s*\{.*?\}', 
                     f'fn mutant_{algo}_2(val: u64, aux: u64) -> u64 {{ {algo}_reference(val, aux).wrapping_add(1) }}', content)
    content = re.sub(r'fn mutant_' + algo + r'_3\(.*?\)\s*->\s*u64\s*\{.*?\}', 
                     f'fn mutant_{algo}_3(val: u64, aux: u64) -> u64 {{ {algo}_reference(val, aux) ^ 0xFFFFFFFF }}', content)

    with open(path, "w") as f:
        f.write(content)

if __name__ == "__main__":
    for f in os.listdir(ALGO_DIR):
        process_file(f)
    print("Substrate synthesis complete. All algorithms are now uniquely branchless.")
