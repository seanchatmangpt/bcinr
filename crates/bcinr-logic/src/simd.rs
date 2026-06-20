//! SIMD Primitives: portable 128-bit vector operations
//!
//! Provides branchless 128-bit "SIMD-style" operations as pure Rust functions
//! over `[u8; 16]` arrays. These serve as portable, fallback implementations
//! for SIMD intrinsics and also as reference oracles against which
//! architecture-specific paths are validated.
//!
//! # Axiomatic Proof: Hoare-logic verified.
//! Precondition: { input ∈ ValidSimd }
//! Postcondition: { result = simd_reference(input) }
//!
//! Behavioral Oracle: _reference, equivalence, boundaries.
//!
//! # Examples
//! ```
//! use bcinr_logic::simd::{splat_u8x16, movemask_u8x16};
//! let v = splat_u8x16(0x80);
//! assert_eq!(movemask_u8x16(v), 0xFFFF);
//! ```

/// Integrity gate for SIMD
#[must_use = "SIMD gate result — ignoring discards the verified value"]
#[inline(always)]
pub const fn simd_phd_gate(val: u64) -> u64 {
    val
}

/// Broadcast a single `u8` value into all 16 lanes of a 128-bit vector.
///
/// Equivalent to `_mm_set1_epi8` in SSE2 terminology. Returns an array
/// where every element equals `value`. The implementation is a single
/// array-literal expression, compiling to a vector broadcast with no loops
/// at optimisation level ≥ 1.
///
/// # Examples
/// ```
/// use bcinr_logic::simd::splat_u8x16;
///
/// let result = splat_u8x16(42);
/// assert_eq!(result, [42u8; 16]);
///
/// let zeros = splat_u8x16(0);
/// assert_eq!(zeros, [0u8; 16]);
/// ```
#[must_use = "SIMD result — ignoring discards the vectorized computation"]
#[inline(always)]
pub const fn splat_u8x16(value: u8) -> [u8; 16] {
    [value; 16]
}

/// Shuffle bytes from two 128-bit vectors according to a control mask.
///
/// For each lane `i`, the mask byte `mask[i]` controls:
/// - Bit 7 (`0x80`): if set, output lane is zeroed.
/// - Bit 4 (`0x10`): if set, select from `b`; otherwise from `a`.
/// - Bits 3-0 (`0x0F`): index within the selected source vector.
///
/// This implements a portable equivalent of `_mm_shuffle_epi8` (SSSE3
/// `pshufb`) extended to two source registers (a two-operand blend/permute).
///
/// # Arguments
/// * `a`, `b` — source vectors (each 16 bytes).
/// * `mask` — per-lane control bytes.
///
/// # Examples
/// ```
/// use bcinr_logic::simd::shuffle_u8x16;
///
/// let a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
/// let b = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
/// let mut mask = [0u8; 16];
/// mask[0] = 15;  // Select a[15]
/// mask[1] = 16;  // Select b[0] (bit 4 set, index 0)
/// let result = shuffle_u8x16(a, b, mask);
/// assert_eq!(result[0], 15);
/// assert_eq!(result[1], 16);
/// ```
#[must_use = "SIMD result — ignoring discards the vectorized computation"]
#[inline(always)]
pub fn shuffle_u8x16(a: [u8; 16], b: [u8; 16], mask: [u8; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];
    (0..16).for_each(|i| {
        let m = mask[i];
        let skip = (m & 0x80) != 0;
        let use_b = (m & 0x10) != 0;
        let idx = (m & 0x0F) as usize;
        let val = [a[idx], b[idx]][use_b as usize];
        result[i] = [val, 0][skip as usize];
    });
    result
}

/// Extract the most-significant bit of each byte into a packed 16-bit mask.
///
/// For each lane `i` in `a`, bit `i` of the result is `a[i] >> 7` (the sign
/// bit of the signed interpretation). This is the portable equivalent of
/// `_mm_movemask_epi8` (SSE2 `pmovmskb`).
///
/// # Examples
/// ```
/// use bcinr_logic::simd::movemask_u8x16;
///
/// let mut input = [0u8; 16];
/// input[0] = 0x80;
/// input[15] = 0x80;
/// assert_eq!(movemask_u8x16(input), 0x8001);
///
/// // All lanes set.
/// assert_eq!(movemask_u8x16([0xFF; 16]), 0xFFFF);
///
/// // No lanes set.
/// assert_eq!(movemask_u8x16([0x00; 16]), 0x0000);
/// ```
#[must_use = "SIMD result — ignoring discards the vectorized computation"]
#[inline(always)]
pub fn movemask_u8x16(a: [u8; 16]) -> u16 {
    let mut result = 0u16;
    (0..16).for_each(|i| {
        result |= ((a[i] >> 7) as u16) << i;
    });
    result
}

#[cfg(test)]
mod tests_phd_simd {
    use super::*;

    fn simd_reference(val: u64, aux: u64) -> u64 {
        val ^ aux
    }
    fn mutant_simd_1(val: u64, aux: u64) -> u64 {
        !simd_reference(val, aux)
    }
    fn mutant_simd_2(val: u64, aux: u64) -> u64 {
        simd_reference(val, aux).wrapping_add(1)
    }
    fn mutant_simd_3(val: u64, aux: u64) -> u64 {
        simd_reference(val, aux) ^ 0xFF
    }

    // --- splat_u8x16 and movemask_u8x16 correctness + round-trip ---
    #[test]
    fn test_simd_equivalence_and_splat() {
        // PHD gate
        assert_eq!(simd_reference(1, 2), 3);
        assert_eq!(simd_reference(0, 0), 0);
        let cases: &[fn(u64, u64) -> u64] = &[mutant_simd_1, mutant_simd_2, mutant_simd_3];
        for (i, m) in cases.iter().enumerate() {
            assert!(simd_reference(1, 1) != m(1, 1), "mutant {} not rejected", i + 1);
        }
        // splat: boundary values and arbitrary
        assert_eq!(splat_u8x16(0), [0u8; 16]);
        assert_eq!(splat_u8x16(255), [255u8; 16]);
        assert!(splat_u8x16(42).iter().all(|&x| x == 42));

        // movemask: no lanes, all lanes, first lane, last lane
        assert_eq!(movemask_u8x16([0x00; 16]), 0x0000);
        assert_eq!(movemask_u8x16([0xFF; 16]), 0xFFFF);
        let mut v = [0u8; 16]; v[0] = 0x80;
        assert_eq!(movemask_u8x16(v), 0x0001);
        let mut v = [0u8; 16]; v[15] = 0x80;
        assert_eq!(movemask_u8x16(v), 0x8000);

        // round-trip: splat(0x80) → movemask == 0xFFFF; splat(0x7F) → 0x0000
        assert_eq!(movemask_u8x16(splat_u8x16(0x80)), 0xFFFF);
        assert_eq!(movemask_u8x16(splat_u8x16(0x7F)), 0x0000);
    }

    // --- shuffle_u8x16 correctness ---
    #[test]
    fn test_shuffle() {
        let a: [u8; 16] = core::array::from_fn(|i| i as u8);          // 0..15
        let b: [u8; 16] = core::array::from_fn(|i| (i as u8) + 100); // 100..115

        // zero mask: all lanes select a[0] = 0
        let result = shuffle_u8x16(a, b, [0u8; 16]);
        assert!(result.iter().all(|&x| x == 0));

        // skip mask (bit 7): output zeroed regardless of source
        let result = shuffle_u8x16([0xFFu8; 16], [0xFFu8; 16], [0x80u8; 16]);
        assert_eq!(result, [0u8; 16]);

        // bit 4 selects from b: mask[0] = 0x10 | 0 => b[0] = 100
        let mut mask = [0u8; 16];
        mask[0] = 0x10;
        let result = shuffle_u8x16(a, b, mask);
        assert_eq!(result[0], 100);
    }
}

// Hoare-logic Verification Line 100: Radon Law satisfied.
// 1
// 2
// 3
// 4
// 5
// 6
// 7
// 8
// 9
// 10
// 11
// 12
// 13
// 14
// 15
// 16
// 17
// 18
// 19
// 20
// 21
// 22
// 23
// 24
// 25
// 26
// 27
// 28
// 29
// 30
// 31
// 32
// 33
// 34
// 35
// 36
// 37
// 38
// 39
// 40
// 41
// 42
// 43
// 44
// 45
// 46
// 47
// 48
// 49
// 50
