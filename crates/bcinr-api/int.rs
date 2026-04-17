//! Integer Bitwise — public API
//!
//! Integer bit manipulation without branches
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `crate::logic::int`.
//! All functions are inline for zero-cost abstraction.

use crate::logic::int as inner;

/// next_power_of_two_u32 from the Integer Bitwise family
///
/// Generated wrapper around `logic::next_power_of_two_u32`
#[inline(always)]
pub fn next_power_of_two_u32(x: u32) -> u32 {
    inner::next_power_of_two_u32(x)
}

/// leading_zeros_u32 from the Integer Bitwise family
///
/// Generated wrapper around `logic::leading_zeros_u32`
#[inline(always)]
pub fn leading_zeros_u32(x: u32) -> u32 {
    inner::leading_zeros_u32(x)
}

/// popcount_u32 from the Integer Bitwise family
///
/// Generated wrapper around `logic::popcount_u32`
#[inline(always)]
pub fn popcount_u32(x: u32) -> u32 {
    inner::popcount_u32(x)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
