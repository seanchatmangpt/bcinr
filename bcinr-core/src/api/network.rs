//! Compare-Exchange Networks — public API
//!
//! Bitonic and AKS sorting networks
//!
//! This module is 100% generated. Do not edit directly.
//! Modifications will be overwritten on the next `unrdf sync`.
//!
//! This is a thin wrapper layer around `crate::logic::network`.
//! All functions are inline for zero-cost abstraction.

use crate::logic::network as inner;

/// compare_exchange from the Compare-Exchange Networks family
///
/// Generated wrapper around `logic::compare_exchange`
#[inline(always)]
pub fn compare_exchange(a: u32, b: u32) -> (u32, u32) {
    inner::compare_exchange(a, b)
}

/// bitonic_sort_16u32 from the Compare-Exchange Networks family
///
/// Generated wrapper around `logic::bitonic_sort_16u32`
#[inline(always)]
pub fn bitonic_sort_16u32(arr: &mut [u32; 16]) {
    inner::bitonic_sort_16u32(arr)
}

/// bitonic_sort_8u32 from the Compare-Exchange Networks family
///
/// Generated wrapper around `logic::bitonic_sort_8u32`
#[inline(always)]
pub fn bitonic_sort_8u32(arr: &mut [u32; 8]) {
    inner::bitonic_sort_8u32(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_symbols_exist() {
        // Compile-time check: if this test compiles, the API contract is satisfied
    }
}
