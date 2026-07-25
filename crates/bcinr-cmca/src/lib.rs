#![allow(clippy::cargo_common_metadata)]
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
//! let kappa_hat = NonNegativeFixed::from_value_bits(131072);       // Estimated condition number (2.0)
//! let kappa_under = NonNegativeFixed::from_value_bits(131072);     // Lower bound condition number (2.0)
//! let epsilon_on = NonNegativeFixed::from_value_bits(65536);       // Upper limit threshold (1.0)
//! let gamma_min_plus_hat = NonNegativeFixed::from_value_bits(131072); // Estimated positive eigenvalue (2.0)
//! let gamma_min_plus_under = NonNegativeFixed::from_value_bits(131072); // Lower bound positive eigenvalue (2.0)
//! let epsilon_gram = NonNegativeFixed::from_value_bits(65536);     // Gram matrix threshold (1.0)
//! let d_js = NonNegativeFixed::ZERO;                         // Divergence/Drift measurement (0.0)
//! let epsilon_drift = NonNegativeFixed::from_value_bits(65536);    // Maximum allowed drift threshold (1.0)
//! let s_meas = NonNegativeFixed::ONE;                        // Measured scale (1.0)
//! let s_leaf = NonNegativeFixed::from_value_bits(32768);           // Leaf scale target (0.5)
//! let round_identity: u64 = 7;                               // Identity of this evaluation round
//!
//! // Provide the artifact
//! use bcinr_cmca::observatory::{MeasurementArtifact, SupportStanding, ModeDelta};
//! use bcinr_cmca::allocator::CertificateReceipt;
//!
//! let artifact = MeasurementArtifact {
//!     point_estimate: kappa_hat,
//!     lower_bound: kappa_under,
//!     upper_bound: kappa_hat,
//!     support_standing: SupportStanding { is_supported: true, smoothing_applied: false },
//!     effective_sample_size: NonNegativeFixed::ONE,
//!     dependence_standing: 0,
//!     numeric_error: NonNegativeFixed::ZERO,
//!     drift: d_js,
//!     gram_lower_bound: gamma_min_plus_under,
//!     graph_digest: 0,
//!     control_mode_digest: 42,
//!     proposal: ModeDelta::ProposeDelta,
//! };
//!
//! // Evaluate the calibration state branchlessly
//! let outcome = evaluate_calibration(
//!     &artifact,
//!     epsilon_on,
//!     epsilon_gram,
//!     epsilon_drift,
//!     s_meas,
//!     s_leaf,
//!     round_identity,
//! );
//!
//! // The calibration succeeds, proposing recertification.
//! //
//! // This doctest demonstrates baseline (non-mutant) behavior. Under the `mutant_11`
//! // hostile-mutation feature (which inverts `gamma_under_off`'s comparison direction
//! // in `evaluate_calibration`, src/observatory.rs), this fixture's `RecertificationCandidate`
//! // condition is deliberately erased by that mutant's own dedicated oracle
//! // (`kill_mutant_11_false_gram_degenerate` in tests/hostile_mutants.rs) — so the
//! // baseline assertion below is skipped, at doctest run time via `cfg!`, specifically
//! // under that feature rather than weakened for the default build.
//! if !cfg!(feature = "mutant_11") {
//!     assert!(outcome.flags.contains(ObservatoryFlag::RecertificationCandidate));
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "std")]
extern crate std;

pub mod alloc_counter;
pub mod allocator;
pub mod certification;
pub mod fixed;
pub mod generated;
pub mod jump;
pub mod lrc;
pub mod mode_switch;
pub mod observatory;
pub mod proposal;
pub mod shadow;
pub mod stability;
// Test-time-only Gamma_CMCA artifact verification (VerifyGeneratedProfile).
// Gated to #[cfg(test)] because its dependencies (blake3, serde, serde_json)
// are dev-dependencies only, keeping non-test builds of this crate free of
// any additional runtime dependency beyond bcinr-logic. See src/artifact.rs
// module docs for the full rationale.
#[cfg(test)]
pub mod artifact;

// mfw-producer-sourced artifact modules (fresh Gamma_CMCA output, generator
// v2.0.0-mfw), reachable as `bcinr_cmca::generated_artifact::case_studies` /
// `bcinr_cmca::generated_artifact::generalization`. This is now the live,
// unconditional (default-features) path that `allocator.rs`/`observatory.rs`
// and the test suite import from.
//
// RECONCILIATION HISTORY (2026-07-20/21): earlier revisions of this comment
// claimed compiling this module unconditionally produced 611 errors (a
// `SignedFixed::from_bits`/private-field mismatch against `src/fixed.rs`'s
// real API). That claim was stale even at the time it was checked in: a
// clean-rebuild (`cargo clean -p bcinr-cmca && cargo build -p bcinr-cmca
// --features generated_artifact_pending`) and a grep of both
// `generated-artifact/*/cmca_generated.rs` files for `from_bits(` found zero
// matches -- every call site already used `NonNegativeFixed::from_value_bits`,
// which `src/fixed.rs` has always exposed. There was no real API mismatch to
// fix; the module built cleanly (0 errors, 0 warnings) before this
// reconciliation, gated only behind the now-removed `generated_artifact_pending`
// feature. That feature flag has been removed from `Cargo.toml` and this
// module is unconditionally compiled.
//
// NOTE ON PLACEMENT: this module is declared here rather than folded into
// `src/generated/mod.rs` (which still declares the superseded
// `crate::generated::case_studies` / `crate::generated::generalization`,
// alongside the still-live `crate::generated::stability_profile`) because
// `src/generated/` carries a repo-level invariant (`cmca/rdf-generation.md`)
// that only the generator produces files there, enforced for interactive
// sessions by `scripts/gates/block-generated-edit.sh`. Consolidating the two
// module trees is a separate, larger structural change than this
// reconciliation pass; `crate::generated::case_studies` is dead code (no
// production or test consumer imports it after this pass -- see its own
// header comment) but left in place rather than deleted, since deleting it
// touches the same gate-protected directory. See
// `src/generated_artifact/mod.rs`'s module doc for the old-symbol ->
// new-symbol correspondence table this reconciliation verified.
pub mod generated_artifact;

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
