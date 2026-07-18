//! # Jump analysis — Authority hop 3 of the C3 chain
//!
//! Classifies a shadow-execution receipt's comparison outcome into one of three
//! mutually-exclusive jump categories: a deliberate policy-driven jump, a jump internal to
//! the fixed-point/state trajectory itself, or a switching disturbance (transient noise
//! from the act of switching, not a "real" jump in either sense). This is analysis only —
//! it never actuates a switch and never mints a certificate.

use crate::proposal::mix64;
use crate::shadow::ShadowExecutionReceipt;

/// The three mutually-exclusive jump classifications this stage can produce.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JumpKind {
    /// The observed change is attributable to a deliberate policy/control decision (the
    /// proposal itself), not to internal state dynamics.
    PolicyJump,
    /// The observed change is attributable to the fixed-point trajectory's own internal
    /// state evolving, independent of the proposed policy delta.
    FixedPointStateJump,
    /// The observed change is attributable to switching disturbance — transient noise
    /// introduced by the act of switching mode, distinct from either of the above.
    SwitchingDisturbance,
}

/// Sealed classification of a shadow-execution receipt's jump behavior. Constructible only
/// via [`analyze_jump`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct JumpAnalysisReceipt {
    kind: JumpKind,
    shadow_receipt_digest: u64,
    magnitude: i64,
    analysis_digest: u64,
}

impl JumpAnalysisReceipt {
    #[inline(always)]
    pub fn kind(self) -> JumpKind {
        self.kind
    }

    #[inline(always)]
    pub fn shadow_receipt_digest(self) -> u64 {
        self.shadow_receipt_digest
    }

    #[inline(always)]
    pub fn magnitude(self) -> i64 {
        self.magnitude
    }

    #[inline(always)]
    pub fn analysis_digest(self) -> u64 {
        self.analysis_digest
    }

    #[inline(always)]
    fn seal(kind: JumpKind, shadow_receipt_digest: u64, magnitude: i64) -> u64 {
        let kind_tag = match kind {
            JumpKind::PolicyJump => 0u64,
            JumpKind::FixedPointStateJump => 1u64,
            JumpKind::SwitchingDisturbance => 2u64,
        };
        mix64(mix64(shadow_receipt_digest, kind_tag), magnitude as u64)
    }
}

/// Classifies the jump behavior evidenced by a [`ShadowExecutionReceipt`].
///
/// Classification rule (documented, not asserted from thin air):
/// - if `|comparison_value| <= switching_noise_bound`, it is a [`JumpKind::SwitchingDisturbance`]
///   — the change is within the expected transient noise band of switching itself;
/// - otherwise, if `proposed_delta_magnitude >= comparison_magnitude / 2`, the observed
///   change is at least half explained by the proposed policy delta, so it is a
///   [`JumpKind::PolicyJump`];
/// - otherwise the change is dominated by something other than the proposed delta, so it is
///   a [`JumpKind::FixedPointStateJump`].
pub fn analyze_jump(
    shadow: &ShadowExecutionReceipt,
    proposed_delta_magnitude: i64,
    switching_noise_bound: i64,
) -> JumpAnalysisReceipt {
    let magnitude = shadow.comparison_value();
    let abs_magnitude = magnitude.unsigned_abs() as i64;

    let kind = if abs_magnitude <= switching_noise_bound {
        JumpKind::SwitchingDisturbance
    } else if proposed_delta_magnitude.unsigned_abs() as i64 * 2 >= abs_magnitude {
        JumpKind::PolicyJump
    } else {
        JumpKind::FixedPointStateJump
    };

    let analysis_digest = JumpAnalysisReceipt::seal(kind, shadow.receipt_digest(), magnitude);

    JumpAnalysisReceipt {
        kind,
        shadow_receipt_digest: shadow.receipt_digest(),
        magnitude,
        analysis_digest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::SignedFixed;
    use crate::observatory::{ObservatoryFlag, ObservatoryFlagSet};
    use crate::proposal::{admit_proposal, ModeProposal};
    use crate::shadow::execute_shadow;

    fn shadow_fixture(current: u64, candidate: u64) -> ShadowExecutionReceipt {
        let flags = ObservatoryFlagSet::EMPTY.insert(ObservatoryFlag::RecertificationCandidate);
        let p = ModeProposal::test_fixture(SignedFixed::from_value_bits(1), 1, current, 3, flags);
        let admitted = admit_proposal(p, 3, current, SignedFixed::from_value_bits(100)).unwrap();
        execute_shadow(&admitted, current, candidate)
    }

    #[test]
    fn small_delta_within_noise_band_is_switching_disturbance() {
        let shadow = shadow_fixture(100, 102); // comparison_value == 2
        let r = analyze_jump(&shadow, 1, 5);
        assert_eq!(r.kind(), JumpKind::SwitchingDisturbance);
    }

    #[test]
    fn large_delta_explained_by_policy_is_policy_jump() {
        let shadow = shadow_fixture(100, 200); // comparison_value == 100
        let r = analyze_jump(&shadow, 80, 5); // 2*80=160 >= 100
        assert_eq!(r.kind(), JumpKind::PolicyJump);
    }

    #[test]
    fn large_delta_not_explained_by_policy_is_fixed_point_state_jump() {
        let shadow = shadow_fixture(100, 200); // comparison_value == 100
        let r = analyze_jump(&shadow, 1, 5); // 2*1=2 < 100
        assert_eq!(r.kind(), JumpKind::FixedPointStateJump);
    }

    #[test]
    fn categories_are_mutually_exclusive_across_the_full_boundary() {
        let shadow = shadow_fixture(0, 10); // comparison_value == 10
        assert_eq!(
            analyze_jump(&shadow, 0, 10).kind(),
            JumpKind::SwitchingDisturbance
        );
        assert_eq!(
            analyze_jump(&shadow, 0, 9).kind(),
            JumpKind::FixedPointStateJump
        );
        assert_eq!(analyze_jump(&shadow, 5, 9).kind(), JumpKind::PolicyJump);
    }

    #[test]
    fn analysis_digest_binds_shadow_receipt_and_kind() {
        let shadow = shadow_fixture(100, 200);
        let r1 = analyze_jump(&shadow, 80, 5);
        let r2 = analyze_jump(&shadow, 1, 5);
        assert_ne!(r1.analysis_digest(), r2.analysis_digest());
    }
}
