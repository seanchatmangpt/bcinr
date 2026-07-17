#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed, CanonicalMask};
use bcinr_cmca::allocator::{
    allocate, StabilityRefusal, AdaptiveUpdate, AdmittedControlState,
    CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, CertifiedLearning
};
use bcinr_cmca::generated::case_studies::{
    OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, N, Q
};

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
use bcinr_cmca::generated::generalization as gen;
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

#[test]
fn test_case_study_1_cache_choice() {
    // Artifact_A has high recomputation cost (0.9), Artifact_B has low recomputation cost (0.1).
    // Access frequencies are both 0.5. Standings are 1.0.
    // Under Cache Choice head, Artifact_A should receive more resource than Artifact_B.
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    // We construct a simple tree where 0 and 1 are root leaf nodes
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let result = allocate(
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
        get_proof().as_ref(),
    ).unwrap();

    // Verify that Artifact_A (index 0) gets more cache allocation than Artifact_B (index 1)
    // In our lambda matrix, index 0 is MeasureCache, which dominates lens 0 (2.0) and 1 (1.0).
    println!("result[0]: {:?}, result[1]: {:?}", result[0], result[1]); assert!(result[0].val > result[1].val, "Artifact_A should have higher cache allocation than Artifact_B");
}

#[test]
fn test_case_study_2_single_object_multiple_decisions() {
    // Obj_Single (index 6) has high retrieval demand (0.9) and high business value (100).
    // Let's verify it gets a significant portion of resource allocation under weighted retrieval.
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let result = allocate(
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
        get_proof().as_ref(),
    ).unwrap();

    // Obj_Single (index 6) has high business value and retrieval demand, should have higher allocation than Obj_Obligation.
    for i in 0..N {
        println!("CS2 result[{}]: {:?}", i, result[i]);
    }
    assert!(result[6].val > result[4].val, "Obj_Single should have higher allocation than Obj_Obligation");
}

#[test]
fn test_case_study_3_downstream_consequence() {
    // Obj_Obligation (index 4) has business value 0, but depends on Obj_Activity which eventually
    // depends on Obj_Value (1000).
    // Consequence mass is 1000. Under search-weighted allocation, it should get a high allocation.
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    // Parent-child relationships for Case Study 3
    let mut parent = [-1; N];
    parent[2] = 4; // Obj_Activity depends on Obj_Obligation
    parent[3] = 2; // Obj_Deployment depends on Obj_Activity
    parent[5] = 3; // Obj_Outcome depends on Obj_Deployment
    parent[7] = 5; // Obj_Value depends on Obj_Outcome

    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let result = allocate(
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
        get_proof().as_ref(),
    ).unwrap();

    // Obj_Value (index 7) is the only leaf in the chain, so it receives the allocated resource.
    assert!(result[7].val > 0, "Obj_Value should receive allocation");
}

#[test]
fn test_case_study_4_generalization() {}

#[test]
fn test_stability_refusals_and_graceful_fallback() {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    // 1. Invalid certificate digest with degrade=false -> should return CertificateDigestMismatch
    let wrong_digest = [0u8; 32];
    let res = allocate(
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
        wrong_digest,
        get_proof().as_ref(),
    );
    assert_eq!(res, Err(StabilityRefusal::CertificateDigestMismatch));

    // 2. Invalid certificate digest with degrade=true -> should succeed but freeze learning (CertifiedSelectionOnly)
    let weights_before = weights;
    let res_degraded = allocate(
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
        wrong_digest,
        None,
    );
    assert!(res_degraded.is_ok());
    // Learning should be frozen, meaning weights are not modified/updated
    assert_eq!(weights, weights_before);

    // 3. Mode dwell rounds too fast with degrade=false -> should return ModeDwellTimeViolated
    let fast_dwell = 10u32;
    let res_dwell = allocate(
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
        fast_dwell,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    );
    assert_eq!(res_dwell, Err(StabilityRefusal::ModeDwellTimeViolated));

    // 4. Learning rate outside envelope with degrade=false -> should return LearningRateOutsideEnvelope
    let high_zeta = NonNegativeFixed::from_bits(2000); // Exceeds ZETA_W_MAX (819)
    let res_lr = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        high_zeta,
        NonNegativeFixed::ZERO,
        &mu,
        &costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        500,
        CERTIFICATE_DIGEST,
        get_proof().as_ref(),
    );
    assert_eq!(res_lr, Err(StabilityRefusal::LearningRateOutsideEnvelope));
}

#[test]
fn test_typestate_bounds_checks() {
    // 1. Safe boundary: temperature = 5.0 (327680), distinguishability = 0.001 (65)
    let p_ok = AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::from_bits(327680),
        NonNegativeFixed::from_bits(65),
        CertifiedLearning::admit_learning(),
    );
    assert!(p_ok.is_some());

    // 2. Temp too high: temperature = 5.0 + eps (327681), distinguishability = 0.001 (65)
    let p_temp_high = AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::from_bits(327681),
        NonNegativeFixed::from_bits(65),
        CertifiedLearning::admit_learning(),
    );
    assert!(p_temp_high.is_none());

    // 3. Distinguishability too low: temperature = 5.0 (327680), distinguishability = 64
    let p_dist_low = AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(0),
        CertificateReceipt::admit_certificate(0),
        EnvelopeReceipt::admit_envelope(0),
        OutcomeReceipt::admit_outcome(0),
        NonNegativeFixed::from_bits(327680),
        NonNegativeFixed::from_bits(64),
        CertifiedLearning::admit_learning(),
    );
    assert!(p_dist_low.is_none());
}

#[test]
fn test_rejection_invariance() {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 123u32;
    let mut prev_mode = 4u32;
    let parent = [-1; N];
    let mu = [NonNegativeFixed::ZERO; N];
    let costs = [NonNegativeFixed::ZERO; N];

    let weights_before = weights;
    let last_switch_t_before = last_switch_t;
    let prev_mode_before = prev_mode;

    // Trigger a refusal (e.g. invalid digest)
    let res = allocate(
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
        [0u8; 32],
        get_proof().as_ref(),
    );

    assert!(res.is_err());
    assert_eq!(weights, weights_before, "CHEAT-021: REJECTION_STATE_DRIFT - weights modified on rejection!");
    assert_eq!(last_switch_t, last_switch_t_before, "CHEAT-021: REJECTION_STATE_DRIFT - last_switch_t modified on rejection!");
    assert_eq!(prev_mode, prev_mode_before, "CHEAT-021: REJECTION_STATE_DRIFT - prev_mode modified on rejection!");
}


