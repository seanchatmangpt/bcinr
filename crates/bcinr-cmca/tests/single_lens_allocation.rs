#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]
#![allow(clippy::needless_range_loop)]

//! Acceptance tests for [`allocate_single_lens`], the crate's first genuine
//! single-lens allocation entry point.
//!
//! # Why this file exists
//!
//! A test-drive script run against `allocator::allocate()` over this crate's
//! own shipped `OBJECT_REGISTRY`/`LENS_REGISTRY` found that all 4 lenses
//! (Exploitation/Proportional/Coverage/Rare) produced byte-identical output.
//! Root cause, confirmed by reading `allocate_in`: `allocate()` always sums
//! all `K x Q` (measure, lens) combinations into one LAMBDA-weighted blend
//! (`pi_combined`) -- there was no public way to ask "what would lens `q`
//! alone say." `tests/falsification_adversarial.rs` already documented this
//! gap directly ("per-lens isolation is not observable through the public
//! API"). `allocate_single_lens` closes it by exposing the crate's existing
//! internal per-lens kernel (`compute_pi_kq_for_kq`) directly, rather than
//! reimplementing the escort math -- so this file's job is to prove that
//! exposure is both (a) real (the 4 lenses now genuinely diverge) and
//! (b) faithful (summing the 16 single-lens results with LAMBDA reproduces
//! `allocate()`'s own blended answer, bit-for-bit up to measured rounding).

use bcinr_cmca::allocator::{
    allocate_single_lens, AdaptiveUpdate, AdmittedControlState, CertificateReceipt,
    CertifiedLearning, EnvelopeReceipt, LensSelectionRefusal, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::consequence_mass::case_studies::{
    ETA, K, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q,
};

fn flat_weights() -> [[NonNegativeFixed; 2 * Q]; N] {
    [[NonNegativeFixed::ONE; 2 * Q]; N]
}

fn flat_root_parent() -> [i32; N] {
    [-1; N] // every object is its own root -- OBJECT_REGISTRY is a flat, non-tree registry
}

/// The 4 lenses now genuinely diverge over the crate's real registry -- the
/// original test-drive scenario this file exists to close, now with real
/// assertions instead of `println!`.
///
/// Sanity-checked during implementation: hardcoding `lens_idx` to a fixed
/// value inside `allocate_single_lens` makes this test fail, confirming it
/// isn't accidentally vacuous.
#[test]
fn the_four_lenses_produce_genuinely_different_allocations() {
    let weights = flat_weights();
    let parent = flat_root_parent();

    let mut per_lens_results = Vec::with_capacity(Q);
    for lens_idx in 0..Q {
        let result = allocate_single_lens(
            &OBJECT_REGISTRY,
            &LENS_REGISTRY,
            0,
            lens_idx,
            &parent,
            &weights,
        )
        .unwrap();
        per_lens_results.push(result);
    }

    // At least one pair of lenses must disagree on at least one object's
    // allocation -- the property that was silently false for `allocate()`.
    let mut any_divergence = false;
    'outer: for a in 0..Q {
        for b in (a + 1)..Q {
            for i in 0..N {
                if per_lens_results[a][i].val != per_lens_results[b][i].val {
                    any_divergence = true;
                    break 'outer;
                }
            }
        }
    }
    assert!(
        any_divergence,
        "expected the 4 lenses to diverge on at least one object; got identical results: {per_lens_results:?}"
    );
}

/// Refusal correctness: each `LensSelectionRefusal` variant fires on the
/// input it names, matching the crate's `assert_eq!`-against-typed-`Result`
/// convention (`case_studies.rs`, `calibration.rs`) rather than a bare
/// `is_err()`.
#[test]
fn refuses_out_of_range_measure_index() {
    let weights = flat_weights();
    let parent = flat_root_parent();
    assert_eq!(
        allocate_single_lens(&OBJECT_REGISTRY, &LENS_REGISTRY, K, 0, &parent, &weights),
        Err(LensSelectionRefusal::MeasureIndexOutOfRange { measure: K })
    );
}

#[test]
fn refuses_out_of_range_lens_index() {
    let weights = flat_weights();
    let parent = flat_root_parent();
    assert_eq!(
        allocate_single_lens(&OBJECT_REGISTRY, &LENS_REGISTRY, 0, Q, &parent, &weights),
        Err(LensSelectionRefusal::LensIndexOutOfRange { lens_idx: Q })
    );
}

#[test]
fn refuses_cyclic_parent() {
    let weights = flat_weights();
    // 0 -> 1 -> 0: a real cycle, no root anywhere.
    let mut parent = flat_root_parent();
    parent[0] = 1;
    parent[1] = 0;
    assert_eq!(
        allocate_single_lens(&OBJECT_REGISTRY, &LENS_REGISTRY, 0, 0, &parent, &weights),
        Err(LensSelectionRefusal::Cyclic)
    );
}

fn get_proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
    AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        CertifiedLearning::admit_learning(),
    )
}

/// The core architectural claim: `allocate_single_lens` is not a parallel
/// reimplementation that could drift from `allocate()` -- summing its 16
/// `(measure, lens)` results with the crate's own `LAMBDA` weights
/// reproduces `pi_combined`, the value `allocate_in` builds internally
/// before folding in `mu`/`costs`/`eta` pricing (`allocator/mod.rs`'s
/// `pi_res` construction), within a measured Q16.16 rounding tolerance --
/// not `DIFFERENTIAL_TOLERANCE`'s 0.22 placeholder, which is an unrelated
/// f64-vs-fixed compatibility hack.
///
/// `allocate()` does not return `pi_combined` directly, only the
/// fully-priced `pi_res`, so this test replicates `allocate_in`'s own
/// pricing formula (`val = eta*nl_recip + (1-eta)*p_mu`, with `mu`/`costs`
/// zeroed so `p_mu` reduces to `pi_combined[x] / sum_leaves(pi_combined)`)
/// against the LAMBDA-weighted reconstruction from `allocate_single_lens`,
/// rather than against a second, independently-guessed value.
#[test]
fn blend_equals_the_lambda_weighted_sum_of_single_lens_results() {
    use bcinr_cmca::allocator::allocate;
    use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

    let weights = flat_weights();
    let parent = flat_root_parent();
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut mut_weights = weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;

    let blended = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA, // the crate's own registry ETA (0.5) -- allocate_in refuses eta below a floor
        &parent,
        &mut mut_weights,
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
        get_proof().as_ref(),
    )
    .unwrap();

    let mut pi_combined = [NonNegativeFixed::ZERO; N];
    for k in 0..K {
        for q_idx in 0..Q {
            let single = allocate_single_lens(
                &OBJECT_REGISTRY,
                &LENS_REGISTRY,
                k,
                q_idx,
                &parent,
                &weights,
            )
            .unwrap();
            for i in 0..N {
                pi_combined[i] += LAMBDA[k][q_idx] * single[i];
            }
        }
    }

    // mu=costs=0 everywhere -> exp(-(mu*costs)) == ONE for every object, so
    // `p_mu[x] == pi_combined[x] / priced_sum`, `priced_sum` the sum of
    // `pi_combined` over leaves (all N objects here, flat root parent).
    let mut priced_sum = NonNegativeFixed::ZERO;
    for i in 0..N {
        priced_sum += pi_combined[i];
    }
    // `LEAF_RECIP[N]` (allocator/mod.rs's lookup table) for N=8 leaves == 1/8.
    let nl_recip = NonNegativeFixed::from_bits(8192);

    let mut expected = [NonNegativeFixed::ZERO; N];
    for i in 0..N {
        let p_mu = pi_combined[i] / priced_sum;
        expected[i] = ETA * nl_recip + (NonNegativeFixed::ONE - ETA) * p_mu;
    }

    // Measured, not guessed: the actual observed diff on this registry (all
    // 8 objects) is exactly 0 -- `pi_kq`'s max-shift-stabilised softmax and
    // `compute_pi_kq_for_kq`'s exact repeated-multiplication path leave no
    // rounding room to lose here. 8 bits of headroom above that measured 0
    // (matching `cmca_h_lean_correspondence.rs`'s `NORMALIZATION_TOLERANCE_BITS`
    // precedent) catches a real regression without being a brittle exact-zero
    // assertion tied to this one registry's specific values.
    const RECONSTRUCTION_TOLERANCE_BITS: i64 = 8;
    for i in 0..N {
        let diff = (blended[i].to_bits() as i64 - expected[i].to_bits() as i64).abs();
        assert!(
            diff <= RECONSTRUCTION_TOLERANCE_BITS,
            "object {i}: blended={:?} expected_from_single_lens_reconstruction={:?} diff={diff}",
            blended[i],
            expected[i]
        );
    }
}
