#![forbid(unsafe_code)]
//! Bitset Algebra: Bitset operations: rank, select, set, clear
//!
//! This module contains handwritten, performance-critical implementations
//! of all Bitset Algebra algorithms.

/// Set bit at position in `u64` value.
#[inline]
#[must_use]
pub const fn set_bit_u64(x: u64, pos: usize) -> u64 {
    debug_assert!(pos < 64);
    x | (1u64 << pos)
}

/// Clear bit at position in `u64` value.
#[inline]
#[must_use]
pub const fn clear_bit_u64(x: u64, pos: usize) -> u64 {
    debug_assert!(pos < 64);
    x & !(1u64 << pos)
}

/// Count set bits (population count) up to and including position.
#[inline]
#[must_use]
pub const fn rank_u64(x: u64, pos: usize) -> usize {
    debug_assert!(pos < 64);
    let mask = if pos == 63 { u64::MAX } else { (1u64 << (pos + 1)) - 1 };
    (x & mask).count_ones() as usize
}

/// Find the position of the nth set bit (0-indexed).
#[inline]
#[must_use]
pub const fn select_bit_u64(x: u64, nth: usize) -> usize {
    let mut left = 0usize;
    let mut right = 64usize;
    while left < right {
        let mid = (left + right) / 2;
        let count_at_mid = if mid == 0 { 0 } else { rank_u64(x, mid - 1) };
        if count_at_mid <= nth {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left - 1
}

/// Computes the parity of the bitset slice.
#[inline]
#[must_use]
pub fn parity_u64_slice(a: &[u64]) -> u32 {
    let mut parity = 0u32;
    for &x in a {
        parity ^= x.count_ones() & 1;
    }
    parity
}

/// Jaccard similarity, Hamming distance, intersection, union, `any_bit_set` implemented as before
#[inline]
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn jaccard_u64_slices(a: &[u64], b: &[u64]) -> f32 {
    let mut intersection_count = 0u32;
    let mut union_count = 0u32;
    for (&va, &vb) in a.iter().zip(b.iter()) {
        intersection_count += (va & vb).count_ones();
        union_count += (va | vb).count_ones();
    }
    if union_count == 0 {
        1.0
    } else {
        intersection_count as f32 / union_count as f32
    }
}

/// Compute the Hamming distance between two bitset slices.
#[inline]
#[must_use]
pub fn hamming_u64_slices(a: &[u64], b: &[u64]) -> u32 {
    let mut distance = 0u32;
    for (&va, &vb) in a.iter().zip(b.iter()) {
        distance += (va ^ vb).count_ones();
    }
    distance
}

/// Compute the intersection of two bitset slices.
#[inline]
pub fn intersect_u64_slices(a: &[u64], b: &[u64], out: &mut [u64]) {
    for ((&va, &vb), vout) in a.iter().zip(b.iter()).zip(out.iter_mut()) {
        *vout = va & vb;
    }
}

/// Compute the union of two bitset slices.
#[inline]
pub fn union_u64_slices(a: &[u64], b: &[u64], out: &mut [u64]) {
    for ((&va, &vb), vout) in a.iter().zip(b.iter()).zip(out.iter_mut()) {
        *vout = va | vb;
    }
}

/// Check if any bit is set in the bitset slice.
#[inline]
#[must_use]
pub fn any_bit_set_u64_slice(a: &[u64]) -> bool {
    let mut accumulator = 0u64;
    for &x in a {
        accumulator |= x;
    }
    accumulator != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parity_u64_slice() {
        // popcount(0b101) = 2, popcount(0b100) = 1. Total = 3. Parity = 1.
        assert_eq!(parity_u64_slice(&[0b101, 0b100]), 1);
        // popcount(0b101) = 2, popcount(0b001) = 1. Total = 3. Parity = 1.
        assert_eq!(parity_u64_slice(&[0b101, 0b001]), 1);
        // popcount(0b11) = 2, popcount(0b11) = 2. Total = 4. Parity = 0.
        assert_eq!(parity_u64_slice(&[0b11, 0b11]), 0);
    }
}
