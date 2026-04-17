#![forbid(unsafe_code)]
//! SWAR Reductions: SIMD Within A Register: horizontal reductions
//!
//! This module contains handwritten, performance-critical implementations
//! of all SWAR Reductions algorithms.

/// Horizontal bitwise OR of u32 slice.
#[inline(always)]
#[must_use]
pub fn horizontal_or_u32(slice: &[u32]) -> u32 {
    let mut acc = 0u32;
    for &x in slice { acc |= x; }
    acc
}

/// Horizontal bitwise AND of u32 slice.
#[inline(always)]
#[must_use]
pub fn horizontal_and_u32(slice: &[u32]) -> u32 {
    if slice.is_empty() { return 0; }
    let mut acc = slice[0];
    for &x in &slice[1..] { acc &= x; }
    acc
}

/// Horizontal bitwise XOR of u32 slice.
#[inline(always)]
#[must_use]
pub fn horizontal_xor_u32(slice: &[u32]) -> u32 {
    let mut acc = 0u32;
    for &x in slice { acc ^= x; }
    acc
}

/// Branchless Murmur3-like hash for u64.
#[inline(always)]
#[must_use]
pub fn murmur3_hash_u64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^= x >> 33;
    x
}

/// Horizontal sum of 8 bytes in a slice (first 8 bytes).
#[inline(always)]
#[must_use]
pub fn horizontal_sum_u8x8(slice: &[u8]) -> u64 {
    if slice.len() < 8 {
        return 0;
    }
    let mut sum = 0u64;
    for i in 0..8 {
        sum += slice[i] as u64;
    }
    sum
}

/// Horizontal maximum of 8 bytes in a slice (first 8 bytes).
#[inline(always)]
#[must_use]
pub fn horizontal_max_u8x8(slice: &[u8]) -> u8 {
    if slice.is_empty() {
        return 0;
    }
    let mut max = slice[0];
    let end = 8.min(slice.len());
    for i in 1..end {
        max = core::cmp::max(max, slice[i]);
    }
    max
}

/// Horizontal minimum of 8 bytes in a slice (first 8 bytes).
#[inline(always)]
#[must_use]
pub fn horizontal_min_u8x8(slice: &[u8]) -> u8 {
    if slice.is_empty() {
        return 0;
    }
    let mut min = slice[0];
    let end = 8.min(slice.len());
    for i in 1..end {
        min = core::cmp::min(min, slice[i]);
    }
    min
}

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
