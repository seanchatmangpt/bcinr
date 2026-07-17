//! # CMCA: Covariance Monitoring and Calibration Assessment
//!
//! `bcinr-cmca` is an authoritative systems library implementing deterministic,
//! branchless, and allocation-free algorithms for runtime telemetry, calibration,
//! and monitoring of bounded computational systems.
//!
//! Under the strict constraints of the **Radon Law** (Cyclomatic Complexity $CC=1$),
//! this module provides tools to audit the health and stability of mathematical
//! substrates without introducing timing side-channels, dynamic heap allocations,
//! or panic-prone execution paths.
//!
//! ## Architectural Foundations
//!
//! The crate enforces an autonomic feedback loop structured around the standard **MAPE-K**
//! (Monitor, Analyze, Plan, Execute, Shared Knowledge) control loop:
//! 1. **Observe**: Collect telemetry metrics (such as scaling factors, condition bounds, and drift metrics).
//! 2. **Infer**: Compute safety flags branchlessly using fixed-point arithmetic invariants.
//! 3. **Propose**: Report validation states indicating whether to proceed or trigger recertification.
//!
//! ## Key Modules
//!
//! * [`fixed`]: Q16.16 fixed-point arithmetic implementation designed for $CC=1$ operations.
//! * [`allocator`]: Linear bounds-checked and panic-free bump allocators.
//! * [`observatory`]: The evaluation engine that computes calibration safety flags based on mathematical thresholds.
//!
//! ## Example Usage
//!
//! ```rust
//! use bcinr_cmca::fixed::NonNegativeFixed;
//! use bcinr_cmca::observatory::{evaluate_calibration, ObservatoryFlag};
//!
//! // Define typical parameters for telemetry monitoring
//! let kappa_hat = NonNegativeFixed::from_bits(131072);       // Estimated condition number (2.0)
//! let kappa_under = NonNegativeFixed::from_bits(131072);     // Lower bound condition number (2.0)
//! let epsilon_on = NonNegativeFixed::from_bits(65536);       // Upper limit threshold (1.0)
//! let gamma_min_plus_hat = NonNegativeFixed::from_bits(131072); // Estimated positive eigenvalue (2.0)
//! let gamma_min_plus_under = NonNegativeFixed::from_bits(131072); // Lower bound positive eigenvalue (2.0)
//! let epsilon_gram = NonNegativeFixed::from_bits(65536);     // Gram matrix threshold (1.0)
//! let d_js = NonNegativeFixed::ZERO;                         // Divergence/Drift measurement (0.0)
//! let epsilon_drift = NonNegativeFixed::from_bits(65536);    // Maximum allowed drift threshold (1.0)
//! let s_meas = NonNegativeFixed::ONE;                        // Measured scale (1.0)
//! let s_leaf = NonNegativeFixed::from_bits(32768);           // Leaf scale target (0.5)
//!
//! // Evaluate the calibration state branchlessly
//! let status = evaluate_calibration(
//!     kappa_hat,
//!     kappa_under,
//!     epsilon_on,
//!     gamma_min_plus_hat,
//!     gamma_min_plus_under,
//!     epsilon_gram,
//!     d_js,
//!     epsilon_drift,
//!     s_meas,
//!     s_leaf,
//! );
//!
//! // The calibration succeeds, proposing recertification
//! assert_eq!(status, Ok(()));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

pub mod fixed;
pub mod generated;
pub mod allocator;
pub mod observatory;

pub use allocator::StabilityRefusal;

/// A branchless mock/dummy function utilized in tests and contract validation.
///
/// Under release profiles, this function compiles to a single wrapping addition
/// instruction without any conditional branches or memory accesses, conforming to $CC=1$.
///
/// # Examples
///
/// ```rust
/// use bcinr_cmca::dummy_branchless;
///
/// assert_eq!(dummy_branchless(42), 43);
/// assert_eq!(dummy_branchless(u64::MAX), 0);
/// ```
///
/// # Branchless Contract
///
/// ```text
/// u64_contract!(
///     requires: true,
///     ensures: true
/// )
/// ```
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
