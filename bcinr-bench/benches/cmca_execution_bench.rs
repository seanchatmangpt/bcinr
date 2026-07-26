//! Divan benchmarks for the complete public CMCA execution surface.
//!
//! The suite deliberately separates primitive kernels from admitted jobs so a
//! regression can be localized to arithmetic, observatory inference, receipt
//! admission, LRC adaptation, allocator execution, or end-to-end orchestration.
//! Refusal and selection-only paths are benchmarked as first-class outcomes:
//! they are production execution rails, not exceptional test scaffolding.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt, StabilityRefusal,
};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::generated::case_studies::{ETA, K, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;
use bcinr_cmca::lrc::{LrcParams, LrcState};
use bcinr_cmca::observatory::{
    evaluate_calibration, measure_kappa, MeasurementArtifact, ModeDelta, ObservatoryFlag,
    SupportStanding,
};
use std::time::Instant;

const CONTROL_MODE_DIGEST: u64 = 42;
const SAFE_DWELL_ROUNDS: u32 = 500;
const WRONG_CERTIFICATE_DIGEST: [u8; 32] = [0; 32];

const CASCADE_PARENT: [i32; N] = [-1, 0, 0, 1, 1, 2, 2, 2];
const CASCADE_IS_LEAF: [bool; N] = [false, false, false, true, true, true, true, true];
const CASCADE_SUBTREE_LEAF: [[bool; N]; N] = [
    [false, false, false, true, true, true, true, true],
    [false, false, false, true, true, false, false, false],
    [false, false, false, false, false, true, true, true],
    [false, false, false, true, false, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, false, false, true, false],
    [false, false, false, false, false, false, false, true],
];

fn measurement(
    point_estimate: NonNegativeFixed,
    lower_bound: NonNegativeFixed,
    gram_lower_bound: NonNegativeFixed,
    drift: NonNegativeFixed,
    proposal: ModeDelta,
) -> MeasurementArtifact {
    MeasurementArtifact {
        point_estimate,
        lower_bound,
        upper_bound: point_estimate,
        support_standing: SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift,
        gram_lower_bound,
        graph_digest: 0,
        control_mode_digest: CONTROL_MODE_DIGEST,
        proposal,
    }
}

fn stable_measurement() -> MeasurementArtifact {
    measurement(
        NonNegativeFixed::from_bits(131_072),
        NonNegativeFixed::from_bits(131_072),
        NonNegativeFixed::from_bits(131_072),
        NonNegativeFixed::ZERO,
        ModeDelta::ProposeDelta,
    )
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
    let certificate = evaluate(&stable_measurement())
        .expect("stable benchmark telemetry must produce a certificate");
    AdaptiveUpdate::admit_adaptive_update(
        AdmittedControlState::admit_control_state(CONTROL_MODE_DIGEST),
        certificate,
        EnvelopeReceipt::admit_envelope(CONTROL_MODE_DIGEST),
        OutcomeReceipt::admit_outcome(CONTROL_MODE_DIGEST),
        NonNegativeFixed::ZERO,
        NonNegativeFixed::from_bits(65),
        CertifiedLearning::admit_learning(),
    )
    .expect("matching benchmark receipts must admit adaptive execution")
}

fn node_masses() -> [[NonNegativeFixed; N]; K] {
    let mut masses = [[NonNegativeFixed::ONE; N]; K];
    for (k, row) in masses.iter_mut().enumerate() {
        for (i, mass) in row.iter_mut().enumerate() {
            *mass = NonNegativeFixed::from_bits(
                65_536u32
                    .saturating_add((k as u32 + 1).saturating_mul(4_096))
                    .saturating_add((i as u32 + 1).saturating_mul(1_024)),
            );
        }
    }
    masses
}

fn lrc_params() -> LrcParams {
    LrcParams {
        alpha: NonNegativeFixed::from_bits(8_192),
        phi_max: NonNegativeFixed::ONE,
        phi_min: NonNegativeFixed::from_bits(6_554),
        zeta_0: NonNegativeFixed::from_bits(328),
        zeta_min: NonNegativeFixed::from_bits(33),
        zeta_max: NonNegativeFixed::from_bits(819),
        eta_0: NonNegativeFixed::from_bits(655),
        eta_min: NonNegativeFixed::from_bits(328),
        eta_max: NonNegativeFixed::from_bits(6_554),
        k_kappa: NonNegativeFixed::from_bits(32_768),
        k_d: NonNegativeFixed::from_bits(32_768),
        gamma: NonNegativeFixed::ONE,
        theta: NonNegativeFixed::from_bits(131_072),
    }
}

#[derive(Clone)]
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
            divan::black_box(&OBJECT_REGISTRY),
            divan::black_box(&LENS_REGISTRY),
            divan::black_box(&LAMBDA),
            divan::black_box(ETA),
            divan::black_box(&self.parent),
            &mut self.weights,
            divan::black_box(&self.payoffs),
            divan::black_box(zeta),
            divan::black_box(NonNegativeFixed::ZERO),
            divan::black_box(&self.mu),
            divan::black_box(&self.costs),
            divan::black_box(SAFE_DWELL_ROUNDS),
            &mut self.last_switch_t,
            &mut self.prev_mode,
            divan::black_box(tau_d),
            divan::black_box(digest),
            divan::black_box(proof),
        )
    }
}

mod fixed_point {
    use super::*;

    #[divan::bench]
    fn nonnegative_add() -> NonNegativeFixed {
        divan::black_box(NonNegativeFixed::from_bits(98_304))
            .saturating_add(divan::black_box(NonNegativeFixed::from_bits(24_576)))
    }

    #[divan::bench]
    fn nonnegative_sub() -> NonNegativeFixed {
        divan::black_box(NonNegativeFixed::from_bits(98_304))
            .saturating_sub(divan::black_box(NonNegativeFixed::from_bits(24_576)))
    }

    #[divan::bench]
    fn nonnegative_mul() -> NonNegativeFixed {
        divan::black_box(NonNegativeFixed::from_bits(98_304))
            .saturating_mul(divan::black_box(NonNegativeFixed::from_bits(49_152)))
    }

    #[divan::bench]
    fn nonnegative_div() -> NonNegativeFixed {
        divan::black_box(NonNegativeFixed::from_bits(98_304))
            .saturating_div(divan::black_box(NonNegativeFixed::from_bits(49_152)))
    }

    #[divan::bench]
    fn nonnegative_log2() -> SignedFixed {
        divan::black_box(NonNegativeFixed::from_bits(98_304)).log2()
    }

    #[divan::bench]
    fn signed_exp2() -> NonNegativeFixed {
        divan::black_box(SignedFixed::from_bits(32_768)).exp2()
    }

    #[divan::bench]
    fn signed_exp() -> NonNegativeFixed {
        divan::black_box(SignedFixed::from_bits(16_384)).exp()
    }
}

mod lrc {
    use super::*;

    #[divan::bench]
    fn adaptive_rate_update() -> (NonNegativeFixed, NonNegativeFixed) {
        let mut state = LrcState::default();
        state.update(
            divan::black_box(NonNegativeFixed::from_bits(4_096)),
            divan::black_box(NonNegativeFixed::from_bits(8_192)),
            divan::black_box(NonNegativeFixed::from_bits(2_048)),
            divan::black_box(&lrc_params()),
        )
    }
}

mod observatory {
    use super::*;

    #[divan::bench]
    fn measure_kappa_kernel() -> MeasurementArtifact {
        let masses = node_masses();
        measure_kappa(
            divan::black_box(0),
            divan::black_box(0),
            divan::black_box(0),
            divan::black_box(&CASCADE_PARENT),
            divan::black_box(&CASCADE_IS_LEAF),
            divan::black_box(&CASCADE_SUBTREE_LEAF[0]),
            divan::black_box(&CASCADE_SUBTREE_LEAF),
            divan::black_box(&masses),
            divan::black_box(SignedFixed::ONE),
        )
    }

    #[divan::bench]
    fn recertification_candidate() -> Result<CertificateReceipt, ObservatoryFlag> {
        evaluate(divan::black_box(&stable_measurement()))
    }

    #[divan::bench]
    fn drift_refusal() -> Result<CertificateReceipt, ObservatoryFlag> {
        let artifact = measurement(
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            ModeDelta::ProposeDelta,
        );
        evaluate(divan::black_box(&artifact))
    }

    #[divan::bench]
    fn scale_inert_refusal() -> Result<CertificateReceipt, ObservatoryFlag> {
        evaluate_calibration(
            divan::black_box(&stable_measurement()),
            NonNegativeFixed::from_bits(65_536),
            NonNegativeFixed::from_bits(65_536),
            NonNegativeFixed::from_bits(65_536),
            NonNegativeFixed::ONE,
            NonNegativeFixed::ONE,
        )
    }

    #[divan::bench]
    fn numerical_uncertainty_refusal() -> Result<CertificateReceipt, ObservatoryFlag> {
        let artifact = measurement(
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(32_768),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::ZERO,
            ModeDelta::ProposeDelta,
        );
        evaluate(divan::black_box(&artifact))
    }

    #[divan::bench]
    fn gram_degeneracy_refusal() -> Result<CertificateReceipt, ObservatoryFlag> {
        let artifact = measurement(
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(32_768),
            NonNegativeFixed::ZERO,
            ModeDelta::ProposeDelta,
        );
        evaluate(divan::black_box(&artifact))
    }

    #[divan::bench]
    fn unadmitted_mode_delta_refusal() -> Result<CertificateReceipt, ObservatoryFlag> {
        let artifact = measurement(
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::ZERO,
            ModeDelta::Retain,
        );
        evaluate(divan::black_box(&artifact))
    }
}

mod admission {
    use super::*;

    #[divan::bench]
    fn matching_receipts() -> Option<AdaptiveUpdate<CertifiedLearning>> {
        AdaptiveUpdate::admit_adaptive_update(
            divan::black_box(AdmittedControlState::admit_control_state(
                CONTROL_MODE_DIGEST,
            )),
            divan::black_box(CertificateReceipt::admit_certificate(CONTROL_MODE_DIGEST)),
            divan::black_box(EnvelopeReceipt::admit_envelope(CONTROL_MODE_DIGEST)),
            divan::black_box(OutcomeReceipt::admit_outcome(CONTROL_MODE_DIGEST)),
            divan::black_box(NonNegativeFixed::ZERO),
            divan::black_box(NonNegativeFixed::from_bits(65)),
            divan::black_box(CertifiedLearning::admit_learning()),
        )
    }

    #[divan::bench]
    fn mismatched_receipts() -> Option<AdaptiveUpdate<CertifiedLearning>> {
        AdaptiveUpdate::admit_adaptive_update(
            divan::black_box(AdmittedControlState::admit_control_state(
                CONTROL_MODE_DIGEST + 1,
            )),
            divan::black_box(CertificateReceipt::admit_certificate(CONTROL_MODE_DIGEST)),
            divan::black_box(EnvelopeReceipt::admit_envelope(CONTROL_MODE_DIGEST)),
            divan::black_box(OutcomeReceipt::admit_outcome(CONTROL_MODE_DIGEST)),
            divan::black_box(NonNegativeFixed::ZERO),
            divan::black_box(NonNegativeFixed::from_bits(65)),
            divan::black_box(CertifiedLearning::admit_learning()),
        )
    }
}

mod allocator {
    use super::*;

    #[divan::bench]
    fn selection_only() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        AllocatorHarness::new().run(
            NonNegativeFixed::ZERO,
            SAFE_DWELL_ROUNDS,
            CERTIFICATE_DIGEST,
            None,
        )
    }

    #[divan::bench]
    fn certified_learning() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        let proof = admitted_proof();
        AllocatorHarness::new().run(
            NonNegativeFixed::from_bits(64),
            SAFE_DWELL_ROUNDS,
            CERTIFICATE_DIGEST,
            Some(&proof),
        )
    }

    #[divan::bench]
    fn certificate_mismatch_refusal() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        let proof = admitted_proof();
        AllocatorHarness::new().run(
            NonNegativeFixed::ZERO,
            SAFE_DWELL_ROUNDS,
            WRONG_CERTIFICATE_DIGEST,
            Some(&proof),
        )
    }

    #[divan::bench]
    fn dwell_time_refusal() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        let proof = admitted_proof();
        AllocatorHarness::new().run(NonNegativeFixed::ZERO, 10, CERTIFICATE_DIGEST, Some(&proof))
    }

    #[divan::bench]
    fn price_envelope_refusal() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        let proof = admitted_proof();
        let mut harness = AllocatorHarness::new();
        harness.mu[0] = NonNegativeFixed::from_bits(6_553_601);
        harness.run(
            NonNegativeFixed::ZERO,
            SAFE_DWELL_ROUNDS,
            CERTIFICATE_DIGEST,
            Some(&proof),
        )
    }

    #[divan::bench(args = [1, 8, 64])]
    fn selection_only_batch(rounds: usize) -> u32 {
        let mut harness = AllocatorHarness::new();
        let mut checksum = 0u32;
        for _ in 0..rounds {
            let allocation = harness
                .run(
                    NonNegativeFixed::ZERO,
                    SAFE_DWELL_ROUNDS,
                    CERTIFICATE_DIGEST,
                    None,
                )
                .expect("selection-only benchmark execution must remain admitted");
            checksum ^= allocation[0].val;
        }
        divan::black_box(checksum)
    }
}

mod end_to_end {
    use super::*;

    #[divan::bench]
    fn certified_observe_admit_allocate() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        let artifact = stable_measurement();
        let certificate = evaluate(divan::black_box(&artifact))
            .expect("stable end-to-end telemetry must certify");
        let proof = AdaptiveUpdate::admit_adaptive_update(
            AdmittedControlState::admit_control_state(CONTROL_MODE_DIGEST),
            certificate,
            EnvelopeReceipt::admit_envelope(CONTROL_MODE_DIGEST),
            OutcomeReceipt::admit_outcome(CONTROL_MODE_DIGEST),
            NonNegativeFixed::ZERO,
            NonNegativeFixed::from_bits(65),
            CertifiedLearning::admit_learning(),
        )
        .expect("matching end-to-end receipts must admit execution");

        AllocatorHarness::new().run(
            NonNegativeFixed::from_bits(64),
            SAFE_DWELL_ROUNDS,
            CERTIFICATE_DIGEST,
            Some(&proof),
        )
    }

    #[divan::bench]
    fn drift_refusal_to_selection_only() -> Result<[NonNegativeFixed; N], StabilityRefusal> {
        let artifact = measurement(
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            NonNegativeFixed::from_bits(131_072),
            ModeDelta::ProposeDelta,
        );
        let observatory_result = evaluate(divan::black_box(&artifact));
        divan::black_box(observatory_result);

        AllocatorHarness::new().run(
            NonNegativeFixed::ZERO,
            SAFE_DWELL_ROUNDS,
            WRONG_CERTIFICATE_DIGEST,
            None,
        )
    }
}

fn main() {
    let start = Instant::now();
    divan::main();
    eprintln!("cmca_execution_bench wall clock: {:?}", start.elapsed());
}
