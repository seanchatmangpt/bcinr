// Named law: bounds explicitly reject mutant divergence
//! JTBD 2 — Regulated/auditable adaptive-policy ops: independently reconstructable audit trail.
//!
//! ## What this validates (falsifiable property under test)
//!
//! For one real, successful, fully-certified adaptation — a real `CertifiedModeSwitch`
//! produced by driving the actual `bcinr-cmca` chain
//! (`observatory::evaluate_calibration` -> `proposal::admit_proposal` ->
//! `shadow::execute_shadow` -> `jump::analyze_jump` ->
//! `stability::derive_stability_candidate` -> `certification::seal_certificate` ->
//! `certification::observe_dwell` -> `mode_switch::prepare_mode_switch` /
//! `mode_switch::apply_mode_switch`) — an INDEPENDENT downstream reader can:
//!
//! 1. reconstruct, from public receipt/digest getters alone, which proposal was admitted
//!    (its round identity, observation digest, and proposal digest);
//! 2. reconstruct which stability witness (candidate) justified the certification (its
//!    public `candidate_digest()` / `comparison_derivation_identity()`);
//! 3. reconstruct which certificate digest bound the transition, WITHOUT ever reading the
//!    private `CertificateReceipt` digest field directly (that field is `pub(crate)`, not
//!    visible outside `bcinr-cmca` — this test lives in `tests/`, a separate crate, so it
//!    has no access to it and never gets one);
//! 4. prove the three digests are mutually consistent for exactly one candidate: this test
//!    constructs TWO independently-derived candidate proposals/chains (distinguished by a
//!    different `graph_digest` observation input and a different target-mode digest, both
//!    real inputs to the real `evaluate_calibration`/`execute_shadow` production functions),
//!    certifies only ONE of them, and shows that an independent reader — which never reuses
//!    any value from the production call path, only the public receipts/digests it
//!    produced — can determine WHICH candidate was certified by re-deriving a certificate
//!    from each candidate's own public data via the real `seal_certificate` function and
//!    comparing the result to the sealed receipt with `==` (the only operation
//!    `CertificateReceipt`'s public API supports: it derives `PartialEq`/`Eq` but exposes no
//!    digest getter). This is genuine non-repudiation evidence: the sealed receipt, opaque
//!    on its own, unambiguously identifies its candidate only when replayed through the real
//!    production sealing function — it is not a label attached out-of-band.
//! 5. go one hop further than (1)-(4) and reach actuation itself: drive the real
//!    `mode_switch::prepare_mode_switch` / `mode_switch::apply_mode_switch` for one candidate
//!    to a real, applied `ModeState` transition, attempt to actuate the OTHER candidate's
//!    switch by replaying the first candidate's real certificate against it (refused), and
//!    have an external-auditor function determine — from the public
//!    `mode_switch::ActuationEvidence` record `apply_mode_switch` now returns alongside its
//!    `Result`, and nothing else — which candidate's certificate actually drove a real state
//!    transition. See `independent_reader_disambiguates_which_candidate_was_actuated_via_public_evidence_only`
//!    below.
//!
//! ## Historical note: the actuation hop is now closed
//!
//! An earlier revision of this test (see the "NOTE on scope" comment preserved in
//! `independent_reader_disambiguates_which_of_two_candidates_was_certified` below) recorded a
//! genuine, structural finding: `apply_mode_switch` required a caller-supplied
//! `expected_certificate_digest: u64`, and an external reader holding only an opaque
//! `CertificateReceipt` (private `digest` field, no public getter — by design, per
//! `authority-and-c3.md`'s sealing invariants) had no safe-Rust way to produce that `u64`, so
//! the audit trail this crate could prove stopped one hop short of actuation for anyone
//! outside the crate.
//!
//! That gap is now closed by two changes in `mode_switch.rs`, both exercised by test (5)
//! above:
//!
//! 1. `apply_mode_switch`'s `expected_certificate_digest: u64` parameter was replaced with
//!    `expected_certificate: CertificateReceipt` — the certificate check now compares two full
//!    receipts with `==` (the receipt's own derived equality) instead of requiring a raw
//!    digest extracted from a private field. An external caller obtains a matching
//!    `expected_certificate` by calling the real, public `seal_certificate` a second time with
//!    the same inputs (deterministic, so the two receipts compare equal) — exactly the
//!    "re-derive via the real production function, compare with `==`" idiom test (4) already
//!    used one hop earlier, extended to actuation.
//! 2. `apply_mode_switch` now additionally returns a [`mode_switch::ActuationEvidence`] — a
//!    sealed, purpose-built public evidence artifact (no external constructor; see
//!    `mode_switch.rs`'s own doc comment and its `tests/ui/*actuation_evidence*` compile-fail
//!    suite) exposing `certificate_digest()`, `old_control_mode_digest()`,
//!    `new_control_mode_digest()`, `round_identity()`, and `outcome()`. An external reader
//!    combines this with `CertificateReceipt`'s existing public `admit_certificate`
//!    constructor (wrap the reported digest back into a receipt, compare with `==`) to
//!    identify which candidate's certificate actually drove the transition — never reading
//!    `CertificateReceipt`'s private field, on either side of the comparison.
//!
//! All types used (`ModeProposal`, `AdmittedProposal`, `ShadowExecutionReceipt`,
//! `JumpAnalysisReceipt`, `StabilityCandidate`, `CertificateReceipt`, `DwellSatisfied`,
//! `CertifiedModeSwitch`, `ModeState`, `ActuationEvidence`) are the crate's real, sealed
//! production types, constructed only through their real production entry points. Nothing in
//! this file mocks or stubs any `bcinr-cmca` collaborator.
//!
//! ## What this does NOT validate (explicitly out of scope)
//!
//! - Long-term receipt storage or retention (this test only exercises one in-process run;
//!   it makes no claim about durability, replay after restart, or archival).
//! - Cryptographic tamper-resistance of the digest algorithm itself (`mix64` in
//!   `proposal.rs` is explicitly documented as "not a cryptographic hash" — an avalanche mix
//!   for equality binding, not collision-resistant hashing). This test proves digest-level
//!   *disambiguation* between two known candidates under the real sealing function, not
//!   resistance to a forger with unbounded compute.
//! - Any regulatory-specific audit format (SOC2, FDA, or otherwise). This test proves an
//!   engineering property (digest-chain disambiguation) that such a format might build on,
//!   not compliance with any named standard.
//! - Whether `admit_adaptive_update`'s own (4-binding) digest check independently implies
//!   all 11 `seal_certificate` bindings changed — that evidence gap is recorded separately in
//!   `PHASE2_RUNTIME_CLOSURE_VERDICT.md` and is not re-litigated here.
//!
//! ## Chicago-style TDD note
//!
//! Per `/Users/sac/chicago-tdd-tools`'s conventions (state-based assertions against real
//! collaborators, e.g. `tests/proposer_substrate.rs` in `bcinr-pddl`), every assertion below
//! is a state comparison (`assert_eq!`/`assert_ne!`/`assert!(x == y)`) against values
//! actually returned by real `bcinr-cmca` functions — there is no mock, stub, or spy of any
//! `bcinr-cmca` type in this file.

use bcinr_cmca::allocator::{AdmittedControlState, CertificateReceipt, CertifiedLearning};
use bcinr_cmca::certification::{
    observe_dwell, seal_certificate, CertificateBindings, CertificationRefusal,
};
use bcinr_cmca::fixed::{NonNegativeFixed, SignedFixed};
use bcinr_cmca::jump::{analyze_jump, JumpAnalysisReceipt};
use bcinr_cmca::mode_switch::{
    apply_mode_switch, prepare_mode_switch, ActuationEvidence, ActuationOutcome, ModeState,
    ModeSwitchRefusal,
};
use bcinr_cmca::observatory::{
    evaluate_calibration, MeasurementArtifact, ModeDelta, ObservatoryFlag, SupportStanding,
};
use bcinr_cmca::proposal::{admit_proposal, AdmittedProposal};
use bcinr_cmca::shadow::{execute_shadow, ShadowExecutionReceipt};
use bcinr_cmca::stability::{derive_stability_candidate, StabilityCandidate, DIM, SCALE};

use chicago_tdd_tools::core::governance::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticSink, RunSummary, Severity,
};
use chicago_tdd_tools::observability::ocel::OcelCollector;
use chicago_tdd_tools::test;
use std::collections::HashMap;
use std::path::PathBuf;

const ROUND_IDENTITY: u64 = 7;
const CURRENT_MODE_DIGEST: u64 = 42;

/// Shared telemetry shape that reliably clears the Observatory as
/// `RecertificationCandidate`-only (telemetry-admissible), matching the known-good fixture
/// in `tests/calibration.rs::test_f01_material_scale_information`. Only `graph_digest` is
/// varied between the two candidates built in this test — everything else is identical
/// telemetry, so any downstream digest difference between the two candidate chains traces
/// back to that one real, distinguishing input (plus the different candidate-mode target
/// each chain shadow-executes against).
fn clean_artifact(graph_digest: u64) -> MeasurementArtifact {
    MeasurementArtifact {
        point_estimate: NonNegativeFixed::from_value_bits(131072),
        lower_bound: NonNegativeFixed::from_value_bits(131072),
        upper_bound: NonNegativeFixed::from_value_bits(131072),
        support_standing: SupportStanding {
            is_supported: true,
            smoothing_applied: false,
        },
        effective_sample_size: NonNegativeFixed::ONE,
        dependence_standing: 0,
        numeric_error: NonNegativeFixed::ZERO,
        drift: NonNegativeFixed::ZERO,
        gram_lower_bound: NonNegativeFixed::from_value_bits(131072),
        graph_digest,
        control_mode_digest: CURRENT_MODE_DIGEST,
        proposal: ModeDelta::ProposeDelta,
    }
}

/// One full production-path run from telemetry through a sealed `StabilityCandidate`,
/// carrying every intermediate real receipt so the test (and the independent reader) can
/// inspect them via their public getters.
struct CandidateChain {
    admitted: AdmittedProposal,
    shadow: ShadowExecutionReceipt,
    /// Kept on the struct (and asserted on below) purely as evidence that jump analysis was
    /// actually run as a real hop of the chain, even though its `kind()` and
    /// `analysis_digest()` are not separately consumed downstream in this test's scenario.
    #[allow(dead_code)]
    jump: JumpAnalysisReceipt,
    stability: StabilityCandidate,
    target_mode_digest: u64,
}

/// Drives the real production chain (Observatory -> admission -> shadow -> jump ->
/// stability) for one candidate, distinguished by `graph_digest` (the real
/// `evaluate_calibration` observation input) and `target_mode_digest` (the real
/// `execute_shadow` candidate-mode input).
fn build_candidate_chain(graph_digest: u64, target_mode_digest: u64) -> CandidateChain {
    let artifact = clean_artifact(graph_digest);
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
        outcome
            .flags
            .contains(ObservatoryFlag::RecertificationCandidate),
        "fixture must clear the Observatory as RecertificationCandidate; got {:?}",
        outcome.flags
    );
    assert!(outcome.flags.telemetry_admissible());

    let admitted = admit_proposal(
        outcome.proposal,
        ROUND_IDENTITY,
        CURRENT_MODE_DIGEST,
        SignedFixed::from_value_bits(100),
    )
    .expect("real proposal must be admitted under matching round/mode/delta-bound expectations");

    let shadow = execute_shadow(&admitted, CURRENT_MODE_DIGEST, target_mode_digest);

    let jump = analyze_jump(
        &shadow, /* proposed_delta_magnitude */ 1, /* switching_noise_bound */ 0,
    );

    // Stability is derived from a real 0.5-contracting witness (matches
    // src/stability.rs's own `derives_candidate_when_witness_holds` fixture), with the
    // comparison-derivation identity bound to this candidate's own shadow-receipt digest —
    // exactly the value the independent reader will use to tell candidates apart.
    let g: [[i64; DIM]; DIM] = [[SCALE / 2, 0], [0, SCALE / 2]];
    let d: [i64; DIM] = [SCALE, SCALE];
    let stability = derive_stability_candidate(
        jump.kind(),
        g,
        d,
        SCALE / 4,
        0,
        0,
        SCALE,
        0,
        1,
        0,
        shadow.receipt_digest(),
    )
    .expect("real witness must verify for the fixed 0.5-contracting G/d fixture");

    CandidateChain {
        admitted,
        shadow,
        jump,
        stability,
        target_mode_digest,
    }
}

/// Bindings a real certification registry would hand `seal_certificate` for a given
/// candidate chain: shared substrate identities held fixed, plus the two fields that are
/// genuinely candidate-specific in this test — `control_mode` (the candidate's own target
/// mode digest) and `comparison_derivation` (the candidate's own shadow-receipt digest,
/// already bound into its `StabilityCandidate`).
fn bindings_for(chain: &CandidateChain) -> CertificateBindings {
    CertificateBindings {
        admitted_graph: 1,
        generated_payload: 2,
        kernel_specialization_identity: 3,
        numeric_profile: 4,
        q_registry: 5,
        pricing_law: 6,
        floor_law: 7,
        control_mode: chain.target_mode_digest,
        influence_state: 9,
        comparison_derivation: chain.shadow.receipt_digest(),
        round_identity: ROUND_IDENTITY,
    }
}

/// The independent downstream reader.
///
/// Reuses NO internal state from the production call path above — it is handed only the
/// sealed `CertificateReceipt` under audit and the two candidates' PUBLIC receipts
/// (`StabilityCandidate`, `CertificateBindings`, both obtained through public getters/plain
/// public fields). It has no access to `CertificateReceipt`'s private digest field (this is
/// a separate test crate; that field is `pub(crate)`), so the only operation available to it
/// is calling the real `seal_certificate` function again on each candidate and comparing the
/// result to the receipt under audit with `==` — the receipt's own derived `PartialEq`.
///
/// Returns `Some(0)` if the receipt matches candidate A's re-derivation, `Some(1)` if it
/// matches candidate B's, `None` if it matches neither (which would itself be a finding:
/// the receipt is not reproducible from either known candidate).
fn independent_reader_identify_certified_candidate(
    receipt_under_audit: CertificateReceipt,
    candidate_a: (StabilityCandidate, CertificateBindings),
    candidate_b: (StabilityCandidate, CertificateBindings),
) -> Option<usize> {
    let redo_a = seal_certificate(candidate_a.0, candidate_a.1, candidate_a.1);
    let redo_b = seal_certificate(candidate_b.0, candidate_b.1, candidate_b.1);

    let matches_a = redo_a == Ok(receipt_under_audit);
    let matches_b = redo_b == Ok(receipt_under_audit);

    // The two candidates must not both re-derive the same receipt — otherwise "which one
    // was certified" would be genuinely undecidable from the receipt alone, and this
    // function must say so rather than silently picking one.
    assert!(
        !(matches_a && matches_b),
        "digest chain is ambiguous: both candidates re-derive the receipt under audit"
    );

    if matches_a {
        Some(0)
    } else if matches_b {
        Some(1)
    } else {
        None
    }
}

#[test]
fn independent_reader_disambiguates_which_of_two_candidates_was_certified() {
    let run_id = "jtbd-auditable-adaptive-policy-1";
    let output_path = PathBuf::from("target/jtbd_auditable_adaptive_policy.ocel.jsonl");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("create target dir for OCEL trace");
    }
    if output_path.exists() {
        let _ = std::fs::remove_file(&output_path);
    }
    let collector = OcelCollector::new(Some(output_path.clone()));
    let mut diagnostics_emitted = 0usize;
    let diag = |source_module: &'static str, message: String, elapsed_ns: u64| {
        collector
            .emit(Diagnostic {
                code: DiagnosticCode::new(
                    "bcinr-cmca".to_string(),
                    DiagnosticCategory::Admission,
                    1,
                ),
                category: DiagnosticCategory::Admission,
                severity: Severity::Info,
                location: None,
                message,
                context: HashMap::new(),
                run_id: run_id.to_string(),
                agent_id: None,
                source_module,
                elapsed_ns,
            })
            .expect("emit diagnostic");
    };

    // Arrange: two independently-derived candidate proposals/chains, distinguished by a
    // real observation input (graph_digest) and a real shadow-execution target
    // (candidate_mode_digest). Neither chain is aware of the other.
    let candidate_a = build_candidate_chain(
        /* graph_digest */ 100, /* target_mode_digest */ 5000,
    );
    let candidate_b = build_candidate_chain(
        /* graph_digest */ 200, /* target_mode_digest */ 6000,
    );
    diag(
        "propose",
        format!(
            "two candidates admitted: A.proposal_digest={} B.proposal_digest={}",
            candidate_a.admitted.proposal().proposal_digest(),
            candidate_b.admitted.proposal().proposal_digest()
        ),
        0,
    );
    diagnostics_emitted += 1;

    assert_ne!(
        candidate_a.admitted.proposal().proposal_digest(),
        candidate_b.admitted.proposal().proposal_digest(),
        "the two candidates must be genuinely distinct admitted proposals"
    );
    assert_ne!(
        candidate_a.stability.candidate_digest(),
        candidate_b.stability.candidate_digest(),
        "the two candidates must derive genuinely distinct stability witnesses"
    );

    let bindings_a = bindings_for(&candidate_a);
    let bindings_b = bindings_for(&candidate_b);

    // Act: certify ONLY candidate A, via the real, sole certificate-minting function.
    let cert_a = seal_certificate(candidate_a.stability, bindings_a, bindings_a)
        .expect("candidate A must certify under its own matching bindings");
    diag(
        "certify",
        "candidate A certified via seal_certificate".to_string(),
        1,
    );
    diagnostics_emitted += 1;

    // A real, structural sanity check: certifying candidate B's stability witness under
    // candidate A's expected bindings must be refused (they diverge on control_mode and
    // comparison_derivation) — this is not the disambiguation test itself, but confirms the
    // registry-mismatch path is live, not a no-op.
    let cross_refusal = seal_certificate(candidate_b.stability, bindings_b, bindings_a);
    assert!(
        matches!(
            cross_refusal,
            Err(CertificationRefusal::ControlModeMismatch)
                | Err(CertificationRefusal::ComparisonDerivationMismatch)
        ),
        "certifying B's witness against A's expected bindings must refuse on a named binding mismatch, got {:?}",
        cross_refusal
    );

    // Assert (the falsifiable property): an INDEPENDENT reader, given only the sealed
    // receipt and the two candidates' PUBLIC receipts, can determine unambiguously which
    // candidate was certified — using only the real seal_certificate function and the
    // receipt's own derived equality, never a private field read.
    let identified = independent_reader_identify_certified_candidate(
        cert_a,
        (candidate_a.stability, bindings_a),
        (candidate_b.stability, bindings_b),
    );
    assert_eq!(
        identified,
        Some(0),
        "independent reader must identify candidate A (index 0) as the certified one"
    );
    diag(
        "audit",
        format!(
            "independent reader identified certified candidate index={:?}",
            identified
        ),
        2,
    );
    diagnostics_emitted += 1;

    // NOTE on scope (historical): this test deliberately stops at the certificate-seal hop and
    // does not itself drive `mode_switch::apply_mode_switch`. When this test was first written,
    // `apply_mode_switch` admitted a switch by comparing `certificate.digest ==
    // expected_certificate_digest` against `CertificateReceipt`'s `pub(crate)` digest field — a
    // field this test, living in `tests/` as a separate crate, structurally could not read, and
    // `CertificateReceipt` exposed no public digest getter. That meant an external, independent
    // auditor holding only `cert_a` had no safe-Rust way to supply the matching
    // `expected_certificate_digest` `apply_mode_switch` required, so the audit trail this test
    // proves (proposal/witness/certificate disambiguation) stopped one hop short of actuation
    // for anyone outside the crate. That finding is no longer current: it is fixed by
    // `apply_mode_switch`'s certificate parameter changing from a raw `u64` to a full
    // `CertificateReceipt` (compared with `==`) and by the new `ActuationEvidence` return
    // value — see the module doc comment's "Historical note" and
    // `independent_reader_disambiguates_which_candidate_was_actuated_via_public_evidence_only`
    // below, which drives the actuation hop from outside the crate using only those two
    // changes. This test itself is left as-is (still stopping at the certificate-seal hop) as
    // a minimal, focused regression for the certification step in isolation.

    collector
        .close(RunSummary {
            run_id: run_id.to_string(),
            total_diagnostics: diagnostics_emitted,
            ..Default::default()
        })
        .expect("close OCEL collector");

    assert!(
        output_path.exists(),
        "OCEL audit trace file must be written"
    );
    let contents = std::fs::read_to_string(&output_path).expect("read OCEL trace file");
    assert!(
        !contents.is_empty(),
        "OCEL audit trace file must be non-empty"
    );
    assert!(
        contents.contains(run_id),
        "OCEL audit trace must reference the run id"
    );
}

/// The external-auditor function for the actuation hop — one hop further downstream than
/// [`independent_reader_identify_certified_candidate`] above, reusing the exact same
/// `Option<usize>` disambiguation idiom. Given only a [`mode_switch::ActuationEvidence`]
/// record and the two candidates' own (public) `CertificateReceipt` values, determines which
/// candidate's certificate actually drove a real, applied state transition — using nothing but
/// `ActuationEvidence`'s public accessors and `CertificateReceipt`'s existing public
/// `admit_certificate` constructor plus its derived `==`. No crate-internal access; no digest
/// getter on `CertificateReceipt` itself is ever used, on either side of the comparison.
///
/// Returns `Some(0)`/`Some(1)` only when the evidence's outcome is
/// [`mode_switch::ActuationOutcome::Applied`] AND its reported certificate digest matches
/// exactly one of the two candidates. Returns `None` for a refused attempt (nothing was
/// actually applied, regardless of which certificate was presented and refused) or for a
/// certificate matching neither known candidate.
fn external_auditor_identify_actuated_candidate(
    evidence: ActuationEvidence,
    candidate_a_cert: CertificateReceipt,
    candidate_b_cert: CertificateReceipt,
) -> Option<usize> {
    // Wrap the evidence's reported digest back into a `CertificateReceipt` via the existing
    // public constructor and compare with `==` — never a raw-digest comparison, and never a
    // read of either candidate's own private field.
    let presented = CertificateReceipt::admit_certificate(evidence.certificate_digest());
    let matches_a = presented == candidate_a_cert;
    let matches_b = presented == candidate_b_cert;
    assert!(
        !(matches_a && matches_b),
        "digest chain is ambiguous: the actuation evidence's certificate matches both candidates"
    );

    if !evidence.outcome().is_applied() {
        // A refused attempt still names which certificate was PRESENTED (and refused) via
        // `certificate_digest()`, but nothing was actually applied to persistent state — that
        // is a distinct, weaker claim than "candidate N was applied," so it is reported here
        // as "no candidate applied," not as an identification of a successful transition.
        return None;
    }

    if matches_a {
        Some(0)
    } else if matches_b {
        Some(1)
    } else {
        None
    }
}

#[test]
fn independent_reader_disambiguates_which_candidate_was_actuated_via_public_evidence_only() {
    // Arrange: two independently-derived candidate chains, each certified for real via the
    // real, sole certificate-minting function (mirrors the disambiguation test above, with its
    // own distinct graph/target digests so the two runs cannot be confused).
    let candidate_a = build_candidate_chain(
        /* graph_digest */ 300, /* target_mode_digest */ 7000,
    );
    let candidate_b = build_candidate_chain(
        /* graph_digest */ 400, /* target_mode_digest */ 8000,
    );
    let bindings_a = bindings_for(&candidate_a);
    let bindings_b = bindings_for(&candidate_b);

    let cert_a = seal_certificate(candidate_a.stability, bindings_a, bindings_a)
        .expect("candidate A must certify under its own matching bindings");
    let cert_b = seal_certificate(candidate_b.stability, bindings_b, bindings_b)
        .expect("candidate B must certify under its own matching bindings");
    assert_ne!(
        cert_a, cert_b,
        "two genuinely distinct candidates must not collide on a certificate"
    );

    // Act 1: certify-and-actuate candidate A for real, all the way to a real applied
    // `ModeState` transition. `expected_certificate` is obtained by an INDEPENDENT second call
    // to the real `seal_certificate` (never by reusing `cert_a`'s value directly) — the same
    // "re-derive via the real production function, compare with `==`" idiom the certification
    // hop above already establishes, now extended one hop further to actuation. This is what
    // makes driving `apply_mode_switch` possible at all from a crate outside `bcinr-cmca`: no
    // step here ever reads a `CertificateReceipt`'s private `digest` field.
    let expected_for_a = seal_certificate(candidate_a.stability, bindings_a, bindings_a)
        .expect("independent re-derivation of candidate A's certificate must succeed identically");

    let admitted_state = AdmittedControlState::admit_control_state(CURRENT_MODE_DIGEST);
    let switch_a = prepare_mode_switch(
        admitted_state,
        CertifiedLearning::admit_learning(),
        candidate_a.target_mode_digest,
    );
    let dwell_a = observe_dwell(ROUND_IDENTITY, /* transition_identity */ 1, 10, 10)
        .expect("dwell must be observed as satisfied for this round/transition");
    let mut state_a = ModeState {
        mode_digest: CURRENT_MODE_DIGEST,
        generation: 0,
    };
    let (result_a, evidence_a) = apply_mode_switch(
        &mut state_a,
        switch_a,
        dwell_a,
        ROUND_IDENTITY,
        /* transition_identity */ 1,
        cert_a,
        expected_for_a,
    );
    assert!(
        result_a.is_ok(),
        "candidate A's real certificate must actuate its own real switch: {:?}",
        result_a
    );
    assert_eq!(
        state_a.mode_digest, candidate_a.target_mode_digest,
        "persistent state must actually reach candidate A's target mode digest"
    );
    assert_eq!(evidence_a.outcome(), ActuationOutcome::Applied);

    // Act 2: attempt to actuate candidate B's switch, but present candidate A's REAL
    // certificate — the only certificate anyone in this test has actually driven through a
    // genuine, successful actuation — against candidate B's own independently-rederived
    // expected certificate. Distinct candidates yield distinct seal digests (asserted above),
    // so this must refuse: a real certificate cannot be replayed to actuate a different
    // candidate's switch.
    let expected_for_b = seal_certificate(candidate_b.stability, bindings_b, bindings_b)
        .expect("independent re-derivation of candidate B's certificate must succeed identically");
    let switch_b = prepare_mode_switch(
        admitted_state,
        CertifiedLearning::admit_learning(),
        candidate_b.target_mode_digest,
    );
    let dwell_b = observe_dwell(ROUND_IDENTITY, /* transition_identity */ 2, 10, 10)
        .expect("dwell must be observed as satisfied for this round/transition");
    let mut state_b = ModeState {
        mode_digest: CURRENT_MODE_DIGEST,
        generation: 0,
    };
    let (result_b, evidence_b) = apply_mode_switch(
        &mut state_b,
        switch_b,
        dwell_b,
        ROUND_IDENTITY,
        /* transition_identity */ 2,
        cert_a,         // WRONG certificate for this switch: candidate A's real one, replayed
        expected_for_b, // candidate B's own independently-rederived expected certificate
    );
    assert_eq!(
        result_b,
        Err(ModeSwitchRefusal::CertificateDigestMismatch),
        "actuating B's switch while presenting A's real certificate must be refused on the \
         certificate binding, got {:?}",
        result_b
    );
    assert_eq!(
        state_b,
        ModeState {
            mode_digest: CURRENT_MODE_DIGEST,
            generation: 0
        },
        "refused actuation must leave persistent state byte-for-byte unchanged"
    );
    assert_eq!(
        evidence_b.outcome(),
        ActuationOutcome::Refused(ModeSwitchRefusal::CertificateDigestMismatch)
    );

    // Assert (the falsifiable property that closes the previously-recorded gap): an
    // INDEPENDENT reader, given ONLY the two `ActuationEvidence` records this test received
    // from `apply_mode_switch` and the two candidates' PUBLIC `CertificateReceipt` values, can
    // determine which candidate's certificate actually drove a real state transition — with no
    // crate-internal access, and no digest getter on `CertificateReceipt` itself.
    let identified_applied =
        external_auditor_identify_actuated_candidate(evidence_a, cert_a, cert_b);
    assert_eq!(
        identified_applied,
        Some(0),
        "external auditor must identify candidate A (index 0) as the actually-applied one, \
         from evidence_a alone"
    );

    let identified_refused =
        external_auditor_identify_actuated_candidate(evidence_b, cert_a, cert_b);
    assert_eq!(
        identified_refused, None,
        "external auditor must report that the refused attempt applied NEITHER candidate, \
         even though the presented certificate matched candidate A"
    );
}
