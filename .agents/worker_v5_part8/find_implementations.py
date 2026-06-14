import os

algos = [
    "ascii_to_lowercase_simd", "ascii_to_uppercase_simd", "is_alphanumeric_simd_u8x16",
    "is_digit_simd_u8x16", "is_space_simd_u8x16", "trim_whitespace_branchless",
    "split_lines_simd", "csv_scan_row_simd", "json_find_string_escapes_simd",
    "json_find_structural_simd", "levenshtein_dist_branchless", "hamming_dist_simd",
    "jaro_winkler_branchless", "soundex_encode_branchless", "metaphone_encode_branchless",
    "url_encode_branchless", "url_decode_branchless", "punycode_encode_branchless",
    "simd_strstr_branchless", "simd_memchr_u8x16", "simd_memrchr_u8x16",
    "wildcard_match_branchless", "regex_nfa_simd_step", "aho_corasick_simd_step",
    "suffix_array_step_branchless", "lcp_array_step_branchless", "burrows_wheeler_transform_step",
    "move_to_front_branchless", "huffman_decode_table_step", "prefix_sum_simd_u32x8",
    "suffix_sum_simd_u32x8"
]

all_py_files = [f for f in os.listdir("/Users/sac/bcinr") if f.endswith(".py") and f not in ["implement_201_300.py", "generate_algorithms.py"]]
all_py_files.sort()

for algo in algos:
    print(f"=== {algo} ===")
    matches = []
    for filename in all_py_files:
        filepath = os.path.join("/Users/sac/bcinr", filename)
        with open(filepath, "r") as f:
            content = f.read()
        
        if f'"{algo}"' in content or f"'{algo}'" in content:
            lines = content.split('\n')
            for i, line in enumerate(lines):
                if f'"{algo}"' in line or f"'{algo}'" in line:
                    # check if the surrounding contains actual logic
                    # E.g. let, if, return, or code characteristics
                    surrounding = "\n".join(lines[max(0, i-5):min(len(lines), i+15)])
                    if "let " in surrounding or "if " in surrounding or "wrapping_" in surrounding or "pub fn" in surrounding:
                        matches.append((filename, i+1, line.strip()))
                        break
    for m in matches:
        print(f"  Found in {m[0]} at line {m[1]}: {m[2]}")
