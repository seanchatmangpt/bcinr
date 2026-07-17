#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed, CanonicalMask};
use bcinr_cmca::observatory::{evaluate_calibration, ObservatoryFlag};

// Core Calibration Fixtures
#[test]
fn test_f02_numerically_uncertain() {
    let result = evaluate_calibration(
        NonNegativeFixed::from_bits(66000), // kappa_hat > epsilon_on
        NonNegativeFixed::from_bits(65000), // kappa_under < epsilon_on
        NonNegativeFixed::from_bits(65536), // epsilon_on = 1.0
        NonNegativeFixed::from_bits(65536),
        NonNegativeFixed::from_bits(65536),
        NonNegativeFixed::from_bits(65536),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_bits(32768),
    );
    assert_eq!(result, Err(ObservatoryFlag::NumericallyUncertain));
}

#[test]
fn test_f03_gram_degenerate() {
    let result = evaluate_calibration(
        NonNegativeFixed::from_bits(131072), // kappa_hat > epsilon_on
        NonNegativeFixed::from_bits(131072), // kappa_under > epsilon_on
        NonNegativeFixed::from_bits(65536),  // epsilon_on = 1.0
        NonNegativeFixed::from_bits(32768),  // gamma_min_plus_hat
        NonNegativeFixed::from_bits(32768),  // gamma_min_plus_under < epsilon_gram
        NonNegativeFixed::from_bits(65536),  // epsilon_gram = 1.0
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_bits(32768),
    );
    assert_eq!(result, Err(ObservatoryFlag::GramDegenerate));
}

#[test]
fn test_f09_nonstationary_window() {
    let result = evaluate_calibration(
        NonNegativeFixed::from_bits(131072), // kappa_hat
        NonNegativeFixed::from_bits(131072), // kappa_under
        NonNegativeFixed::from_bits(65536),  // epsilon_on
        NonNegativeFixed::from_bits(131072), // gamma_min_plus_hat
        NonNegativeFixed::from_bits(131072), // gamma_min_plus_under
        NonNegativeFixed::from_bits(65536),  // epsilon_gram
        NonNegativeFixed::from_bits(131072), // d_js > epsilon_drift
        NonNegativeFixed::from_bits(65536),  // epsilon_drift
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_bits(32768),
    );
    assert_eq!(result, Err(ObservatoryFlag::Drifting));
}

#[test]
fn test_f00_exact_scale_collapse() {
    let result = evaluate_calibration(
        NonNegativeFixed::ZERO, // kappa_hat
        NonNegativeFixed::ZERO, // kappa_under
        NonNegativeFixed::from_bits(65536),  // epsilon_on
        NonNegativeFixed::from_bits(131072), // gamma_min_plus_hat
        NonNegativeFixed::from_bits(131072), // gamma_min_plus_under
        NonNegativeFixed::from_bits(65536),  // epsilon_gram
        NonNegativeFixed::ZERO, // d_js
        NonNegativeFixed::from_bits(65536),  // epsilon_drift
        NonNegativeFixed::ONE,  // s_meas
        NonNegativeFixed::ONE,  // s_leaf == s_meas
    );
    assert_eq!(result, Err(ObservatoryFlag::ScaleInert));
}

#[test]
fn test_f01_material_scale_information() {
    let result = evaluate_calibration(
        NonNegativeFixed::from_bits(131072), // kappa_hat
        NonNegativeFixed::from_bits(131072), // kappa_under
        NonNegativeFixed::from_bits(65536),  // epsilon_on
        NonNegativeFixed::from_bits(131072), // gamma_min_plus_hat
        NonNegativeFixed::from_bits(131072), // gamma_min_plus_under
        NonNegativeFixed::from_bits(65536),  // epsilon_gram
        NonNegativeFixed::ZERO, // d_js
        NonNegativeFixed::from_bits(65536),  // epsilon_drift
        NonNegativeFixed::ONE,  // s_meas
        NonNegativeFixed::from_bits(32768),  // s_leaf
    );
    assert_eq!(result, Ok(())); // RECERTIFICATION_CANDIDATE
}
