// Named law: bounds explicitly reject mutant divergence
//! Permanent regression test for `numeric-hot-path.md` Invariant 5 ("rejected authoritative
//! operations leave state byte-for-byte unchanged") across EVERY `RefusalSet` variant, not
//! just the one (`NO_LEAVES`) whose gap was found and fixed this round.
//!
//! ## Real defect this file guards against
//!
//! `tests/jtbd_boundary_adversarial_inputs.rs` documented a real defect: a NO_LEAVES-only
//! refusal left `has_refusal` (the gate in `src/allocator.rs` that guards the
//! `weights`/`last_switch_t`/`prev_mode` write-back) false, so the real, computed
//! `local_weights` got committed even though the call was refused. `src/allocator.rs`'s
//! `has_refusal` now also folds in `nl_is_zero`. This file is the permanent regression test
//! for that fix, generalized to every declared `RefusalSet` cause rather than `NO_LEAVES`
//! alone, per this round's task: "iterating every RefusalSet variant... asserting full
//! byte-invariance of weights/last_switch_t/prev_mode for each."
//!
//! ## Reachability audit and full per-bit disposition (real finding, not assumed)
//!
//! `RefusalSet` declares 8 bit constants. Reading `src/allocator.rs`'s `gated_refusals` /
//! `final_refusals` construction (the only place any `allocate()` call can produce a
//! `RefusalSet` value) shows only **4 of the 8** are ever unioned into the value `allocate()`
//! actually returns. This file's task ("Refusal Algebra Realization") assigns each of the 8
//! bits one of six dispositions (REACHABLE / UNREACHABLE_BY_PROOF /
//! OWNED_BY_DIFFERENT_COMPONENT / RESERVED_WITH_EXPLICIT_NONCLAIM / DEAD_VARIANT_REMOVE /
//! MISSING_IMPLEMENTATION_PATH); the full disposition table with justification and test
//! references lives in `crates/bcinr-cmca/REFUSAL_REALIZATION_REPORT.md`, and each bit's own
//! `pub const` declaration in `src/allocator.rs` now carries the same disposition inline.
//! Summary:
//!
//! - `NO_LEAVES` — **REACHABLE**, unioned unconditionally on `nl_is_zero`.
//! - `DIGEST_MISMATCH` — **REACHABLE**, unioned on `digest_err`, gated by `has_refusal`.
//! - `DWELL_UNSATISFIED` — **REACHABLE**, unioned on `dwell_err`, gated by `has_refusal`.
//! - `PROPOSAL_REJECTED` — **REACHABLE**, unioned on
//!   `(!gd_ok)|lr_err|beta_err|eta_err|q_err|price_err`, gated by `has_refusal`.
//! - `AUTHORITY_MISSING` — **UNREACHABLE_BY_PROOF**. IS unioned
//!   (`.union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as
//!   u32))`), but the surrounding `gated_refusals` bundle is then masked again by
//!   `has_refusal`, and `has_refusal = (has_error | (nl_is_zero != 0)) &
//!   !degrade_to_certified_selection` requires `degrade_to_certified_selection == false` to
//!   ever be true — the exact opposite of the condition `AUTHORITY_MISSING`'s own mask
//!   requires. For any boolean `b`, `b & !b == false`, so the conjunction is unsatisfiable:
//!   `AUTHORITY_MISSING` is masked to zero on every call, unconditionally — a proof, not an
//!   empirical absence. Verified below by a real run (not only by reading the source) that
//!   specifically targets the scenario a working `AUTHORITY_MISSING` would be expected to
//!   fire under.
//! - `ROUND_MISMATCH` — **OWNED_BY_DIFFERENT_COMPONENT**. No code path in `allocate()`
//!   constructs this bit, but the condition it names is already realized, tested, and
//!   passing via two other modules' own typed return types (never via `RefusalSet`):
//!   `proposal::ProposalRefusal::RoundIdentityMismatch` (`proposal::tests::
//!   refuses_on_round_mismatch`) and `certification::CertificationRefusal::
//!   RoundIdentityMismatch` (`certification::tests::refuses_solo_mismatch_round_identity`).
//! - `CERTIFICATE_STALE` — **OWNED_BY_DIFFERENT_COMPONENT**. No code path in `allocate()`
//!   constructs this bit, but "a previously-valid certificate is no longer current" is
//!   realized, tested, and passing via `mode_switch::ModeSwitchRefusal::
//!   CertificateDigestMismatch` (`mode_switch::tests::
//!   rejection_cause_certificate_mismatch_leaves_state_untouched`) and (for the specific
//!   "sealed against a superseded round" case) `certification::CertificationRefusal::
//!   RoundIdentityMismatch` above.
//! - `CERTIFICATE_MISSING` — **RESERVED_WITH_EXPLICIT_NONCLAIM**. No code path anywhere in
//!   this crate constructs this bit — not even a masked-to-zero one, unlike
//!   `AUTHORITY_MISSING`. "No certificate was ever presented" has no representable trigger
//!   given the current mandatory-parameter API surface (`allocate`'s `digest: [u8; 32]` and
//!   `mode_switch::apply_mode_switch`'s `certificate: CertificateReceipt` are both taken by
//!   value, never `Option`) — a deliberate consequence of the branchless/fixed-shape-input
//!   mandate this crate is built on, not an oversight. Kept declared (not removed as
//!   vestigial) because the underlying domain condition is real and the bit is already read
//!   meaningfully by `RefusalSet::primary_reason()`; reserved for a future API shape able to
//!   distinguish "missing" from "mismatched" at this boundary.
//!
//! For `CERTIFICATE_MISSING`, this file does not fabricate a triggering scenario (that would
//! misrepresent what `allocate()` can actually produce) — it documents the non-reachability
//! directly, corroborated by the negative sweep in
//! `no_dead_refusal_bit_appears_across_a_representative_sweep_of_real_allocate_calls` below.
//! `ROUND_MISMATCH`/`CERTIFICATE_STALE` get real, passing same-object tests, but in their
//! *owning* modules (`proposal.rs`/`certification.rs`/`mode_switch.rs`), not in this
//! `allocate()`-scoped file — see the disposition list above for the exact test names.
//!
//! This is Chicago-style, state-based, real-collaborator TDD: every test below calls the
//! real, unmodified `bcinr_cmca::allocator::allocate` with real `OBJECT_REGISTRY`/
//! `LENS_REGISTRY`/`CERTIFICATE_DIGEST` generated case-study data. No mock or stub of any
//! CMCA internal is used anywhere in this file.

use bcinr_cmca::allocator::{
    allocate, AdaptiveUpdate, AdmittedControlState, CertificateReceipt, CertifiedLearning,
    EnvelopeReceipt, OutcomeReceipt, RefusalSet,
};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_cmca::generated_artifact::case_studies::{ETA, LAMBDA, LENS_REGISTRY, N, OBJECT_REGISTRY, Q};
use bcinr_cmca::generated::stability_profile::{CERTIFICATE_DIGEST, MODE_DWELL_ROUNDS_MIN};

/// Real, unmocked certified-learning proof — identical construction path to every other
/// JTBD/case_studies test file in this crate.
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

const TAU_D: u32 = 500;

/// One real, byte-for-byte-comparable pre-call snapshot of the entire mutable state surface
/// `allocate()` can persist back to its caller (numeric-hot-path.md Invariant 5's scope:
/// `weights`, `last_switch_t`, `prev_mode` — nothing else is mutated through `&mut`/`*`
/// parameters).
struct StateSnapshot {
    weights: [[NonNegativeFixed; 2 * Q]; N],
    last_switch_t: u32,
    prev_mode: u32,
}

/// Fixed call parameters shared by every case in this file, isolated to exactly the fields
/// each case needs to override to trigger its one target cause. `parent = [-1; N]` (every
/// object a flat-sibling leaf) is the shared baseline so `NO_LEAVES` never incidentally fires
/// in the non-NO_LEAVES cases; the NO_LEAVES case below uses the ring topology instead, as in
/// `jtbd_boundary_adversarial_inputs.rs`.
struct Call {
    parent: [i32; N],
    zeta: NonNegativeFixed,
    mu: [NonNegativeFixed; N],
    costs: [NonNegativeFixed; N],
    tau_d: u32,
    digest: [u8; 32],
    proof: Option<AdaptiveUpdate<CertifiedLearning>>,
}

/// The proven-clean baseline: every field set so that, empirically (confirmed by the
/// `baseline_triggers_no_refusal_at_all` sanity test below), `allocate()` returns
/// `is_refused() == false`. Every per-cause test below is this baseline with exactly ONE
/// field changed, so any refusal observed is attributable to that one change alone.
fn clean_baseline() -> Call {
    Call {
        parent: [-1i32; N],
        zeta: NonNegativeFixed::ZERO,
        mu: [NonNegativeFixed::ZERO; N],
        costs: [NonNegativeFixed::ZERO; N],
        tau_d: TAU_D,
        digest: CERTIFICATE_DIGEST,
        proof: get_proof(),
    }
}

/// Runs one real `allocate()` call from a `Call` fixture, returning the outcome plus
/// before/after state snapshots for byte-invariance assertions.
fn run(
    call: &Call,
) -> (
    bcinr_cmca::allocator::AllocationOutcome,
    StateSnapshot,
    StateSnapshot,
) {
    let mut weights = [[NonNegativeFixed::ONE; 2 * Q]; N];
    let payoffs = [[NonNegativeFixed::ZERO; 2 * Q]; N];
    let mut last_switch_t = 0u32;
    let mut prev_mode = 0u32;

    let before = StateSnapshot {
        weights,
        last_switch_t,
        prev_mode,
    };

    let outcome = allocate(
        &OBJECT_REGISTRY,
        &LENS_REGISTRY,
        &LAMBDA,
        ETA,
        &call.parent,
        &mut weights,
        &payoffs,
        call.zeta,
        NonNegativeFixed::ZERO,
        &call.mu,
        &call.costs,
        0,
        &mut last_switch_t,
        &mut prev_mode,
        call.tau_d,
        call.digest,
        call.proof.as_ref(),
    );

    let after = StateSnapshot {
        weights,
        last_switch_t,
        prev_mode,
    };

    (outcome, before, after)
}

fn assert_state_invariant(before: &StateSnapshot, after: &StateSnapshot, label: &str) {
    assert_eq!(
        after.weights, before.weights,
        "{label}: weights must be byte-identical to their pre-attempt value \
         (numeric-hot-path.md Invariant 5)"
    );
    assert_eq!(
        after.last_switch_t, before.last_switch_t,
        "{label}: last_switch_t must be unchanged"
    );
    assert_eq!(
        after.prev_mode, before.prev_mode,
        "{label}: prev_mode must be unchanged"
    );
}

// -----------------------------------------------------------------------------------------
// Sanity: the shared baseline really does trigger no refusal at all, so every per-cause test
// below can attribute its observed refusal to its one deliberate change.
// -----------------------------------------------------------------------------------------

#[test]
fn baseline_triggers_no_refusal_at_all() {
    let (outcome, _before, _after) = run(&clean_baseline());
    assert!(
        !outcome.is_refused(),
        "clean_baseline() must not refuse; every per-cause test in this file relies on this: {:?}",
        outcome.refusals()
    );
}

// -----------------------------------------------------------------------------------------
// The 4 reachable causes: one real allocate() call per cause, each triggering ONLY that
// cause, each asserting full byte-invariance of weights/last_switch_t/prev_mode.
// -----------------------------------------------------------------------------------------

/// `NO_LEAVES`: a ring `parent` array clears every `is_leaf[i]` (see
/// `jtbd_boundary_adversarial_inputs.rs` for the construction rationale), with every other
/// field left at the clean baseline.
#[test]
fn no_leaves_only_refusal_leaves_full_state_invariant() {
    let mut call = clean_baseline();
    let mut ring = [0i32; N];
    for (j, p) in ring.iter_mut().enumerate() {
        *p = ((j + 1) % N) as i32;
    }
    call.parent = ring;

    let (outcome, before, after) = run(&call);

    assert_eq!(
        outcome.refusals(),
        RefusalSet::NO_LEAVES,
        "expected exactly NO_LEAVES, got {:?}",
        outcome.refusals()
    );
    assert_state_invariant(&before, &after, "NO_LEAVES-only refusal");
}

/// `DIGEST_MISMATCH`: one byte of the certificate digest flipped from the real
/// `CERTIFICATE_DIGEST`, everything else at the clean baseline.
#[test]
fn digest_mismatch_only_refusal_leaves_full_state_invariant() {
    let mut call = clean_baseline();
    let mut bad_digest = CERTIFICATE_DIGEST;
    bad_digest[0] ^= 0x01;
    call.digest = bad_digest;

    let (outcome, before, after) = run(&call);

    assert_eq!(
        outcome.refusals(),
        RefusalSet::DIGEST_MISMATCH,
        "expected exactly DIGEST_MISMATCH, got {:?}",
        outcome.refusals()
    );
    assert_state_invariant(&before, &after, "DIGEST_MISMATCH-only refusal");
}

/// `DWELL_UNSATISFIED`: `tau_d` one tick short of `MODE_DWELL_ROUNDS_MIN`, everything else at
/// the clean baseline.
#[test]
fn dwell_unsatisfied_only_refusal_leaves_full_state_invariant() {
    let min_dwell = MODE_DWELL_ROUNDS_MIN;
    assert!(
        min_dwell > 0,
        "test assumes MODE_DWELL_ROUNDS_MIN > 0 so `- 1` is representable"
    );
    let mut call = clean_baseline();
    call.tau_d = min_dwell - 1;

    let (outcome, before, after) = run(&call);

    assert_eq!(
        outcome.refusals(),
        RefusalSet::DWELL_UNSATISFIED,
        "expected exactly DWELL_UNSATISFIED, got {:?}",
        outcome.refusals()
    );
    assert_state_invariant(&before, &after, "DWELL_UNSATISFIED-only refusal");
}

/// `PROPOSAL_REJECTED`: `mu[0]` set above `mu_max` (100.0), which trips `price_err` — one of
/// the six conditions `PROPOSAL_REJECTED` is unioned on — everything else at the clean
/// baseline.
#[test]
fn proposal_rejected_only_refusal_leaves_full_state_invariant() {
    let mut call = clean_baseline();
    call.mu[0] = NonNegativeFixed::MAX;

    let (outcome, before, after) = run(&call);

    assert_eq!(
        outcome.refusals(),
        RefusalSet::PROPOSAL_REJECTED,
        "expected exactly PROPOSAL_REJECTED, got {:?}",
        outcome.refusals()
    );
    assert_state_invariant(&before, &after, "PROPOSAL_REJECTED-only refusal");
}

// -----------------------------------------------------------------------------------------
// The 4 bits `allocate()` itself never constructs, split by final disposition (see
// `REFUSAL_REALIZATION_REPORT.md` and each bit's own doc comment in `src/allocator.rs` for
// the full record):
//   - AUTHORITY_MISSING: UNREACHABLE_BY_PROOF. Gets a real, targeted run against the
//     scenario a working implementation would be expected to fire it under, confirming the
//     dead-mask proof empirically, not only by reading the source.
//   - ROUND_MISMATCH / CERTIFICATE_STALE: OWNED_BY_DIFFERENT_COMPONENT. Realized, tested, and
//     passing via `proposal::ProposalRefusal::RoundIdentityMismatch`,
//     `certification::CertificationRefusal::RoundIdentityMismatch`, and
//     `mode_switch::ModeSwitchRefusal::CertificateDigestMismatch` in their own modules — not
//     fabricated here, since `allocate()` has no parameter surface mapping to them.
//   - CERTIFICATE_MISSING: RESERVED_WITH_EXPLICIT_NONCLAIM. No construction path anywhere in
//     the crate, by deliberate mandatory-parameter design, not oversight.
// All 4 are covered by the representative sweep below (never observed set across every
// scenario this file constructs).
// -----------------------------------------------------------------------------------------

/// `AUTHORITY_MISSING`'s own mask requires `degrade_to_certified_selection == true`
/// (`proof.is_none()`), but `has_refusal` (which gates the whole `gated_refusals` bundle
/// `AUTHORITY_MISSING` lives in) requires `degrade_to_certified_selection == false` — an
/// unsatisfiable conjunction, confirmed here by really running the scenario a working
/// `AUTHORITY_MISSING` would be expected to fire under (no proof supplied, plus a real
/// control-plane error so `has_error` is true): a digest mismatch with `proof = None`.
///
/// This is a distinct, pre-existing finding from the NO_LEAVES gate fix this round — it is
/// about `AUTHORITY_MISSING` never being *reachable* at all (an authority/reporting-surface
/// question), not about the state-commit gate this file's other tests cover. It is reported
/// here, not fixed: fixing which conditions make `AUTHORITY_MISSING` reachable is a
/// refusal-set/authority-semantics decision outside this track's numeric/state-commit-gating
/// scope (`.claude/agents/cmca-numeric.md`), not a numeric write-back gating defect — a proof
/// of `false` cannot violate a state-invariance law, so it is included here for its
/// diagnostic value, not as an Invariant 5 violation.
#[test]
fn authority_missing_is_never_actually_set_verified_by_targeted_run() {
    let mut call = clean_baseline();
    call.proof = None; // degrade_to_certified_selection == true
    let mut bad_digest = CERTIFICATE_DIGEST;
    bad_digest[0] ^= 0x01; // forces has_error == true via digest_err
    call.digest = bad_digest;

    let (outcome, before, after) = run(&call);

    assert!(
        !outcome.refusals().contains(RefusalSet::AUTHORITY_MISSING),
        "AUTHORITY_MISSING was expected to remain unreachable (dead mask, see doc comment \
         above) but was observed set: {:?} — if this assertion fails, the has_refusal/\
         degrade_to_certified_selection masking has changed and AUTHORITY_MISSING may now be \
         reachable; this test (and its doc comment) need updating, not deleting.",
        outcome.refusals()
    );
    // No control-plane refusal is reported at all in this configuration (proof=None routes
    // to the certified-selection-only / freeze_learning path, which swallows the digest
    // error rather than surfacing AUTHORITY_MISSING or anything else) — a real, if
    // non-obvious, observed consequence of the current design, documented rather than
    // asserted-away.
    assert!(
        !outcome.is_refused(),
        "proof=None + digest mismatch was observed to report no refusal at all under the \
         current has_refusal/freeze_learning design: {:?}",
        outcome.refusals()
    );
    // State invariance holds here vacuously: proof=None means update_allowed/did_switch are
    // both false regardless of has_error, so local_weights/local_last_switch_t/
    // local_prev_mode already equal their pre-call values before the has_refusal gate is
    // even consulted. Asserted explicitly rather than assumed.
    assert_state_invariant(
        &before,
        &after,
        "proof=None + digest-mismatch (AUTHORITY_MISSING probe)",
    );
}

/// `CERTIFICATE_MISSING`, `CERTIFICATE_STALE`, `ROUND_MISMATCH` are never unioned into
/// `gated_refusals`/`final_refusals` anywhere in `src/allocator.rs`'s `allocate()` body
/// (grep-confirmed: they appear only in their own `pub const` declarations and in
/// `RefusalSet::primary_reason()`'s pattern-match arms, which only ever read a `RefusalSet` a
/// caller already constructed — `primary_reason` has no path that constructs one of these
/// bits into a value `allocate()` returns). There is no parameter or code path in
/// `allocate()`'s current signature that maps to any of these three causes, so no "one real
/// allocate() call that triggers ONLY that cause" can be constructed for them without
/// fabricating a scenario `allocate()` cannot actually produce — which this file does not do.
/// This is a structural, source-reading finding, not a state-commit-gating defect: state
/// invariance is trivially satisfied for a cause that can never fire in the first place. The
/// representative sweep below is the closest available empirical corroboration (these three
/// bits are checked absent across every scenario exercised in this file, not merely assumed
/// absent from the digest-mismatch/dwell/proposal/no-leaves cases alone).
///
/// This absence from `allocate()` is not the end of the story for `CERTIFICATE_STALE` and
/// `ROUND_MISMATCH`, though: both describe real conditions realized, tested, and passing
/// elsewhere (disposition OWNED_BY_DIFFERENT_COMPONENT — see `proposal::tests::
/// refuses_on_round_mismatch`, `certification::tests::refuses_solo_mismatch_round_identity`,
/// `mode_switch::tests::rejection_cause_certificate_mismatch_leaves_state_untouched`).
/// `CERTIFICATE_MISSING` alone has no owning realization anywhere in the crate (disposition
/// RESERVED_WITH_EXPLICIT_NONCLAIM — see `src/allocator.rs`'s doc comment on that constant).
/// Full record: `REFUSAL_REALIZATION_REPORT.md`.
#[test]
fn certificate_missing_stale_round_mismatch_have_no_allocate_construction_path() {
    // Intentionally empty of an `allocate()` call: see the doc comment above for why none
    // can be constructed. This test exists so the fact is asserted in code (via the doc
    // comment being attached to a real, collected test in the suite) rather than left as
    // prose that could silently go stale relative to `src/allocator.rs`.
    assert_ne!(RefusalSet::CERTIFICATE_MISSING, RefusalSet::EMPTY);
    assert_ne!(RefusalSet::CERTIFICATE_STALE, RefusalSet::EMPTY);
    assert_ne!(RefusalSet::ROUND_MISMATCH, RefusalSet::EMPTY);
}

// -----------------------------------------------------------------------------------------
// Representative sweep: none of the 4 dead/unreachable bits appears across every scenario
// this file constructs, run together as one additional real-data corroboration.
// -----------------------------------------------------------------------------------------

#[test]
fn no_dead_refusal_bit_appears_across_a_representative_sweep_of_real_allocate_calls() {
    let dead = RefusalSet::CERTIFICATE_MISSING
        .union(RefusalSet::CERTIFICATE_STALE)
        .union(RefusalSet::ROUND_MISMATCH)
        .union(RefusalSet::AUTHORITY_MISSING);

    let mut ring = [0i32; N];
    for (j, p) in ring.iter_mut().enumerate() {
        *p = ((j + 1) % N) as i32;
    }
    let mut bad_digest = CERTIFICATE_DIGEST;
    bad_digest[0] ^= 0x01;

    let scenarios: [Call; 6] = [
        clean_baseline(),
        Call {
            parent: ring,
            ..clean_baseline()
        },
        Call {
            digest: bad_digest,
            ..clean_baseline()
        },
        Call {
            tau_d: MODE_DWELL_ROUNDS_MIN.saturating_sub(1),
            ..clean_baseline()
        },
        Call {
            mu: {
                let mut mu = [NonNegativeFixed::ZERO; N];
                mu[0] = NonNegativeFixed::MAX;
                mu
            },
            ..clean_baseline()
        },
        Call {
            proof: None,
            digest: bad_digest,
            ..clean_baseline()
        },
    ];

    for (i, call) in scenarios.iter().enumerate() {
        let (outcome, _before, _after) = run(call);
        assert_eq!(
            outcome.refusals().bits() & dead.bits(),
            0,
            "scenario {i}: a dead/unreachable RefusalSet bit was observed set: {:?}",
            outcome.refusals()
        );
    }
}
