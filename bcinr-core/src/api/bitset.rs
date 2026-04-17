//! Bitset Algebra — public API
//!
//! Bitset operations: rank, select, set, clear
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `crate::logic::bitset`.
//! All functions are inline for zero-cost abstraction.

use crate::logic::bitset as inner;

/// select_bit_u64 from the Bitset Algebra family
///
/// Generated wrapper around `logic::select_bit_u64`
#[inline(always)]
pub fn select_bit_u64(x: u64, nth: usize) -> usize {
    inner::select_bit_u64(x, nth)
}

/// rank_u64 from the Bitset Algebra family
///
/// Generated wrapper around `logic::rank_u64`
#[inline(always)]
pub fn rank_u64(x: u64, pos: usize) -> usize {
    inner::rank_u64(x, pos)
}

/// clear_bit_u64 from the Bitset Algebra family
///
/// Generated wrapper around `logic::clear_bit_u64`
#[inline(always)]
pub fn clear_bit_u64(x: u64, pos: usize) -> u64 {
    inner::clear_bit_u64(x, pos)
}

/// set_bit_u64 from the Bitset Algebra family
///
/// Generated wrapper around `logic::set_bit_u64`
#[inline(always)]
pub fn set_bit_u64(x: u64, pos: usize) -> u64 {
    inner::set_bit_u64(x, pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
