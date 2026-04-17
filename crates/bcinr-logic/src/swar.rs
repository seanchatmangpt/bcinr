#![forbid(unsafe_code)]
//! SWAR (SIMD Within A Register) primitives for byte-parallel operations.

/// Returns a mask with the high bit set for each byte that is zero.
#[inline(always)]
#[must_use]
pub fn has_zero_byte_u64(x: u64) -> u64 {
    (x.wrapping_sub(0x0101_0101_0101_0101)) & !x & 0x8080_8080_8080_8080
}

/// Returns a mask with the high bit set for each byte that is less than `val`.
#[inline(always)]
#[must_use]
pub fn has_less_than_byte_u64(x: u64, val: u8) -> u64 {
    let broadcast = val as u64 * 0x0101_0101_0101_0101;
    (x.wrapping_sub(broadcast)) & !x & 0x8080_8080_8080_8080
}
