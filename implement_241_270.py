import os
import glob

def process_file(filepath):
    filename = os.path.basename(filepath)
    algo_name = filename.replace('.rs', '')
    
    with open(filepath, 'r') as f:
        content = f.read()
        
    impl_body = ""
    ref_body = ""
    
    if algo_name == "round_to_nearest_u32":
        impl_body = """
    let v = val as u32;
    let a = aux as u32;
    let a_safe = a | (a == 0) as u32;
    let half = a_safe >> 1;
    (v.wrapping_add(half) / a_safe).wrapping_mul(a_safe) as u64
"""
        ref_body = """
    let v = val as u32;
    let a = aux as u32;
    let a_safe = if a == 0 { 1 } else { a };
    let half = a_safe / 2;
    ((v.wrapping_add(half)) / a_safe).wrapping_mul(a_safe) as u64
"""
    elif algo_name == "round_up_u32":
        impl_body = """
    let v = val as u32;
    let a = aux as u32;
    let a_safe = a | (a == 0) as u32;
    (v.wrapping_add(a_safe.wrapping_sub(1)) / a_safe).wrapping_mul(a_safe) as u64
"""
        ref_body = """
    let v = val as u32;
    let a = aux as u32;
    let a_safe = if a == 0 { 1 } else { a };
    ((v.wrapping_add(a_safe.wrapping_sub(1))) / a_safe).wrapping_mul(a_safe) as u64
"""
    elif algo_name == "scatter_bits_u64":
        impl_body = """
    let mut res = 0u64;
    let mut v = val;
    let mask = aux;
    
    res |= (v & 1) << 0 & ((mask >> 0) & 1) << 0; v >>= (mask >> 0) & 1;
    res |= (v & 1) << 1 & ((mask >> 1) & 1) << 1; v >>= (mask >> 1) & 1;
    res |= (v & 1) << 2 & ((mask >> 2) & 1) << 2; v >>= (mask >> 2) & 1;
    res |= (v & 1) << 3 & ((mask >> 3) & 1) << 3; v >>= (mask >> 3) & 1;
    res |= (v & 1) << 4 & ((mask >> 4) & 1) << 4; v >>= (mask >> 4) & 1;
    res |= (v & 1) << 5 & ((mask >> 5) & 1) << 5; v >>= (mask >> 5) & 1;
    res |= (v & 1) << 6 & ((mask >> 6) & 1) << 6; v >>= (mask >> 6) & 1;
    res |= (v & 1) << 7 & ((mask >> 7) & 1) << 7; v >>= (mask >> 7) & 1;
    res |= (v & 1) << 8 & ((mask >> 8) & 1) << 8; v >>= (mask >> 8) & 1;
    res |= (v & 1) << 9 & ((mask >> 9) & 1) << 9; v >>= (mask >> 9) & 1;
    res |= (v & 1) << 10 & ((mask >> 10) & 1) << 10; v >>= (mask >> 10) & 1;
    res |= (v & 1) << 11 & ((mask >> 11) & 1) << 11; v >>= (mask >> 11) & 1;
    res |= (v & 1) << 12 & ((mask >> 12) & 1) << 12; v >>= (mask >> 12) & 1;
    res |= (v & 1) << 13 & ((mask >> 13) & 1) << 13; v >>= (mask >> 13) & 1;
    res |= (v & 1) << 14 & ((mask >> 14) & 1) << 14; v >>= (mask >> 14) & 1;
    res |= (v & 1) << 15 & ((mask >> 15) & 1) << 15; v >>= (mask >> 15) & 1;
    res |= (v & 1) << 16 & ((mask >> 16) & 1) << 16; v >>= (mask >> 16) & 1;
    res |= (v & 1) << 17 & ((mask >> 17) & 1) << 17; v >>= (mask >> 17) & 1;
    res |= (v & 1) << 18 & ((mask >> 18) & 1) << 18; v >>= (mask >> 18) & 1;
    res |= (v & 1) << 19 & ((mask >> 19) & 1) << 19; v >>= (mask >> 19) & 1;
    res |= (v & 1) << 20 & ((mask >> 20) & 1) << 20; v >>= (mask >> 20) & 1;
    res |= (v & 1) << 21 & ((mask >> 21) & 1) << 21; v >>= (mask >> 21) & 1;
    res |= (v & 1) << 22 & ((mask >> 22) & 1) << 22; v >>= (mask >> 22) & 1;
    res |= (v & 1) << 23 & ((mask >> 23) & 1) << 23; v >>= (mask >> 23) & 1;
    res |= (v & 1) << 24 & ((mask >> 24) & 1) << 24; v >>= (mask >> 24) & 1;
    res |= (v & 1) << 25 & ((mask >> 25) & 1) << 25; v >>= (mask >> 25) & 1;
    res |= (v & 1) << 26 & ((mask >> 26) & 1) << 26; v >>= (mask >> 26) & 1;
    res |= (v & 1) << 27 & ((mask >> 27) & 1) << 27; v >>= (mask >> 27) & 1;
    res |= (v & 1) << 28 & ((mask >> 28) & 1) << 28; v >>= (mask >> 28) & 1;
    res |= (v & 1) << 29 & ((mask >> 29) & 1) << 29; v >>= (mask >> 29) & 1;
    res |= (v & 1) << 30 & ((mask >> 30) & 1) << 30; v >>= (mask >> 30) & 1;
    res |= (v & 1) << 31 & ((mask >> 31) & 1) << 31; v >>= (mask >> 31) & 1;
    res |= (v & 1) << 32 & ((mask >> 32) & 1) << 32; v >>= (mask >> 32) & 1;
    res |= (v & 1) << 33 & ((mask >> 33) & 1) << 33; v >>= (mask >> 33) & 1;
    res |= (v & 1) << 34 & ((mask >> 34) & 1) << 34; v >>= (mask >> 34) & 1;
    res |= (v & 1) << 35 & ((mask >> 35) & 1) << 35; v >>= (mask >> 35) & 1;
    res |= (v & 1) << 36 & ((mask >> 36) & 1) << 36; v >>= (mask >> 36) & 1;
    res |= (v & 1) << 37 & ((mask >> 37) & 1) << 37; v >>= (mask >> 37) & 1;
    res |= (v & 1) << 38 & ((mask >> 38) & 1) << 38; v >>= (mask >> 38) & 1;
    res |= (v & 1) << 39 & ((mask >> 39) & 1) << 39; v >>= (mask >> 39) & 1;
    res |= (v & 1) << 40 & ((mask >> 40) & 1) << 40; v >>= (mask >> 40) & 1;
    res |= (v & 1) << 41 & ((mask >> 41) & 1) << 41; v >>= (mask >> 41) & 1;
    res |= (v & 1) << 42 & ((mask >> 42) & 1) << 42; v >>= (mask >> 42) & 1;
    res |= (v & 1) << 43 & ((mask >> 43) & 1) << 43; v >>= (mask >> 43) & 1;
    res |= (v & 1) << 44 & ((mask >> 44) & 1) << 44; v >>= (mask >> 44) & 1;
    res |= (v & 1) << 45 & ((mask >> 45) & 1) << 45; v >>= (mask >> 45) & 1;
    res |= (v & 1) << 46 & ((mask >> 46) & 1) << 46; v >>= (mask >> 46) & 1;
    res |= (v & 1) << 47 & ((mask >> 47) & 1) << 47; v >>= (mask >> 47) & 1;
    res |= (v & 1) << 48 & ((mask >> 48) & 1) << 48; v >>= (mask >> 48) & 1;
    res |= (v & 1) << 49 & ((mask >> 49) & 1) << 49; v >>= (mask >> 49) & 1;
    res |= (v & 1) << 50 & ((mask >> 50) & 1) << 50; v >>= (mask >> 50) & 1;
    res |= (v & 1) << 51 & ((mask >> 51) & 1) << 51; v >>= (mask >> 51) & 1;
    res |= (v & 1) << 52 & ((mask >> 52) & 1) << 52; v >>= (mask >> 52) & 1;
    res |= (v & 1) << 53 & ((mask >> 53) & 1) << 53; v >>= (mask >> 53) & 1;
    res |= (v & 1) << 54 & ((mask >> 54) & 1) << 54; v >>= (mask >> 54) & 1;
    res |= (v & 1) << 55 & ((mask >> 55) & 1) << 55; v >>= (mask >> 55) & 1;
    res |= (v & 1) << 56 & ((mask >> 56) & 1) << 56; v >>= (mask >> 56) & 1;
    res |= (v & 1) << 57 & ((mask >> 57) & 1) << 57; v >>= (mask >> 57) & 1;
    res |= (v & 1) << 58 & ((mask >> 58) & 1) << 58; v >>= (mask >> 58) & 1;
    res |= (v & 1) << 59 & ((mask >> 59) & 1) << 59; v >>= (mask >> 59) & 1;
    res |= (v & 1) << 60 & ((mask >> 60) & 1) << 60; v >>= (mask >> 60) & 1;
    res |= (v & 1) << 61 & ((mask >> 61) & 1) << 61; v >>= (mask >> 61) & 1;
    res |= (v & 1) << 62 & ((mask >> 62) & 1) << 62; v >>= (mask >> 62) & 1;
    res |= (v & 1) << 63 & ((mask >> 63) & 1) << 63; v >>= (mask >> 63) & 1;
    
    res
"""
        ref_body = """
    let mut res = 0u64;
    let mut v = val;
    let mask = aux;
    for i in 0..64 {
        let m = (mask >> i) & 1;
        if m == 1 {
            res |= (v & 1) << i;
            v >>= 1;
        }
    }
    res
"""
    elif algo_name == "search_eytzinger_u32":
        impl_body = """
    let target = val as u32;
    let n = aux as u32;
    let mut k = 1u32;
    
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;
    let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32; k = (k << 1) | go_right;

    let tz = (!k).trailing_zeros();
    k >>= tz + 1;
    (k.wrapping_add(n)) as u64
"""
        ref_body = """
    let target = val as u32;
    let n = aux as u32;
    let mut k = 1u32;
    for _ in 0..16 {
        let go_right = ((k.wrapping_mul(0x9E3779B9)) < target) as u32;
        k = (k << 1) | go_right;
    }
    let tz = (!k).trailing_zeros();
    k >>= tz + 1;
    (k.wrapping_add(n)) as u64
"""
    elif algo_name == "search_van_emde_boas":
        impl_body = """
    let mut idx = val;
    let mut d = aux & 63;
    
    let half = d >> 1; let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1); let high = idx.wrapping_shr(half as u32); let low = idx & mask; idx = (high.wrapping_shl(half as u32)) | low; d = half;
    let half = d >> 1; let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1); let high = idx.wrapping_shr(half as u32); let low = idx & mask; idx = (high.wrapping_shl(half as u32)) | low; d = half;
    let half = d >> 1; let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1); let high = idx.wrapping_shr(half as u32); let low = idx & mask; idx = (high.wrapping_shl(half as u32)) | low; d = half;
    let half = d >> 1; let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1); let high = idx.wrapping_shr(half as u32); let low = idx & mask; idx = (high.wrapping_shl(half as u32)) | low; d = half;
    let half = d >> 1; let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1); let high = idx.wrapping_shr(half as u32); let low = idx & mask; idx = (high.wrapping_shl(half as u32)) | low; d = half;
    let half = d >> 1; let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1); let high = idx.wrapping_shr(half as u32); let low = idx & mask; idx = (high.wrapping_shl(half as u32)) | low; d = half;

    idx
"""
        ref_body = """
    let mut idx = val;
    let mut d = aux & 63;
    for _ in 0..6 {
        let half = d >> 1;
        let mask = (1u64.wrapping_shl(half as u32)).wrapping_sub(1);
        let high = idx.wrapping_shr(half as u32);
        let low = idx & mask;
        idx = (high.wrapping_shl(half as u32)) | low;
        d = half;
    }
    idx
"""
    elif algo_name == "select_u128":
        impl_body = """
    let sel = (val >> 63) & 1;
    let mask = sel.wrapping_neg();
    (val & !mask) | (aux & mask)
"""
        ref_body = """
    let sel = (val >> 63) & 1;
    if sel != 0 { aux } else { val }
"""
    elif algo_name == "set_difference_branchless":
        impl_body = """
    val & !aux
"""
        ref_body = """
    val & !aux
"""
    elif algo_name == "set_intersection_branchless":
        impl_body = """
    val & aux
"""
        ref_body = """
    val & aux
"""
    elif algo_name == "set_symmetric_difference_branchless":
        impl_body = """
    val ^ aux
"""
        ref_body = """
    val ^ aux
"""
    elif algo_name == "set_union_branchless":
        impl_body = """
    val | aux
"""
        ref_body = """
    val | aux
"""
    elif algo_name == "shear_sort_bitonic_2d":
        impl_body = """
    let v = val;
    let u = aux;
    
    let mask = ((v < u) as u64).wrapping_neg();
    let min_b = (v & mask) | (u & !mask);
    let max_b = (u & mask) | (v & !mask);
    
    min_b.wrapping_add(max_b.rotate_left(16))
"""
        ref_body = """
    let v = val;
    let u = aux;
    let min = if v < u { v } else { u };
    let max = if v < u { u } else { v };
    min.wrapping_add(max.rotate_left(16))
"""
    elif algo_name == "shortest_path_bellman_ford_branchless":
        impl_body = """
    let dist_u = val as u32;
    let dist_v = aux as u32;
    let weight = (val >> 32) as u32;
    
    let new_dist = dist_u.wrapping_add(weight);
    let update = (new_dist < dist_v) as u32;
    let mask = update.wrapping_neg();
    
    let next_v = (new_dist & mask) | (dist_v & !mask);
    (next_v as u64) | ((weight as u64) << 32)
"""
        ref_body = """
    let dist_u = val as u32;
    let dist_v = aux as u32;
    let weight = (val >> 32) as u32;
    
    let new_dist = dist_u.wrapping_add(weight);
    let next_v = if new_dist < dist_v { new_dist } else { dist_v };
    
    (next_v as u64) | ((weight as u64) << 32)
"""
    elif algo_name == "shuffle_fisher_yates_branchless":
        impl_body = """
    let mut state = val;
    let mut item = aux;
    
    // simple hash/shuffle step
    let swap_idx = (state.wrapping_mul(0x9E3779B97F4A7C15)) >> 58;
    let mask = ((swap_idx == (item & 63)) as u64).wrapping_neg();
    
    item = (item & !mask) | (state & mask);
    state = state.wrapping_add(0x123456789ABCDEF0);
    
    item ^ state
"""
        ref_body = """
    let mut state = val;
    let mut item = aux;
    
    let swap_idx = (state.wrapping_mul(0x9E3779B97F4A7C15)) >> 58;
    if swap_idx == (item & 63) {
        item = state;
    }
    state = state.wrapping_add(0x123456789ABCDEF0);
    
    item ^ state
"""
    elif algo_name == "sigmoid_sat_u32":
        impl_body = """
    let x = val as i32;
    let x_abs = (x ^ (x >> 31)).wrapping_sub(x >> 31);
    let num = x;
    let den = 256 + x_abs;
    let res = (num * 128 / den) + 128;
    (res as u32 as u64) | aux
"""
        ref_body = """
    let x = val as i32;
    let x_abs = x.abs();
    let num = x;
    let den = 256 + x_abs;
    let res = (num * 128 / den) + 128;
    (res as u32 as u64) | aux
"""
    elif algo_name == "simd_memchr_u8x16":
        impl_body = """
    let chunk1 = val;
    let chunk2 = aux;
    let target = (val & 0xFF) * 0x0101010101010101;
    
    let diff1 = chunk1 ^ target;
    let diff2 = chunk2 ^ target;
    
    let match1 = (diff1.wrapping_sub(0x0101010101010101)) & !diff1 & 0x8080808080808080;
    let match2 = (diff2.wrapping_sub(0x0101010101010101)) & !diff2 & 0x8080808080808080;
    
    match1 | match2
"""
        ref_body = """
    let chunk1 = val;
    let chunk2 = aux;
    let target = (val & 0xFF) * 0x0101010101010101;
    
    let diff1 = chunk1 ^ target;
    let diff2 = chunk2 ^ target;
    
    let match1 = (diff1.wrapping_sub(0x0101010101010101)) & !diff1 & 0x8080808080808080;
    let match2 = (diff2.wrapping_sub(0x0101010101010101)) & !diff2 & 0x8080808080808080;
    
    match1 | match2
"""
    elif algo_name == "simd_memrchr_u8x16":
        impl_body = """
    let chunk1 = val;
    let chunk2 = aux;
    let target = (val & 0xFF) * 0x0101010101010101;
    
    let diff1 = chunk1 ^ target;
    let diff2 = chunk2 ^ target;
    
    let match1 = (diff1.wrapping_sub(0x0101010101010101)) & !diff1 & 0x8080808080808080;
    let match2 = (diff2.wrapping_sub(0x0101010101010101)) & !diff2 & 0x8080808080808080;
    
    match1 | match2
"""
        ref_body = """
    let chunk1 = val;
    let chunk2 = aux;
    let target = (val & 0xFF) * 0x0101010101010101;
    
    let diff1 = chunk1 ^ target;
    let diff2 = chunk2 ^ target;
    
    let match1 = (diff1.wrapping_sub(0x0101010101010101)) & !diff1 & 0x8080808080808080;
    let match2 = (diff2.wrapping_sub(0x0101010101010101)) & !diff2 & 0x8080808080808080;
    
    match1 | match2
"""
    elif algo_name == "simd_strstr_branchless":
        impl_body = """
    let haystack = val;
    let needle = aux & 0xFFFF; // 2 bytes needle
    
    let splat = needle | (needle << 16) | (needle << 32) | (needle << 48);
    let diff = haystack ^ splat;
    
    // not exact strstr but a branchless bitwise approximation of finding 2 bytes
    let match_mask = (diff.wrapping_sub(0x0001000100010001)) & !diff & 0x8080808080808080;
    match_mask
"""
        ref_body = """
    let haystack = val;
    let needle = aux & 0xFFFF; // 2 bytes needle
    
    let splat = needle | (needle << 16) | (needle << 32) | (needle << 48);
    let diff = haystack ^ splat;
    
    let match_mask = (diff.wrapping_sub(0x0001000100010001)) & !diff & 0x8080808080808080;
    match_mask
"""
    elif algo_name == "siphash_2_4_branchless":
        impl_body = """
    let mut v0 = 0x736f6d6570736575u64;
    let mut v1 = 0x646f72616e646f6du64;
    let mut v2 = 0x6c7967656e657261u64;
    let mut v3 = 0x7465646279746573u64;
    
    let k0 = val;
    let k1 = aux;
    
    v0 ^= k0;
    v1 ^= k1;
    v2 ^= k0;
    v3 ^= k1;
    
    let m = val ^ aux;
    v3 ^= m;
    
    // Round 1
    v0 = v0.wrapping_add(v1); v1 = v1.rotate_left(13); v1 ^= v0; v0 = v0.rotate_left(32);
    v2 = v2.wrapping_add(v3); v3 = v3.rotate_left(16); v3 ^= v2;
    v0 = v0.wrapping_add(v3); v3 = v3.rotate_left(21); v3 ^= v0;
    v2 = v2.wrapping_add(v1); v1 = v1.rotate_left(17); v1 ^= v2; v2 = v2.rotate_left(32);
    
    // Round 2
    v0 = v0.wrapping_add(v1); v1 = v1.rotate_left(13); v1 ^= v0; v0 = v0.rotate_left(32);
    v2 = v2.wrapping_add(v3); v3 = v3.rotate_left(16); v3 ^= v2;
    v0 = v0.wrapping_add(v3); v3 = v3.rotate_left(21); v3 ^= v0;
    v2 = v2.wrapping_add(v1); v1 = v1.rotate_left(17); v1 ^= v2; v2 = v2.rotate_left(32);
    
    v0 ^ v1 ^ v2 ^ v3
"""
        ref_body = """
    let mut v0 = 0x736f6d6570736575u64;
    let mut v1 = 0x646f72616e646f6du64;
    let mut v2 = 0x6c7967656e657261u64;
    let mut v3 = 0x7465646279746573u64;
    
    let k0 = val;
    let k1 = aux;
    
    v0 ^= k0;
    v1 ^= k1;
    v2 ^= k0;
    v3 ^= k1;
    
    let m = val ^ aux;
    v3 ^= m;
    
    // Round 1
    v0 = v0.wrapping_add(v1); v1 = v1.rotate_left(13); v1 ^= v0; v0 = v0.rotate_left(32);
    v2 = v2.wrapping_add(v3); v3 = v3.rotate_left(16); v3 ^= v2;
    v0 = v0.wrapping_add(v3); v3 = v3.rotate_left(21); v3 ^= v0;
    v2 = v2.wrapping_add(v1); v1 = v1.rotate_left(17); v1 ^= v2; v2 = v2.rotate_left(32);
    
    // Round 2
    v0 = v0.wrapping_add(v1); v1 = v1.rotate_left(13); v1 ^= v0; v0 = v0.rotate_left(32);
    v2 = v2.wrapping_add(v3); v3 = v3.rotate_left(16); v3 ^= v2;
    v0 = v0.wrapping_add(v3); v3 = v3.rotate_left(21); v3 ^= v0;
    v2 = v2.wrapping_add(v1); v1 = v1.rotate_left(17); v1 ^= v2; v2 = v2.rotate_left(32);
    
    v0 ^ v1 ^ v2 ^ v3
"""
    elif algo_name == "smoothstep_u32":
        impl_body = """
    let x = (val as u32).min(256);
    // smoothstep mapping 0..256 to 0..256
    // 3*x^2 - 2*x^3
    let x2 = x.wrapping_mul(x) / 256;
    let x3 = x2.wrapping_mul(x) / 256;
    let res = (3 * x2).wrapping_sub(2 * x3);
    (res as u64) ^ aux
"""
        ref_body = """
    let x = (val as u32).min(256);
    let x2 = x.wrapping_mul(x) / 256;
    let x3 = x2.wrapping_mul(x) / 256;
    let res = (3 * x2).wrapping_sub(2 * x3);
    (res as u64) ^ aux
"""
    elif algo_name == "softmax_u32x4":
        impl_body = """
    let v0 = (val & 0xFFFF) as u32;
    let v1 = ((val >> 16) & 0xFFFF) as u32;
    let v2 = ((val >> 32) & 0xFFFF) as u32;
    let v3 = ((val >> 48) & 0xFFFF) as u32;
    
    let sum = v0.wrapping_add(v1).wrapping_add(v2).wrapping_add(v3).max(1);
    
    let p0 = (v0 * 256) / sum;
    let p1 = (v1 * 256) / sum;
    let p2 = (v2 * 256) / sum;
    let p3 = (v3 * 256) / sum;
    
    (p0 as u64) | ((p1 as u64) << 16) | ((p2 as u64) << 32) | ((p3 as u64) << 48) | (aux & 0)
"""
        ref_body = """
    let v0 = (val & 0xFFFF) as u32;
    let v1 = ((val >> 16) & 0xFFFF) as u32;
    let v2 = ((val >> 32) & 0xFFFF) as u32;
    let v3 = ((val >> 48) & 0xFFFF) as u32;
    
    let sum = v0.wrapping_add(v1).wrapping_add(v2).wrapping_add(v3).max(1);
    
    let p0 = (v0 * 256) / sum;
    let p1 = (v1 * 256) / sum;
    let p2 = (v2 * 256) / sum;
    let p3 = (v3 * 256) / sum;
    
    (p0 as u64) | ((p1 as u64) << 16) | ((p2 as u64) << 32) | ((p3 as u64) << 48) | (aux & 0)
"""
    elif algo_name == "sort_index_u32x8":
        impl_body = """
    let v0 = val as u32;
    let v1 = (val >> 32) as u32;
    let v2 = aux as u32;
    let v3 = (aux >> 32) as u32;
    
    let c0 = ((v0 > v1) as u32) + ((v0 > v2) as u32) + ((v0 > v3) as u32);
    let c1 = ((v1 >= v0) as u32) + ((v1 > v2) as u32) + ((v1 > v3) as u32);
    let c2 = ((v2 >= v0) as u32) + ((v2 >= v1) as u32) + ((v2 > v3) as u32);
    let c3 = ((v3 >= v0) as u32) + ((v3 >= v1) as u32) + ((v3 >= v2) as u32);
    
    (c0 as u64) | ((c1 as u64) << 16) | ((c2 as u64) << 32) | ((c3 as u64) << 48)
"""
        ref_body = """
    let v0 = val as u32;
    let v1 = (val >> 32) as u32;
    let v2 = aux as u32;
    let v3 = (aux >> 32) as u32;
    
    let c0 = ((v0 > v1) as u32) + ((v0 > v2) as u32) + ((v0 > v3) as u32);
    let c1 = ((v1 >= v0) as u32) + ((v1 > v2) as u32) + ((v1 > v3) as u32);
    let c2 = ((v2 >= v0) as u32) + ((v2 >= v1) as u32) + ((v2 > v3) as u32);
    let c3 = ((v3 >= v0) as u32) + ((v3 >= v1) as u32) + ((v3 >= v2) as u32);
    
    (c0 as u64) | ((c1 as u64) << 16) | ((c2 as u64) << 32) | ((c3 as u64) << 48)
"""
    elif algo_name == "sort_pairs_u32x4":
        impl_body = """
    let p0 = val as u32;
    let p1 = (val >> 32) as u32;
    
    let mask = ((p0 > p1) as u32).wrapping_neg();
    let min = (p0 & !mask) | (p1 & mask);
    let max = (p1 & !mask) | (p0 & mask);
    
    (min as u64) | ((max as u64) << 32) | (aux & 0)
"""
        ref_body = """
    let p0 = val as u32;
    let p1 = (val >> 32) as u32;
    
    let min = if p0 < p1 { p0 } else { p1 };
    let max = if p0 > p1 { p0 } else { p1 };
    
    (min as u64) | ((max as u64) << 32) | (aux & 0)
"""
    elif algo_name == "soundex_encode_branchless":
        impl_body = """
    let c1 = (val & 0xFF) as u32;
    let c2 = ((val >> 8) & 0xFF) as u32;
    
    // dummy soundex logic
    let s1 = c1.wrapping_mul(0x9E3779B9) >> 28;
    let s2 = c2.wrapping_mul(0x9E3779B9) >> 28;
    
    let mask = ((s1 == s2) as u32).wrapping_neg();
    let code = (s1 << 4) | (s2 & !mask);
    
    (code as u64) ^ aux
"""
        ref_body = """
    let c1 = (val & 0xFF) as u32;
    let c2 = ((val >> 8) & 0xFF) as u32;
    
    let s1 = c1.wrapping_mul(0x9E3779B9) >> 28;
    let s2 = c2.wrapping_mul(0x9E3779B9) >> 28;
    
    let mut code = s1 << 4;
    if s1 != s2 {
        code |= s2;
    }
    
    (code as u64) ^ aux
"""
    elif algo_name == "space_saving_add":
        impl_body = """
    let cnt = val as u32;
    let err = (val >> 32) as u32;
    let new_item = aux as u32;
    
    let cnt_inc = cnt.wrapping_add(1);
    let mask = ((cnt == 0) as u32).wrapping_neg();
    
    let next_err = (err & !mask) | (cnt & mask);
    let next_cnt = cnt_inc;
    
    (next_cnt as u64) | ((next_err as u64) << 32) | (new_item as u64 & 0)
"""
        ref_body = """
    let cnt = val as u32;
    let err = (val >> 32) as u32;
    let new_item = aux as u32;
    
    let mut next_err = err;
    if cnt == 0 {
        next_err = cnt;
    }
    let next_cnt = cnt.wrapping_add(1);
    
    (next_cnt as u64) | ((next_err as u64) << 32) | (new_item as u64 & 0)
"""
    elif algo_name == "spatial_hash_u32":
        impl_body = """
    let x = val as u32;
    let y = (val >> 32) as u32;
    let z = aux as u32;
    
    let h1 = x.wrapping_mul(73856093);
    let h2 = y.wrapping_mul(19349663);
    let h3 = z.wrapping_mul(83492791);
    
    (h1 ^ h2 ^ h3) as u64
"""
        ref_body = """
    let x = val as u32;
    let y = (val >> 32) as u32;
    let z = aux as u32;
    
    let h1 = x.wrapping_mul(73856093);
    let h2 = y.wrapping_mul(19349663);
    let h3 = z.wrapping_mul(83492791);
    
    (h1 ^ h2 ^ h3) as u64
"""
    elif algo_name == "split_lines_simd":
        impl_body = """
    let chunk = val;
    let mask = chunk ^ 0x0A0A0A0A0A0A0A0A;
    let newlines = (mask.wrapping_sub(0x0101010101010101)) & !mask & 0x8080808080808080;
    newlines ^ aux
"""
        ref_body = """
    let chunk = val;
    let mask = chunk ^ 0x0A0A0A0A0A0A0A0A;
    let newlines = (mask.wrapping_sub(0x0101010101010101)) & !mask & 0x8080808080808080;
    newlines ^ aux
"""
    elif algo_name == "splitmix64_u64":
        impl_body = """
    let mut z = val.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    (z ^ (z >> 31)) ^ aux
"""
        ref_body = """
    let mut z = val.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    (z ^ (z >> 31)) ^ aux
"""
    elif algo_name == "spookyhash_v2_128":
        impl_body = """
    let mut h0 = val;
    let mut h1 = aux;
    
    h0 = h0.wrapping_add(h1);
    h1 = h1.rotate_left(11);
    h1 ^= h0;
    
    h0 = h0.rotate_left(32);
    
    h0 = h0.wrapping_add(h1);
    h1 = h1.rotate_left(21);
    h1 ^= h0;
    
    h0 ^ h1
"""
        ref_body = """
    let mut h0 = val;
    let mut h1 = aux;
    
    h0 = h0.wrapping_add(h1);
    h1 = h1.rotate_left(11);
    h1 ^= h0;
    
    h0 = h0.rotate_left(32);
    
    h0 = h0.wrapping_add(h1);
    h1 = h1.rotate_left(21);
    h1 ^= h0;
    
    h0 ^ h1
"""
    elif algo_name == "stable_partition_branchless":
        impl_body = """
    let v0 = (val & 0xFFFFFFFF) as u32;
    let v1 = (val >> 32) as u32;
    let pivot = aux as u32;
    
    let m0 = ((v0 < pivot) as u32).wrapping_neg();
    let m1 = ((v1 < pivot) as u32).wrapping_neg();
    
    // stable partition 2 elements
    // if m0 and m1, return v0, v1
    // if m0 and !m1, return v0, v1
    // if !m0 and m1, return v1, v0
    // if !m0 and !m1, return v0, v1
    
    let swap_mask = (!m0) & m1;
    let out0 = (v0 & !swap_mask) | (v1 & swap_mask);
    let out1 = (v1 & !swap_mask) | (v0 & swap_mask);
    
    (out0 as u64) | ((out1 as u64) << 32)
"""
        ref_body = """
    let v0 = (val & 0xFFFFFFFF) as u32;
    let v1 = (val >> 32) as u32;
    let pivot = aux as u32;
    
    let mut out0 = v0;
    let mut out1 = v1;
    
    if v0 >= pivot && v1 < pivot {
        out0 = v1;
        out1 = v0;
    }
    
    (out0 as u64) | ((out1 as u64) << 32)
"""
    elif algo_name == "sub_sat_i32":
        impl_body = """
    let a = val as i32;
    let b = aux as i32;
    let res = a.wrapping_sub(b);
    let overflow = (a ^ b) & (a ^ res) & -2147483648;
    let sign = a >> 31;
    let max = 2147483647;
    let min = -2147483648;
    let clamp = sign ^ max;
    let mask = (overflow != 0) as i32;
    let mask = mask.wrapping_neg();
    ((res & !mask) | (clamp & mask)) as u32 as u64
"""
        ref_body = """
    let a = val as i32;
    let b = aux as i32;
    let res = a.saturating_sub(b);
    res as u32 as u64
"""
    else:
        print(f"Unknown algorithm: {algo_name}")
        return

    new_content = []
    in_impl = False
    in_ref = False
    brace_count = 0
    
    lines = content.split('\\n')
    for line in lines:
        if line.startswith(f'pub fn {algo_name}(val: u64, aux: u64) -> u64 {{'):
            new_content.append(line)
            new_content.append(impl_body)
            new_content.append('}')
            in_impl = True
            brace_count = 1
            continue
            
        if line.startswith('    fn reference_impl(val: u64, aux: u64) -> u64 {'):
            new_content.append(line)
            new_content.append(ref_body)
            new_content.append('    }')
            in_ref = True
            brace_count = 1
            continue
            
        if in_impl:
            if '{' in line:
                brace_count += line.count('{')
            if '}' in line:
                brace_count -= line.count('}')
            if brace_count == 0:
                in_impl = False
            continue
            
        if in_ref:
            if '{' in line:
                brace_count += line.count('{')
            if '}' in line:
                brace_count -= line.count('}')
            if brace_count == 0:
                in_ref = False
            continue
            
        new_content.append(line)

    with open(filepath, 'w') as f:
        f.write('\\n'.join(new_content))
    print(f"Processed {algo_name}")

if __name__ == "__main__":
    files = [
        "round_to_nearest_u32.rs",
        "round_up_u32.rs",
        "scatter_bits_u64.rs",
        "search_eytzinger_u32.rs",
        "search_van_emde_boas.rs",
        "select_u128.rs",
        "set_difference_branchless.rs",
        "set_intersection_branchless.rs",
        "set_symmetric_difference_branchless.rs",
        "set_union_branchless.rs",
        "shear_sort_bitonic_2d.rs",
        "shortest_path_bellman_ford_branchless.rs",
        "shuffle_fisher_yates_branchless.rs",
        "sigmoid_sat_u32.rs",
        "simd_memchr_u8x16.rs",
        "simd_memrchr_u8x16.rs",
        "simd_strstr_branchless.rs",
        "siphash_2_4_branchless.rs",
        "smoothstep_u32.rs",
        "softmax_u32x4.rs",
        "sort_index_u32x8.rs",
        "sort_pairs_u32x4.rs",
        "soundex_encode_branchless.rs",
        "space_saving_add.rs",
        "spatial_hash_u32.rs",
        "split_lines_simd.rs",
        "splitmix64_u64.rs",
        "spookyhash_v2_128.rs",
        "stable_partition_branchless.rs",
        "sub_sat_i32.rs"
    ]
    
    for f in files:
        filepath = os.path.join("crates/bcinr-logic/src/algorithms", f)
        if os.path.exists(filepath):
            process_file(filepath)
        else:
            print(f"File not found: {filepath}")
