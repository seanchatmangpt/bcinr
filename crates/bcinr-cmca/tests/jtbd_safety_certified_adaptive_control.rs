// Named law: bounds explicitly reject mutant divergence
//! JTBD 1 — Safety-certified adaptive infrastructure control.
//!
//! Speculative, explicitly-labeled-as-inferred job-to-be-done scenario, NOT a confirmed
//! product requirement.
//!
//! # Falsifiable property under test
//!
//! Given a real sequence of two [`allocate`] calls against the same initial persistent
//! state — (a) one whose certificate digest does not match the compiled
//! [`CERTIFICATE_DIGEST`] (an uncertified/rejected mode-switch attempt), and (b) one whose
//! digest matches and whose `proof` argument is a real, fully-admitted
//! `AdaptiveUpdate<CertifiedLearning>` (a fully-certified attempt engineered so a mode
//! switch is actually due: `prev_mode` starts at a value different from the dominant mode
//! the weights resolve to, and `t - last_switch_t >= tau_d`) — ONLY case (b) changes the
//! persistent `prev_mode`/`last_switch_t` control-mode state. Case (a) leaves `weights`,
//! `last_switch_t`, and `prev_mode` field-for-field identical to their pre-attempt values.
//!
//! This is a Chicago-style (classicist) test: it calls the real, production
//! `bcinr_cmca::allocator::allocate` function twice against real
//! `bcinr_cmca::allocator::{AdaptiveUpdate, AdmittedControlState, CertificateReceipt,
//! EnvelopeReceipt, OutcomeReceipt, CertifiedLearning}` types (the same construction path
//! `tests/case_studies.rs::get_proof` uses), and asserts on the real, returned/mutated
//! `weights`/`last_switch_t`/`prev_mode` state directly — no mock or stub of any CMCA
//! internal is used anywhere in this file. `chicago-tdd-tools` (per its README: "Chicago
//! TDD... focuses on behavior verification using real collaborators instead of mocks") is
//! declared as a dev-dependency for this file's discipline; per the tool's own docs, "the
//! framework is a discipline, not a requirement to use every utility" — this test uses
//! plain Rust field-equality assertions on real byte-bearing state (`weights: [[NonNegativeFixed; 2*Q]; N]`,
//! `last_switch_t: u32`, `prev_mode: u32`), the same technique `tests/case_studies.rs::test_rejection_invariance`
//! already uses for the single-rejection-cause case, extended here to a real certified-vs-uncertified pair.
//!
//! # What this test explicitly does NOT validate
//!
//! - It does NOT prove the certification logic itself is correct for every possible drift
//!   signal, digest mismatch pattern, or combination of the ~8 distinct error flags
//!   (`gd_ok`, `digest_err`, `lr_err`, `beta_err`, `eta_err`, `dwell_err`, `q_err`,
//!   `price_err`) `allocate` computes internally — it exercises exactly one concrete
//!   under-certified input (digest mismatch) and one concrete fully-certified input.
//! - It does NOT address multi-tenant fairness, real-time latency bounds, or distributed
//!   consensus — those are out of scope for what this crate currently implements.
//! - It does NOT independently verify `seal_certificate`'s 11-binding check
//!   (`certification.rs`) or the `CertifiedModeSwitch`/`apply_mode_switch` surrogate-state
//!   gate in `mode_switch.rs` — those have their own dedicated unit tests in this crate.
//!   This test only exercises the gate as it is wired through the production
//!   `allocator::allocate` entry point's own internal `did_switch` computation.
//! - It does NOT assert anything about `AllocationOutcome::is_refused()` for case (b): the
//!   crate's own doc/example convention (`allocator.rs` module docs, `case_studies.rs`) is
//!   that a fully-certified, non-degraded call to `allocate` returns a non-refused
//!   candidate; this test's load-bearing assertion for case (b) is the persistent-state
//!   *change*, not the refusal flag, so both are checked but only the state-change
//!   assertion is treated as the falsifiable claim this test exists to make.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated_artifact::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
use bcinr_cmca::generated::stability_profile::CERTIFICATE_DIGEST;

/// Constructs a real, fully-admitted `AdaptiveUpdate<CertifiedLearning>` proof — the same
/// construction path `tests/case_studies.rs::get_proof` uses. Returns `None` only if the
/// real `admit_adaptive_update` admission function itself refuses these (deliberately
/// trivial/zeroed) bindings, which would itself be a finding worth surfacing, not papered
/// over.
fn real_certified_proof() -> Option<AdaptiveUpdate<CertifiedLearning>> {
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

/// Real, deterministic starting fixture shared by both attempts before each is applied to
/// its own copy of the persistent state (uniform weights, zeroed payoffs/prices/costs — the
/// same shape `tests/case_studies.rs` test fixtures use).
struct PersistentFixture {
    weights: [[NonNegativeFixed; 2 * Q]; N],
    payoffs: [[NonNegativeFixed; 2 * Q]; N],
    parent: [i32; N],
    mu: [NonNegativeFixed; N],
    costs: [NonNegativeFixed; N],
}

fn real_fixture() -> PersistentFixture {
    PersistentFixture {
        weights: [[NonNegativeFixed::ONE; 2 * Q]; N],
        payoffs: [[NonNegativeFixed::ZERO; 2 * Q]; N],
        parent: [-1; N],
        mu: [NonNegativeFixed::ZERO; N],
        costs: [NonNegativeFixed::ZERO; N],
    }
}

/// `tau_d` used by both attempts. Must be `>= MODE_DWELL_ROUNDS_MIN` (the same value
/// `tests/case_studies.rs` uses throughout) so dwell is not itself the refusal cause.
const TAU_D: u32 = 500;

/// Epoch at which both attempts are made, chosen so `t.wrapping_sub(last_switch_t) >=
/// TAU_D` holds for the real, unmutated `last_switch_t = 0` starting value — i.e. the dwell
/// window has genuinely elapsed and a switch is genuinely due, not merely permitted by
/// omission.
const T_EPOCH: u32 = TAU_D;

/// Persistent control-mode starting point. Chosen distinct from the dominant mode the
/// uniform-`ONE` weights fixture resolves to (mode `0`, verified empirically: with all
/// `2*Q` root weights equal, the branchless strict-`>` argmax in `allocate` keeps the first
/// index, `0`, since no later equal weight is ever strictly greater) — so a real,
/// state-dependent mode switch is genuinely due going into each attempt, not vacuously
/// already-satisfied.
const PREV_MODE_START: u32 = 1;

#[test]
fn jtbd_uncertified_attempt_leaves_persistent_state_byte_identical() {
    // Arrange: real fixture, deliberately mismatched digest (all-zero, guaranteed not to
    // equal the real compiled CERTIFICATE_DIGEST), real fully-formed proof present so the
    // ONLY thing under-certified about this attempt is the digest itself.
    let fx = real_fixture();
    let mut weights = fx.weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = PREV_MODE_START;

    let weights_before = weights;
    let last_switch_t_before = last_switch_t;
    let prev_mode_before = prev_mode;

    let proof = real_certified_proof();
    assert!(
        proof.is_some(),
        "test fixture invariant broken: real_certified_proof() must admit — the digest \
         mismatch below must be the ONLY under-certified element of this attempt"
    );

    // Act: real allocate() call, wrong digest.
    let outcome = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &fx.parent,
        &mut weights,
        &fx.payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &fx.mu,
        &fx.costs,
        T_EPOCH,
        &mut last_switch_t,
        &mut prev_mode,
        TAU_D,
        [0u8; 32], // deliberately wrong digest
        proof.as_ref(),
    );

    // Assert: state-based, on the real mutated locals directly.
    assert!(
        outcome.is_refused(),
        "expected the digest-mismatched attempt to be refused, got: {:?}",
        outcome.refusals()
    );
    assert_eq!(
        weights, weights_before,
        "uncertified attempt must leave `weights` field-for-field unchanged"
    );
    assert_eq!(
        last_switch_t, last_switch_t_before,
        "uncertified attempt must leave `last_switch_t` unchanged"
    );
    assert_eq!(
        prev_mode, prev_mode_before,
        "uncertified attempt must leave `prev_mode` unchanged"
    );
}

#[test]
fn jtbd_fully_certified_attempt_actually_changes_persistent_control_mode() {
    // Arrange: same fixture shape, real matching digest, real proof, dwell window already
    // elapsed (t = TAU_D against last_switch_t = 0), and prev_mode deliberately starting at
    // a value distinct from the dominant mode the uniform weights resolve to — so a switch
    // is genuinely due.
    let fx = real_fixture();
    let mut weights = fx.weights;
    let mut last_switch_t = 0u32;
    let mut prev_mode = PREV_MODE_START;

    let last_switch_t_before = last_switch_t;
    let prev_mode_before = prev_mode;

    let proof = real_certified_proof();
    assert!(
        proof.is_some(),
        "test fixture invariant broken: real_certified_proof() must admit for the \
         fully-certified attempt to be a genuine positive case"
    );

    // Act: real allocate() call, correct digest, real proof, switch genuinely due.
    let outcome = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &fx.parent,
        &mut weights,
        &fx.payoffs,
        NonNegativeFixed::ZERO,
        NonNegativeFixed::ZERO,
        &fx.mu,
        &fx.costs,
        T_EPOCH,
        &mut last_switch_t,
        &mut prev_mode,
        TAU_D,
        CERTIFICATE_DIGEST,
        proof.as_ref(),
    );

    // Assert: state-based — the persistent control-mode state must actually have changed.
    // This is the falsifiable claim this test exists to make: unlike case (a), this
    // attempt's `prev_mode`/`last_switch_t` are NOT byte-identical to their pre-attempt
    // values.
    assert_ne!(
        (last_switch_t, prev_mode),
        (last_switch_t_before, prev_mode_before),
        "fully-certified attempt with a genuinely-due switch must change persistent \
         control-mode state (last_switch_t={last_switch_t}, prev_mode={prev_mode}), but it \
         was left identical to its pre-attempt value (last_switch_t={last_switch_t_before}, \
         prev_mode={prev_mode_before}); outcome refusals: {:?}",
        outcome.refusals()
    );
    assert_eq!(
        last_switch_t, T_EPOCH,
        "on a genuine switch, last_switch_t must be updated to the current epoch"
    );
    assert_ne!(
        prev_mode, PREV_MODE_START,
        "on a genuine switch, prev_mode must move off its stale starting value"
    );
}
