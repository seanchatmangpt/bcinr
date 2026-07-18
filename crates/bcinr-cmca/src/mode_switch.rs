//! # Certified mode switch — Authority hop 6 (final) of the C3 chain
//!
//! [`CertifiedModeSwitch`] is prepared only from an [`AdmittedControlState`], a
//! [`CertifiedLearning`] mode token, and a target-mode digest — it carries no bare boolean
//! authority of its own. Actually *applying* a switch additionally requires a
//! [`DwellSatisfied`] token bound to the same round/transition and a [`CertificateReceipt`]
//! matching an independently supplied expected [`CertificateReceipt`], verified inside
//! [`apply_mode_switch`]. On rejection, [`apply_mode_switch`] leaves every persistent byte of
//! [`ModeState`] it could have touched completely unchanged — proved for three independent
//! rejection causes in the tests below, per `authority-and-c3.md` Invariant 5.
//!
//! ## External-auditor actuation evidence
//!
//! [`apply_mode_switch`] also returns an [`ActuationEvidence`] record — a sealed, purpose-built
//! public artifact (never constructible outside this function) that lets a reader *external to
//! this crate* independently confirm which certified proposal produced which actual state
//! transition, without ever reading `CertificateReceipt`'s private digest field. See
//! [`ActuationEvidence`]'s own doc comment for its precondition/postcondition/nonclaims.

use crate::allocator::{AdmittedControlState, CertificateReceipt, CertifiedLearning};
use crate::certification::DwellSatisfied;
use crate::proposal::mix64;

/// Surrogate persistent mode-state for this stage of the chain. This is intentionally a
/// small, self-contained struct rather than a reach into `bcinr-cmca`'s existing packed
/// allocator state — wiring `CertifiedModeSwitch` into the allocator's actual persistent
/// state (`PackedSemanticState`/`CertifiedLearning` constructor) is the explicitly deferred
/// follow-up phase named in the task description, not this one. All fields are `pub(crate)`
/// so this module's tests can snapshot/diff them directly without `unsafe` byte
/// reinterpretation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModeState {
    pub mode_digest: u64,
    pub generation: u64,
}

/// Sealed proof that a mode switch was prepared from an admitted control state and
/// certified-learning mode token, targeting a specific mode digest. Constructible only via
/// [`prepare_mode_switch`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CertifiedModeSwitch {
    admitted_state_digest: u64,
    target_mode_digest: u64,
    prepared_digest: u64,
}

impl CertifiedModeSwitch {
    #[inline(always)]
    pub fn admitted_state_digest(&self) -> u64 {
        self.admitted_state_digest
    }

    #[inline(always)]
    pub fn target_mode_digest(&self) -> u64 {
        self.target_mode_digest
    }
}

/// Prepares a [`CertifiedModeSwitch`] from an admitted control state, a certified-learning
/// mode token, and the target mode digest. The `CertifiedLearning` parameter is consumed by
/// value purely as a capability proof — this function does not branch on its contents (it
/// is a unit-like sealed marker) and takes no other path to a `CertifiedModeSwitch`.
#[inline(always)]
pub fn prepare_mode_switch(
    state: AdmittedControlState,
    _learning: CertifiedLearning,
    target_mode_digest: u64,
) -> CertifiedModeSwitch {
    let admitted_state_digest = state.digest;
    let prepared_digest = mix64(admitted_state_digest, target_mode_digest);
    CertifiedModeSwitch {
        admitted_state_digest,
        target_mode_digest,
        prepared_digest,
    }
}

/// Refusal reasons for [`apply_mode_switch`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModeSwitchRefusal {
    /// The certificate's digest does not match the expected certificate digest for this
    /// switch.
    ///
    /// This is one of the owning realizations of `allocator::RefusalSet::CERTIFICATE_STALE`
    /// (a certificate that was valid when sealed but has since been superseded surfaces
    /// here as a digest mismatch against the freshly-expected certificate) — `allocate()`
    /// never constructs that bit itself; see `REFUSAL_REALIZATION_REPORT.md` for the full
    /// reconciliation.
    CertificateDigestMismatch,
    /// The dwell token's bound round/transition identity does not match the identity of
    /// the transition actually being attempted.
    DwellIdentityMismatch,
    /// The switch's admitted-state digest does not match the persistent state's current
    /// mode digest — the world moved on since the switch was prepared.
    StaleAdmittedState,
}

/// Outcome discriminant carried by [`ActuationEvidence`]: whether the attempted switch was
/// actually applied, or refused (naming exactly which of [`ModeSwitchRefusal`]'s named laws
/// was violated). This is a real enum discriminant, not a collapsed boolean — a refusal
/// carries its specific typed reason, per `authority-and-c3.md` Invariant 2's "never a lossy
/// projection" spirit applied to actuation outcomes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActuationOutcome {
    /// The switch was admitted; `persistent`'s mode digest and generation were updated.
    Applied,
    /// The switch was refused for the named reason; `persistent` was left byte-for-byte
    /// unchanged (see [`apply_mode_switch`]'s masked-commit law).
    Refused(ModeSwitchRefusal),
}

impl ActuationOutcome {
    /// `true` iff the switch was actually applied.
    #[inline(always)]
    pub fn is_applied(&self) -> bool {
        matches!(self, ActuationOutcome::Applied)
    }

    /// `Some(reason)` iff the switch was refused; `None` iff it was applied.
    #[inline(always)]
    pub fn refusal(&self) -> Option<ModeSwitchRefusal> {
        match self {
            ActuationOutcome::Applied => None,
            ActuationOutcome::Refused(reason) => Some(*reason),
        }
    }
}

/// A bounded, purpose-built public evidence artifact minted at the actuation boundary,
/// closing the external-auditor gap named in `tests/jtbd_auditable_adaptive_policy.rs`:
/// `CertificateReceipt`'s digest field is `pub(crate)` with no public getter (by design — see
/// `authority-and-c3.md`'s sealing invariants), so a reader outside `bcinr-cmca` has no other
/// safe-Rust way to learn which certificate actuated which real state transition.
/// `ActuationEvidence` reports exactly the four digests and the outcome discriminant needed to
/// answer that one question — nothing about `ModeState`'s internal representation, nothing
/// about `DwellSatisfied` or `CertifiedModeSwitch`'s internal digests, and no general digest
/// getter on `CertificateReceipt` itself.
///
/// # Hoare contract
///
/// **Precondition:** an `ActuationEvidence` value exists only because [`apply_mode_switch`]
/// actually ran to completion on some real `(persistent, switch, dwell, certificate,
/// expected_certificate)` tuple. There is no other safe-Rust constructor, public or
/// crate-internal — see the `tests/ui/*actuation_evidence*` compile-fail suite.
///
/// **Postcondition:** the returned value's digests are exactly those of the transition that
/// actually occurred (on [`ActuationOutcome::Applied`]) or was actually attempted (on
/// [`ActuationOutcome::Refused`]):
/// - [`certificate_digest`](Self::certificate_digest) is exactly the `digest` field of the
///   `certificate: CertificateReceipt` argument [`apply_mode_switch`] was actually called
///   with — the receipt presented for this attempt, regardless of outcome.
/// - [`old_control_mode_digest`](Self::old_control_mode_digest) is exactly `persistent`'s
///   `mode_digest` *before* the call.
/// - [`new_control_mode_digest`](Self::new_control_mode_digest) is exactly `switch`'s
///   `target_mode_digest` — the digest actually written to `persistent.mode_digest` on
///   `Applied`, or the digest that was attempted and refused on `Refused`.
/// - [`round_identity`](Self::round_identity) is exactly the `round_identity` argument
///   [`apply_mode_switch`] was called with.
/// - [`outcome`](Self::outcome) names `Applied` iff `persistent` was actually mutated, or
///   `Refused(reason)` naming the exact [`ModeSwitchRefusal`] variant that fired.
///
/// # Nonclaims
///
/// - This type does **not** itself re-verify the certificate, the dwell token, or the
///   admitted-state digest — it *reports* what [`apply_mode_switch`]'s own admission checks
///   already computed. An `ActuationEvidence` with `outcome() == Applied` is only as trustworthy
///   as the caller's own inputs to `apply_mode_switch` (a caller can self-fabricate a
///   `CertificateReceipt`/`AdmittedControlState`/`DwellSatisfied` via their own public
///   constructors today, exactly as it could before this type existed); this type adds
///   *legibility* of a real actuation attempt's outcome to external readers, not additional
///   forgery-resistance beyond what those upstream sealed types already provide.
/// - An external holder of an `ActuationEvidence` cannot forge a *new* evidence record from it,
///   combine two records into a third, or mint one for a transition that was never attempted —
///   the type is `Copy`/`Clone`/`PartialEq`/`Eq` only; it derives no constructor from its own
///   data. It can only be read, moved, and compared.
/// - This type makes no claim about long-term storage, retention, or replay after process
///   restart — it describes the one `apply_mode_switch` call that produced it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ActuationEvidence {
    certificate_digest: u64,
    old_control_mode_digest: u64,
    new_control_mode_digest: u64,
    round_identity: u64,
    outcome: ActuationOutcome,
}

impl ActuationEvidence {
    /// Crate-internal constructor. Deliberately **not** `pub(crate)`: the only call site is
    /// inside [`apply_mode_switch`] in this same module, tighter than every other sealed type
    /// in this chain (which use `pub(crate)` because a registry/orchestrator module elsewhere
    /// in the crate legitimately constructs them). `ActuationEvidence` has no such legitimate
    /// producer other than the actuation boundary itself.
    #[inline(always)]
    fn new(
        certificate_digest: u64,
        old_control_mode_digest: u64,
        new_control_mode_digest: u64,
        round_identity: u64,
        outcome: ActuationOutcome,
    ) -> Self {
        Self {
            certificate_digest,
            old_control_mode_digest,
            new_control_mode_digest,
            round_identity,
            outcome,
        }
    }

    /// The digest of the `CertificateReceipt` presented to [`apply_mode_switch`] for this
    /// attempt (regardless of whether it matched). An external reader can re-derive a
    /// candidate's own certificate via the real `certification::seal_certificate` and compare
    /// it to this evidence by wrapping this digest back into a `CertificateReceipt` via the
    /// existing public `CertificateReceipt::admit_certificate` constructor and comparing with
    /// `==` — the same disambiguation idiom already used in
    /// `tests/jtbd_auditable_adaptive_policy.rs` for the certification hop.
    #[inline(always)]
    pub fn certificate_digest(&self) -> u64 {
        self.certificate_digest
    }

    /// `persistent.mode_digest` immediately before this actuation attempt.
    #[inline(always)]
    pub fn old_control_mode_digest(&self) -> u64 {
        self.old_control_mode_digest
    }

    /// The mode digest this attempt targeted — written to persistent state on
    /// [`ActuationOutcome::Applied`], or the digest that was attempted and refused otherwise.
    #[inline(always)]
    pub fn new_control_mode_digest(&self) -> u64 {
        self.new_control_mode_digest
    }

    /// The round identity this actuation attempt was made under.
    #[inline(always)]
    pub fn round_identity(&self) -> u64 {
        self.round_identity
    }

    /// Whether the switch was applied, or refused and why.
    #[inline(always)]
    pub fn outcome(&self) -> ActuationOutcome {
        self.outcome
    }
}

/// Atomically applies a prepared, certified mode switch to persistent `ModeState`, and returns
/// an [`ActuationEvidence`] record of the attempt alongside the existing `Result`.
///
/// # Masked-commit law (AGENTS.md §10)
///
/// The candidate next-state is computed fully before any admission check runs. Every
/// admission predicate is combined into a single boolean `admitted`. The actual write is a
/// masked select between the candidate and the untouched current state — `persistent` is
/// written exactly once, with either the candidate (if `admitted`) or a value structurally
/// identical to its own prior contents (if not). On any single rejection cause,
/// `*persistent` is therefore left byte-for-byte (here: field-for-field, since this crate
/// forbids unsafe code and there is no `bytemuck`/transmute dependency to compare raw
/// bytes) identical to its pre-call value — proved per rejection cause in the tests below.
///
/// # External-auditor closure
///
/// The certificate check compares the presented `certificate` against an independently
/// supplied `expected_certificate` — both full `CertificateReceipt` values, compared with the
/// receipt's own derived `==` — rather than requiring the caller to supply a raw expected
/// digest (`u64`) extracted from a receipt's private field. A caller outside this crate can
/// legitimately obtain a `CertificateReceipt` to pass as `expected_certificate` by calling the
/// real, public `certification::seal_certificate` again with the same inputs it used to obtain
/// `certificate` in the first place (deterministic, so the two receipts compare equal) —
/// exactly the "re-derive via the real production function, compare with `==`" idiom
/// `tests/jtbd_auditable_adaptive_policy.rs` already uses one hop earlier for the certification
/// step itself. This is the change that makes it possible for that test to drive
/// `apply_mode_switch` at all from outside the crate; see that file's module doc for the
/// closed finding.
pub fn apply_mode_switch(
    persistent: &mut ModeState,
    switch: CertifiedModeSwitch,
    dwell: DwellSatisfied,
    round_identity: u64,
    transition_identity: u64,
    certificate: CertificateReceipt,
    expected_certificate: CertificateReceipt,
) -> (Result<(), ModeSwitchRefusal>, ActuationEvidence) {
    let cert_ok = certificate == expected_certificate;
    let dwell_ok = dwell.round_identity() == round_identity
        && dwell.transition_identity() == transition_identity;
    let state_ok = switch.admitted_state_digest == persistent.mode_digest;

    let admitted = cert_ok && dwell_ok && state_ok;

    // Candidate is computed unconditionally (no branch gates its computation), per the
    // masked-commit law: "compute the candidate structurally," then select.
    let candidate = ModeState {
        mode_digest: switch.target_mode_digest,
        generation: persistent.generation.wrapping_add(1),
    };

    let old_control_mode_digest = persistent.mode_digest;
    let next = if admitted { candidate } else { *persistent };
    *persistent = next;

    let result = if admitted {
        Ok(())
    } else if !cert_ok {
        Err(ModeSwitchRefusal::CertificateDigestMismatch)
    } else if !dwell_ok {
        Err(ModeSwitchRefusal::DwellIdentityMismatch)
    } else {
        Err(ModeSwitchRefusal::StaleAdmittedState)
    };

    let outcome = match result {
        Ok(()) => ActuationOutcome::Applied,
        Err(reason) => ActuationOutcome::Refused(reason),
    };

    let evidence = ActuationEvidence::new(
        certificate.digest,
        old_control_mode_digest,
        switch.target_mode_digest,
        round_identity,
        outcome,
    );

    (result, evidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certification::observe_dwell;

    fn admitted_state(digest: u64) -> AdmittedControlState {
        AdmittedControlState::admit_control_state(digest)
    }

    fn prepared_switch(current_digest: u64, target_digest: u64) -> CertifiedModeSwitch {
        prepare_mode_switch(
            admitted_state(current_digest),
            CertifiedLearning::admit_learning(),
            target_digest,
        )
    }

    #[test]
    fn applies_when_every_binding_matches() {
        let mut state = ModeState {
            mode_digest: 10,
            generation: 0,
        };
        let switch = prepared_switch(10, 20);
        let dwell = observe_dwell(1, 2, 10, 10).unwrap();
        let cert = CertificateReceipt::admit_certificate(99);
        let expected = CertificateReceipt::admit_certificate(99);

        let (res, evidence) = apply_mode_switch(&mut state, switch, dwell, 1, 2, cert, expected);
        assert!(res.is_ok());
        assert_eq!(
            state,
            ModeState {
                mode_digest: 20,
                generation: 1
            }
        );
        // The evidence artifact independently reports the same transition, using only its
        // own public accessors.
        assert_eq!(evidence.outcome(), ActuationOutcome::Applied);
        assert!(evidence.outcome().is_applied());
        assert_eq!(evidence.outcome().refusal(), None);
        assert_eq!(evidence.certificate_digest(), 99);
        assert_eq!(evidence.old_control_mode_digest(), 10);
        assert_eq!(evidence.new_control_mode_digest(), 20);
        assert_eq!(evidence.round_identity(), 1);
    }

    #[test]
    fn rejection_cause_certificate_mismatch_leaves_state_untouched() {
        let mut state = ModeState {
            mode_digest: 10,
            generation: 0,
        };
        let snapshot = state;
        let switch = prepared_switch(10, 20);
        let dwell = observe_dwell(1, 2, 10, 10).unwrap();
        let cert = CertificateReceipt::admit_certificate(1); // wrong digest
        let expected = CertificateReceipt::admit_certificate(99);

        let (res, evidence) = apply_mode_switch(&mut state, switch, dwell, 1, 2, cert, expected);
        assert_eq!(res, Err(ModeSwitchRefusal::CertificateDigestMismatch));
        assert_eq!(
            state, snapshot,
            "persistent state must be byte-for-byte unchanged on rejection"
        );
        assert_eq!(
            evidence.outcome(),
            ActuationOutcome::Refused(ModeSwitchRefusal::CertificateDigestMismatch)
        );
        assert!(!evidence.outcome().is_applied());
        assert_eq!(
            evidence.outcome().refusal(),
            Some(ModeSwitchRefusal::CertificateDigestMismatch)
        );
        // certificate_digest reports the PRESENTED certificate (the wrong one), not the
        // expected one — evidence of what was actually attempted, not what was hoped for.
        assert_eq!(evidence.certificate_digest(), 1);
        // old_control_mode_digest reports the real pre-attempt state (unchanged, per the
        // masked-commit law); new_control_mode_digest still names the target that was
        // attempted and refused — an auditor can tell these apart precisely because Applied
        // is the only outcome where the persistent state's digest actually becomes
        // new_control_mode_digest.
        assert_eq!(evidence.old_control_mode_digest(), 10);
        assert_eq!(evidence.new_control_mode_digest(), 20);
    }

    #[test]
    fn rejection_cause_dwell_mismatch_leaves_state_untouched() {
        let mut state = ModeState {
            mode_digest: 10,
            generation: 0,
        };
        let snapshot = state;
        let switch = prepared_switch(10, 20);
        let dwell = observe_dwell(1, 2, 10, 10).unwrap();
        let cert = CertificateReceipt::admit_certificate(99);
        let expected = CertificateReceipt::admit_certificate(99);

        // Attempted transition identity (1, 999) does not match the dwell token's (1, 2).
        let (res, evidence) =
            apply_mode_switch(&mut state, switch, dwell, 1, 999, cert, expected);
        assert_eq!(res, Err(ModeSwitchRefusal::DwellIdentityMismatch));
        assert_eq!(
            state, snapshot,
            "persistent state must be byte-for-byte unchanged on rejection"
        );
        assert_eq!(
            evidence.outcome(),
            ActuationOutcome::Refused(ModeSwitchRefusal::DwellIdentityMismatch)
        );
        assert_eq!(evidence.round_identity(), 1);
        assert_eq!(evidence.old_control_mode_digest(), 10);
        assert_eq!(evidence.new_control_mode_digest(), 20);
    }

    #[test]
    fn rejection_cause_stale_admitted_state_leaves_state_untouched() {
        let mut state = ModeState {
            mode_digest: 10,
            generation: 0,
        };
        let snapshot = state;
        // Switch prepared against a state digest of 999, but persistent's current digest
        // is 10 — stale.
        let switch = prepared_switch(999, 20);
        let dwell = observe_dwell(1, 2, 10, 10).unwrap();
        let cert = CertificateReceipt::admit_certificate(99);
        let expected = CertificateReceipt::admit_certificate(99);

        let (res, evidence) = apply_mode_switch(&mut state, switch, dwell, 1, 2, cert, expected);
        assert_eq!(res, Err(ModeSwitchRefusal::StaleAdmittedState));
        assert_eq!(
            state, snapshot,
            "persistent state must be byte-for-byte unchanged on rejection"
        );
        assert_eq!(
            evidence.outcome(),
            ActuationOutcome::Refused(ModeSwitchRefusal::StaleAdmittedState)
        );
        // The evidence reports the REAL pre-attempt persistent digest (10), not the switch's
        // stale `admitted_state_digest` (999) it was prepared against — old_control_mode_digest
        // always describes the actual `persistent` snapshot, never the switch's internal claim.
        assert_eq!(evidence.old_control_mode_digest(), 10);
        assert_eq!(evidence.new_control_mode_digest(), 20);
    }

    #[test]
    fn expected_certificate_can_be_independently_rederived_and_still_match() {
        // Mirrors the external-auditor idiom: a caller need not hold the SAME CertificateReceipt
        // value twice — it can independently construct an equal one (here, standing in for a
        // fresh `seal_certificate` replay) and the equality check still succeeds, because
        // `CertificateReceipt`'s `PartialEq` is structural, not identity-based.
        let mut state = ModeState {
            mode_digest: 10,
            generation: 0,
        };
        let switch = prepared_switch(10, 20);
        let dwell = observe_dwell(1, 2, 10, 10).unwrap();
        let presented = CertificateReceipt::admit_certificate(77);
        let independently_rederived = CertificateReceipt::admit_certificate(77);

        let (res, evidence) = apply_mode_switch(
            &mut state,
            switch,
            dwell,
            1,
            2,
            presented,
            independently_rederived,
        );
        assert!(res.is_ok());
        assert!(evidence.outcome().is_applied());
        assert_eq!(evidence.certificate_digest(), 77);
    }

    #[test]
    fn dwell_cannot_be_forged_for_a_different_round_or_transition() {
        // observe_dwell binds exactly the round/transition passed to it; there is no safe
        // way to relabel an existing DwellSatisfied to a different pair.
        let dwell = observe_dwell(1, 2, 10, 10).unwrap();
        assert_eq!(dwell.round_identity(), 1);
        assert_eq!(dwell.transition_identity(), 2);
        assert!(observe_dwell(1, 2, 9, 10).is_none());
    }
}
