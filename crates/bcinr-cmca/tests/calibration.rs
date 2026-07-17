#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

use bcinr_cmca::fixed::Fixed;
use bcinr_cmca::observatory::{evaluate_calibration, ObservatoryFlag};

// Core Calibration Fixtures
#[test]
fn test_f02_numerically_uncertain() {
    let result = evaluate_calibration(
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
    assert_eq!(result, Err(ObservatoryFlag::NumericallyUncertain));
}

#[test]
fn test_f03_gram_degenerate() {
    let result = evaluate_calibration(
        Fixed::from_bits(131072), // kappa_hat > epsilon_on
        Fixed::from_bits(131072), // kappa_under > epsilon_on
        Fixed::from_bits(65536),  // epsilon_on = 1.0
        Fixed::from_bits(32768),  // gamma_min_plus_hat
        Fixed::from_bits(32768),  // gamma_min_plus_under < epsilon_gram
        Fixed::from_bits(65536),  // epsilon_gram = 1.0
        Fixed::ZERO,
        Fixed::ONE,
        Fixed::ONE,
        Fixed::from_bits(32768),
    );
    assert_eq!(result, Err(ObservatoryFlag::GramDegenerate));
}

#[test]
fn test_f09_nonstationary_window() {
    let result = evaluate_calibration(
        Fixed::from_bits(131072), // kappa_hat
        Fixed::from_bits(131072), // kappa_under
        Fixed::from_bits(65536),  // epsilon_on
        Fixed::from_bits(131072), // gamma_min_plus_hat
        Fixed::from_bits(131072), // gamma_min_plus_under
        Fixed::from_bits(65536),  // epsilon_gram
        Fixed::from_bits(131072), // d_js > epsilon_drift
        Fixed::from_bits(65536),  // epsilon_drift
        Fixed::ONE,
        Fixed::from_bits(32768),
    );
    assert_eq!(result, Err(ObservatoryFlag::Drifting));
}

#[test]
fn test_f00_exact_scale_collapse() {
    let result = evaluate_calibration(
        Fixed::ZERO, // kappa_hat
        Fixed::ZERO, // kappa_under
        Fixed::from_bits(65536),  // epsilon_on
        Fixed::from_bits(131072), // gamma_min_plus_hat
        Fixed::from_bits(131072), // gamma_min_plus_under
        Fixed::from_bits(65536),  // epsilon_gram
        Fixed::ZERO, // d_js
        Fixed::from_bits(65536),  // epsilon_drift
        Fixed::ONE,  // s_meas
        Fixed::ONE,  // s_leaf == s_meas
    );
    assert_eq!(result, Err(ObservatoryFlag::ScaleInert));
}

#[test]
fn test_f01_material_scale_information() {
    let result = evaluate_calibration(
        Fixed::from_bits(131072), // kappa_hat
        Fixed::from_bits(131072), // kappa_under
        Fixed::from_bits(65536),  // epsilon_on
        Fixed::from_bits(131072), // gamma_min_plus_hat
        Fixed::from_bits(131072), // gamma_min_plus_under
        Fixed::from_bits(65536),  // epsilon_gram
        Fixed::ZERO, // d_js
        Fixed::from_bits(65536),  // epsilon_drift
        Fixed::ONE,  // s_meas
        Fixed::from_bits(32768),  // s_leaf
    );
    assert_eq!(result, Ok(())); // RECERTIFICATION_CANDIDATE
}
