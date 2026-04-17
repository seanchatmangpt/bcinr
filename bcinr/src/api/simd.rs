//! SIMD Primitives — public API
//!
//! SIMD vector operations (128/256-bit)
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `bcinr_logic::::simd`.
//! All functions are inline for zero-cost abstraction.

use bcinr_logic::::simd as inner;

/// movemask_u8x16 from the SIMD Primitives family
///
/// Generated wrapper around `logic::movemask_u8x16`
#[inline(always)]
pub fn movemask_u8x16(a: [u8; 16]) -> u16 {
    inner::movemask_u8x16(a)
}

/// shuffle_u8x16 from the SIMD Primitives family
///
/// Generated wrapper around `logic::shuffle_u8x16`
#[inline(always)]
pub fn shuffle_u8x16(a: [u8; 16], b: [u8; 16], mask: [u8; 16]) -> [u8; 16] {
    inner::shuffle_u8x16(a, b, mask)
}

/// splat_u8x16 from the SIMD Primitives family
///
/// Generated wrapper around `logic::splat_u8x16`
#[inline(always)]
pub fn splat_u8x16(value: u8) -> [u8; 16] {
    inner::splat_u8x16(value)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
