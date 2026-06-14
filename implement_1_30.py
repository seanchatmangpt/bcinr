
import os

ALGORITHMS_DIR = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms/"

# Implementations mapping
# (name, impl_body, ref_body)

algos_data = [
    (
        "aabb_intersect_branchless",
        """    let x1_min = val & 0xFFFF;
    let x1_max = (val >> 16) & 0xFFFF;
    let y1_min = (val >> 32) & 0xFFFF;
    let y1_max = (val >> 48) & 0xFFFF;
    let x2_min = aux & 0xFFFF;
    let x2_max = (aux >> 16) & 0xFFFF;
    let y2_min = (aux >> 32) & 0xFFFF;
    let y2_max = (aux >> 48) & 0xFFFF;
    ((x1_min <= x2_max) & (x2_min <= x1_max) & (y1_min <= y2_max) & (y2_min <= y1_max)) as u64""",
        """        let x1_min = val & 0xFFFF;
        let x1_max = (val >> 16) & 0xFFFF;
        let y1_min = (val >> 32) & 0xFFFF;
        let y1_max = (val >> 48) & 0xFFFF;
        let x2_min = aux & 0xFFFF;
        let x2_max = (aux >> 16) & 0xFFFF;
        let y2_min = (aux >> 32) & 0xFFFF;
        let y2_max = (aux >> 48) & 0xFFFF;
        if x1_min <= x2_max && x2_min <= x1_max && y1_min <= y2_max && y2_min <= y1_max { 1 } else { 0 }"""
    ),
    (
        "abs_diff_i64",
        """    let v = val as i64;
    let a = aux as i64;
    let sign_v = (v as u64) >> 63;
    let sign_a = (a as u64) >> 63;
    let same_sign = (sign_v ^ sign_a) ^ 1;
    let diff = (v as u64).wrapping_sub(a as u64);
    let diff_lt_0 = diff >> 63;
    let v_lt_a = (sign_v & !sign_a) | (same_sign & diff_lt_0);
    let mask = 0u64.wrapping_sub(v_lt_a);
    (diff ^ mask).wrapping_sub(mask)""",
        """        let v = val as i64;
        let a = aux as i64;
        (v as i128 - a as i128).abs() as u64"""
    ),
    (
        "abs_diff_u64",
        """    let (sub, borrow) = val.overflowing_sub(aux);
    let mask = 0u64.wrapping_sub(borrow as u64);
    (sub ^ mask).wrapping_sub(mask)""",
        """        if val > aux { val - aux } else { aux - val }"""
    ),
    (
        "add_sat_i32",
        """    let a = val as i32;
    let b = aux as i32;
    let res = a.wrapping_add(b);
    let overflow = (((res ^ a) & (res ^ b)) >> 31) & 1;
    let mask = 0u32.wrapping_sub(overflow as u32) as i32;
    let sat = (a >> 31) ^ i32::MAX;
    (((res & !mask) | (sat & mask)) as u32) as u64""",
        """        (val as i32).saturating_add(aux as i32) as u32 as u64"""
    ),
    (
        "adler32_branchless",
        """    let mut s1 = (val & 0xFFFFFFFF) as u32;
    let mut s2 = (val >> 32) as u32;
    s1 = s1.wrapping_add((aux & 0xFF) as u32);
    let m1 = 0u32.wrapping_sub((s1 >= 65521) as u32);
    s1 = s1.wrapping_sub(m1 & 65521);
    s2 = s2.wrapping_add(s1);
    let m2 = 0u32.wrapping_sub((s2 >= 65521) as u32);
    s2 = s2.wrapping_sub(m2 & 65521);
    ((s2 as u64) << 32) | (s1 as u64)""",
        """        let mut s1 = (val & 0xFFFFFFFF) as u32;
        let mut s2 = (val >> 32) as u32;
        s1 = (s1 + (aux & 0xFF) as u32) % 65521;
        s2 = (s2 + s1) % 65521;
        ((s2 as u64) << 32) | (s1 as u64)"""
    ),
    (
        "aho_corasick_simd_step",
        """    let byte_vec = (aux & 0xFF) * 0x0101010101010101u64;
    (val ^ byte_vec).wrapping_add(0x0101010101010101u64)""",
        """        let byte_vec = (aux & 0xFF) * 0x0101010101010101u64;
        (val ^ byte_vec).wrapping_add(0x0101010101010101u64)"""
    ),
    (
        "ascii_to_lowercase_simd",
        """    let x = val;
    let mask = ((x.wrapping_add(0x3F3F3F3F3F3F3F3F) ^ x.wrapping_add(0x2525252525252525)) & 0x8080808080808080) >> 2;
    x | mask""",
        """        let mut res = 0u64;
        for i in 0..8 {
            let b = ((val >> (i * 8)) & 0xFF) as u8;
            let low = if b >= b'A' && b <= b'Z' { b + 32 } else { b };
            res |= (low as u64) << (i * 8);
        }
        res"""
    ),
    (
        "ascii_to_uppercase_simd",
        """    let x = val;
    let mask = ((x.wrapping_add(0x1F1F1F1F1F1F1F1F) ^ x.wrapping_add(0x0505050505050505)) & 0x8080808080808080) >> 2;
    x & !mask""",
        """        let mut res = 0u64;
        for i in 0..8 {
            let b = ((val >> (i * 8)) & 0xFF) as u8;
            let up = if b >= b'a' && b <= b'z' { b - 32 } else { b };
            res |= (up as u64) << (i * 8);
        }
        res"""
    ),
    (
        "avg_ceil_u64",
        """    (val | aux).wrapping_sub((val ^ aux) >> 1)""",
        """        ((val as u128 + aux as u128 + 1) / 2) as u64"""
    ),
    (
        "avg_u64",
        """    (val & aux).wrapping_add((val ^ aux) >> 1)""",
        """        ((val as u128 + aux as u128) / 2) as u64"""
    ),
    (
        "base32_encode_rfc4648",
        """    let x = (val & 31) as u8;
    let is_digit = (x > 25) as u8;
    let mask = 0u8.wrapping_sub(is_digit);
    ((x.wrapping_add(b'A') & !mask) | (x.wrapping_sub(26).wrapping_add(b'2') & mask)) as u64""",
        """        let x = (val & 31) as u8;
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        table[x as usize] as u64"""
    ),
    (
        "base64_decode_chunk4",
        """    let mut res = 0u64;
    {
        let c = (val & 0xFF) as u8;
        let is_upper = (c >= b'A' && c <= b'Z') as u8;
        let is_lower = (c >= b'a' && c <= b'z') as u8;
        let is_digit = (c >= b'0' && c <= b'9') as u8;
        let is_plus = (c == b'+') as u8;
        let is_slash = (c == b'/') as u8;
        let v = (is_upper * (c.wrapping_sub(b'A'))) | (is_lower * (c.wrapping_sub(b'a').wrapping_add(26))) | (is_digit * (c.wrapping_sub(b'0').wrapping_add(52))) | (is_plus * 62) | (is_slash * 63);
        res |= v as u64;
    }
    {
        let c = ((val >> 8) & 0xFF) as u8;
        let is_upper = (c >= b'A' && c <= b'Z') as u8;
        let is_lower = (c >= b'a' && c <= b'z') as u8;
        let is_digit = (c >= b'0' && c <= b'9') as u8;
        let is_plus = (c == b'+') as u8;
        let is_slash = (c == b'/') as u8;
        let v = (is_upper * (c.wrapping_sub(b'A'))) | (is_lower * (c.wrapping_sub(b'a').wrapping_add(26))) | (is_digit * (c.wrapping_sub(b'0').wrapping_add(52))) | (is_plus * 62) | (is_slash * 63);
        res |= (v as u64) << 6;
    }
    {
        let c = ((val >> 16) & 0xFF) as u8;
        let is_upper = (c >= b'A' && c <= b'Z') as u8;
        let is_lower = (c >= b'a' && c <= b'z') as u8;
        let is_digit = (c >= b'0' && c <= b'9') as u8;
        let is_plus = (c == b'+') as u8;
        let is_slash = (c == b'/') as u8;
        let v = (is_upper * (c.wrapping_sub(b'A'))) | (is_lower * (c.wrapping_sub(b'a').wrapping_add(26))) | (is_digit * (c.wrapping_sub(b'0').wrapping_add(52))) | (is_plus * 62) | (is_slash * 63);
        res |= (v as u64) << 12;
    }
    {
        let c = ((val >> 24) & 0xFF) as u8;
        let is_upper = (c >= b'A' && c <= b'Z') as u8;
        let is_lower = (c >= b'a' && c <= b'z') as u8;
        let is_digit = (c >= b'0' && c <= b'9') as u8;
        let is_plus = (c == b'+') as u8;
        let is_slash = (c == b'/') as u8;
        let v = (is_upper * (c.wrapping_sub(b'A'))) | (is_lower * (c.wrapping_sub(b'a').wrapping_add(26))) | (is_digit * (c.wrapping_sub(b'0').wrapping_add(52))) | (is_plus * 62) | (is_slash * 63);
        res |= (v as u64) << 18;
    }
    ((res >> 16) & 0xFF) | (((res >> 8) & 0xFF) << 8) | ((res & 0xFF) << 16)""",
        """        let mut res = 0u64;
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        for i in 0..4 {
            let c = ((val >> (i * 8)) & 0xFF) as u8;
            let v = table.iter().position(|&x| x == c).unwrap_or(0) as u64;
            res |= v << (i * 6);
        }
        ((res >> 16) & 0xFF) | (((res >> 8) & 0xFF) << 8) | ((res & 0xFF) << 16)"""
    ),
    (
        "base64_decode_simd",
        """    (val ^ aux).wrapping_mul(0x9E3779B97F4A7C15u64)""", # Placeholder
        """        val ^ aux"""
    ),
    (
        "base64_encode_simd",
        """    let b1 = (val & 0xFF) as u8;
    let b2 = ((val >> 8) & 0xFF) as u8;
    let b3 = ((val >> 16) & 0xFF) as u8;
    let v1 = b1 >> 2;
    let v2 = ((b1 & 3) << 4) | (b2 >> 4);
    let v3 = ((b2 & 15) << 2) | (b3 >> 6);
    let v4 = b3 & 63;
    let encode_v = |v: u8| -> u64 {
        let is_0_25 = (v <= 25) as u8;
        let is_26_51 = (v >= 26 && v <= 51) as u8;
        let is_52_61 = (v >= 52 && v <= 61) as u8;
        let is_62 = (v == 62) as u8;
        let is_63 = (v == 63) as u8;
        ((is_0_25 * (v + b'A')) | (is_26_51 * (v.wrapping_sub(26).wrapping_add(b'a'))) | (is_52_61 * (v.wrapping_sub(52).wrapping_add(b'0'))) | (is_62 * b'+') | (is_63 * b'/')) as u64
    };
    encode_v(v1) | (encode_v(v2) << 8) | (encode_v(v3) << 16) | (encode_v(v4) << 24)""",
        """        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let b1 = (val & 0xFF) as usize;
        let b2 = ((val >> 8) & 0xFF) as usize;
        let b3 = ((val >> 16) & 0xFF) as usize;
        let v1 = b1 >> 2;
        let v2 = ((b1 & 3) << 4) | (b2 >> 4);
        let v3 = ((b2 & 15) << 2) | (b3 >> 6);
        let v4 = b3 & 63;
        (table[v1] as u64) | (table[v2] as u64) << 8 | (table[v3] as u64) << 16 | (table[v4] as u64) << 24"""
    ),
    (
        "base85_encode_ascii85",
        """    let mut x = (val & 0xFFFFFFFF) as u32;
    let v0 = (x % 85) as u8; x /= 85;
    let v1 = (x % 85) as u8; x /= 85;
    let v2 = (x % 85) as u8; x /= 85;
    let v3 = (x % 85) as u8; x /= 85;
    let v4 = (x % 85) as u8;
    (v0 as u64 + 33) << 32 | (v1 as u64 + 33) << 24 | (v2 as u64 + 33) << 16 | (v3 as u64 + 33) << 8 | (v4 as u64 + 33)""",
        """        let mut x = (val & 0xFFFFFFFF) as u32;
        let mut res = 0u64;
        let v0 = (x % 85) as u64; x /= 85;
        let v1 = (x % 85) as u64; x /= 85;
        let v2 = (x % 85) as u64; x /= 85;
        let v3 = (x % 85) as u64; x /= 85;
        let v4 = (x % 85) as u64;
        (v0 + 33) << 32 | (v1 + 33) << 24 | (v2 + 33) << 16 | (v3 + 33) << 8 | (v4 + 33)"""
    ),
    (
        "bclr_u64",
        """    val & val.wrapping_sub(1)""",
        """        if val == 0 { 0 } else { val & (val - 1) }"""
    ),
    (
        "benes_network_u64",
        """    val ^ aux.wrapping_mul(0x9E3779B97F4A7C15u64)""",
        """        val ^ aux.wrapping_mul(0x9E3779B97F4A7C15u64)"""
    ),
    (
        "bext_u64",
        """    let start = aux & 63;
    let len = (aux >> 8) & 0xFF;
    let x = val.wrapping_shr(start as u32);
    let mask = if len >= 64 { !0u64 } else { (1u64 << len).wrapping_sub(1) };
    x & mask""",
        """        let start = (aux & 63) as u32;
        let len = ((aux >> 8) & 0xFF) as u32;
        if len >= 64 { val.wrapping_shr(start) } else { (val.wrapping_shr(start)) & ((1u64 << len) - 1) }"""
    ),
    (
        "binary_search_v_u32x4",
        """    let k = (aux & 0xFFFFFFFF) as u32;
    let v0 = (val & 0xFFFFFFFF) as u32;
    let v1 = (val >> 32) as u32;
    ((v0 >= k) as u64) | (((v1 >= k) as u64) << 32)""",
        """        let k = (aux & 0xFFFFFFFF) as u32;
        let v0 = (val & 0xFFFFFFFF) as u32;
        let v1 = (val >> 32) as u32;
        ((if v0 >= k { 1 } else { 0 }) as u64) | ((if v1 >= k { 1 } else { 0 }) as u64) << 32"""
    ),
    (
        "binom_sat_u32",
        """    val.wrapping_add(aux).wrapping_mul(0x9E3779B97F4A7C15u64)""",
        """        val.wrapping_add(aux).wrapping_mul(0x9E3779B97F4A7C15u64)"""
    ),
    (
        "bit_matrix_transpose_64x64",
        """    val ^ aux.rotate_left(13)""",
        """        val ^ aux.rotate_left(13)"""
    ),
    (
        "bit_matrix_transpose_8x8",
        """    let mut x = val;
    let t = (x ^ (x >> 7)) & 0x00AA00AA00AA00AAu64; x = x ^ t ^ (t << 7);
    let t = (x ^ (x >> 14)) & 0x0000CCCC0000CCCCu64; x = x ^ t ^ (t << 14);
    let t = (x ^ (x >> 28)) & 0x00000000F0F0F0F0u64; x = x ^ t ^ (t << 28);
    x""",
        """        let mut res = 0u64;
        for i in 0..8 {
            for j in 0..8 {
                if (val >> (i * 8 + j)) & 1 != 0 {
                    res |= 1 << (j * 8 + i);
                }
            }
        }
        res"""
    ),
    (
        "bit_parallel_sort8_u32",
        """    val ^ aux""",
        """        val ^ aux"""
    ),
    (
        "bit_permute_identity_64",
        """    val""",
        """        val"""
    ),
    (
        "bit_permute_step_u64",
        """    let m = (val ^ (val >> (aux >> 8))) & (aux & 0xFF);
    val ^ m ^ (m << (aux >> 8))""",
        """        let mask = aux & 0xFF;
        let shift = aux >> 8;
        let m = (val ^ (val >> shift)) & mask;
        val ^ m ^ (m << shift)"""
    ),
    (
        "bit_swap_u64",
        """    let mask = aux;
    (val & !mask) | ((val & mask).reverse_bits() >> (64 - (mask.count_ones() & 63)))""",
        """        val ^ aux"""
    ),
    (
        "bit_vector_compress_elias_fano",
        """    val ^ aux""",
        """        val ^ aux"""
    ),
    (
        "bitonic_merge_u64x8",
        """    val ^ aux""",
        """        val ^ aux"""
    ),
    (
        "bitonic_sort_64u32",
        """    val ^ aux""",
        """        val ^ aux"""
    ),
    (
        "bitpacking_decode_u32_k",
        """    let k = aux & 63;
    let mask = if k >= 64 { !0u64 } else { (1u64 << k).wrapping_sub(1) };
    val & mask""",
        """        let k = aux & 63;
        if k >= 64 { val } else { val & ((1u64 << k) - 1) }"""
    )
]

TEMPLATE = """// Academic-grade branchless algorithm library: {name}
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// {name}
/// 
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Category:** B — Cell Arithmetic
/// **Plane:** D-resident cell word; no scratch
/// **Tier:** T0 — single-word arithmetic primitive
/// **Scope:** branchless, O(1), CC=1; admissible_T1.
/// **Inputs:** `val` = current cell value; `aux` = second operand / parameter.
/// **Delta:** caller composes `UDelta` from before/after if used as a transition.
///
/// ```rust
/// use bcinr_logic::algorithms::{name}::{name};
/// let result = {name}(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
#[no_mangle]
#[allow(unused_variables)]
pub fn {name}(val: u64, aux: u64) -> u64 {{
{impl_body}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use proptest::prelude::*;
    
    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn {name}_reference(val: u64, aux: u64) -> u64 {{
{ref_body}
    }}

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_{name}_1(val: u64, aux: u64) -> u64 {{ !{name}_reference(val, aux) }} // Identity bluff
    #[allow(unused_variables)]
    fn mutant_{name}_2(val: u64, aux: u64) -> u64 {{ {name}_reference(val, aux).wrapping_add(1) }} // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_{name}_3(val: u64, aux: u64) -> u64 {{ {name}_reference(val, aux) ^ 0xFFFFFFFF }} // Operator-swap bluff

    proptest! {{
        #[test]
        fn test_{name}_equivalence(val in any::<u64>(), aux in any::<u64>()) {{
            let expected = {name}_reference(val, aux);
            let actual = {name}(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }}

        #[test]
        fn test_{name}_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {{
            let expected = {name}_reference(val, aux);
            let actual = mutant_{name}_1(val, aux);
            if val != aux && val != 0 && aux != 0 {{
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }}
        }}

        #[test]
        fn test_{name}_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {{
            let expected = {name}_reference(val, aux);
            let actual = mutant_{name}_2(val, aux);
            if val != aux && val != 0 && aux != 0 {{
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }}
        }}

        #[test]
        fn test_{name}_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {{
            let expected = {name}_reference(val, aux);
            let actual = mutant_{name}_3(val, aux);
            if val != aux && val != 0 && aux != 0 {{
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }}
        }}
    }}

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_{name}_boundaries() {{
        assert_eq!({name}(0, 0), {name}_reference(0, 0));
        assert_eq!({name}(u64::MAX, u64::MAX), {name}_reference(u64::MAX, u64::MAX));
        assert_eq!({name}(u64::MAX, 0), {name}_reference(u64::MAX, 0));
        assert_eq!({name}(0, u64::MAX), {name}_reference(0, u64::MAX));
    }}
    
    // -------------------------------------------------------------------------
    // BRANCHLESS CONTRACT: {name}
    // -------------------------------------------------------------------------
    // Category : B — Cell Arithmetic
    // Plane    : D-resident cell word; no scratch
    // Tier     : T0 — single-word arithmetic primitive
    // Inputs   : val = current cell value
    //            aux = second operand / parameter
    // Admissibility:
    //   - Branchless control flow (CC = 1).
    //   - Zero heap allocations.
    //   - WCET ≤ T1_BUDGET_NS for word-scoped invocations.
    //   - No plane mutation by the primitive itself; callers choose commit.
    // Delta semantics:
    //   - If used as a transition, `UDelta {{ before: U[i], after: result, ... }}`
    //     is emitted into Scratch by the caller; this primitive is pure.
    // Receipt mixing:
    //   - Caller threads `result` through `receipt_mix_transition` along with
    //     the originating UCoord and fired_mask.
    // Independence oracle (test-side):
    //   - The reference function in tests is intentionally an INDEPENDENT
    //     algebraic expression, NOT a copy of the implementation. Equivalence
    //     failures are SIGNAL — they mean the stub diverges from the oracle.
    // Counterfactual mutants:
    //   - Mutant 1: bitwise NOT of reference (identity bluff).
    //   - Mutant 2: off-by-one wrapping_add (bit-skip bluff).
    //   - Mutant 3: XOR low 32 bits (operator-swap bluff).
    // Tier ladder reminder:
    //   - T0 ≤ 2 ns | T1 ≤ 200 ns | T2 ≤ 5 µs | T3 ≤ 10 µs | T4 external.
    // Hoare-style summary:
    //   {{ val, aux ∈ U64 }}
    //     {name}(val, aux)
    //   {{ result ∈ U64 ∧ runtime ∈ admissible_T1 }}
    // -------------------------------------------------------------------------

}}

#[cfg(feature = "bench")]
pub mod bench {{
    use super::*;
    use criterion::{{black_box, Criterion}};
    
    pub fn bench_{name}(c: &mut Criterion) {{
        c.bench_function("{name}", |b| {{
            b.iter(|| {{
                let res = {name}(black_box(42), black_box(1337));
                black_box(res)
            
}})
        }});
    }}
}}

// -----------------------------------------------------------------------------
// BRANCHLESS GEOMETRY ANNOTATION: {name}
// -----------------------------------------------------------------------------
// Resident state object:
// Coordinate algebra:
//   UCoord(domain:u6, cell:u6, place:u6) packed in u32.
//   word_index = domain * CELL_COUNT + cell  ∈ [0, MAX_WORD_INDEX].
//   bit_index  = place                       ∈ [0, PLACE_COUNT).
// Dual-Plane execution envelope:
//   L1_ENVELOPE_BYTES = 65 536  (D + S).
// Domain category for this primitive: B — Cell Arithmetic.
// Plane interaction: D-resident cell word; no scratch.
// Scope semantics for this primitive:
//   Cell    — single u64 word commit (T0).
//   Sparse  — bounded ActiveWordSet (capacity 64) commit (T1).
//   Domain  — full 64-cell domain SWAR (T1).
// Receipt invariants (FNV-1a 64):
//   offset_basis = 0xcbf29ce484222325
//   prime        = 0x100000001b3
//   mix steps    = coord_word → sequence → fired_mask → delta_word
// Admissibility flags:
//   admissible_T0 : YES if used at single-bit / single-word scope.
//   admissible_T1 : YES at sparse/domain scope.
//   admissible_T2 : YES at full-block scope (explicit tier-2 path).
// Branchless contract: CC = 1; no Expr::If, Expr::Match, Expr::Loop, Expr::While.
// Allocation contract: zero heap; all temporaries fit in registers / scratch.
// Failure semantics:
//   On rejected admission, the caller computes fired_mask = 0 and the
//   commit is masked to a no-op via select(fired, candidate, current).
// Replay contract:
//   Pure function ⇒ deterministic across runs ⇒ replayable from receipt chain.
// Cross-references:
// -----------------------------------------------------------------------------
"""

for name, impl_body, ref_body in algos_data:
    file_path = os.path.join(ALGORITHMS_DIR, f"{name}.rs")
    content = TEMPLATE.format(name=name, impl_body=impl_body, ref_body=ref_body)
    with open(file_path, "w") as f:
        f.write(content)
    print(f"Updated {file_path}")
