//! Byte/Word Scanning — public API
//!
//! Scanning and searching within byte/word sequences
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `bcinr_logic::::scan`.
//! All functions are inline for zero-cost abstraction.

use bcinr_logic::::scan as inner;

/// find_byte from the Byte/Word Scanning family
///
/// Find the first occurrence of a target byte in a slice.
///
/// Generated wrapper around `logic::find_byte`
#[inline(always)]
pub fn find_byte(bytes: &[u8], target: u8) -> Option<usize> {
    inner::find_byte(bytes, target)
}

/// count_zero_bytes from the Byte/Word Scanning family
///
/// Count the number of zero bytes in a slice.
///
/// Generated wrapper around `logic::count_zero_bytes`
#[inline(always)]
pub fn count_zero_bytes(bytes: &[u8]) -> usize {
    inner::count_zero_bytes(bytes)
}

/// first_zero_byte from the Byte/Word Scanning family
///
/// Find the index of the first zero byte in a slice.
///
/// Generated wrapper around `logic::first_zero_byte`
#[inline(always)]
pub fn first_zero_byte(bytes: &[u8]) -> Option<usize> {
    inner::first_zero_byte(bytes)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
