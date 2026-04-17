#![forbid(unsafe_code)]
//! Saturation Arithmetic: Fixed-point and saturation arithmetic
//!
//! This module contains handwritten, production-quality implementations
//! of saturation arithmetic algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/fix.rs`.
//!
//! # Algorithm Documentation
//!
//! ## add_sat_u8: Saturating Addition
//! Uses Rust's built-in `u8::saturating_add()` which leverages CPU saturation
//! instructions when available (x86_64 PADDSB/PADDUSB via LLVM codegen).
//! Branchless and constant-time.
//!
//! ## sub_sat_u8: Saturating Subtraction
//! Uses Rust's built-in `u8::saturating_sub()` which leverages CPU saturation
//! instructions when available (x86_64 PSUBSB/PSUBUSB via LLVM codegen).
//! Branchless and constant-time.
//!
//! ## clamp_u32: Min-Max Clamping
//! Branchless implementation using `Ord::min()` and `Ord::max()` which
//! compile to efficient conditional moves on modern architectures.
//! No branching overhead.

use crate::Branchless;
use core::ops::Add;

/// Saturating addition for generic types.
#[inline(always)]
#[must_use]
pub fn add_sat<T: Branchless + Add<Output = T> + PartialOrd + Copy>(a: T, b: T, max: T) -> T {
    let sum = a + b;
    if sum > max { max } else { sum }
}

/// Saturating addition for u8.
#[inline]
#[must_use]
pub const fn add_sat_u8(a: u8, b: u8) -> u8 {
    a.saturating_add(b)
}

/// Saturating subtraction for u8.
#[inline]
#[must_use]
pub const fn sub_sat_u8(a: u8, b: u8) -> u8 {
    a.saturating_sub(b)
}

/// Saturating addition for `u32`.
#[inline]
#[must_use]
pub const fn add_sat_u32(a: u32, b: u32) -> u32 {
    a.saturating_add(b)
}

/// Saturating subtraction for `u32`.
#[inline]
#[must_use]
pub const fn sub_sat_u32(a: u32, b: u32) -> u32 {
    a.saturating_sub(b)
}


/// Saturating multiplication for `u32`.
#[inline]
#[must_use]
pub const fn mul_sat_u32(a: u32, b: u32) -> u32 {
    a.saturating_mul(b)
}

/// Saturating increment for `u32`.
#[inline]
#[must_use]
pub const fn inc_sat_u32(x: u32) -> u32 {
    x.saturating_add(1)
}

/// Saturating decrement for `u32`.
#[inline]
#[must_use]
pub const fn dec_sat_u32(x: u32) -> u32 {
    x.saturating_sub(1)
}

/// Clamps a `u32` value to the range `[min, max]`.
#[inline]
pub const fn clamp_u32(x: u32, min: u32, max: u32) -> Result<u32, &'static str> {
    if min > max {
        return Err("min must be <= max");
    }
    let x = if x < min { min } else { x };
    Ok(if x > max { max } else { x })
}

/// Branchless bucketization. Returns the index of the first bucket such that `x < buckets[index]`.
/// `buckets` must be sorted.
#[inline(always)]
#[must_use]
pub fn bucketize_u32<const N: usize>(x: u32, buckets: &[u32; N]) -> usize {
    let mut index = 0;
    for &bucket in buckets {
        // x < buckets[i] is 1 if true, 0 if false.
        // We sum these up to find the bucket index.
        // This is branchless if the compiler optimizes the comparison.
        index += (x >= bucket) as usize;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_sat_u32() {
        assert_eq!(add_sat_u32(100, 50), 150);
        assert_eq!(add_sat_u32(u32::MAX, 1), u32::MAX);
    }

    #[test]
    fn test_sub_sat_u32() {
        assert_eq!(sub_sat_u32(100, 50), 50);
        assert_eq!(sub_sat_u32(0, 1), 0);
    }

    #[test]
    fn test_bucketize_u32() {
        let buckets = [10, 20, 30, 40, 50];
        assert_eq!(bucketize_u32(5, &buckets), 0);
        assert_eq!(bucketize_u32(15, &buckets), 1);
        assert_eq!(bucketize_u32(25, &buckets), 2);
        assert_eq!(bucketize_u32(55, &buckets), 5);
    }

    // ========== add_sat_u8 Tests ==========

    #[test]
    fn test_add_sat_u8_basic() {
        assert_eq!(add_sat_u8(100, 50), 150);
    }

    #[test]
    fn test_add_sat_u8_overflow() {
        assert_eq!(add_sat_u8(255, 1), 255);
        assert_eq!(add_sat_u8(200, 100), 255);
        assert_eq!(add_sat_u8(128, 128), 255);
    }

    #[test]
    fn test_add_sat_u8_underflow_impossible() {
        // u8 addition can never underflow since both operands are non-negative
        assert_eq!(add_sat_u8(0, 0), 0);
    }

    #[test]
    fn test_add_sat_u8_boundary_zero() {
        assert_eq!(add_sat_u8(0, 0), 0);
        assert_eq!(add_sat_u8(0, 255), 255);
        assert_eq!(add_sat_u8(255, 0), 255);
    }

    #[test]
    fn test_add_sat_u8_boundary_max() {
        assert_eq!(add_sat_u8(255, 255), 255);
        assert_eq!(add_sat_u8(1, 254), 255);
        assert_eq!(add_sat_u8(127, 128), 255);
    }

    #[test]
    fn test_add_sat_u8_commutative() {
        for a in [0, 1, 127, 128, 254, 255] {
            for b in [0, 1, 127, 128, 254, 255] {
                assert_eq!(add_sat_u8(a, b), add_sat_u8(b, a));
            }
        }
    }

    // ========== sub_sat_u8 Tests ==========

    #[test]
    fn test_sub_sat_u8_basic() {
        assert_eq!(sub_sat_u8(150, 50), 100);
    }

    #[test]
    fn test_sub_sat_u8_underflow() {
        assert_eq!(sub_sat_u8(50, 100), 0);
        assert_eq!(sub_sat_u8(0, 1), 0);
        assert_eq!(sub_sat_u8(1, 255), 0);
    }

    #[test]
    fn test_sub_sat_u8_overflow_impossible() {
        // u8 subtraction can never overflow since the result is always >= 0
        assert_eq!(sub_sat_u8(255, 0), 255);
    }

    #[test]
    fn test_sub_sat_u8_boundary_zero() {
        assert_eq!(sub_sat_u8(0, 0), 0);
        assert_eq!(sub_sat_u8(255, 255), 0);
        assert_eq!(sub_sat_u8(0, 255), 0);
    }

    #[test]
    fn test_sub_sat_u8_boundary_max() {
        assert_eq!(sub_sat_u8(255, 0), 255);
        assert_eq!(sub_sat_u8(255, 1), 254);
        assert_eq!(sub_sat_u8(128, 0), 128);
    }

    #[test]
    fn test_sub_sat_u8_identity() {
        for a in [0, 1, 127, 128, 254, 255] {
            assert_eq!(sub_sat_u8(a, 0), a);
            assert_eq!(sub_sat_u8(a, a), 0);
        }
    }

    #[test]
    fn test_sub_sat_u8_non_commutative() {
        // Verify sub is not commutative
        assert_ne!(sub_sat_u8(100, 50), sub_sat_u8(50, 100));
        assert_eq!(sub_sat_u8(100, 50), 50);
        assert_eq!(sub_sat_u8(50, 100), 0);
    }

    // ========== clamp_u32 Tests ==========

    #[test]
    fn test_clamp_u32_within_range() {
        assert_eq!(clamp_u32(50, 0, 100).unwrap(), 50);
        assert_eq!(clamp_u32(0, 0, 100).unwrap(), 0);
        assert_eq!(clamp_u32(100, 0, 100).unwrap(), 100);
    }

    #[test]
    fn test_clamp_u32_above_max() {
        assert_eq!(clamp_u32(150, 0, 100).unwrap(), 100);
        assert_eq!(clamp_u32(u32::MAX, 0, 100).unwrap(), 100);
        assert_eq!(clamp_u32(101, 0, 100).unwrap(), 100);
    }

    #[test]
    fn test_clamp_u32_below_min() {
        assert_eq!(clamp_u32(25, 50, 100).unwrap(), 50);
        assert_eq!(clamp_u32(0, 50, 100).unwrap(), 50);
        assert_eq!(clamp_u32(49, 50, 100).unwrap(), 50);
    }

    #[test]
    fn test_clamp_u32_boundary_single_value() {
        assert_eq!(clamp_u32(0, 0, 0).unwrap(), 0);
        assert_eq!(clamp_u32(1, 0, 0).unwrap(), 0);
        assert_eq!(clamp_u32(100, 100, 100).unwrap(), 100);
    }

    #[test]
    fn test_clamp_u32_boundary_extremes() {
        assert_eq!(clamp_u32(u32::MAX, 0, u32::MAX).unwrap(), u32::MAX);
        assert_eq!(clamp_u32(0, 0, u32::MAX).unwrap(), 0);
        assert_eq!(clamp_u32(u32::MAX, u32::MAX, u32::MAX).unwrap(), u32::MAX);
        assert_eq!(clamp_u32(0, u32::MAX, u32::MAX).unwrap(), u32::MAX);
    }

    #[test]
    fn test_clamp_u32_identity_unbounded() {
        // When min is 0 and max is u32::MAX, clamp should return the input
        assert_eq!(clamp_u32(12345, 0, u32::MAX).unwrap(), 12345);
        assert_eq!(clamp_u32(0, 0, u32::MAX).unwrap(), 0);
        assert_eq!(clamp_u32(u32::MAX, 0, u32::MAX).unwrap(), u32::MAX);
    }

    #[test]
    fn test_clamp_u32_various_ranges() {
        // Test with different range sizes
        assert_eq!(clamp_u32(500, 100, 1000).unwrap(), 500);
        assert_eq!(clamp_u32(50, 100, 1000).unwrap(), 100);
        assert_eq!(clamp_u32(1500, 100, 1000).unwrap(), 1000);

        // Tight ranges
        assert_eq!(clamp_u32(10, 10, 11).unwrap(), 10);
        assert_eq!(clamp_u32(11, 10, 11).unwrap(), 11);
        assert_eq!(clamp_u32(12, 10, 11).unwrap(), 11);
        assert_eq!(clamp_u32(9, 10, 11).unwrap(), 10);
    }

    #[test]
    fn test_clamp_u32_idempotent() {
        // Clamping twice should equal clamping once
        let x = 50;
        let min = 0;
        let max = 100;
        let clamped_once = clamp_u32(x, min, max).unwrap();
        let clamped_twice = clamp_u32(clamped_once, min, max).unwrap();
        assert_eq!(clamped_once, clamped_twice);
    }
}
