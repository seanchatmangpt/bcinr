//! CMCA-119: a runnable, end-to-end example for a first-time integrator.
//!
//! Run with:
//!
//! ```text
//! cargo run --example basic_allocation -p bcinr-cmca --features std
//! ```
//!
//! ## Which entry point does this example use, and why
//!
//! This example calls [`allocate`], the crate's top-level, LAMBDA-weighted
//! allocation entry point (see `bcinr_cmca`'s crate-level "Which entry point
//! do I want?" doc section for how it relates to `allocate_in`,
//! `allocate_single_lens`, `cascade::consequence_mass`, and
//! `escort::escort_distribution`). `allocate` is the right starting point
//! for a new integrator who just wants "one allocation vector across the
//! whole object registry" without picking a feasible region or isolating a
//! single lens.
//!
//! The registry below reuses this crate's own compiled-in `N = 8`, `K = 4`,
//! `Q = 4` shape from `generated::consequence_mass::case_studies` -- see
//! `allocate`'s own doc comment (and CMCA-108) for why that shape is fixed
//! rather than caller-generic.
//!
//! ## Authority boundary
//!
//! This first-integrator example deliberately uses `proof = None`. The
//! adaptive-learning authority chain is withheld pending CMCA-102/CMCA-114
//! Hoare-logic verification, so an example must not manufacture a trivial
//! `AdaptiveUpdate` merely to make the allocator run. `None` exercises the
//! allocator's certified-selection/non-adaptive path without widening that
//! authority boundary.

use bcinr_cmca::allocator::{allocate, StabilityRefusal};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

fn main() {
    // A flat forest: every object is its own root (no hierarchy).
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;

    let result: Result<[NonNegativeFixed; N], StabilityRefusal> = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        None,
    );

    match result {
        Ok(shares) => {
            println!("allocate() succeeded -- per-object allocation shares:");
            for (i, share) in shares.iter().enumerate() {
                println!("  object[{i}] = {share:?}");
            }
        }
        Err(refusal) => {
            // `StabilityRefusal` implements `Display` (CMCA-119), so a
            // downstream caller can propagate it with `?` under `std` via
            // `Box<dyn std::error::Error>`, or match on the typed variant
            // directly when finer handling is needed.
            println!("allocate() refused: {refusal} ({refusal:?})");
        }
    }
}
