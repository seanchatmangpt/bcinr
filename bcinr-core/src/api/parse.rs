//! Parsing Primitives — public API
//!
//! Number parsing and format recognition
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `crate::logic::parse`.
//! All functions are inline for zero-cost abstraction.

use crate::logic::parse as inner;

/// skip_whitespace from the Parsing Primitives family
///
/// Generated wrapper around `logic::skip_whitespace`
#[inline(always)]
pub fn skip_whitespace(bytes: &[u8]) -> usize {
    inner::skip_whitespace(bytes)
}

/// parse_hex_u32 from the Parsing Primitives family
///
/// Generated wrapper around `logic::parse_hex_u32`
#[inline(always)]
pub fn parse_hex_u32(bytes: &[u8]) -> Result<u32, &'static str> {
    inner::parse_hex_u32(bytes)
}

/// parse_decimal_u64 from the Parsing Primitives family
///
/// Generated wrapper around `logic::parse_decimal_u64`
#[inline(always)]
pub fn parse_decimal_u64(bytes: &[u8]) -> Result<u64, &'static str> {
    inner::parse_decimal_u64(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
