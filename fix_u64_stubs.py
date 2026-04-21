import glob

algorithms = {
    "parallel_bits_deposit_u64": """    let mut res = 0u64;
    let mut v = val;
    let mut m = aux;
    let mut pos = 1u64;
    let mut i = 0;
    while i < 64 {
        let m_bit = m & 1;
        let v_bit = v & 1;
        res |= (m_bit & v_bit).wrapping_mul(pos);
        v >>= m_bit;
        m >>= 1;
        pos <<= 1;
        i += 1;
    }
    res""",
    "parallel_bits_extract_u64": """    let mut res = 0u64;
    let mut v = val;
    let mut m = aux;
    let mut pos = 1u64;
    let mut i = 0;
    while i < 64 {
        let m_bit = m & 1;
        let v_bit = v & 1;
        res |= (m_bit & v_bit).wrapping_mul(pos);
        pos <<= m_bit;
        v >>= 1;
        m >>= 1;
        i += 1;
    }
    res""",
    "bclr_u64": "    val & !(1u64.wrapping_shl(aux as u32 & 0x3F))",
    "bext_u64": "    (val.wrapping_shr(aux as u32 & 0x3F)) & 1",
    "bset_u64": "    val | 1u64.wrapping_shl(aux as u32 & 0x3F)",
    "btst_u64": "    (val.wrapping_shr(aux as u32 & 0x3F)) & 1",
    "blsi_u64": "    val & val.wrapping_neg()",
    "blsmsk_u64": "    val ^ val.wrapping_sub(1)",
    "blsr_u64": "    val & val.wrapping_sub(1)",
    "tzmsk_u64": "    !val & (val.wrapping_sub(1))",
    "t1mskc_u64": "    !val | (val.wrapping_add(1))",
    "rotate_left_u64": "    val.rotate_left(aux as u32 & 0x3F)",
    "rotate_right_u64": "    val.rotate_right(aux as u32 & 0x3F)",
    "gray_encode_u64": "    val ^ (val >> 1)",
    "gray_decode_u64": """    let mut res = val;
    res ^= res >> 32;
    res ^= res >> 16;
    res ^= res >> 8;
    res ^= res >> 4;
    res ^= res >> 2;
    res ^= res >> 1;
    res""",
    "log2_u64_fixed": """    let nz = (val != 0) as u64;
    let mask = 0u64.wrapping_sub(nz);
    ((63 - val.leading_zeros() as u64) & mask)""",
}

for name, new_body in algorithms.items():
    filepath = f"crates/bcinr-logic/src/algorithms/{name}.rs"
    try:
        with open(filepath, "r") as f:
            lines = f.readlines()
        
        # We expect two functions to replace the body for: pub fn name(..) and fn name_reference(..)
        # Both currently have a 1-line body containing `wrapping_add(!(val & aux) & (val | aux))`
        
        out_lines = []
        for line in lines:
            if "wrapping_add(!(val & aux) & (val | aux))" in line:
                out_lines.append(new_body + "\n")
            else:
                out_lines.append(line)
                
        with open(filepath, "w") as f:
            f.writelines(out_lines)
            
        print(f"Updated {name}")
    except Exception as e:
        print(f"Failed {name}: {e}")

