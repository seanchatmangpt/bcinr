//! CMCA-113 regression test: `stability_profile.rs`'s load-bearing constants
//! (`GAIN_MATRIX`, `WEIGHT_VECTOR`, `CONTRACTION_MARGIN`, `CERTIFICATE_DIGEST`,
//! `MODE_DWELL_ROUNDS_MIN`) were hardcoded with no derivation and no test that
//! could tell a "reasonable-looking but wrong" production-tuning edit from a
//! correct one (FMEA.md, CMCA-113, Detection=9).
//!
//! This test does not invent a formula for constants that never had one
//! (`minimum_dwell_rounds`, the "ARBITRARY" bounds -- see the doc comments on
//! `StabilityProfile` in `src/generated/stability_profile.rs`). It instead
//! locks down the one real, checkable mathematical property `allocate_in`
//! (`src/allocator/mod.rs`) actually requires of `gain_matrix` /
//! `weight_vector` / `deterministic_margin` -- the diagonal-dominance /
//! contraction inequality documented on that struct -- against the live
//! `PROFILE` constants, using the exact same 1e9-scaled fixed-point integer
//! arithmetic `allocate_in` uses (not a floating-point restatement), so this
//! test fails the moment an edit to any of those three constants breaks the
//! property `allocate_in` silently assumes holds.
//!
//! It also locks down `CERTIFICATE_DIGEST`'s actual (documented, not
//! aspirational) behavior: a byte-for-byte match against itself succeeds and
//! a single-byte edit is caught by `allocate_in`'s digest check, so a future
//! reader has a real, running example of what this constant does and does
//! not protect against (see the doc comment on `certificate_digest` for why
//! that is a same-crate self-check, not caller-independent authorization).

#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};
use bcinr_cmca::generated::stability_profile::{
    CERTIFICATE_DIGEST, CONTRACTION_MARGIN, GAIN_MATRIX, MODE_DWELL_ROUNDS_MIN, WEIGHT_VECTOR,
};

/// Mirrors `allocate_in`'s `gd_ok` computation exactly (mod.rs ~1732-1745):
/// for every row `i`, `sum_j gain_matrix[i][j] * weight_vector[j] / 1e9 <=
/// weight_vector[i] - deterministic_margin * weight_vector[i] / 1e9`.
#[test]
fn gain_matrix_weight_vector_margin_satisfy_the_contraction_inequality() {
    for i in 0..5 {
        let mut sum_g_d: u128 = 0;
        for j in 0..5 {
            let g_raw = GAIN_MATRIX[i][j].raw as u128;
            let d_raw = WEIGHT_VECTOR[j].raw as u128;
            sum_g_d += g_raw * d_raw;
        }
        let lhs = sum_g_d / 1_000_000_000;

        let d_i_raw = WEIGHT_VECTOR[i].raw as u128;
        let delta_raw = CONTRACTION_MARGIN.raw as u128;
        let rhs = d_i_raw - (delta_raw * d_i_raw / 1_000_000_000);

        assert!(
            lhs <= rhs,
            "row {i}: contraction inequality violated (lhs={lhs}, rhs={rhs}) -- \
             an edit to GAIN_MATRIX, WEIGHT_VECTOR, or CONTRACTION_MARGIN broke \
             the invariant allocate_in assumes holds; see stability_profile.rs's \
             module doc comment for the formula"
        );
    }
}

/// `minimum_dwell_rounds` has no formula on record (see its doc comment),
/// but it does have one uncontroversial sanity property: it must be a
/// positive round count, or `MODE_DWELL_ROUNDS_MIN` stops meaningfully
/// gating anything (`tau_d < 0` is impossible for a `u32`, so a `0` value
/// would make `dwell_err` permanently `false`).
#[test]
fn mode_dwell_rounds_min_is_a_positive_gate() {
    assert!(
        MODE_DWELL_ROUNDS_MIN > 0,
        "MODE_DWELL_ROUNDS_MIN must stay positive to gate anything"
    );
}

fn sample_allocate_args_ok(
    digest: [u8; 32],
) -> Result<(), bcinr_cmca::allocator::StabilityRefusal> {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;
    let parent = [-1i32; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let proof = AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        CertifiedLearning::admit_learning(),
    );

    allocate(
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
        digest,
        proof.as_ref(),
    )
    .map(|_| ())
}

#[test]
fn certificate_digest_round_trip_matches_and_a_single_flipped_byte_is_caught() {
    // The documented (non-security) behavior: passing the constant straight
    // back in matches.
    assert!(
        sample_allocate_args_ok(CERTIFICATE_DIGEST).is_ok(),
        "allocate_in must accept the live CERTIFICATE_DIGEST constant round-tripped \
         back to it -- if this fails, either the digest check or the constant itself \
         has drifted from what every caller (including this crate's own doctest) relies on"
    );

    // A single flipped byte is still caught by the equality check -- this is
    // exactly the self-consistency property the doc comment on
    // `certificate_digest` describes, demonstrated as a running assertion
    // rather than only claimed in prose.
    let mut tampered = CERTIFICATE_DIGEST;
    tampered[0] ^= 0x01;
    assert!(
        sample_allocate_args_ok(tampered).is_err(),
        "a single-byte-flipped digest must be refused by allocate_in's digest check"
    );
}
