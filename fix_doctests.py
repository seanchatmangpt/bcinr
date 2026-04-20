import re
import os

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Replace crate::... with bcinr_logic::...
    content = re.sub(r'use crate::([a-zA-Z0-9_]+)', r'use bcinr_logic::\1', content)
    content = re.sub(r'use crate::dfa::', r'use bcinr_logic::dfa::', content)
    
    # In simd.rs, it uses simd::movemask_u8x16 directly without import.
    # We should add use bcinr_logic::simd; or replace simd:: with bcinr_logic::simd::
    if 'simd.rs' in filepath:
        content = content.replace('simd::movemask_u8x16', 'bcinr_logic::simd::movemask_u8x16')
        content = content.replace('simd::shuffle_u8x16', 'bcinr_logic::simd::shuffle_u8x16')
        content = content.replace('simd::splat_u8x16', 'bcinr_logic::simd::splat_u8x16')

    # In parse.rs it tries to import parse_decimal_u64 directly from crate root.
    # It should be bcinr_logic::parse::parse_decimal_u64
    if 'parse.rs' in filepath:
        content = content.replace('use bcinr_logic::parse_decimal_u64;', 'use bcinr_logic::parse::parse_decimal_u64;')
        content = content.replace('use bcinr_logic::parse_hex_u32;', 'use bcinr_logic::parse::parse_hex_u32;')
        content = content.replace('use bcinr_logic::skip_whitespace;', 'use bcinr_logic::parse::skip_whitespace;')
        
    if 'dfa.rs' in filepath:
        content = content.replace('use bcinr_logic::dfa_is_accepting;', 'use bcinr_logic::dfa::dfa_is_accepting;')
        content = content.replace('use bcinr_logic::{dfa_run, dfa_advance};', 'use bcinr_logic::dfa::{dfa_run, dfa_advance};')

    with open(filepath, 'w') as f:
        f.write(content)

fix_file('crates/bcinr-logic/src/dfa.rs')
fix_file('crates/bcinr-logic/src/parse.rs')
fix_file('crates/bcinr-logic/src/simd.rs')

print("Fixed doctests")
