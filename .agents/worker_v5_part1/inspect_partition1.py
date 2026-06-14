import os
import re

partition1 = [
    "parallel_bits_deposit_u64", "parallel_bits_extract_u64", "blsr_u64", "blsi_u64", "blsmsk_u64",
    "t1mskc_u64", "tzmsk_u64", "bext_u64", "bset_u64", "bclr_u64", "btst_u64", "popcount_u128",
    "reverse_bits_u128", "clmul_u64", "morton_encode_2d_u32", "morton_decode_2d_u32", "morton_encode_3d_u32",
    "gray_encode_u64", "gray_decode_u64", "parity_check_u128", "next_lexicographic_permutation_u64",
    "count_consecutive_set_bits_u64", "find_nth_set_bit_u128", "mask_range_u64", "rotate_left_u64",
    "rotate_right_u64", "funnel_shift_left_u64", "funnel_shift_right_u64", "bit_swap_u64",
    "gather_bits_u64", "scatter_bits_u64"
]

base_path = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms/"

for algo in partition1:
    path = os.path.join(base_path, f"{algo}.rs")
    if not os.path.exists(path):
        print(f"!!! File {algo}.rs does not exist")
        continue
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Extract impl body
    impl_pattern = rf"pub fn {algo}\((.*?)\) -> u64 {{(.*?)^}}"
    m_impl = re.search(rf"pub fn {algo}\((.*?)\) -> u64 {{(.*?)}}\n\n", content, re.DOTALL | re.MULTILINE)
    
    # Or just extract the text between pub fn algo(...) -> u64 { and #[cfg(test)]
    start_impl_str = f"pub fn {algo}"
    idx = content.find(start_impl_str)
    impl_body = "NOT FOUND"
    if idx != -1:
        # Find next #[cfg(test)] or mod tests
        end_idx = content.find("#[cfg(test)]", idx)
        if end_idx != -1:
            impl_body = content[idx:end_idx].strip()
        else:
            impl_body = content[idx:idx+300].strip()

    # Extract ref body
    ref_pattern = rf"fn {algo}_reference"
    ref_idx = content.find(ref_pattern)
    ref_body = "NOT FOUND"
    if ref_idx != -1:
        # Find the next negative mutants comment or proptest
        end_ref_idx = content.find("// NEGATIVE MUTANTS", ref_idx)
        if end_ref_idx != -1:
            ref_body = content[ref_idx:end_ref_idx].strip()
        else:
            end_ref_idx2 = content.find("proptest!", ref_idx)
            if end_ref_idx2 != -1:
                ref_body = content[ref_idx:end_ref_idx2].strip()
            else:
                ref_body = content[ref_idx:ref_idx+200].strip()

    print(f"=== {algo} ===")
    print("--- IMPLEMENTATION ---")
    print(impl_body)
    print("--- REFERENCE ---")
    print(ref_body)
    print("-" * 50)
