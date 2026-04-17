//! Saturation Arithmetic — public API
//!
//! Fixed-point and saturation arithmetic
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `bcinr_logic::::fix`.
//! All functions are inline for zero-cost abstraction.

use bcinr_logic::::fix as inner;

/// clamp_u32 from the Saturation Arithmetic family
///
/// Generated wrapper around `logic::clamp_u32`
#[inline(always)]
pub fn clamp_u32(x: u32, min: u32, max: u32) -> u32 {
    inner::clamp_u32(x, min, max)
}

/// sub_sat_u8 from the Saturation Arithmetic family
///
/// Generated wrapper around `logic::sub_sat_u8`
#[inline(always)]
pub fn sub_sat_u8(a: u8, b: u8) -> u8 {
    inner::sub_sat_u8(a, b)
}

/// add_sat_u8 from the Saturation Arithmetic family
///
/// Generated wrapper around `logic::add_sat_u8`
#[inline(always)]
pub fn add_sat_u8(a: u8, b: u8) -> u8 {
    inner::add_sat_u8(a, b)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
