import os

ALGO_DIR = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"

def replace_in_file(filename, target, replacement):
    path = os.path.join(ALGO_DIR, filename)
    content = open(path).read()
    if target in content:
        content = content.replace(target, replacement)
        with open(path, "w") as f:
            f.write(content)
        print(f"Fixed {filename}")
    else:
        print(f"Warning: target not found in {filename}")

def fix():
    # branchless_signum_i64.rs
    replace_in_file(
        "branchless_signum_i64.rs",
        "    fn branchless_signum_i64_reference(val: u64, aux: u64) -> u64 { val }",
        "    fn branchless_signum_i64_reference(val: u64, aux: u64) -> u64 {\n        val\n    }"
    )
    replace_in_file(
        "branchless_signum_i64.rs",
        "        assert_eq!(branchless_signum_i64(i64::MAX), branchless_signum_i64_reference(i64::MAX));",
        "        assert_eq!(branchless_signum_i64(u64::MAX, 0), branchless_signum_i64_reference(u64::MAX, 0));"
    )
    
    # clamp_i64.rs
    replace_in_file(
        "clamp_i64.rs",
        "    fn clamp_i64_reference(val: u64, aux: u64) -> u64 { val }",
        "    fn clamp_i64_reference(val: u64, aux: u64) -> u64 {\n        val\n    }"
    )
    
    # bloom_filter_add_u64.rs
    replace_in_file(
        "bloom_filter_add_u64.rs",
        "pub fn bloom_filter_add_u64(val: u64, aux: u64) -> u64 {\n    let h = val.wrapping_mul(0x9e3779b97f4a7c15);\n    let bit1 = 1u64 << (h & 63);\n    let bit2 = 1u64 << ((h >> 6) & 63);\n    let bit3 = 1u64 << ((h >> 12) & 63);\n    filter | bit1 | bit2 | bit3\n}",
        "pub fn bloom_filter_add_u64(val: u64, aux: u64) -> u64 {\n    let h = aux.wrapping_mul(0x9e3779b97f4a7c15);\n    let bit1 = 1u64 << (h & 63);\n    let bit2 = 1u64 << ((h >> 6) & 63);\n    let bit3 = 1u64 << ((h >> 12) & 63);\n    val | bit1 | bit2 | bit3\n}"
    )
    
    # bloom_filter_graph_visited.rs
    replace_in_file(
        "bloom_filter_graph_visited.rs",
        "pub fn bloom_filter_graph_visited(val: u64, aux: u64) -> u64 {\n    let h = val.wrapping_mul(0x9e3779b97f4a7c15);\n    let bit1 = 1u64 << (h & 63);\n    let bit2 = 1u64 << ((h >> 6) & 63);\n    let bit3 = 1u64 << ((h >> 12) & 63);\n    filter | bit1 | bit2 | bit3\n}",
        "pub fn bloom_filter_graph_visited(val: u64, aux: u64) -> u64 {\n    let h = aux.wrapping_mul(0x9e3779b97f4a7c15);\n    let bit1 = 1u64 << (h & 63);\n    let bit2 = 1u64 << ((h >> 6) & 63);\n    let bit3 = 1u64 << ((h >> 12) & 63);\n    val | bit1 | bit2 | bit3\n}"
    )
    
    # bloom_filter_query_u64.rs
    replace_in_file(
        "bloom_filter_query_u64.rs",
        "pub fn bloom_filter_query_u64(val: u64, aux: u64) -> u64 {\n    let h = val.wrapping_mul(0x9e3779b97f4a7c15);\n    let bit1 = 1u64 << (h & 63);\n    let bit2 = 1u64 << ((h >> 6) & 63);\n    let bit3 = 1u64 << ((h >> 12) & 63);\n    let mask = bit1 | bit2 | bit3;\n    ((filter & mask) == mask) as u64\n}",
        "pub fn bloom_filter_query_u64(val: u64, aux: u64) -> u64 {\n    let h = aux.wrapping_mul(0x9e3779b97f4a7c15);\n    let bit1 = 1u64 << (h & 63);\n    let bit2 = 1u64 << ((h >> 6) & 63);\n    let bit3 = 1u64 << ((h >> 12) & 63);\n    let mask = bit1 | bit2 | bit3;\n    ((val & mask) == mask) as u64\n}"
    )

if __name__ == "__main__":
    fix()
