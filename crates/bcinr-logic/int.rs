//! Integer Bitwise: Integer bit manipulation without branches
//!
//! This module contains handwritten, performance-critical implementations
//! of all Integer Bitwise algorithms.

/// Counts the number of set bits (population count) in a u64.
#[inline(always)]
pub fn popcount_u64(x: u64) -> u64 {
    x.count_ones() as u64
}

/// Counts the number of leading zeros in a u64 (from MSB).
#[inline(always)]
pub fn leading_zeros_u64(x: u64) -> u64 {
    x.leading_zeros() as u64
}

/// Counts the number of trailing zeros in a u64 (from LSB).
#[inline(always)]
pub fn trailing_zeros_u64(x: u64) -> u64 {
    x.trailing_zeros() as u64
}

/// Reverses the bits of a u64.
#[inline(always)]
pub fn reverse_bits_u64(mut x: u64) -> u64 {
    x = ((x >> 1) & 0x5555555555555555) | ((x & 0x5555555555555555) << 1);
    x = ((x >> 2) & 0x3333333333333333) | ((x & 0x3333333333333333) << 2);
    x = ((x >> 4) & 0x0F0F0F0F0F0F0F0F) | ((x & 0x0F0F0F0F0F0F0F0F) << 4);
    x = ((x >> 8) & 0x00FF00FF00FF00FF) | ((x & 0x00FF00FF00FF00FF) << 8);
    x = ((x >> 16) & 0x0000FFFF0000FFFF) | ((x & 0x0000FFFF0000FFFF) << 16);
    x = x.rotate_left(32);
    x
}

/// Signed saturating addition for u64 (casting to i64).
#[inline(always)]
pub fn saturating_add_i64(a: i64, b: i64) -> i64 {
    a.saturating_add(b)
}

/// Signed saturating subtraction for u64 (casting to i64).
#[inline(always)]
pub fn saturating_sub_i64(a: i64, b: i64) -> i64 {
    a.saturating_sub(b)
}

/// Signed saturating multiplication for u64 (casting to i64).
#[inline(always)]
pub fn saturating_mul_i64(a: i64, b: i64) -> i64 {
    a.saturating_mul(b)
}

/// Counts the number of set bits (population count) in a u32.
#[inline(always)]
pub fn popcount_u32(x: u32) -> u32 {
    x.count_ones()
}

/// Counts the number of leading zeros in a u32 (from MSB).
#[inline(always)]
pub fn leading_zeros_u32(x: u32) -> u32 {
    x.leading_zeros()
}

/// Rounds up to the next power of two in a branchless, constant-time manner.
#[inline(always)]
pub fn next_power_of_two_u32(mut x: u32) -> u32 {
    x = x.saturating_sub(1);
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x.wrapping_add(1)
}

/// Returns true if x is a power of two.
#[inline(always)]
pub fn is_pow2_u32(x: u32) -> bool {
    x != 0 && (x & (x.wrapping_sub(1))) == 0
}

/// Returns the parity of x (1 if number of set bits is odd, else 0).
#[inline(always)]
pub fn parity_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x ^= x >> 8;
    x ^= x >> 4;
    x &= 0xf;
    (0x6996 >> x) & 1
}

/// Reverses the bits of a u32.
#[inline(always)]
pub fn reverse_bits_u32(mut x: u32) -> u32 {
    x = ((x >> 1) & 0x55555555) | ((x & 0x55555555) << 1);
    x = ((x >> 2) & 0x33333333) | ((x & 0x33333333) << 2);
    x = ((x >> 4) & 0x0F0F0F0F) | ((x & 0x0F0F0F0F) << 4);
    x = ((x >> 8) & 0x00FF00FF) | ((x & 0x00FF00FF) << 8);
    x = x.rotate_left(16);
    x
}

/// Counts the number of trailing zeros in a u32.
#[inline(always)]
pub fn trailing_zeros_u32(x: u32) -> u32 {
    x.trailing_zeros()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcount_u64() {
        assert_eq!(popcount_u64(0), 0);
        assert_eq!(popcount_u64(1), 1);
        assert_eq!(popcount_u64(0xFFFFFFFFFFFFFFFF), 64);
    }

    #[test]
    fn test_reverse_bits_u64() {
        assert_eq!(reverse_bits_u64(1), 0x8000000000000000);
        assert_eq!(reverse_bits_u64(0x8000000000000000), 1);
    }
}
