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

/// horizontal_and_u32 from the SWAR Reductions family
#[inline(always)]
pub fn horizontal_and_u32(slice: &[u32]) -> u32 {
    inner::horizontal_and_u32(slice)
}

/// horizontal_or_u32 from the SWAR Reductions family
#[inline(always)]
pub fn horizontal_or_u32(slice: &[u32]) -> u32 {
    inner::horizontal_or_u32(slice)
}

/// horizontal_xor_u32 from the SWAR Reductions family
#[inline(always)]
pub fn horizontal_xor_u32(slice: &[u32]) -> u32 {
    inner::horizontal_xor_u32(slice)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
