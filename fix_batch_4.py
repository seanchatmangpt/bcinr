import os

files_to_fix = {
    "fp_div_u32_q16.rs": {
        "ref": """
        let dividend = (val as u32) as u64;
        let divisor = (aux as u32) as u64;
        if divisor == 0 { return u64::MAX; }
        ((dividend << 16) / divisor) as u64
        """,
        "impl": """
    let dividend = (val as u32) as u64;
    let divisor = (aux as u32) as u64;
    let is_zero = (divisor == 0) as u64;
    let safe_divisor = divisor | is_zero;
    let res = (dividend << 16) / safe_divisor;
    (res & !is_zero.wrapping_neg()) | (is_zero.wrapping_neg() & u64::MAX)
        """
    },
    "frustum_culling_branchless.rs": {
        "ref": """
        let x = (val >> 32) as i32;
        let y = val as i32;
        let min_x = (aux >> 48) as i16 as i32;
        let max_x = ((aux >> 32) & 0xFFFF) as i16 as i32;
        let min_y = ((aux >> 16) & 0xFFFF) as i16 as i32;
        let max_y = (aux & 0xFFFF) as i16 as i32;
        if x >= min_x && x <= max_x && y >= min_y && y <= max_y { 1 } else { 0 }
        """,
        "impl": """
    let x = (val >> 32) as i32;
    let y = val as i32;
    let min_x = (aux >> 48) as i16 as i32;
    let max_x = ((aux >> 32) & 0xFFFF) as i16 as i32;
    let min_y = ((aux >> 16) & 0xFFFF) as i16 as i32;
    let max_y = (aux & 0xFFFF) as i16 as i32;
    ((x >= min_x) & (x <= max_x) & (y >= min_y) & (y <= max_y)) as u64
        """
    },
    "gather_bits_u64.rs": {
        "ref": """
        let mut res = 0;
        let mut p = 0;
        for i in 0..64 {
            if (aux >> i) & 1 == 1 {
                res |= ((val >> i) & 1) << p;
                p += 1;
            }
        }
        res
        """,
        "impl": """
    let mut res = 0;
    let mut p = 0;
    for i in 0..64 {
        let bit_aux = (aux >> i) & 1;
        let bit_val = (val >> i) & 1;
        res |= (bit_val * bit_aux) << p;
        p += bit_aux;
    }
    res
        """
    },
    "get_mask_boundary_high_u64.rs": {
        "ref": """
        let mut m = val;
        m |= m >> 1; m |= m >> 2; m |= m >> 4;
        m |= m >> 8; m |= m >> 16; m |= m >> 32;
        m
        """,
        "impl": """
    let mut m = val;
    m |= m >> 1; m |= m >> 2; m |= m >> 4;
    m |= m >> 8; m |= m >> 16; m |= m >> 32;
    m
        """
    },
    "graph_bfs_simd_step.rs": {
        "ref": """
        val & !aux
        """,
        "impl": """
    val & !aux
        """
    },
    "graph_dfs_bit_parallel.rs": {
        "ref": """
        let unvisited = val & !aux;
        if unvisited == 0 { 0 } else { 1u64.wrapping_shl(unvisited.trailing_zeros() as u32) }
        """,
        "impl": """
    let unvisited = val & !aux;
    unvisited & unvisited.wrapping_neg()
        """
    },
    "green_sorting_network_16.rs": {
        "ref": """
        let a = (val >> 32) as u32;
        let b = val as u32;
        let min = if a < b { a } else { b };
        let max = if a > b { a } else { b };
        ((max as u64) << 32) | (min as u64)
        """,
        "impl": """
    let a = (val >> 32) as u32;
    let b = val as u32;
    let mask = ((a < b) as u32).wrapping_neg();
    let min = (a & mask) | (b & !mask);
    let max = (a & !mask) | (b & mask);
    ((max as u64) << 32) | (min as u64)
        """
    },
    "heavy_keepers_add.rs": {
        "ref": """
        let idx = (aux & 0xF) * 4;
        let count = (val >> idx) & 0xF;
        let new_count = if count < 15 { count + 1 } else { 15 };
        (val & !(0xF << idx)) | (new_count << idx)
        """,
        "impl": """
    let idx = (aux & 0xF) * 4;
    let count = (val >> idx) & 0xF;
    let is_not_max = (count < 15) as u64;
    let new_count = count + is_not_max;
    (val & !(0xF << idx)) | (new_count << idx)
        """
    },
    "hex_encode_chunk8.rs": {
        "ref": """
        let mut res = 0u64;
        for i in 0..8 {
            let nibble = (val >> (i * 4)) & 0xF;
            let char = if nibble < 10 { 48 + nibble } else { 97 + nibble - 10 };
            res |= char << (i * 8);
        }
        res
        """,
        "impl": """
    let mut res = 0u64;
    for i in 0..8 {
        let nibble = (val >> (i * 4)) & 0xF;
        let is_alpha = (nibble > 9) as u64;
        let char = 48 + nibble + is_alpha * 39;
        res |= char << (i * 8);
    }
    res
        """
    },
    "hex_encode_simd.rs": {
        "ref": """
        let mut res = 0u64;
        for i in 0..8 {
            let nibble = (val >> (i * 4)) & 0xF;
            let char = if nibble < 10 { 48 + nibble } else { 97 + nibble - 10 };
            res |= char << (i * 8);
        }
        res
        """,
        "impl": """
    let mut res = 0u64;
    for i in 0..8 {
        let nibble = (val >> (i * 4)) & 0xF;
        let is_alpha = (nibble > 9) as u64;
        let char = 48 + nibble + is_alpha * 39;
        res |= char << (i * 8);
    }
    res
        """
    }
}

import re

base_dir = "crates/bcinr-logic/src/algorithms"

def rewrite_file(filename, config):
    path = os.path.join(base_dir, filename)
    if not os.path.exists(path):
        return
    with open(path, "r") as f:
        content = f.read()

    # Replace implementation
    # The dummy implementation looks like:
    # pub fn name(val: u64, aux: u64) -> u64 {
    #     (...)
    # }
    
    # We will use regex to find the function body and replace it.
    name = filename.replace(".rs", "")
    
    impl_pattern = re.compile(r"pub fn " + name + r"\(val: u64, aux: u64\) -> u64 \{.*?\n\}", re.DOTALL)
    new_impl = f"pub fn {name}(val: u64, aux: u64) -> u64 {{{config['impl']}}}"
    content = impl_pattern.sub(new_impl, content)
    
    ref_pattern = re.compile(r"fn " + name + r"_reference\(val: u64, aux: u64\) -> u64 \{.*?\n    \}", re.DOTALL)
    new_ref = f"fn {name}_reference(val: u64, aux: u64) -> u64 {{{config['ref']}    }}"
    content = ref_pattern.sub(new_ref, content)
    
    with open(path, "w") as f:
        f.write(content)

for fname, conf in files_to_fix.items():
    rewrite_file(fname, conf)

print("Rewrote the 10 files.")

