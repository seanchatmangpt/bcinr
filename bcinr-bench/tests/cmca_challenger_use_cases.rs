//! Buyer-outcome validation for the CMCA execution rail.
//!
//! These scenarios sit above the microbenchmark suite. Each case states a
//! Challenger Sale narrative and then proves the operational claim through the
//! real public CMCA APIs: observe, certify, admit, allocate, refuse, or fall
//! back safely.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt, StabilityRefusal,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated::case_studies::{ETA, K, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
use bcinr_cmca::observatory::{
    evaluate_calibration, MeasurementArtifact, ModeDelta, ObservatoryFlag, SupportStanding,
};

const CONTROL_MODE_DIGEST: u64 = 42;
const SAFE_DWELL_ROUNDS: u32 = 500;
const WRONG_CERTIFICATE_DIGEST: [u8; 32] = [0; 32];
const CASCADE_PARENT: [i32; N] = [-1, 0, 0, 1, 1, 2, 2, 2];

#[derive(Clone, Copy)]
struct ChallengerCase {
    name: &'static str,
    buyer: &'static str,
    teach: &'static str,
    tailor: &'static str,
    take_control: &'static str,
    proof: &'static str,
}

impl ChallengerCase {
    fn validate_story(self) {
        assert!(!self.name.is_empty());
        assert!(!self.buyer.is_empty());
        assert!(!self.teach.is_empty());
        assert!(!self.tailor.is_empty());
        assert!(!self.take_control.is_empty());
        assert!(!self.proof.is_empty());
    }
}

fn measurement(drift: NonNegativeFixed, proposal: ModeDelta) -> MeasurementArtifact {
    MeasurementArtifact {
        point_estimate: NonNegativeFixed::from_bits(131_072),
        lower_bound: NonNegativeFixed::from_bits(131_072),
        upper_bound: NonNegativeFixed::from_bits(131_072),
        support_standing: SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift,
        gram_lower_bound: NonNegativeFixed::from_bits(131_072),
        graph_digest: 0,
        control_mode_digest: CONTROL_MODE_DIGEST,
        proposal,
    }
}

fn evaluate(artifact: &MeasurementArtifact) -> Result<CertificateReceipt, ObservatoryFlag> {
    evaluate_calibration(
        artifact,
        NonNegativeFixed::from_bits(65_536),
        NonNegativeFixed::from_bits(65_536),
        NonNegativeFixed::from_bits(65_536),
        NonNegativeFixed::ONE,
        NonNegativeFixed::from_bits(32_768),
    )
}

fn admitted_proof() -> AdaptiveUpdate<CertifiedLearning> {
    let certificate = evaluate(&measurement(NonNegativeFixed::ZERO, ModeDelta::ProposeDelta))
        .expect("stable use-case telemetry must certify");

    AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(CONTROL_MODE_DIGEST),
        certificate,
        EnvelopeReceipt::admit_envelope(CONTROL_MODE_DIGEST),
        OutcomeReceipt::admit_outcome(CONTROL_MODE_DIGEST),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::from_bits(65),
        CertifiedLearning::admit_learning(),
    )
    .expect("matching use-case receipts must admit adaptive execution")
}

struct AllocatorHarness {
    weights: [[NonNegativeFixed; 2 * Q]; N],
    payoffs: [[NonNegativeFixed; 2 * Q]; N],
    last_switch_t: u32,
    prev_mode: u32,
    parent: [i32; N],
    mu: [NonNegativeFixed; N],
    costs: [NonNegativeFixed; N],
}

impl AllocatorHarness {
    fn new() -> Self {
        Self {
            weights: [[NonNegativeFixed::ONE; 2 * Q]; N],
            payoffs: [[NonNegativeFixed::from_bits(256); 2 * Q]; N],
            last_switch_t: 0,
            prev_mode: 0,
            parent: CASCADE_PARENT,
            mu: [NonNegativeFixed::ZERO; N],
            costs: [NonNegativeFixed::from_bits(1_024); N],
        }
    }

    fn run(
        &mut self,
        zeta: NonNegativeFixed,
        tau_d: u32,
        digest: [u8; 32],
        proof: Option<&AdaptiveUpdate<CertifiedLearning>>,
    ) -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        allocate(
            &OBJECT_REGISTRY,
            &LENS_REGISTRY,
            &LAMBDA,
            ETA,
            &self.parent,
            &mut self.weights,
            &self.payoffs,
            zeta,
            NonNegativeFixed::ZERO,
            &self.mu,
            &self.costs,
            SAFE_DWELL_ROUNDS,
            &mut self.last_switch_t,
            &mut self.prev_mode,
            tau_d,
            digest,
            proof,
        )
    }
}

#[test]
fn cloud_inference_routing_proves_certified_adaptation() {
    ChallengerCase {
        name: "Cloud inference routing",
        buyer: "Platform engineering and FinOps",
        teach: "The expensive failure is not imperfect routing; it is unbounded adaptation without evidence.",
        tailor: "CMCA permits learning only when telemetry, control state, envelope, and outcome receipts agree.",
        take_control: "Require a certified-learning path before allowing autonomous routing changes.",
        proof: "Stable telemetry certifies, matching receipts admit, and allocation succeeds.",
    }
    .validate_story();

    let proof = admitted_proof();
    let allocation = AllocatorHarness::new().run(
        NonNegativeFixed::from_bits(64),
        SAFE_DWELL_ROUNDS,
        CERTIFICATE_DIGEST,
        Some(&proof),
    );

    assert!(allocation.is_ok(), "certified cloud routing must execute");
}

#[test]
fn fraud_operations_proves_drift_refusal_and_safe_fallback() {
    ChallengerCase {
        name: "Fraud operations under distribution shift",
        buyer: "Risk operations and model governance",
        teach: "A model can remain fast while its operating assumptions have already failed.",
        tailor: "The observatory detects drift before adaptive actuation and preserves a non-learning selection path.",
        take_control: "Make drift refusal and deterministic fallback contractual acceptance criteria.",
        proof: "Drift refuses certification while selection-only allocation remains available.",
    }
    .validate_story();

    let drifted = measurement(
        NonNegativeFixed::from_bits(131_072),
        ModeDelta::ProposeDelta,
    );
    assert!(evaluate(&drifted).is_err(), "drift must refuse certification");

    let fallback = AllocatorHarness::new().run(
        NonNegativeFixed::ZERO,
        SAFE_DWELL_ROUNDS,
        WRONG_CERTIFICATE_DIGEST,
        None,
    );
    assert!(fallback.is_ok(), "selection-only fallback must remain available");
}

#[test]
fn industrial_control_proves_stale_certificate_refusal() {
    ChallengerCase {
        name: "Industrial control rollout",
        buyer: "Operations engineering and safety governance",
        teach: "A previously valid certificate is unsafe evidence after the governed configuration changes.",
        tailor: "CMCA binds adaptive execution to the exact generated stability certificate digest.",
        take_control: "Refuse deployment unless the runtime and certificate identify the same admitted configuration.",
        proof: "A mismatched certificate digest produces a typed allocator refusal.",
    }
    .validate_story();

    let proof = admitted_proof();
    let result = AllocatorHarness::new().run(
        NonNegativeFixed::ZERO,
        SAFE_DWELL_ROUNDS,
        WRONG_CERTIFICATE_DIGEST,
        Some(&proof),
    );

    assert!(result.is_err(), "stale certificates must not actuate");
}

#[test]
fn logistics_dispatch_proves_dwell_time_governance() {
    ChallengerCase {
        name: "Logistics dispatch stabilization",
        buyer: "Network operations and fulfillment leadership",
        teach: "Continuous optimization can create destructive oscillation when modes switch faster than the operation can settle.",
        tailor: "CMCA makes dwell time a runtime admission boundary rather than a dashboard recommendation.",
        take_control: "Set the minimum stabilization interval before approving adaptive dispatch.",
        proof: "A premature mode change is refused before allocation is applied.",
    }
    .validate_story();

    let proof = admitted_proof();
    let result = AllocatorHarness::new().run(
        NonNegativeFixed::ZERO,
        10,
        CERTIFICATE_DIGEST,
        Some(&proof),
    );

    assert!(result.is_err(), "premature switching must be refused");
}

#[test]
fn marketplace_pricing_proves_envelope_enforcement() {
    ChallengerCase {
        name: "Marketplace pricing control",
        buyer: "Commercial operations and finance",
        teach: "An optimizer can improve its objective while violating the economic envelope that makes the business viable.",
        tailor: "CMCA checks admitted price constraints in the allocator execution path.",
        take_control: "Make envelope refusal part of the production go-live decision, not a post-launch audit.",
        proof: "An out-of-envelope multiplier is refused before allocation standing is granted.",
    }
    .validate_story();

    let proof = admitted_proof();
    let mut harness = AllocatorHarness::new();
    harness.mu[0] = NonNegativeFixed::from_bits(6_553_601);
    let result = harness.run(
        NonNegativeFixed::ZERO,
        SAFE_DWELL_ROUNDS,
        CERTIFICATE_DIGEST,
        Some(&proof),
    );

    assert!(result.is_err(), "price-envelope violations must be refused");
}
