//! Chatman Multifractal Consequence Allocation.
//!
//! The crate exposes two admitted boundaries over private implementation
//! kernels:
//! - [`allocator`]: transactional fixed-size certified control rail;
//! - [`cascade`]: arbitrary-shape analysis rail aligned to the same lens domain.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[path = "allocator_checked.rs"]
pub mod allocator;
#[path = "allocator.rs"]
mod allocator_legacy;

#[cfg(feature = "alloc")]
#[path = "cascade_checked.rs"]
pub mod cascade;
#[cfg(feature = "alloc")]
#[path = "cascade.rs"]
mod cascade_legacy;

pub mod fixed;
pub mod generated;
/// Numeric policy manufactured from the admitted ontology profile.
pub mod generated_profile;
pub mod lrc;
pub mod observatory;
pub mod stability_theorem;

pub use allocator::{check_hierarchy_acyclic, HierarchyRefusal, StabilityRefusal};

/// Minimal branchless primitive retained for contract-gate fixtures.
#[must_use]
pub fn dummy_branchless(value: u64) -> u64 {
    value.wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_wraps() {
        assert_eq!(dummy_branchless(42), 43);
        assert_eq!(dummy_branchless(u64::MAX), 0);
    }
}
