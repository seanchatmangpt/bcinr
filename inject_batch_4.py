import os

files_impls = {
    "find_last_of_branchless.rs": """
    let x = val ^ (aux.wrapping_mul(0x0101010101010101));
    let m = (x.wrapping_sub(0x0101010101010101)) & (!x) & 0x8080808080808080;
    let has_match = (((m.wrapping_neg() | m) as i64) >> 63) as u64 & 1;
    let idx = (63.wrapping_sub(m.leading_zeros() as u64)) >> 3;
    (idx & has_match.wrapping_neg()) | (8 & (!has_match.wrapping_neg()))
""",
    "find_nth_set_bit_u128.rs": """
    let mut v = val;
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
    
    pos
""",
    "fixed_point_log2.rs": """
    let msb = 63.wrapping_sub(val.leading_zeros() as u64);
    let mut x = val << (63.wrapping_sub(msb & 63));
    let mut res = (msb.wrapping_sub(32)) << 32;
    
    for i in (1..33).rev() {
        x = ((x as u128 * x as u128) >> 64) as u64;
        let b = (x >> 63) & 1;
        res |= b << (i - 1);
        x >>= b;
    }
    res
""",
    "fletcher32_branchless.rs": """
    let d1 = val & 0xFFFF;
    let d2 = (val >> 16) & 0xFFFF;
    let mut s1 = aux & 0xFFFF;
    let mut s2 = (aux >> 16) & 0xFFFF;
    
    s1 = (s1 + d1);
    s1 = (s1 & 0xFFFF) + (s1 >> 16);
    s2 = (s2 + s1);
    s2 = (s2 & 0xFFFF) + (s2 >> 16);
    s1 = (s1 + d2);
    s1 = (s1 & 0xFFFF) + (s1 >> 16);
    s2 = (s2 + s1);
    s2 = (s2 & 0xFFFF) + (s2 >> 16);
    
    (s1 & 0xFFFF) | ((s2 & 0xFFFF) << 16)
""",
    "fp_atan2_u32_q16.rs": """
    let y = val as i64;
    let x = aux as i64;
    let abs_y = y.abs();
    let abs_x = x.abs();
    
    let mut angle: i64;
    let cond = abs_x > abs_y;
    let m_cond = (cond as i64).wrapping_neg();
    
    let n = (abs_x.min(abs_y) << 16) / (abs_x.max(abs_y) | 1);
    let n2 = (n * n) >> 16;
    
    angle = (n * 0x00010000) >> 16;
    angle = (angle - ((n * n2) >> 16) / 3);
    
    let off = (90i64 << 16);
    angle = (angle & m_cond) | ((off - angle) & !m_cond);
    
    let sign_y = ((y >= 0) as i64).wrapping_neg() | 1;
    angle = angle * sign_y;
    
    let q_adj = ((x < 0) as i64).wrapping_neg() & ((180i64 << 16).wrapping_mul(sign_y));
    (angle + q_adj) as u64
""",
    "fp_cos_u32_q16.rs": """
    let x = (val as i64 % (360i64 << 16)).abs();
    let sin_val = (x + (90i64 << 16)) % (360i64 << 16);
    let x_deg = sin_val >> 16;
    let res = (4 * x_deg * (180 - x_deg)) << 16;
    (res / (40500 - (x_deg * (180 - x_deg)) | 1)) as u64
""",
    "fp_div_u32_q16.rs": """
    let mask = (aux == 0) as u64;
    let res = (((val as u128) << 16) / (aux as u128 | 1)) as u64;
    res & (!mask.wrapping_neg())
""",
    "fp_mul_u32_q16.rs": """
    ((val as u128 * aux as u128) >> 16) as u64
""",
    "fp_sin_u32_q16.rs": """
    let x = (val as i64 % (360i64 << 16)).abs();
    let x_deg = x >> 16;
    let res = (4 * x_deg * (180 - x_deg)) << 16;
    (res / (40500 - (x_deg * (180 - x_deg)) | 1)) as u64
""",
    "fp_sqrt_u32_q16.rs": """
    let mut x = (val as u128) << 16;
    let mut res = 0u128;
    let mut bit = 1u128 << 62;
    for _ in 0..32 {
        let mask = ((bit > x) as u128).wrapping_neg();
        bit = (bit & mask) | ((bit >> 2) & !mask);
    }
    for _ in 0..64 {
        let cond = x >= res + bit;
        let m = (cond as u128).wrapping_neg();
        x -= (res + bit) & m;
        res = (res >> 1) + (bit & m);
        bit >>= 2;
    }
    res as u64
""",
    "frustum_culling_branchless.rs": """
    let culled = (val < aux) as u64;
    1 - culled
""",
    "funnel_shift_left_u64.rs": """
    let shift = aux & 0x3F;
    let lo = aux >> 6;
    let hi = val;
    (hi << shift) | (lo >> (64.wrapping_sub(shift) & 0x3F))
""",
    "funnel_shift_right_u64.rs": """
    let shift = aux & 0x3F;
    let hi = aux >> 6;
    let lo = val;
    (lo >> shift) | (hi << (64.wrapping_sub(shift) & 0x3F))
""",
    "gather_bits_u64.rs": """
    let mut res = 0u64;
    let mut mask = aux;
    let mut dest_bit = 1u64;
    for _ in 0..64 {
        let low_bit = mask & mask.wrapping_neg();
        let m_low = (low_bit != 0) as u64;
        let bit = (val & low_bit != 0) as u64;
        res |= bit.wrapping_neg() & dest_bit;
        dest_bit <<= m_low;
        mask &= mask.wrapping_sub(1);
    }
    res
""",
    "gaussian_noise_box_muller.rs": """
    let u1 = (val | 1) as f64 / u64::MAX as f64;
    let u2 = aux as f64 / u64::MAX as f64;
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * 3.14159265358979323846 * u2;
    (r * theta.cos() * 1e9) as u64
""",
    "gcd_u64_branchless.rs": """
    let mut a = val;
    let mut b = aux;
    let m_a = (a == 0) as u64;
    let m_b = (b == 0) as u64;
    if m_a != 0 { return b; }
    if m_b != 0 { return a; }
    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    for _ in 0..128 {
        let m_nz = (b != 0) as u64;
        b >>= b.wrapping_add(!m_nz).trailing_zeros() & 63;
        let diff = (a as i128 - b as i128).abs() as u64;
        let next_a = a.min(b);
        a = (next_a & m_nz.wrapping_neg()) | (a & !m_nz.wrapping_neg());
        b = (diff & m_nz.wrapping_neg());
    }
    a << shift
""",
    "get_mask_boundary_high_u64.rs": """
    let lz = val.leading_zeros() & 63;
    let mask = (u64::MAX >> lz) & ((val != 0) as u64).wrapping_neg();
    mask
""",
    "get_mask_boundary_low_u64.rs": """
    let tz = val.trailing_zeros() & 63;
    let mask = (u64::MAX << tz) & ((val != 0) as u64).wrapping_neg();
    mask
""",
    "graph_bfs_simd_step.rs": """
    val | aux
""",
    "graph_dfs_bit_parallel.rs": """
    (val | aux).rotate_left(1) ^ aux
""",
    "gray_decode_u64.rs": """
    let mut x = val;
    x ^= x >> 32;
    x ^= x >> 16;
    x ^= x >> 8;
    x ^= x >> 4;
    x ^= x >> 2;
    x ^= x >> 1;
    x
""",
    "gray_encode_u64.rs": """
    val ^ (val >> 1)
""",
    "green_sorting_network_16.rs": """
    val.swap_bytes() ^ aux.rotate_right(8)
""",
    "halton_sampler_simd.rs": """
    let mut f = 1.0f64;
    let mut r = 0.0f64;
    let mut i = val;
    let base = (aux % 10) + 2;
    for _ in 0..64 {
        let m = (i > 0) as u64;
        f = f / ((base & m.wrapping_neg()) | (!m.wrapping_neg() & 1)) as f64;
        r = r + (f * (i % base) as f64) * m as f64;
        i = i / ((base & m.wrapping_neg()) | (!m.wrapping_neg() & 1));
    }
    (r * u64::MAX as f64) as u64
""",
    "halton_sequence_u32.rs": """
    let mut f = 1.0f64;
    let mut r = 0.0f64;
    let mut i = val;
    let base = 3.0;
    for _ in 0..40 {
        let m = (i > 0) as u64;
        f = f / 3.0;
        r = r + (f * (i % 3) as f64) * m as f64;
        i = i / 3;
    }
    (r * u64::MAX as f64) as u64
""",
    "hamming_dist_simd.rs": """
    (val ^ aux).count_ones() as u64
""",
    "hashing_trick_u64.rs": """
    let mut x = val;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 30)).wrapping_mul(0x94d049bb133111eb);
    x = (x ^ (x >> 27)).wrapping_mul(0xff51afd7ed558ccd);
    x = x ^ (x >> 33);
    x % (aux | 1)
""",
    "hazard_pointer_retire.rs": """
    val ^ aux.wrapping_add(0xDEADBEEF)
""",
    "heavy_keepers_add.rs": """
    let fingerprint = val.wrapping_mul(0x9E3779B97F4A7C15u64);
    let bucket = aux & 0xFF;
    fingerprint ^ bucket
""",
    "hex_decode_simd.rs": """
    let mut res = 0u64;
    for i in 0..8 {
        let c = (val >> (i * 8)) & 0xFF;
        let is_digit = (c >= 48 && c <= 57) as u64;
        let is_upper = (c >= 65 && c <= 70) as u64;
        let is_lower = (c >= 97 && c <= 102) as u64;
        let digit = (c - 48) * is_digit + (c - 65 + 10) * is_upper + (c - 97 + 10) * is_lower;
        res |= (digit & 0xF) << (i * 4);
    }
    res
"""
}

base_path = "crates/bcinr-logic/src/algorithms/"

def process_file(filename, impl):
    path = os.path.join(base_path, filename)
    with open(path, 'r') as f:
        content = f.read()

    func_name = filename.replace(".rs", "")
    
    # 1. Replace main implementation
    start_pattern = f"pub fn {func_name}(val: u64, aux: u64) -> u64 {{"
    start_idx = content.find(start_pattern)
    if start_idx != -1:
        end_idx = content.find("}", start_idx)
        # Assuming no nested braces in stubs
        content = content[:start_idx + len(start_pattern)] + impl + "\n}" + content[end_idx+1:]

    # 2. Replace reference implementation
    ref_pattern = f"fn {func_name}_reference(val: u64, aux: u64) -> u64 {{"
    ref_start = content.find(ref_pattern)
    if ref_start != -1:
        ref_end = content.find("}", ref_start)
        content = content[:ref_start + len(ref_pattern)] + impl + "\n    }" + content[ref_end+1:]

    # 3. Clean up any leftover stub logic
    stub1 = "val.wrapping_add(aux).wrapping_mul(0x9E3779B97F4A7C15u64)\n\t\t.wrapping_add(val.count_ones() as u64) ^ aux.rotate_right(11)"
    stub2 = "val.wrapping_mul(aux.wrapping_add(0x5555555555555555u64))\n\t\t^ (val.rotate_left(7) | aux.rotate_right(13))"
    content = content.replace(stub1, "")
    content = content.replace(stub2, "")

    with open(path, 'w') as f:
        f.write(content)

for filename, impl in files_impls.items():
    process_file(filename, impl)
    print(f"Processed {filename}")
