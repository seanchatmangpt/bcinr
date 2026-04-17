//! Probabilistic Sketches — public API
//!
//! Hashing and probabilistic data structures
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `bcinr_logic::::sketch`.
//! All functions are inline for zero-cost abstraction.

use bcinr_logic::::sketch as inner;

/// fnv1a_64 from the Probabilistic Sketches family
///
/// Generated wrapper around `logic::fnv1a_64`
#[inline(always)]
pub fn fnv1a_64(data: &[u8]) -> u64 {
    inner::fnv1a_64(data)
}

/// xxhash32 from the Probabilistic Sketches family
///
/// Generated wrapper around `logic::xxhash32`
#[inline(always)]
pub fn xxhash32(data: &[u8], seed: u32) -> u32 {
    inner::xxhash32(data, seed)
}

/// murmur3_32 from the Probabilistic Sketches family
///
/// Generated wrapper around `logic::murmur3_32`
#[inline(always)]
pub fn murmur3_32(data: &[u8], seed: u32) -> u32 {
    inner::murmur3_32(data, seed)
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
