import os
import re
import json

partition1 = [
    "parallel_bits_deposit_u64", "parallel_bits_extract_u64", "blsr_u64", "blsi_u64", "blsmsk_u64",
    "t1mskc_u64", "tzmsk_u64", "bext_u64", "bset_u64", "bclr_u64", "btst_u64", "popcount_u128",
    "reverse_bits_u128", "clmul_u64", "morton_encode_2d_u32", "morton_decode_2d_u32", "morton_encode_3d_u32",
    "gray_encode_u64", "gray_decode_u64", "parity_check_u128", "next_lexicographic_permutation_u64",
    "count_consecutive_set_bits_u64", "find_nth_set_bit_u128", "mask_range_u64", "rotate_left_u64",
    "rotate_right_u64", "funnel_shift_left_u64", "funnel_shift_right_u64", "bit_swap_u64",
    "gather_bits_u64", "scatter_bits_u64"
]

base_dir = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms/"

# Hardcoded implementation/reference mappings that we want to enforce.
# If they are already in the file correctly, we keep them, otherwise we replace.
# Ensure that references are branchful / mathematically correct / decoupled.
logic_db = {
    "parallel_bits_deposit_u64": (
        """    let mut res = 0;
    let mut v_idx = 0;
    for i in 0..64 {
        let mask_bit = (aux >> i) & 1;
        let val_bit = (val.wrapping_shr(v_idx)) & 1;
        res |= (val_bit & mask_bit) << i;
        v_idx += mask_bit as u32;
    }
    res""",
        """        let mut res = 0;
        let mut v_idx = 0;
        for i in 0..64 {
            if ((aux >> i) & 1) == 1 {
                if ((val.wrapping_shr(v_idx)) & 1) == 1 {
                    res |= 1 << i;
                }
                v_idx += 1;
            }
        }
        res"""
    ),
    "parallel_bits_extract_u64": (
        """    let mut res = 0;
    let mut r_idx = 0;
    for i in 0..64 {
        let mask_bit = (aux >> i) & 1;
        let val_bit = (val >> i) & 1;
        res |= (val_bit & mask_bit).wrapping_shl(r_idx);
        r_idx += mask_bit as u32;
    }
    res""",
        """        let mut res = 0;
        let mut r_idx = 0;
        for i in 0..64 {
            if ((aux >> i) & 1) == 1 {
                if ((val >> i) & 1) == 1 {
                    res |= 1 << r_idx;
                }
                r_idx += 1;
            }
        }
        res"""
    ),
    "blsr_u64": (
        "    val & val.wrapping_sub(1)",
        """        if val == 0 {
            0
        } else {
            val & (val - 1)
        }"""
    ),
    "blsi_u64": (
        "    val & val.wrapping_neg()",
        """        if val == 0 {
            0
        } else {
            val & (!val + 1)
        }"""
    ),
    "blsmsk_u64": (
        "    val ^ val.wrapping_sub(1)",
        """        if val == 0 {
            u64::MAX
        } else {
            val ^ (val - 1)
        }"""
    ),
    "t1mskc_u64": (
        "    (!val) | val.wrapping_add(1)",
        """        if val == u64::MAX {
            0
        } else {
            (!val) | (val + 1)
        }"""
    ),
    "tzmsk_u64": (
        "    (!val) & val.wrapping_sub(1)",
        """        if val == 0 {
            u64::MAX
        } else {
            (!val) & (val - 1)
        }"""
    ),
    "bext_u64": (
        """    let start = aux & 0x3F;
    let len = (aux >> 8) & 0x3F;
    let mask = (0u64.wrapping_sub((len >= 64) as u64)) | (((1u64.wrapping_shl(len as u32 & 0x3F)).wrapping_sub(1)) & (0u64.wrapping_sub((len < 64) as u64)));
    (val.wrapping_shr(start as u32)) & mask""",
        """        let start = aux & 0x3F;
        let len = (aux >> 8) & 0x3F;
        let mut res = 0;
        for i in 0..len {
            if start + i < 64 {
                res |= ((val >> (start + i)) & 1) << i;
            }
        }
        res"""
    ),
    "bset_u64": (
        """    let start = aux & 0x3F;
    let len = (aux >> 8) & 0x3F;
    let mask = (0u64.wrapping_sub((len >= 64) as u64)) | (((1u64.wrapping_shl(len as u32 & 0x3F)).wrapping_sub(1)) & (0u64.wrapping_sub((len < 64) as u64)));
    val | (mask.wrapping_shl(start as u32))""",
        """        let start = aux & 0x3F;
        let len = (aux >> 8) & 0x3F;
        let mut res = val;
        for i in 0..len {
            if start + i < 64 {
                res |= 1 << (start + i);
            }
        }
        res"""
    ),
    "bclr_u64": (
        """    let start = aux & 0x3F;
    let len = (aux >> 8) & 0x3F;
    let mask = (0u64.wrapping_sub((len >= 64) as u64)) | (((1u64.wrapping_shl(len as u32 & 0x3F)).wrapping_sub(1)) & (0u64.wrapping_sub((len < 64) as u64)));
    val & !(mask.wrapping_shl(start as u32))""",
        """        let start = aux & 0x3F;
        let len = (aux >> 8) & 0x3F;
        let mut res = val;
        for i in 0..len {
            if start + i < 64 {
                res &= !(1 << (start + i));
            }
        }
        res"""
    ),
    "btst_u64": (
        "    (val.wrapping_shr((aux & 0x3F) as u32)) & 1",
        """        if (val & (1u64.wrapping_shl((aux & 0x3F) as u32))) != 0 {
            1
        } else {
            0
        }"""
    ),
    "popcount_u128": (
        "    (val.count_ones() + aux.count_ones()) as u64",
        """        let mut c = 0;
        for i in 0..64 {
            c += (val >> i) & 1;
            c += (aux >> i) & 1;
        }
        c"""
    ),
    "reverse_bits_u128": (
        "    val.reverse_bits() ^ aux.reverse_bits()",
        """        let mut r_val = 0u64;
        let mut r_aux = 0u64;
        for i in 0..64 {
            if ((val >> i) & 1) == 1 {
                r_val |= 1u64 << (63 - i);
            }
            if ((aux >> i) & 1) == 1 {
                r_aux |= 1u64 << (63 - i);
            }
        }
        r_val ^ r_aux"""
    ),
    "clmul_u64": (
        """    let mut res = 0;
    for i in 0..64 {
        let bit = (val >> i) & 1;
        let mask = 0u64.wrapping_sub(bit);
        res ^= (aux.wrapping_shl(i as u32)) & mask;
    }
    res""",
        """        let mut res = 0;
        for i in 0..64 {
            if ((val >> i) & 1) == 1 {
                res ^= aux.wrapping_shl(i as u32);
            }
        }
        res"""
    ),
    "morton_encode_2d_u32": (
        """    let mut x = val & 0xFFFFFFFF;
    x = (x ^ (x << 16)) & 0x0000ffff0000ffff;
    x = (x ^ (x << 8)) & 0x00ff00ff00ff00ff;
    x = (x ^ (x << 4)) & 0x0f0f0f0f0f0f0f0f;
    x = (x ^ (x << 2)) & 0x3333333333333333;
    x = (x ^ (x << 1)) & 0x5555555555555555;
    let mut y = aux & 0xFFFFFFFF;
    y = (y ^ (y << 16)) & 0x0000ffff0000ffff;
    y = (y ^ (y << 8)) & 0x00ff00ff00ff00ff;
    y = (y ^ (y << 4)) & 0x0f0f0f0f0f0f0f0f;
    y = (y ^ (y << 2)) & 0x3333333333333333;
    y = (y ^ (y << 1)) & 0x5555555555555555;
    x | (y << 1)""",
        """        let mut res = 0;
        for i in 0..32 {
            if ((val >> i) & 1) == 1 {
                res |= 1 << (2 * i);
            }
            if ((aux >> i) & 1) == 1 {
                res |= 1 << (2 * i + 1);
            }
        }
        res"""
    ),
    "morton_decode_2d_u32": (
        """    let mut x = val & 0x5555555555555555;
    x = (x ^ (x >> 1)) & 0x3333333333333333;
    x = (x ^ (x >> 2)) & 0x0f0f0f0f0f0f0f0f;
    x = (x ^ (x >> 4)) & 0x00ff00ff00ff00ff;
    x = (x ^ (x >> 8)) & 0x0000ffff0000ffff;
    x = (x ^ (x >> 16)) & 0x00000000ffffffff;
    x""",
        """        let mut res = 0;
        for i in 0..32 {
            if ((val >> (2 * i)) & 1) == 1 {
                res |= 1 << i;
            }
        }
        res"""
    ),
    "morton_encode_3d_u32": (
        """    let mut x = val & 0x1FFFFFu64;
    x = (x | (x << 32)) & 0x1F00000000FFFFu64;
    x = (x | (x << 16)) & 0x1F0000FF0000FFu64;
    x = (x | (x << 8)) & 0x100F00F00F00F00Fu64;
    x = (x | (x << 4)) & 0x10c30c30c30c30c3u64;
    x = (x | (x << 2)) & 0x1249249249249249u64;
    x""",
        """        let x = val as u32 & 0x1FFFFF;
        let mut res = 0u64;
        for i in 0..21 {
            if ((x >> i) & 1) == 1 {
                res |= 1u64 << (3 * i);
            }
        }
        res"""
    ),
    "gray_encode_u64": (
        "    val ^ (val >> 1)",
        "        val ^ (val >> 1)"
    ),
    "gray_decode_u64": (
        """    let mut n = val;
    n ^= n >> 32;
    n ^= n >> 16;
    n ^= n >> 8;
    n ^= n >> 4;
    n ^= n >> 2;
    n ^= n >> 1;
    n""",
        """        let mut res = val;
        for i in 1..64 {
            res ^= val >> i;
        }
        res"""
    ),
    "parity_check_u128": (
        "    ((val.count_ones() + aux.count_ones()) & 1) as u64",
        """        let mut count = 0;
        for i in 0..64 {
            if ((val >> i) & 1) == 1 {
                count += 1;
            }
            if ((aux >> i) & 1) == 1 {
                count += 1;
            }
        }
        (count & 1) as u64"""
    ),
    "next_lexicographic_permutation_u64": (
        """    let t = val | val.wrapping_sub(1);
    let c = !t & t.wrapping_add(1);
    let tz = val.trailing_zeros();
    let shift = tz.wrapping_add(1) & 0x3F;
    let o = (c.wrapping_sub(1)).wrapping_shr(shift);
    (t.wrapping_add(1) | o) * (val != 0) as u64""",
        """        if val == 0 {
            0
        } else {
            let t = val | val.wrapping_sub(1);
            let next = t.wrapping_add(1);
            let ones = ((!t & next).wrapping_sub(1)).wrapping_shr(val.trailing_zeros() + 1);
            next | ones
        }"""
    ),
    "count_consecutive_set_bits_u64": (
        """    let mut count = 0;
    let mut v = val;
    for _ in 0..64 {
        let mask = 0u64.wrapping_sub((v != 0) as u64);
        count += 1 & mask;
        v &= v << 1;
    }
    count""",
        """        let mut max_c = 0;
        let mut cur_c = 0;
        for i in 0..64 {
            if ((val >> i) & 1) == 1 {
                cur_c += 1;
                if cur_c > max_c {
                    max_c = cur_c;
                }
            } else {
                cur_c = 0;
            }
        }
        max_c"""
    ),
    "find_nth_set_bit_u128": (
        """    let mut v = val;
    let mut n = aux;
    let mut pos = 0u64;
    
    let c1 = (v as u32).count_ones() as u64;
    let m1 = ((n < c1) as u64).wrapping_neg();
    pos |= (!m1) & 32;
    v = (v & m1) | ((v >> 32) & (!m1));
    n = (n & m1) | ((n.wrapping_sub(c1)) & (!m1));
    
    let c2 = (v as u16).count_ones() as u64;
    let m2 = ((n < c2) as u64).wrapping_neg();
    pos |= (!m2) & 16;
    v = (v & m2) | ((v >> 16) & (!m2));
    n = (n & m2) | ((n.wrapping_sub(c2)) & (!m2));
    
    let c3 = (v as u8).count_ones() as u64;
    let m3 = ((n < c3) as u64).wrapping_neg();
    pos |= (!m3) & 8;
    v = (v & m3) | ((v >> 8) & (!m3));
    n = (n & m3) | ((n.wrapping_sub(c3)) & (!m3));
    
    let c4 = (v & 0xF).count_ones() as u64;
    let m4 = ((n < c4) as u64).wrapping_neg();
    pos |= (!m4) & 4;
    v = (v & m4) | ((v >> 4) & (!m4));
    n = (n & m4) | ((n.wrapping_sub(c4)) & (!m4));
    
    let c5 = (v & 0x3).count_ones() as u64;
    let m5 = ((n < c5) as u64).wrapping_neg();
    pos |= (!m5) & 2;
    v = (v & m5) | ((v >> 2) & (!m5));
    n = (n & m5) | ((n.wrapping_sub(c5)) & (!m5));
    
    let c6 = (v & 0x1).count_ones() as u64;
    let m6 = ((n < c6) as u64).wrapping_neg();
    pos |= (!m6) & 1;
    
    pos""",
        """        let mut count = 0;
        let mut res = 0;
        for i in 0..64 {
            if ((val >> i) & 1) == 1 {
                if count == aux {
                    res = i as u64;
                    break;
                }
                count += 1;
            }
        }
        res"""
    ),
    "mask_range_u64": (
        """    let start = aux & 0x3F;
    let end = (aux >> 8) & 0x3F;
    let is_valid = (start < end) as u64;
    let diff = (end.wrapping_sub(start)) & 0x3F;
    let mask = (0u64.wrapping_sub((end.wrapping_sub(start) >= 64) as u64)) | (((1u64.wrapping_shl(diff as u32)).wrapping_sub(1)) & (0u64.wrapping_sub((end.wrapping_sub(start) < 64) as u64)));
    (mask.wrapping_shl(start as u32)) * is_valid""",
        """        let start = aux & 0x3F;
        let end = (aux >> 8) & 0x3F;
        let mut res = 0;
        if start < end {
            for i in start..end {
                if i < 64 {
                    res |= 1 << i;
                }
            }
        }
        res"""
    ),
    "rotate_left_u64": (
        "    val.rotate_left((aux & 0x3F) as u32)",
        """        let shift = (aux & 0x3F) as u32;
        val.rotate_left(shift)"""
    ),
    "rotate_right_u64": (
        "    val.rotate_right((aux & 0x3F) as u32)",
        """        let shift = (aux & 0x3F) as u32;
        val.rotate_right(shift)"""
    ),
    "funnel_shift_left_u64": (
        """    let shift = (aux & 0x3F) as u32;
    let res = (val.wrapping_shl(shift)) | (aux.wrapping_shr((64u32.wrapping_sub(shift)) & 0x3F) & (0u64.wrapping_sub((shift != 0) as u64)));
    res""",
        """        let shift = aux & 0x3F;
        if shift == 0 {
            val
        } else {
            (val << shift) | (aux >> (64 - shift))
        }"""
    ),
    "funnel_shift_right_u64": (
        """    let shift = (aux & 0x3F) as u32;
    let res = (aux.wrapping_shr(shift)) | (val.wrapping_shl((64u32.wrapping_sub(shift)) & 0x3F) & (0u64.wrapping_sub((shift != 0) as u64)));
    res""",
        """        let shift = aux & 0x3F;
        if shift == 0 {
            aux
        } else {
            (aux >> shift) | (val << (64 - shift))
        }"""
    ),
    "bit_swap_u64": (
        """    let i = (aux & 0x3F) as u32;
    let j = ((aux >> 8) & 0x3F) as u32;
    let bit_i = (val.wrapping_shr(i)) & 1;
    let bit_j = (val.wrapping_shr(j)) & 1;
    let xor_val = bit_i ^ bit_j;
    val ^ ((xor_val.wrapping_shl(i)) | (xor_val.wrapping_shl(j)))""",
        """        let i = aux & 0x3F;
        let j = (aux >> 8) & 0x3F;
        let mut res = val;
        if ((val >> i) & 1) != ((val >> j) & 1) {
            res ^= (1 << i) | (1 << j);
        }
        res"""
    ),
    "gather_bits_u64": (
        """    let mut res = 0;
    let mut r_idx = 0;
    for i in 0..64 {
        let mask_bit = (aux >> i) & 1;
        let val_bit = (val >> i) & 1;
        res |= (val_bit & mask_bit).wrapping_shl(r_idx);
        r_idx += mask_bit as u32;
    }
    res""",
        """        let mut res = 0;
        let mut r_idx = 0;
        for i in 0..64 {
            if ((aux >> i) & 1) == 1 {
                if ((val >> i) & 1) == 1 {
                    res |= 1 << r_idx;
                }
                r_idx += 1;
            }
        }
        res"""
    ),
    "scatter_bits_u64": (
        """    let mut res = 0;
    let mut v_idx = 0;
    for i in 0..64 {
        let mask_bit = (aux >> i) & 1;
        let val_bit = (val.wrapping_shr(v_idx)) & 1;
        res |= (val_bit & mask_bit) << i;
        v_idx += mask_bit as u32;
    }
    res""",
        """        let mut res = 0;
        let mut v_idx = 0;
        for i in 0..64 {
            if ((aux >> i) & 1) == 1 {
                if ((val.wrapping_shr(v_idx)) & 1) == 1 {
                    res |= 1 << i;
                }
                v_idx += 1;
            }
        }
        res"""
    ),
}

# Now check each file and identify what target block to replace.
# In each file, we want to locate the block starting from `pub fn algo` and ending after `fn algo_reference` (before mutants or after them).
# Let's write out the target content and replacement content.

report = {}

for algo in partition1:
    path = os.path.join(base_dir, f"{algo}.rs")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Let's locate `pub fn {algo}`
    start_pat = f"pub fn {algo}"
    start_idx = content.find(start_pat)
    if start_idx == -1:
        print(f"ERROR: could not find pub fn {algo} in {algo}.rs")
        continue
    
    # We want to find where the mutants begin or where tests begin.
    # In these files, we have:
    # pub fn algo(...) -> u64 {
    #    ...
    # }
    #
    # #[cfg(test)]
    # mod tests {
    #     ...
    #     fn algo_reference(...) -> u64 {
    #         ...
    #     }
    #     ...
    #     // NEGATIVE MUTANTS or proptest! or similar.
    # We can capture the range from start_idx up to the mutants comment.
    mutants_comment = "// NEGATIVE MUTANTS"
    mutants_idx = content.find(mutants_comment, start_idx)
    
    # If mutants_comment is not found, try finding mutant_ definitions
    if mutants_idx == -1:
        mutants_idx = content.find(f"fn mutant_{algo}", start_idx)
        
    if mutants_idx == -1:
        print(f"ERROR: could not find mutants block in {algo}.rs")
        continue
    
    # Let's extract the target chunk of text
    target_text = content[start_idx:mutants_idx]
    
    # Construct the clean replacement text
    impl_body, ref_body = logic_db[algo]
    
    replacement_text = f"pub fn {algo}(val: u64, aux: u64) -> u64 {{\n{impl_body}\n}}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n    use proptest::prelude::*;\n    \n    // -------------------------------------------------------------------------\n    // POSITIVE ORACLE: Reference implementation\n    // -------------------------------------------------------------------------\n    fn {algo}_reference(val: u64, aux: u64) -> u64 {{\n{ref_body}\n    }}\n\n    "
    
    report[algo] = {
        "file": path,
        "target": target_text,
        "replacement": replacement_text
    }

with open("/Users/sac/bcinr/.agents/worker_v5_part1/replacements.json", "w") as f:
    json.dump(report, f, indent=2)

print("Done. Generated replacements.json with", len(report), "entries.")
