#![cfg(not(any(
    feature = "mutant_1",
    feature = "mutant_2",
    feature = "mutant_3",
    feature = "mutant_4",
    feature = "mutant_5"
)))]

use bcinr_cmca::fixed::Fixed;
use bcinr_cmca::allocator::{
    allocate, StabilityRefusal, AdaptiveUpdate, AdmittedControlState,
    CertificateReceipt, EnvelopeReceipt, OutcomeReceipt, CertifiedLearning
};
use bcinr_cmca::generated::case_studies::{
    OBJECT_REGISTRY, LENS_REGISTRY, LAMBDA, ETA, N, Q
};

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
use bcinr_cmca::generated::generalization as gen;
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

#[test]
fn test_case_study_1_cache_choice() {
    // Artifact_A has high recomputation cost (0.9), Artifact_B has low recomputation cost (0.1).
    // Access frequencies are both 0.5. Standings are 1.0.
    // Under Cache Choice head, Artifact_A should receive more resource than Artifact_B.
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    // We construct a simple tree where 0 and 1 are root leaf nodes
    let parent = [-1; N];
    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];

    let result = allocate(
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
    ).unwrap();

    // Verify that Artifact_A (index 0) gets more cache allocation than Artifact_B (index 1)
    // In our lambda matrix, index 0 is MeasureCache, which dominates lens 0 (2.0) and 1 (1.0).
    assert!(result[0].0 > result[1].0, "Artifact_A should have higher cache allocation than Artifact_B");
}

#[test]
fn test_case_study_2_single_object_multiple_decisions() {
    // Obj_Single (index 6) has high retrieval demand (0.9) and high business value (100).
    // Let's verify it gets a significant portion of resource allocation under weighted retrieval.
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    let parent = [-1; N];
    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];

    let result = allocate(
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
    ).unwrap();

    // Obj_Single (index 6) has high business value and retrieval demand, should have higher allocation than Obj_Obligation.
    for i in 0..N {
        println!("CS2 result[{}]: {:?}", i, result[i]);
    }
    assert!(result[6].0 > result[4].0, "Obj_Single should have higher allocation than Obj_Obligation");
}

#[test]
fn test_case_study_3_downstream_consequence() {
    // Obj_Obligation (index 4) has business value 0, but depends on Obj_Activity which eventually
    // depends on Obj_Value (1000).
    // Consequence mass is 1000. Under search-weighted allocation, it should get a high allocation.
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    // Parent-child relationships for Case Study 3
    let mut parent = [-1; N];
    parent[2] = 4; // Obj_Activity depends on Obj_Obligation
    parent[3] = 2; // Obj_Deployment depends on Obj_Activity
    parent[5] = 3; // Obj_Outcome depends on Obj_Deployment
    parent[7] = 5; // Obj_Value depends on Obj_Outcome

    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];

    let result = allocate(
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
    ).unwrap();

    // Obj_Value (index 7) is the only leaf in the chain, so it receives the allocated resource.
    assert!(result[7].0 > 0, "Obj_Value should receive allocation");
}

#[test]
fn test_case_study_4_generalization() {
    // Run the allocator on the generalization registry data to verify compatibility and execution.
    let mut weights = [[Fixed::ONE; 2 * gen::Q]; gen::N];
    let payoffs = [[Fixed::ZERO; 2 * gen::Q]; gen::N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    
    let parent = [-1; gen::N];
    let mu = [Fixed::ZERO; gen::N];
    let costs = [Fixed::ZERO; gen::N];

    let gen_states: [bcinr_cmca::generated::case_studies::PackedSemanticState; gen::N] = gen::OBJECT_REGISTRY.map(|state| {
        bcinr_cmca::generated::case_studies::PackedSemanticState {
            id: state.id,
            factors: state.factors,
        }
    });
    let gen_lenses: [bcinr_cmca::generated::case_studies::LensSpec; gen::Q] = gen::LENS_REGISTRY.map(|lens| {
        bcinr_cmca::generated::case_studies::LensSpec {
            id: lens.id,
            q: lens.q,
        }
    });

    let result = allocate(
        &gen_states,
        &gen_lenses,
        &gen::LAMBDA,
        gen::ETA,
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
    ).unwrap();

    // Verify all allocations are valid and sum to 1.0 (Fixed::ONE)
    let mut sum = 0u64;
    for i in 0..gen::N {
        println!("result[{}]: {:?}", i, result[i]);
        sum += result[i].0 as u64;
    }
    println!("sum: {}", sum);
    // We allow small rounding tolerance
    assert!((sum as i64 - Fixed::ONE.0 as i64).abs() < 50, "Total allocation should sum to 1.0");
}

#[test]
fn test_stability_refusals_and_graceful_fallback() {
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0;
    let mut prev_mode = 0;
    let parent = [-1; N];
    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];

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
        Fixed::ZERO,
        Fixed::ZERO,
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
        Fixed::ZERO,
        Fixed::ZERO,
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
        Fixed::ZERO,
        Fixed::ZERO,
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
    let high_zeta = Fixed(2000); // Exceeds ZETA_W_MAX (819)
    let res_lr = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &parent,
        &mut weights,
        &payoffs,
        high_zeta,
        Fixed::ZERO,
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
    let p_ok = AdaptiveUpdate::new(
        AdmittedControlState,
        CertificateReceipt,
        EnvelopeReceipt,
        OutcomeReceipt,
        Fixed(327680),
        Fixed(65),
    );
    assert!(p_ok.is_some());

    // 2. Temp too high: temperature = 5.0 + eps (327681), distinguishability = 0.001 (65)
    let p_temp_high = AdaptiveUpdate::new(
        AdmittedControlState,
        CertificateReceipt,
        EnvelopeReceipt,
        OutcomeReceipt,
        Fixed(327681),
        Fixed(65),
    );
    assert!(p_temp_high.is_none());

    // 3. Distinguishability too low: temperature = 5.0 (327680), distinguishability = 64
    let p_dist_low = AdaptiveUpdate::new(
        AdmittedControlState,
        CertificateReceipt,
        EnvelopeReceipt,
        OutcomeReceipt,
        Fixed(327680),
        Fixed(64),
    );
    assert!(p_dist_low.is_none());
}

#[test]
fn test_rejection_invariance() {
    let mut weights = [[Fixed::ONE; 2 * Q]; N];
    let payoffs = [[Fixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 123u32;
    let mut prev_mode = 4u32;
    let parent = [-1; N];
    let mu = [Fixed::ZERO; N];
    let costs = [Fixed::ZERO; N];

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
        Fixed::ZERO,
        Fixed::ZERO,
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


