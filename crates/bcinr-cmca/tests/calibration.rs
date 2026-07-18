#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

use bcinr_cmca::fixed::{CanonicalMask, NonNegativeFixed, SignedFixed};
use bcinr_cmca::observatory::{
    evaluate_calibration, MeasurementArtifact, ModeDelta, ObservatoryFlag, SupportStanding,
};

const ROUND_IDENTITY: u64 = 7;

fn make_artifact(
    kappa_hat: NonNegativeFixed,
    kappa_under: NonNegativeFixed,
    gamma_min_plus_under: NonNegativeFixed,
    d_js: NonNegativeFixed,
    proposal: ModeDelta,
) -> MeasurementArtifact {
    MeasurementArtifact {
        point_estimate: kappa_hat,
        lower_bound: kappa_under,
        upper_bound: kappa_hat,
        support_standing: SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: d_js,
        gram_lower_bound: gamma_min_plus_under,
        graph_digest: 0,
        control_mode_digest: 42,
        proposal,
    }
}

// Core Calibration Fixtures
#[test]
fn test_f02_numerically_uncertain() {
    let artifact = make_artifact(
        NonNegativeFixed::from_value_bits(66000), // kappa_hat > epsilon_on
        NonNegativeFixed::from_value_bits(65000), // kappa_under < epsilon_on
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ZERO,
        ModeDelta::ProposeDelta,
    );
    let outcome = evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536), // epsilon_on = 1.0
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        ROUND_IDENTITY,
    );
    assert!(
        outcome
            .flags
            .contains(ObservatoryFlag::NumericallyUncertain),
        "expected NumericallyUncertain flag, got {:?}",
        outcome.flags
    );
    assert!(!outcome.flags.telemetry_admissible());
}

#[test]
fn test_f03_gram_degenerate() {
    let artifact = make_artifact(
        NonNegativeFixed::from_value_bits(131072), // kappa_hat > epsilon_on
        NonNegativeFixed::from_value_bits(131072), // kappa_under > epsilon_on
        NonNegativeFixed::from_value_bits(32768),  // gamma_min_plus_under < epsilon_gram
        NonNegativeFixed::ZERO,
        ModeDelta::ProposeDelta,
    );
    let outcome = evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536), // epsilon_on = 1.0
        NonNegativeFixed::from_value_bits(65536), // epsilon_gram = 1.0
        NonNegativeFixed::ONE,
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        ROUND_IDENTITY,
    );
    assert!(
        outcome.flags.contains(ObservatoryFlag::GramDegenerate),
        "expected GramDegenerate flag, got {:?}",
        outcome.flags
    );
    assert!(!outcome.flags.telemetry_admissible());
}

#[test]
fn test_f09_nonstationary_window() {
    let artifact = make_artifact(
        NonNegativeFixed::from_value_bits(131072), // kappa_hat
        NonNegativeFixed::from_value_bits(131072), // kappa_under
        NonNegativeFixed::from_value_bits(131072), // gamma_min_plus_under
        NonNegativeFixed::from_value_bits(131072), // d_js > epsilon_drift
        ModeDelta::ProposeDelta,
    );
    let outcome = evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536), // epsilon_on
        NonNegativeFixed::from_value_bits(65536), // epsilon_gram
        NonNegativeFixed::from_value_bits(65536), // epsilon_drift
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        ROUND_IDENTITY,
    );
    assert!(
        outcome.flags.contains(ObservatoryFlag::Drifting),
        "expected Drifting flag, got {:?}",
        outcome.flags
    );
    assert!(!outcome.flags.telemetry_admissible());
}

#[test]
fn test_f00_exact_scale_collapse() {
    let artifact = make_artifact(
        NonNegativeFixed::ZERO,                    // kappa_hat
        NonNegativeFixed::ZERO,                    // kappa_under
        NonNegativeFixed::from_value_bits(131072), // gamma_min_plus_under
        NonNegativeFixed::ZERO,                    // d_js
        ModeDelta::ProposeDelta,
    );
    let outcome = evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536), // epsilon_on
        NonNegativeFixed::from_value_bits(65536), // epsilon_gram
        NonNegativeFixed::from_value_bits(65536), // epsilon_drift
        NonNegativeFixed::ONE,                    // s_meas
        NonNegativeFixed::ONE,                    // s_leaf == s_meas
        ROUND_IDENTITY,
    );
    assert!(
        outcome.flags.contains(ObservatoryFlag::ScaleInert),
        "expected ScaleInert flag, got {:?}",
        outcome.flags
    );
    assert!(!outcome.flags.telemetry_admissible());
}

#[test]
fn test_f01_material_scale_information() {
    let artifact = make_artifact(
        NonNegativeFixed::from_value_bits(131072), // kappa_hat
        NonNegativeFixed::from_value_bits(131072), // kappa_under
        NonNegativeFixed::from_value_bits(131072), // gamma_min_plus_under
        NonNegativeFixed::ZERO,                    // d_js
        ModeDelta::ProposeDelta,
    );
    let outcome = evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536), // epsilon_on
        NonNegativeFixed::from_value_bits(65536), // epsilon_gram
        NonNegativeFixed::from_value_bits(65536), // epsilon_drift
        NonNegativeFixed::ONE,                    // s_meas
        NonNegativeFixed::from_value_bits(32768), // s_leaf
        ROUND_IDENTITY,
    );
    assert!(
        outcome
            .flags
            .contains(ObservatoryFlag::RecertificationCandidate),
        "expected RecertificationCandidate flag, got {:?}",
        outcome.flags
    );
    assert!(outcome.flags.telemetry_admissible());
    assert_eq!(outcome.proposal.proposed_control_delta().value_bits(), 1);
    assert_eq!(outcome.proposal.round_identity(), ROUND_IDENTITY);
    assert_eq!(outcome.proposal.current_mode_digest(), 42);
}

#[test]
fn test_f04_mode_delta_unadmitted() {
    let artifact = make_artifact(
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::from_value_bits(131072),
        NonNegativeFixed::ZERO,
        ModeDelta::Retain, // Retain!
    );
    let outcome = evaluate_calibration(
        &artifact,
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::from_value_bits(65536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_value_bits(32768),
        ROUND_IDENTITY,
    );
    assert!(
        outcome.flags.contains(ObservatoryFlag::ModeDeltaUnadmitted),
        "expected ModeDeltaUnadmitted flag, got {:?}",
        outcome.flags
    );
    assert!(!outcome.flags.telemetry_admissible());
}
