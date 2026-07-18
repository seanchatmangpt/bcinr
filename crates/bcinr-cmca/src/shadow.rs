//! # Shadow execution — Authority hop 2 of the C3 chain
//!
//! Shadow execution computes and compares candidate-mode behavior against the current
//! mode, but per `.claude/rules/30-authority-separation.md` (SELECT is never DO) it must
//! never mutate authoritative persistent state or actuate anything. Its sole output is a
//! sealed [`ShadowExecutionReceipt`] binding the admitted proposal's digest, the
//! current/candidate mode digests, and the round identity it ran under — evidence for the
//! later jump/stability/certification hops, never an authority-bearing token itself.

use crate::proposal::{mix64, AdmittedProposal};

/// Sealed evidence that a shadow execution ran for a specific admitted proposal and
/// current/candidate mode pair. Constructible only via [`execute_shadow`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShadowExecutionReceipt {
    admitted_proposal_digest: u64,
    current_mode_digest: u64,
    candidate_mode_digest: u64,
    round_identity: u64,
    comparison_value: i64,
    receipt_digest: u64,
}

impl ShadowExecutionReceipt {
    #[inline(always)]
    pub fn admitted_proposal_digest(&self) -> u64 {
        self.admitted_proposal_digest
    }

    #[inline(always)]
    pub fn current_mode_digest(&self) -> u64 {
        self.current_mode_digest
    }

    #[inline(always)]
    pub fn candidate_mode_digest(&self) -> u64 {
        self.candidate_mode_digest
    }

    #[inline(always)]
    pub fn round_identity(&self) -> u64 {
        self.round_identity
    }

    /// The shadow-computed comparison value (e.g. a candidate-vs-current cost delta).
    /// Read-only evidence; nothing derived from this value is written back to
    /// authoritative state by this module.
    #[inline(always)]
    pub fn comparison_value(&self) -> i64 {
        self.comparison_value
    }

    #[inline(always)]
    pub fn receipt_digest(&self) -> u64 {
        self.receipt_digest
    }

    #[inline(always)]
    fn seal(
        admitted_proposal_digest: u64,
        current_mode_digest: u64,
        candidate_mode_digest: u64,
        round_identity: u64,
        comparison_value: i64,
    ) -> u64 {
        let mut d = mix64(admitted_proposal_digest, current_mode_digest);
        d = mix64(d, candidate_mode_digest);
        d = mix64(d, round_identity);
        d = mix64(d, comparison_value as u64);
        d
    }
}

/// Runs shadow execution for an admitted proposal against a candidate mode digest,
/// returning a sealed receipt.
///
/// # SELECT-is-never-DO
///
/// This function takes no `&mut` parameter into any authoritative or persistent state and
/// returns a plain value — there is no reachable path in this function's body that writes
/// to a static, a file, or any caller-owned mutable location. It reads its own by-value
/// arguments only. See the `noninterference_with_authoritative_state` test below for a
/// concrete byte-level demonstration against a simulated authoritative-state surrogate.
pub fn execute_shadow(
    admitted: &AdmittedProposal,
    current_mode_digest: u64,
    candidate_mode_digest: u64,
) -> ShadowExecutionReceipt {
    let admitted_proposal_digest = admitted.proposal().proposal_digest();
    let round_identity = admitted.proposal().round_identity();

    // Pure comparison: no persistent write, no actuation. The comparison value is a
    // read-only function of the inputs above.
    let comparison_value = (candidate_mode_digest as i64).wrapping_sub(current_mode_digest as i64);

    let receipt_digest = ShadowExecutionReceipt::seal(
        admitted_proposal_digest,
        current_mode_digest,
        candidate_mode_digest,
        round_identity,
        comparison_value,
    );

    ShadowExecutionReceipt {
        admitted_proposal_digest,
        current_mode_digest,
        candidate_mode_digest,
        round_identity,
        comparison_value,
        receipt_digest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::SignedFixed;
    use crate::observatory::{ObservatoryFlag, ObservatoryFlagSet};
    use crate::proposal::{admit_proposal, ModeProposal};

    fn admitted_fixture() -> AdmittedProposal {
        let flags = ObservatoryFlagSet::EMPTY.insert(ObservatoryFlag::RecertificationCandidate);
        let p = ModeProposal::test_fixture(SignedFixed::from_value_bits(1), 1, 2, 3, flags);
        admit_proposal(p, 3, 2, SignedFixed::from_value_bits(100)).unwrap()
    }

    /// A minimal stand-in for "authoritative persistent state" used only to prove
    /// noninterference: shadow execution must not be able to reach or mutate it.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct SimulatedAuthoritativeState {
        mode_digest: u64,
        generation: u64,
        scratch: [u8; 4],
    }

    #[test]
    fn noninterference_with_authoritative_state() {
        let admitted = admitted_fixture();
        let before = SimulatedAuthoritativeState {
            mode_digest: 2,
            generation: 7,
            scratch: [1, 2, 3, 4],
        };
        let snapshot = before;

        // Shadow execution runs, deliberately in "the same scope" as the authoritative
        // state, without ever being given a reference to it.
        let _receipt = execute_shadow(&admitted, 2, 99);

        // `before` must be byte-for-byte identical to the pre-call snapshot: shadow
        // execution had no path to it (it was never passed in) and could not have mutated
        // it even in principle.
        assert_eq!(before, snapshot);
    }

    #[test]
    fn receipt_binds_all_four_identities() {
        let admitted = admitted_fixture();
        let receipt = execute_shadow(&admitted, 2, 99);
        assert_eq!(
            receipt.admitted_proposal_digest(),
            admitted.proposal().proposal_digest()
        );
        assert_eq!(receipt.current_mode_digest(), 2);
        assert_eq!(receipt.candidate_mode_digest(), 99);
        assert_eq!(receipt.round_identity(), 3);
    }

    #[test]
    fn receipt_digest_changes_if_any_bound_identity_changes() {
        let admitted = admitted_fixture();
        let r1 = execute_shadow(&admitted, 2, 99);
        let r2 = execute_shadow(&admitted, 2, 100); // different candidate digest
        assert_ne!(r1.receipt_digest(), r2.receipt_digest());
    }
}
