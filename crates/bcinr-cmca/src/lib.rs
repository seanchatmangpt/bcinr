#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

pub mod fixed;
pub mod generated;
pub mod allocator;
pub mod observatory;

pub use allocator::StabilityRefusal;

/// Branchless Contract
/// u64_contract!(
///     requires: true,
///     ensures: true
/// )
///
/// A simple dummy branchless function.
pub fn dummy_branchless(val: u64) -> u64 {
    val.wrapping_add(1)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_branchless() {
        assert_eq!(dummy_branchless(0), 1);
        assert_eq!(dummy_branchless(42), 43);
        assert_eq!(dummy_branchless(u64::MAX), 0);
    }
}
