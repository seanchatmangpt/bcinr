import os
import re

targets = {
    "crates/bcinr-logic/src/simd.rs": ["shuffle_u8x16"],
    "crates/bcinr-logic/src/utf8.rs": ["count_codepoints", "first_invalid_byte"],
    "crates/bcinr-logic/src/reduce.rs": ["horizontal_or_u32", "horizontal_and_u32", "horizontal_xor_u32", "horizontal_sum_u8x8", "horizontal_max_u8x8", "horizontal_min_u8x8"],
    "crates/bcinr-logic/src/utils/dense_kernel.rs": ["fnv1a_64"],
    "crates/bcinr-logic/src/sketch.rs": ["murmur3_32", "fnv1a_64", "xxhash32"],
    "crates/bcinr-logic/src/parse.rs": ["skip_whitespace", "parse_hex_u32", "parse_decimal_u64"],
    "crates/bcinr-logic/src/fix.rs": ["add_sat", "clamp_u32", "bucketize_u32"],
    "crates/bcinr-logic/src/scan.rs": ["find_byte_mask", "skip_spaces", "is_ascii_u64_slice"],
    "crates/bcinr-logic/src/dfa.rs": ["dfa_advance", "dfa_run"],
    "crates/bcinr-logic/src/network.rs": ["compare_exchange", "bitonic_sort_32u32"],
    "crates/bcinr-logic/src/bitset.rs": ["rank_u64", "select_bit_u64", "parity_u64_slice", "jaccard_u64_slices", "hamming_u64_slices", "intersect_u64_slices", "union_u64_slices", "any_bit_set_u64_slice"]
}

for filepath, funcs in targets.items():
    if not os.path.exists(filepath):
        continue
    with open(filepath, "r") as f:
        content = f.read()

    for func in funcs:
        # Find `pub fn func(...) -> ... {` or `pub const fn func(...) -> ... {`
        # We need to extract the signature to create the wrapper
        pattern = r'(pub (?:const )?fn ' + func + r'\s*<.*?>\s*\((.*?)\)(?:\s*->\s*(.*?))?\s*\{)'
        match = re.search(pattern, content)
        if not match:
            pattern = r'(pub (?:const )?fn ' + func + r'\s*\((.*?)\)(?:\s*->\s*(.*?))?\s*\{)'
            match = re.search(pattern, content)
        
        if match:
            # We rename the original to `_func_inner` and make it private
            # Then we insert a new `pub fn func` that calls `_func_inner`
            sig_full = match.group(1)
            params_raw = match.group(2)
            ret_type = match.group(3) if match.group(3) else ""
            
            # Extract just the argument names for the call
            # e.g. "a: [u8; 16], b: [u8; 16]" -> "a, b"
            # Ignore types and mut
            args = []
            for param in params_raw.split(','):
                if not param.strip(): continue
                # handle `mut a: u32`
                name_part = param.split(':')[0].strip()
                name = name_part.split(' ')[-1] # get last word, usually the name
                args.append(name)
            arg_str = ", ".join(args)
            
            # Replace `pub fn func` with `fn _func_inner`
            new_sig_full = sig_full.replace("pub ", "").replace("fn " + func, "fn _" + func + "_inner")
            
            wrapper_ret = f" -> {ret_type}" if ret_type else ""
            wrapper_sig = sig_full.replace("pub const fn", "pub fn") # Remove const from wrapper just in case
            
            wrapper = f"{wrapper_sig}\n    _{func}_inner({arg_str})\n}}\n\n#[inline(always)]\n#[allow(dead_code)]\n{new_sig_full}"
            
            content = content.replace(sig_full, wrapper)

    with open(filepath, "w") as f:
        f.write(content)

print("Legacy files refactored.")
