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

dir_path = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms/"

for algo in algos:
    filepath = os.path.join(dir_path, f"{algo}.rs")
    if not os.path.exists(filepath):
        print(f"File {algo}.rs does NOT exist!")
        continue
    
    with open(filepath, "r") as f:
        lines = f.readlines()
    
    content = "".join(lines)
    
    # Check lines
    line_count = len(lines)
    
    # Check "Branchless Contract"
    has_contract = "Branchless Contract" in content
    
    # Check if implementation body is a dummy logic
    # We can inspect the pub fn body
    # Let's locate the implementation body of the pub fn
    import re
    m_impl = re.search(r"pub fn " + algo + r"\s*\((.*?)\)\s*->\s*u64\s*\{(.*?)\n\}", content, re.DOTALL)
    impl_body = m_impl.group(2).strip() if m_impl else "NOT FOUND"
    
    # Let's check for reference body
    m_ref = re.search(r"fn " + algo + r"_reference\s*\((.*?)\)\s*->\s*u64\s*\{(.*?)\n\s*\}", content, re.DOTALL)
    ref_body = m_ref.group(2).strip() if m_ref else "NOT FOUND"
    
    # Let's check if the file has dummy logic.
    # Dummy logic has patterns like wrapping_add(aux) ^ (val.rotate_left(7))
    is_dummy_impl = "0x9E3779B" in impl_body or "0xDEADBEEF" in impl_body or ("wrapping_add(aux)" in impl_body and "rotate_left(7)" in impl_body) or ("count_ones()" in impl_body and "rotate_left(11)" in impl_body)
    
    # Print status
    print(f"File: {algo}.rs | Lines: {line_count} | HasContract: {has_contract} | DummyImpl: {is_dummy_impl}")
    if is_dummy_impl or line_count < 100 or not has_contract or impl_body == "NOT FOUND" or ref_body == "NOT FOUND":
        print(f"  --> REQUIRES REFACTOR!")
        print(f"      Impl: {impl_body[:100]}")
        print(f"      Ref:  {ref_body[:100]}")
