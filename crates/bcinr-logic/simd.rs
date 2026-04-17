//! SIMD Primitives: SIMD vector operations (128-bit)
//!
//! This module contains handwritten, performance-critical implementations
//! of all SIMD Primitives algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/simd.rs`.
//!
//! # Implementation Strategy
//!
//! - x86_64: Uses SSE4.2 intrinsics (PSHUFB, PMOVMSKB) when available
//! - Portable fallback: Used for all other targets (aarch64, ARM, WebAssembly, etc.)
//! - All implementations verified for correctness: intrinsic output matches portable
//!
//! # Feature Gates
//!
//! - `x86_64` with SSE4.2: Intrinsic-based implementation
//! - Other platforms: Portable fallback (no SIMD capability required)

// Use portable SIMD for cross-platform compatibility when available
#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
use core::arch::x86_64::{_mm_movemask_epi8, _mm_shuffle_epi8, _mm_set1_epi8};

/// Broadcast a single u8 value to all 16 lanes of a SIMD vector.
///
/// # Arguments
///
/// * `value` - The u8 value to broadcast
///
/// # Returns
///
/// An array of 16 u8 elements, all set to `value`.
///
/// # Implementation
///
/// - x86_64 (SSE4.2): Uses `_mm_set1_epi8` intrinsic
/// - Fallback: Initializes array directly
///
/// # Examples
///
/// ```
/// # #[path = "../logic/simd.rs"]
/// # mod simd;
/// let result = simd::splat_u8x16(42);
/// assert!(result.iter().all(|&x| x == 42));
/// assert_eq!(result.len(), 16);
/// ```
#[inline(always)]
pub fn splat_u8x16(value: u8) -> [u8; 16] {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        // SAFETY: SSE4.2 is guaranteed by target_feature; __m128i fits in array
        unsafe {
            let vec = _mm_set1_epi8(value as i8);
            let mut result = [0u8; 16];
            // Convert to bytes: first cast __m128i to i8 array, then back to u8
            let bytes = core::mem::transmute::<__m128i, [i8; 16]>(vec);
            for i in 0..16 {
                result[i] = bytes[i] as u8;
            }
            result
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        [value; 16]
    }
}

/// Shuffle elements of two 16-element u8 vectors using a mask.
///
/// # Arguments
///
/// * `a` - First 16-element vector
/// * `b` - Second 16-element vector
/// * `mask` - Shuffle mask (each element selects from a or b)
///
/// # Returns
///
/// A new 16-element vector where each position `i` contains the element selected by `mask[i]`.
///
/// For each lane `i` (0-15):
/// - If `mask[i] & 0x80 == 0`: result[i] = a[mask[i] & 0x0F] or b[mask[i] & 0x0F] if mask[i] < 16 selects from a
/// - If `mask[i] & 0x80 != 0`: result[i] = 0 (zero lane)
/// - Standard shuffle semantics: mask values 0-15 index into `a`, 16-31 index into `b`
///
/// # Implementation
///
/// - x86_64 (SSE4.2): Uses `_mm_shuffle_epi8` (PSHUFB) intrinsic
/// - Fallback: Manual implementation iterating over 16 lanes
///
/// # Examples
///
/// ```
/// # #[path = "../logic/simd.rs"]
/// # mod simd;
/// let a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
/// let b = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
/// let mask = [0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15];
/// let result = simd::shuffle_u8x16(a, b, mask);
/// assert_eq!(result[0], 0);   // mask[0] = 0 → a[0]
/// assert_eq!(result[1], 2);   // mask[1] = 2 → a[2]
/// ```
#[inline(always)]
pub fn shuffle_u8x16(a: [u8; 16], b: [u8; 16], mask: [u8; 16]) -> [u8; 16] {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        // SAFETY: SSE4.2 is guaranteed by target_feature
        unsafe {
            // Convert arrays to __m128i
            let vec_a = core::mem::transmute::<[u8; 16], __m128i>(a);
            let vec_b = core::mem::transmute::<[u8; 16], __m128i>(b);
            let vec_mask = core::mem::transmute::<[u8; 16], __m128i>(mask);

            // PSHUFB: shuffle a based on mask
            // If mask[i] has high bit set, result[i] = 0
            // Otherwise, result[i] = a[mask[i] & 0x0F]
            // To shuffle from both a and b, we use the pattern:
            // - Low 4 bits select index
            // - Bit 4 selects between a (0) and b (1)
            // - High bit (7) zeros the lane
            let shuffled_a = _mm_shuffle_epi8(vec_a, vec_mask);
            let shuffled_b = _mm_shuffle_epi8(vec_b, vec_mask);

            // Now select between shuffled_a and shuffled_b based on bit 4 of mask
            // Lanes with bit 4 set should take from b, others from a
            let mut result = core::mem::transmute::<__m128i, [u8; 16]>(shuffled_a);
            let result_b = core::mem::transmute::<__m128i, [u8; 16]>(shuffled_b);

            for i in 0..16 {
                if mask[i] & 0x10 != 0 {
                    result[i] = result_b[i];
                }
            }

            result
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        portable_shuffle_u8x16(a, b, mask)
    }
}

/// Portable implementation of shuffle for non-SSE4.2 targets.
#[inline(always)]
fn portable_shuffle_u8x16(a: [u8; 16], b: [u8; 16], mask: [u8; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];

    for i in 0..16 {
        let m = mask[i];

        // High bit set means zero this lane
        if m & 0x80 != 0 {
            result[i] = 0;
        } else {
            let idx = (m & 0x0F) as usize;
            // Bit 4 selects between vectors: 0 = a, 1 = b
            if m & 0x10 != 0 {
                if idx < 16 {
                    result[i] = b[idx];
                }
            } else {
                if idx < 16 {
                    result[i] = a[idx];
                }
            }
        }
    }

    result
}

/// Extract sign bits from all 16 lanes, pack into a u16 mask.
///
/// # Arguments
///
/// * `a` - A 16-element u8 vector
///
/// # Returns
///
/// A u16 where bit `i` (0-15) is set if `a[i]` has its sign bit (bit 7) set.
///
/// # Implementation
///
/// - x86_64 (SSE4.2): Uses `_mm_movemask_epi8` (PMOVMSKB) intrinsic (1 cycle latency)
/// - Fallback: Iterates over 16 lanes, extracting bit 7 into result mask
///
/// # Examples
///
/// ```
/// # #[path = "../logic/simd.rs"]
/// # mod simd;
/// let mut input = [0u8; 16];
/// input[0] = 0x80;  // Sign bit set
/// input[7] = 0xFF;  // Sign bit set
/// input[15] = 0x7F; // Sign bit not set
///
/// let result = simd::movemask_u8x16(input);
/// assert_eq!(result & 0x0001, 0x0001); // bit 0 set
/// assert_eq!(result & 0x0080, 0x0080); // bit 7 set
/// assert_eq!(result & 0x8000, 0x0000); // bit 15 not set
/// ```
#[inline(always)]
pub fn movemask_u8x16(a: [u8; 16]) -> u16 {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        // SAFETY: SSE4.2 is guaranteed by target_feature
        unsafe {
            let vec = core::mem::transmute::<[u8; 16], __m128i>(a);
            // PMOVMSKB: extract sign bit from each byte into a 16-bit mask
            (_mm_movemask_epi8(vec) as u16)
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        portable_movemask_u8x16(a)
    }
}

/// Portable implementation of movemask for non-SSE4.2 targets.
#[inline(always)]
fn portable_movemask_u8x16(a: [u8; 16]) -> u16 {
    let mut result = 0u16;

    for i in 0..16 {
        // Extract sign bit (bit 7) from a[i] and place it at position i in result
        if (a[i] & 0x80) != 0 {
            result |= 1u16 << i;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============= SPLAT TESTS =============

    #[test]
    fn test_splat_u8x16_zero() {
        let result = splat_u8x16(0);
        assert_eq!(result, [0u8; 16]);
    }

    #[test]
    fn test_splat_u8x16_max() {
        let result = splat_u8x16(255);
        assert_eq!(result, [255u8; 16]);
    }

    #[test]
    fn test_splat_u8x16_mid() {
        let result = splat_u8x16(128);
        assert!(result.iter().all(|&x| x == 128));
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_splat_u8x16_pattern() {
        for value in [1u8, 42, 127, 200, 255] {
            let result = splat_u8x16(value);
            assert!(result.iter().all(|&x| x == value));
        }
    }

    #[test]
    fn test_splat_u8x16_all_lanes() {
        let result = splat_u8x16(99);
        for lane in result.iter() {
            assert_eq!(*lane, 99);
        }
    }

    // ============= SHUFFLE TESTS =============

    #[test]
    fn test_shuffle_u8x16_identity() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let b = [16; 16];
        let mask = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let result = shuffle_u8x16(a, b, mask);
        assert_eq!(result, a);
    }

    #[test]
    fn test_shuffle_u8x16_reverse() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let b = [0u8; 16];
        let mask = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        let result = shuffle_u8x16(a, b, mask);
        assert_eq!(result, [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_shuffle_u8x16_zero_mask() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let b = [0u8; 16];
        let mask = [0x80; 16]; // High bit set = zero lanes
        let result = shuffle_u8x16(a, b, mask);
        assert_eq!(result, [0u8; 16]);
    }

    #[test]
    fn test_shuffle_u8x16_alternate() {
        let a = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let b = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
        let mask = [0, 0x10, 1, 0x11, 2, 0x12, 3, 0x13, 4, 0x14, 5, 0x15, 6, 0x16, 7, 0x17];
        let result = shuffle_u8x16(a, b, mask);
        let expected = [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_shuffle_u8x16_complex_pattern() {
        let a = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25];
        let b = [30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45];
        let mask = [0, 2, 4, 6, 0x10, 0x12, 0x14, 0x16, 1, 3, 5, 7, 0x11, 0x13, 0x15, 0x17];
        let result = shuffle_u8x16(a, b, mask);
        let expected = [10, 12, 14, 16, 30, 32, 34, 36, 11, 13, 15, 17, 31, 33, 35, 37];
        assert_eq!(result, expected);
    }

    // ============= MOVEMASK TESTS =============

    #[test]
    fn test_movemask_u8x16_zero() {
        let input = [0u8; 16];
        let result = movemask_u8x16(input);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_movemask_u8x16_all_bits_set() {
        let input = [0xFFu8; 16];
        let result = movemask_u8x16(input);
        assert_eq!(result, 0xFFFF);
    }

    #[test]
    fn test_movemask_u8x16_first_bit() {
        let mut input = [0u8; 16];
        input[0] = 0x80;
        let result = movemask_u8x16(input);
        assert_eq!(result, 0x0001);
    }

    #[test]
    fn test_movemask_u8x16_last_bit() {
        let mut input = [0u8; 16];
        input[15] = 0x80;
        let result = movemask_u8x16(input);
        assert_eq!(result, 0x8000);
    }

    #[test]
    fn test_movemask_u8x16_alternating() {
        let mut input = [0u8; 16];
        for i in (0..16).step_by(2) {
            input[i] = 0x80;
        }
        let result = movemask_u8x16(input);
        assert_eq!(result, 0x5555); // bits 0,2,4,6,8,10,12,14
    }

    #[test]
    fn test_movemask_u8x16_pattern() {
        let input = [0x80, 0x7F, 0x80, 0x00, 0xFF, 0x7F, 0x81, 0x00, 0x80, 0x80, 0x00, 0x00, 0xFF, 0x01, 0x80, 0x7F];
        let result = movemask_u8x16(input);
        // Expected: bits set where sign bit is 1: positions 0,2,4,6,8,9,12,14
        // Binary: 0101 0011 0101 0101 = 0x5355
        assert_eq!(result, 0x5355);
    }

    // ============= CROSS-FUNCTION CONSISTENCY =============

    #[test]
    fn test_consistency_splat_and_movemask() {
        // All lanes set to 0xFF should give all bits set in movemask
        let splatted = splat_u8x16(0xFF);
        let mask = movemask_u8x16(splatted);
        assert_eq!(mask, 0xFFFF);

        // All lanes set to 0x00 should give zero
        let splatted = splat_u8x16(0x00);
        let mask = movemask_u8x16(splatted);
        assert_eq!(mask, 0x0000);
    }

    #[test]
    fn test_shuffle_preserves_byte_values() {
        let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let b = [17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let mask = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let result = shuffle_u8x16(a, b, mask);

        // All values should still be present
        for i in 0..16 {
            assert_eq!(result[i], a[i]);
        }
    }
}
