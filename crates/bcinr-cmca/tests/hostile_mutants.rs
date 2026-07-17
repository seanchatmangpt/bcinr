#![allow(dead_code)]

use bcinr_cmca::fixed::Fixed;
use bcinr_cmca::observatory::ObservatoryFlag;

// M01: Ignore numeric error in underline kappa. Use kappa_hat instead of kappa_under.
pub fn evaluate_m01(
    kappa_hat: Fixed,
    _kappa_under: Fixed,
    epsilon_on: Fixed,
    _gamma_min_plus_hat: Fixed,
    gamma_min_plus_under: Fixed,
    epsilon_gram: Fixed,
    d_js: Fixed,
    epsilon_drift: Fixed,
    s_meas: Fixed,
    s_leaf: Fixed,
) -> Result<(), ObservatoryFlag> {
    bcinr_cmca::observatory::evaluate_calibration(
        kappa_hat,
        kappa_hat, // MUTANT!
        epsilon_on,
        _gamma_min_plus_hat,
        gamma_min_plus_under,
        epsilon_gram,
        d_js,
        epsilon_drift,
        s_meas,
        s_leaf,
    )
}

#[test]
fn kill_m01_ignore_numeric_error() {
    let result = evaluate_m01(
        Fixed::from_bits(66000), // kappa_hat > epsilon_on
        Fixed::from_bits(65000), // kappa_under < epsilon_on
        Fixed::from_bits(65536), // epsilon_on = 1.0
        Fixed::from_bits(65536),
        Fixed::from_bits(65536),
        Fixed::from_bits(65536),
        Fixed::ZERO,
        Fixed::ONE,
        Fixed::ONE,
        Fixed::from_bits(32768),
    );
    assert_ne!(result, Err(ObservatoryFlag::NumericallyUncertain));
}

// M03: Use point-estimate Gram gate without subtracting epsilon_gram.
pub fn evaluate_m03(
    kappa_hat: Fixed,
    kappa_under: Fixed,
    epsilon_on: Fixed,
    gamma_min_plus_hat: Fixed,
    _gamma_min_plus_under: Fixed,
    epsilon_gram: Fixed,
    d_js: Fixed,
    epsilon_drift: Fixed,
    s_meas: Fixed,
    s_leaf: Fixed,
) -> Result<(), ObservatoryFlag> {
    bcinr_cmca::observatory::evaluate_calibration(
        kappa_hat,
        kappa_under,
        epsilon_on,
        gamma_min_plus_hat,
        gamma_min_plus_hat, // MUTANT!
        epsilon_gram,
        d_js,
        epsilon_drift,
        s_meas,
        s_leaf,
    )
}

#[test]
fn kill_m03_point_estimate_gram_gate() {
    let result = evaluate_m03(
        Fixed::from_bits(131072),
        Fixed::from_bits(131072),
        Fixed::from_bits(65536),
        Fixed::from_bits(131072), // gamma_hat > epsilon_gram
        Fixed::from_bits(32768),  // gamma_under < epsilon_gram
        Fixed::from_bits(65536),
        Fixed::ZERO,
        Fixed::ONE,
        Fixed::ONE,
        Fixed::from_bits(32768),
    );
    assert_ne!(result, Err(ObservatoryFlag::GramDegenerate));
}

// M05: Ignore drift.
pub fn evaluate_m05(
    kappa_hat: Fixed,
    kappa_under: Fixed,
    epsilon_on: Fixed,
    gamma_min_plus_hat: Fixed,
    gamma_min_plus_under: Fixed,
    epsilon_gram: Fixed,
    _d_js: Fixed,
    epsilon_drift: Fixed,
    s_meas: Fixed,
    s_leaf: Fixed,
) -> Result<(), ObservatoryFlag> {
    bcinr_cmca::observatory::evaluate_calibration(
        kappa_hat,
        kappa_under,
        epsilon_on,
        gamma_min_plus_hat,
        gamma_min_plus_under,
        epsilon_gram,
        Fixed::ZERO, // MUTANT! Ignores drift by passing 0.
        epsilon_drift,
        s_meas,
        s_leaf,
    )
}

#[test]
fn kill_m05_ignore_drift() {
    let result = evaluate_m05(
        Fixed::from_bits(131072),
        Fixed::from_bits(131072),
        Fixed::from_bits(65536),
        Fixed::from_bits(131072),
        Fixed::from_bits(131072),
        Fixed::from_bits(65536),
        Fixed::from_bits(131072), // d_js > epsilon_drift
        Fixed::from_bits(65536),
        Fixed::ONE,
        Fixed::from_bits(32768),
    );
    assert_ne!(result, Err(ObservatoryFlag::Drifting));
}

// M07: Activate learner based on kappa only, ignoring Gram distinguishability.
pub fn evaluate_m07(
    kappa_hat: Fixed,
    kappa_under: Fixed,
    epsilon_on: Fixed,
    gamma_min_plus_hat: Fixed,
    _gamma_min_plus_under: Fixed,
    epsilon_gram: Fixed,
    d_js: Fixed,
    epsilon_drift: Fixed,
    s_meas: Fixed,
    s_leaf: Fixed,
) -> Result<(), ObservatoryFlag> {
    bcinr_cmca::observatory::evaluate_calibration(
        kappa_hat,
        kappa_under,
        epsilon_on,
        gamma_min_plus_hat,
        Fixed::from_bits(1310720), // MUTANT! Forcing gamma_under to be large, ignoring actual Gram
        epsilon_gram,
        d_js,
        epsilon_drift,
        s_meas,
        s_leaf,
    )
}

#[test]
fn kill_m07_ignore_gram() {
    let result = evaluate_m07(
        Fixed::from_bits(131072),
        Fixed::from_bits(131072),
        Fixed::from_bits(65536),
        Fixed::from_bits(32768),
        Fixed::from_bits(32768), // Both gamma < epsilon_gram
        Fixed::from_bits(65536),
        Fixed::ZERO,
        Fixed::ONE,
        Fixed::ONE,
        Fixed::from_bits(32768),
    );
    assert_ne!(result, Err(ObservatoryFlag::GramDegenerate));
}

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt,
    EnvelopeReceipt, OutcomeReceipt, CertifiedLearning
};
use bcinr_cmca::generated::case_studies::{
    OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, N, Q
};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

fn get_proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
    AdaptiveUpdate::new(
        AdmittedControlState,
        CertificateReceipt,
        EnvelopeReceipt,
        OutcomeReceipt,
        Fixed::ZERO,
        Fixed::ONE,
    )
}

fn run_alloc_baseline() -> [Fixed; N] {
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];
    
    allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        Fixed::ZERO,
        Fixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    ).unwrap()
}

fn run_alloc_tree() -> [Fixed; N] {
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    let mut parent = [-1; N];
    parent[1] = 0;
    parent[2] = 0;

    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];
    
    allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        Fixed::ZERO,
        Fixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    ).unwrap()
}

fn run_alloc_mu_cost() -> [Fixed; N] {
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    
    // Set mu negative so clipping to zero differs from unclipped
    let mu = [Fixed(0u32.wrapping_sub(327680)); N];
    let costs = [Fixed::ONE; N];
    
    allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        Fixed::ZERO,
        Fixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        None, // degrade_to_certified_selection = true, freezes learning but succeeds!
    ).unwrap()
}

const CORRECT_BASELINE: [u32; N] = [8349, 7741, 6684, 6684, 6684, 6684, 7973, 14733];
const CORRECT_TREE: [u32; N] = [0, 9391, 6623, 8066, 8066, 8066, 9275, 16043];
const CORRECT_MU_COST: [u32; N] = [4096, 4096, 4096, 4096, 4096, 4096, 4096, 4096];

#[cfg(feature = "mutant_1")]
#[test]
fn kill_mutant_1_single_measure_collapse() {
    let result_mutant = run_alloc_baseline().map(|x| x.0);
    assert_ne!(result_mutant, CORRECT_BASELINE, "Mutant 1 should deviate from correct baseline");
}

#[cfg(feature = "mutant_2")]
#[test]
fn kill_mutant_2_q_sign_inversion() {
    let result_mutant = run_alloc_baseline().map(|x| x.0);
    assert_ne!(result_mutant, CORRECT_BASELINE, "Mutant 2 should deviate from correct baseline");
}

#[cfg(feature = "mutant_3")]
#[test]
fn kill_mutant_3_broken_normalization() {
    let result_mutant = run_alloc_tree().map(|x| x.0);
    assert_ne!(result_mutant, CORRECT_TREE, "Mutant 3 should deviate from correct tree baseline");
}

#[cfg(feature = "mutant_4")]
#[test]
fn kill_mutant_4_rdf_identity_skew() {
    let result_mutant = run_alloc_baseline().map(|x| x.0);
    assert_ne!(result_mutant, CORRECT_BASELINE, "Mutant 4 should deviate from correct baseline");
}

#[cfg(feature = "mutant_5")]
#[test]
fn kill_mutant_5_consequence_truncation() {
    let result_mutant = run_alloc_mu_cost().map(|x| x.0);
    assert_ne!(result_mutant, CORRECT_MU_COST, "Mutant 5 should deviate from correct mu_cost baseline");
}

#[cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]
#[test]
fn verify_correctness_baselines() {
    assert_eq!(run_alloc_baseline().map(|x| x.0), CORRECT_BASELINE);
    assert_eq!(run_alloc_tree().map(|x| x.0), CORRECT_TREE);
    assert_eq!(run_alloc_mu_cost().map(|x| x.0), CORRECT_MU_COST);
}
