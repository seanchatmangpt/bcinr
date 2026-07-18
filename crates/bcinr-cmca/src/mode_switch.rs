//! # Certified mode switch — Authority hop 6 (final) of the C3 chain
//!
//! [`CertifiedModeSwitch`] is prepared only from an [`AdmittedControlState`], a
//! [`CertifiedLearning`] mode token, and a target-mode digest — it carries no bare boolean
//! authority of its own. Actually *applying* a switch additionally requires a
//! [`DwellSatisfied`] token bound to the same round/transition and a [`CertificateReceipt`]
//! matching the expected certificate digest, verified inside [`apply_mode_switch`]. On
//! rejection, [`apply_mode_switch`] leaves every persistent byte of [`ModeState`] it could
//! have touched completely unchanged — proved for three independent rejection causes in the
//! tests below, per `authority-and-c3.md` Invariant 5.

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
    CertificateDigestMismatch,
    /// The dwell token's bound round/transition identity does not match the identity of
    /// the transition actually being attempted.
    DwellIdentityMismatch,
    /// The switch's admitted-state digest does not match the persistent state's current
    /// mode digest — the world moved on since the switch was prepared.
    StaleAdmittedState,
}

/// Atomically applies a prepared, certified mode switch to persistent `ModeState`.
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
pub fn apply_mode_switch(
    persistent: &mut ModeState,
    switch: CertifiedModeSwitch,
    dwell: DwellSatisfied,
    round_identity: u64,
    transition_identity: u64,
    certificate: CertificateReceipt,
    expected_certificate_digest: u64,
) -> Result<(), ModeSwitchRefusal> {
    let cert_ok = certificate.digest == expected_certificate_digest;
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

    let next = if admitted { candidate } else { *persistent };
    *persistent = next;

    if admitted {
        Ok(())
    } else if !cert_ok {
        Err(ModeSwitchRefusal::CertificateDigestMismatch)
    } else if !dwell_ok {
        Err(ModeSwitchRefusal::DwellIdentityMismatch)
    } else {
        Err(ModeSwitchRefusal::StaleAdmittedState)
    }
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

        let res = apply_mode_switch(&mut state, switch, dwell, 1, 2, cert, 99);
        assert!(res.is_ok());
        assert_eq!(
            state,
            ModeState {
                mode_digest: 20,
                generation: 1
            }
        );
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

        let res = apply_mode_switch(&mut state, switch, dwell, 1, 2, cert, 99);
        assert_eq!(res, Err(ModeSwitchRefusal::CertificateDigestMismatch));
        assert_eq!(
            state, snapshot,
            "persistent state must be byte-for-byte unchanged on rejection"
        );
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

        // Attempted transition identity (1, 999) does not match the dwell token's (1, 2).
        let res = apply_mode_switch(&mut state, switch, dwell, 1, 999, cert, 99);
        assert_eq!(res, Err(ModeSwitchRefusal::DwellIdentityMismatch));
        assert_eq!(
            state, snapshot,
            "persistent state must be byte-for-byte unchanged on rejection"
        );
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

        let res = apply_mode_switch(&mut state, switch, dwell, 1, 2, cert, 99);
        assert_eq!(res, Err(ModeSwitchRefusal::StaleAdmittedState));
        assert_eq!(
            state, snapshot,
            "persistent state must be byte-for-byte unchanged on rejection"
        );
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
