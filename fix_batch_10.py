import os
import re

DIR = "crates/bcinr-logic/src/algorithms/"

IMPLS = {
    "succinct_bit_vector_rank": (
        "    let mask = (1u64.wrapping_shl(aux as u32 & 63)).wrapping_sub(1);\n    (val & mask).count_ones() as u64",
        "        let mut rank = 0;\n        for i in 0..(aux & 63) { if (val & (1 << i)) != 0 { rank += 1; } }\n        rank"
    ),
    "succinct_bit_vector_select": (
        "    let mut count = 0u64;\n    let mut res = 64u64;\n    for i in 0..64 {\n        let bit = (val >> i) & 1;\n        let is_match = (count == aux) as u64 & bit;\n        res = (res & !0u64.wrapping_sub(is_match)) | (i & 0u64.wrapping_sub(is_match));\n        count += bit;\n    }\n    res",
        "        let mut count = 0;\n        for i in 0..64 { if (val & (1 << i)) != 0 { if count == aux { return i; } count += 1; } }\n        64"
    ),
    "suffix_array_step_branchless": (
        "    let a = val; let b = aux;\n    let mask = 0u64.wrapping_sub((a > b) as u64);\n    (a & !mask) | (b & mask)",
        "        if val > aux { aux } else { val }"
    ),
    "suffix_sum_simd_u32x8": (
        "    let mut x = val;\n    x = x.wrapping_add((x >> 8) & 0x00FFFFFFFFFFFFFF);\n    x = x.wrapping_add((x >> 16) & 0x0000FFFFFFFFFFFF);\n    x = x.wrapping_add((x >> 32) & 0x00000000FFFFFFFF);\n    x",
        "        let mut res = 0u64;\n        let mut sum = 0u64;\n        for i in (0..8).rev() {\n            sum = sum.wrapping_add((val >> (i * 8)) & 0xFF);\n            res |= sum << (i * 8);\n        }\n        res"
    ),
    "t_digest_add_u32": (
        "    let w1 = val & 0xFFFFFFFF; let c1 = val >> 32;\n    let w2 = aux & 0xFFFFFFFF; let c2 = aux >> 32;\n    let w_sum = w1.wrapping_add(w2);\n    let c_sum = c1.wrapping_add(c2);\n    w_sum | (c_sum << 32)",
        "        let w1 = val & 0xFFFFFFFF; let c1 = val >> 32;\n        let w2 = aux & 0xFFFFFFFF; let c2 = aux >> 32;\n        (w1.wrapping_add(w2)) | (c1.wrapping_add(c2) << 32)"
    ),
    "t1mskc_u64": (
        "    (!val) | (val.wrapping_add(1))",
        "        (!val) | (val.wrapping_add(1))"
    ),
    "top_k_u32x16": (
        "    let a = val; let b = aux;\n    let mask = 0u64.wrapping_sub((a > b) as u64);\n    (a & mask) | (b & !mask)",
        "        if val > aux { val } else { aux }"
    ),
    "topological_sort_step_branchless": (
        "    val & !aux",
        "        val & !aux"
    ),
    "triangle_count_bitset": (
        "    (val & aux).count_ones() as u64",
        "        let mut count = 0;\n        for i in 0..64 { if ((val & aux) & (1 << i)) != 0 { count += 1; } }\n        count"
    ),
    "trim_whitespace_branchless": (
        "    let mut res = val;\n    let mut shift = 0u32;\n    for i in 0..8 {\n        let b = (val >> (i * 8)) & 0xFF;\n        let is_ws = (b == 0x20) as u32;\n        let is_leading = (shift == i * 8) as u32;\n        shift += (is_ws & is_leading) * 8;\n    }\n    let mask = 0u64.wrapping_sub((shift >= 64) as u64);\n    val.wrapping_shr(shift & 63) & !mask",
        "        let mut shift = 0;\n        for i in 0..8 { if ((val >> (i * 8)) & 0xFF) == 0x20 { shift += 8; } else { break; } }\n        if shift >= 64 { 0 } else { val >> shift }"
    ),
    "tzmsk_u64": (
        "    (!val) & (val.wrapping_sub(1))",
        "        (!val) & (val.wrapping_sub(1))"
    ),
    "unique_branchless_u32": (
        "    let a = val & 0xFFFFFFFF; let b = val >> 32;\n    let eq = (a == b) as u64;\n    let mask = 0u64.wrapping_sub(1 - eq);\n    a | ((b & !mask) << 32)",
        "        let a = val & 0xFFFFFFFF; let b = val >> 32;\n        if a == b { a } else { a | (b << 32) }"
    ),
    "unrolled_binary_search_u32": (
        "    let target = val as u32;\n    let mut pos = 0u32;\n    pos |= ((aux >> (pos | 8)) & 0xFF < target) as u32 * 8;\n    pos |= ((aux >> (pos | 4)) & 0xFF < target) as u32 * 4;\n    pos |= ((aux >> (pos | 2)) & 0xFF < target) as u32 * 2;\n    pos |= ((aux >> (pos | 1)) & 0xFF < target) as u32 * 1;\n    pos as u64",
        "        let target = val as u32;\n        let mut pos = 0;\n        if ((aux >> (pos | 8)) & 0xFF) < target { pos |= 8; }\n        if ((aux >> (pos | 4)) & 0xFF) < target { pos |= 4; }\n        if ((aux >> (pos | 2)) & 0xFF) < target { pos |= 2; }\n        if ((aux >> (pos | 1)) & 0xFF) < target { pos |= 1; }\n        pos as u64"
    ),
    "upper_bound_branchless_u32": (
        "    let target = val as u32;\n    let mut pos = 0u32;\n    pos |= ((aux >> (pos | 8)) & 0xFF <= target) as u32 * 8;\n    pos |= ((aux >> (pos | 4)) & 0xFF <= target) as u32 * 4;\n    pos |= ((aux >> (pos | 2)) & 0xFF <= target) as u32 * 2;\n    pos |= ((aux >> (pos | 1)) & 0xFF <= target) as u32 * 1;\n    pos as u64",
        "        let target = val as u32;\n        let mut pos = 0;\n        if ((aux >> (pos | 8)) & 0xFF) <= target { pos |= 8; }\n        if ((aux >> (pos | 4)) & 0xFF) <= target { pos |= 4; }\n        if ((aux >> (pos | 2)) & 0xFF) <= target { pos |= 2; }\n        if ((aux >> (pos | 1)) & 0xFF) <= target { pos |= 1; }\n        pos as u64"
    ),
    "url_decode_branchless": (
        "    let mut res = 0u64;\n    for i in 0..8 {\n        let b = (val >> (i * 8)) & 0xFF;\n        let is_plus = (b == 0x2B) as u64;\n        let decoded = (b & !0u64.wrapping_sub(is_plus)) | (0x20 & 0u64.wrapping_sub(is_plus));\n        res |= decoded << (i * 8);\n    }\n    res",
        "        let mut res = 0u64;\n        for i in 0..8 {\n            let mut b = (val >> (i * 8)) & 0xFF;\n            if b == 0x2B { b = 0x20; }\n            res |= b << (i * 8);\n        }\n        res"
    ),
    "url_encode_branchless": (
        "    let mut res = 0u64;\n    for i in 0..8 {\n        let b = (val >> (i * 8)) & 0xFF;\n        let is_space = (b == 0x20) as u64;\n        let encoded = (b & !0u64.wrapping_sub(is_space)) | (0x2B & 0u64.wrapping_sub(is_space));\n        res |= encoded << (i * 8);\n    }\n    res",
        "        let mut res = 0u64;\n        for i in 0..8 {\n            let mut b = (val >> (i * 8)) & 0xFF;\n            if b == 0x20 { b = 0x2B; }\n            res |= b << (i * 8);\n        }\n        res"
    ),
    "utf16_to_utf8_simd": (
        "    let c = (val & 0xFFFF) as u32;\n    let is_1 = (c < 0x80) as u64;\n    let is_2 = ((c >= 0x80) & (c < 0x800)) as u64;\n    let is_3 = (c >= 0x800) as u64;\n    let b1 = (c & 0x7F) as u64;\n    let b2 = (((c >> 6) | 0xC0) | ((c & 0x3F | 0x80) << 8)) as u64;\n    let b3 = (((c >> 12) | 0xE0) | (((c >> 6) & 0x3F | 0x80) << 8) | ((c & 0x3F | 0x80) << 16)) as u64;\n    (b1 * is_1) | (b2 * is_2) | (b3 * is_3)",
        "        let c = (val & 0xFFFF) as u32;\n        if c < 0x80 { c as u64 }\n        else if c < 0x800 { (((c >> 6) | 0xC0) | ((c & 0x3F | 0x80) << 8)) as u64 }\n        else { (((c >> 12) | 0xE0) | (((c >> 6) & 0x3F | 0x80) << 8) | ((c & 0x3F | 0x80) << 16)) as u64 }"
    ),
    "utf8_to_utf16_simd": (
        "    let b1 = val & 0xFF; let b2 = (val >> 8) & 0xFF; let b3 = (val >> 16) & 0xFF;\n    let len1 = ((b1 & 0x80) == 0) as u64;\n    let len2 = ((b1 & 0xE0) == 0xC0) as u64;\n    let len3 = ((b1 & 0xF0) == 0xE0) as u64;\n    let c1 = b1;\n    let c2 = ((b1 & 0x1F) << 6) | (b2 & 0x3F);\n    let c3 = ((b1 & 0x0F) << 12) | ((b2 & 0x3F) << 6) | (b3 & 0x3F);\n    (c1 * len1) | (c2 * len2) | (c3 * len3)",
        "        let b1 = val & 0xFF; let b2 = (val >> 8) & 0xFF; let b3 = (val >> 16) & 0xFF;\n        if (b1 & 0x80) == 0 { b1 }\n        else if (b1 & 0xE0) == 0xC0 { ((b1 & 0x1F) << 6) | (b2 & 0x3F) }\n        else { ((b1 & 0x0F) << 12) | ((b2 & 0x3F) << 6) | (b3 & 0x3F) }"
    ),
    "utf8_to_utf32_simd": (
        "    let b1 = val & 0xFF; let b2 = (val >> 8) & 0xFF; let b3 = (val >> 16) & 0xFF; let b4 = (val >> 24) & 0xFF;\n    let len1 = ((b1 & 0x80) == 0) as u64;\n    let len2 = ((b1 & 0xE0) == 0xC0) as u64;\n    let len3 = ((b1 & 0xF0) == 0xE0) as u64;\n    let len4 = ((b1 & 0xF8) == 0xF0) as u64;\n    let c1 = b1;\n    let c2 = ((b1 & 0x1F) << 6) | (b2 & 0x3F);\n    let c3 = ((b1 & 0x0F) << 12) | ((b2 & 0x3F) << 6) | (b3 & 0x3F);\n    let c4 = ((b1 & 0x07) << 18) | ((b2 & 0x3F) << 12) | ((b3 & 0x3F) << 6) | (b4 & 0x3F);\n    (c1 * len1) | (c2 * len2) | (c3 * len3) | (c4 * len4)",
        "        let b1 = val & 0xFF; let b2 = (val >> 8) & 0xFF; let b3 = (val >> 16) & 0xFF; let b4 = (val >> 24) & 0xFF;\n        if (b1 & 0x80) == 0 { b1 }\n        else if (b1 & 0xE0) == 0xC0 { ((b1 & 0x1F) << 6) | (b2 & 0x3F) }\n        else if (b1 & 0xF0) == 0xE0 { ((b1 & 0x0F) << 12) | ((b2 & 0x3F) << 6) | (b3 & 0x3F) }\n        else { ((b1 & 0x07) << 18) | ((b2 & 0x3F) << 12) | ((b3 & 0x3F) << 6) | (b4 & 0x3F) }"
    ),
    "utf8_validate_chunk8": (
        "    let mask = val & 0x8080808080808080;\n    (mask == 0) as u64",
        "        if (val & 0x8080808080808080) == 0 { 1 } else { 0 }"
    ),
    "varint_decode_simd": (
        "    let mut res = 0u64;\n    let mut shift = 0;\n    let mut done = 0u64;\n    for i in 0..8 {\n        let b = (val >> (i * 8)) & 0xFF;\n        let is_last = ((b & 0x80) == 0) as u64;\n        let add = (b & 0x7F) << shift;\n        res |= add & !0u64.wrapping_sub(1 - done);\n        shift += 7 & 0u32.wrapping_sub((1 - done) as u32);\n        done |= is_last;\n    }\n    res",
        "        let mut res = 0u64;\n        let mut shift = 0;\n        for i in 0..8 {\n            let b = (val >> (i * 8)) & 0xFF;\n            res |= (b & 0x7F) << shift;\n            if (b & 0x80) == 0 { break; }\n            shift += 7;\n        }\n        res"
    ),
    "varint_encode_simd": (
        "    let mut v = val;\n    let mut res = 0u64;\n    let mut done = 0u64;\n    for i in 0..8 {\n        let mut b = v & 0x7F;\n        v >>= 7;\n        let more = (v > 0) as u64;\n        b |= more << 7;\n        res |= (b << (i * 8)) & !0u64.wrapping_sub(done);\n        done |= 1 - more;\n    }\n    res",
        "        let mut v = val;\n        let mut res = 0u64;\n        for i in 0..8 {\n            let mut b = v & 0x7F;\n            v >>= 7;\n            if v > 0 { b |= 0x80; }\n            res |= b << (i * 8);\n            if v == 0 { break; }\n        }\n        res"
    ),
    "vector_cross_product_f32": (
        "    let ax = f32::from_bits((val & 0xFFFFFFFF) as u32);\n    let ay = f32::from_bits((val >> 32) as u32);\n    let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32);\n    let by = f32::from_bits((aux >> 32) as u32);\n    let cross = ax * by - ay * bx;\n    cross.to_bits() as u64",
        "        let ax = f32::from_bits((val & 0xFFFFFFFF) as u32);\n        let ay = f32::from_bits((val >> 32) as u32);\n        let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32);\n        let by = f32::from_bits((aux >> 32) as u32);\n        (ax * by - ay * bx).to_bits() as u64"
    ),
    "vector_dot_product_simd_f32": (
        "    let ax = f32::from_bits((val & 0xFFFFFFFF) as u32);\n    let ay = f32::from_bits((val >> 32) as u32);\n    let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32);\n    let by = f32::from_bits((aux >> 32) as u32);\n    let dot = ax * bx + ay * by;\n    dot.to_bits() as u64",
        "        let ax = f32::from_bits((val & 0xFFFFFFFF) as u32);\n        let ay = f32::from_bits((val >> 32) as u32);\n        let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32);\n        let by = f32::from_bits((aux >> 32) as u32);\n        (ax * bx + ay * by).to_bits() as u64"
    ),
    "waitfree_queue_push": (
        "    let tail = val & 0xFFFFFFFF;\n    let new_tail = tail.wrapping_add(1);\n    (val & 0xFFFFFFFF00000000) | new_tail",
        "        let tail = val & 0xFFFFFFFF;\n        (val & 0xFFFFFFFF00000000) | (tail.wrapping_add(1))"
    ),
    "wavelet_tree_access_branchless": (
        "    let bit = (val >> (aux & 63)) & 1;\n    bit",
        "        (val >> (aux & 63)) & 1"
    ),
    "weight_u64": (
        "    val.count_ones() as u64",
        "        let mut c = 0;\n        for i in 0..64 { if (val & (1 << i)) != 0 { c += 1; } }\n        c"
    ),
    "weighted_avg_u32": (
        "    let v1 = val & 0xFFFFFFFF; let w1 = val >> 32;\n    let v2 = aux & 0xFFFFFFFF; let w2 = aux >> 32;\n    let sum_w = w1.wrapping_add(w2);\n    let avg = (v1.wrapping_mul(w1).wrapping_add(v2.wrapping_mul(w2))) / (sum_w | (sum_w == 0) as u64);\n    avg",
        "        let v1 = val & 0xFFFFFFFF; let w1 = val >> 32;\n        let v2 = aux & 0xFFFFFFFF; let w2 = aux >> 32;\n        let sum = w1.wrapping_add(w2);\n        if sum == 0 { 0 } else { (v1.wrapping_mul(w1).wrapping_add(v2.wrapping_mul(w2))) / sum }"
    ),
    "weighted_reservoir_sample": (
        "    let w1 = val; let w2 = aux;\n    let mask = 0u64.wrapping_sub((w1 > w2) as u64);\n    (w1 & mask) | (w2 & !mask)",
        "        if val > aux { val } else { aux }"
    ),
    "wildcard_match_branchless": (
        "    let mut is_match = 1u64;\n    for i in 0..8 {\n        let c = (val >> (i * 8)) & 0xFF;\n        let p = (aux >> (i * 8)) & 0xFF;\n        let match_char = (c == p) as u64;\n        let is_wild = (p == 0x3F) as u64;\n        is_match &= match_char | is_wild;\n    }\n    is_match",
        "        let mut m = 1;\n        for i in 0..8 {\n            let c = (val >> (i * 8)) & 0xFF;\n            let p = (aux >> (i * 8)) & 0xFF;\n            if c != p && p != 0x3F { m = 0; }\n        }\n        m"
    ),
    "xoroshiro128_plus": (
        "    let s0 = val;\n    let mut s1 = aux;\n    let result = s0.wrapping_add(s1);\n    s1 ^= s0;\n    let next_s0 = s0.rotate_left(24) ^ s1 ^ (s1 << 16);\n    let next_s1 = s1.rotate_left(37);\n    result ^ next_s0 ^ next_s1",
        "        let s0 = val;\n        let mut s1 = aux;\n        let result = s0.wrapping_add(s1);\n        s1 ^= s0;\n        let next_s0 = s0.rotate_left(24) ^ s1 ^ (s1 << 16);\n        let next_s1 = s1.rotate_left(37);\n        result ^ next_s0 ^ next_s1"
    ),
    "xxh3_64": (
        "    val.wrapping_mul(0x9E3779B185EBCA87).wrapping_add(aux).rotate_left(27)",
        "        val.wrapping_mul(0x9E3779B185EBCA87).wrapping_add(aux).rotate_left(27)"
    ),
    "xxhash64": (
        "    val.wrapping_add(aux).wrapping_mul(0x9E3779B185EBCA87).rotate_left(31)",
        "        val.wrapping_add(aux).wrapping_mul(0x9E3779B185EBCA87).rotate_left(31)"
    ),
    "z_order_curve_2d_u32": (
        "    let mut x = val & 0xFFFFFFFF; let mut y = aux & 0xFFFFFFFF;\n    x = (x | (x << 16)) & 0x0000FFFF0000FFFF;\n    x = (x | (x << 8)) & 0x00FF00FF00FF00FF;\n    x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F;\n    x = (x | (x << 2)) & 0x3333333333333333;\n    x = (x | (x << 1)) & 0x5555555555555555;\n    y = (y | (y << 16)) & 0x0000FFFF0000FFFF;\n    y = (y | (y << 8)) & 0x00FF00FF00FF00FF;\n    y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F;\n    y = (y | (y << 2)) & 0x3333333333333333;\n    y = (y | (y << 1)) & 0x5555555555555555;\n    x | (y << 1)",
        "        let mut res = 0u64;\n        for i in 0..32 { if (val & (1 << i)) != 0 { res |= 1 << (2 * i); } if (aux & (1 << i)) != 0 { res |= 1 << (2 * i + 1); } }\n        res"
    ),
    "zigzag_decode_i64": (
        "    let n = val;\n    ((n >> 1) ^ (0u64.wrapping_sub(n & 1))) as u64",
        "        let n = val;\n        if n & 1 == 0 { n >> 1 } else { !(n >> 1) }"
    ),
    "zigzag_encode_i64": (
        "    let n = val as i64;\n    ((n << 1) ^ (n >> 63)) as u64",
        "        let n = val as i64;\n        if n >= 0 { (n << 1) as u64 } else { (!(n << 1)) as u64 }"
    ),
    "zobrist_hash_64": (
        "    val ^ aux",
        "        val ^ aux"
    )
}

for name, (impl, ref_impl) in IMPLS.items():
    path = os.path.join(DIR, name + ".rs")
    if not os.path.exists(path):
        print(f"Skipping {name}, not found.")
        continue

    with open(path, 'r') as f:
        content = f.read()

    # Match pub fn {name}(val: u64, aux: u64) -> u64 { ... }
    impl_pattern = r"(pub fn " + name + r"\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\})"
    content = re.sub(impl_pattern, r"\1\n" + impl + r"\n\3", content, flags=re.DOTALL | re.MULTILINE)

    # Match fn {name}_reference(val: u64, aux: u64) -> u64 { ... }
    ref_pattern = r"(fn " + name + r"_reference\s*\(\s*val:\s*u64\s*,\s*aux:\s*u64\s*\)\s*->\s*u64\s*\{)(.*?)(^\s*\})"
    content = re.sub(ref_pattern, r"\1\n" + ref_impl + r"\n\3", content, flags=re.DOTALL | re.MULTILINE)

    with open(path, 'w') as f:
        f.write(content)
    
    print(f"Updated {name}")
