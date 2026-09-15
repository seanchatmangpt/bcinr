//! Divan benchmarks for the CMCA allocation kernel -- the crate's first
//! benchmark suite (the repo's bench-auditor had been truthfully reporting
//! zero coverage). Convention matches `bcinr-powl/benches/*`: Divan,
//! `harness = false`, black-boxed inputs.
//!
//! # What is measured, and honestly not
//!
//! - `allocate_kernel`: one full `allocate()` call over the case-studies
//!   fixture (N=8, K=4, Q=4, fully unrolled kernel), including the cheap
//!   `Copy` of the weights array in (state initialization is part of the
//!   real call contract -- `allocate` mutates caller state).
//! - `allocate_single_lens_query`: one stateless single-lens query
//!   (rebuilds its topology tables per call -- that is the API's
//!   documented cost shape).
//! - `escort_exact_integer_q` / `escort_fractional_q` (require `--features
//!   std|alloc`): the escort distribution on the exact repeated-multiply
//!   path vs the approximate log2/exp2 path.
//! - `power_fractional`: the underlying `allocator::power` primitive.
//!
//! Numbers are receipts, not claims: see `docs/BENCH_BASELINE_v26.9.15.md`
//! for the recorded environment and reproduction command.

use bcinr_cmca::allocator::{allocate, allocate_single_lens, power};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
use divan::black_box;

fn main() {
    divan::main();
}

fn flat_parent() -> [i32; N] {
    [-1; N]
}

#[divan::bench]
fn allocate_kernel() {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let parent = flat_parent();
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let result = allocate(
        black_box(&OBJECT_REGISTRY),
        black_box(&LENS_REGISTRY),
        black_box(&LAMBDA),
        black_box(ETA),
        black_box(&parent),
        &mut weights,
        black_box(&payoffs),
        black_box(NonNegativeFixed::ZERO),
        black_box(NonNegativeFixed::ZERO),
        black_box(&mu),
        black_box(&costs),
        black_box(0),
        &mut last_switch_t,
        &mut prev_mode,
        black_box(500),
        black_box(CERTIFICATE_DIGEST),
        black_box(None),
    );
    let _ = black_box(result);
}

#[divan::bench]
fn allocate_single_lens_query() {
    let weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let parent = flat_parent();
    let result = allocate_single_lens(
        black_box(&OBJECT_REGISTRY),
        black_box(&LENS_REGISTRY),
        black_box(0),
        black_box(0),
        black_box(&parent),
        black_box(&weights),
    );
    let _ = black_box(result);
}

#[divan::bench]
fn power_fractional() {
    let out = power(
        black_box(NonNegativeFixed::from_bits(2 << 16)),
        black_box(SignedFixed::from_bits(1 << 15)), // q = 0.5
    );
    black_box(out);
}

// The escort paths live behind the `alloc` feature (Vec-returning API);
// without it these benchmarks compile out rather than silently vanishing
// from a default-feature run -- run `cargo bench --features std` for the
// full suite.
#[cfg(feature = "alloc")]
mod escort {
    use super::*;

    const MASSES: [NonNegativeFixed; 8] = [
        NonNegativeFixed::from_bits(1 << 16),
        NonNegativeFixed::from_bits(2 << 16),
        NonNegativeFixed::from_bits(3 << 16),
        NonNegativeFixed::from_bits(4 << 16),
        NonNegativeFixed::from_bits(5 << 16),
        NonNegativeFixed::from_bits(6 << 16),
        NonNegativeFixed::from_bits(7 << 16),
        NonNegativeFixed::from_bits(8 << 16),
    ];

    #[divan::bench]
    fn escort_exact_integer_q() {
        let out = bcinr_cmca::escort::escort_distribution(
            black_box(&MASSES),
            black_box(SignedFixed::from_num(3i32)),
        );
        let _ = black_box(out);
    }

    #[divan::bench]
    fn escort_fractional_q() {
        let out = bcinr_cmca::escort::escort_distribution(
            black_box(&MASSES),
            black_box(SignedFixed::from_bits(1 << 15)), // q = 0.5
        );
        let _ = black_box(out);
    }
}
