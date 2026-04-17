//! SWAR Reductions — public API
//!
//! SIMD Within A Register: horizontal reductions
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `crate::logic::reduce`.
//! All functions are inline for zero-cost abstraction.

use crate::logic::reduce as inner;

/// horizontal_min_u8x8 from the SWAR Reductions family
///
/// Generated wrapper around `logic::horizontal_min_u8x8`
#[inline(always)]
pub fn horizontal_min_u8x8(arr: &[u8; 8]) -> u8 {
    inner::horizontal_min_u8x8(arr)
}

/// horizontal_max_u8x8 from the SWAR Reductions family
///
/// Generated wrapper around `logic::horizontal_max_u8x8`
#[inline(always)]
pub fn horizontal_max_u8x8(arr: &[u8; 8]) -> u8 {
    inner::horizontal_max_u8x8(arr)
}

/// horizontal_sum_u8x8 from the SWAR Reductions family
///
/// Generated wrapper around `logic::horizontal_sum_u8x8`
#[inline(always)]
pub fn horizontal_sum_u8x8(arr: &[u8; 8]) -> u16 {
    inner::horizontal_sum_u8x8(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
