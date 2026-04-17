//! SWAR Reductions: SIMD Within A Register: horizontal reductions
//!
//! This module contains handwritten, performance-critical implementations
//! of all SWAR Reductions algorithms.

/// Horizontal bitwise OR of u32 slice.
#[inline(always)]
pub fn horizontal_or_u32(slice: &[u32]) -> u32 {
    let mut acc = 0u32;
    for &x in slice { acc |= x; }
    acc
}

/// Horizontal bitwise AND of u32 slice.
#[inline(always)]
pub fn horizontal_and_u32(slice: &[u32]) -> u32 {
    if slice.is_empty() { return 0; }
    let mut acc = slice[0];
    for &x in &slice[1..] { acc &= x; }
    acc
}

/// Horizontal bitwise XOR of u32 slice.
#[inline(always)]
pub fn horizontal_xor_u32(slice: &[u32]) -> u32 {
    let mut acc = 0u32;
    for &x in slice { acc ^= x; }
    acc
}

/// Branchless Murmur3-like hash for u64.
#[inline(always)]
pub fn murmur3_hash_u64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^= x >> 33;
    x
}

// ... existing reduction methods ...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reductions() {
        assert_eq!(horizontal_or_u32(&[1, 2, 4]), 7);
        assert_eq!(horizontal_and_u32(&[0b11, 0b10]), 2);
        assert_eq!(horizontal_xor_u32(&[1, 1, 2]), 2);
    }
}
