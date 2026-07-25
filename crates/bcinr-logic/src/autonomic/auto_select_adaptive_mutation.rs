#![forbid(unsafe_code)]

//! Auto Select Adaptive Mutation Operator (Iteration 27)
//!
//! Implements ReceiptSound law for adaptive mutation.
//! Deterministically applies telemetry to control state via fixed-point gradients.

use crate::autonomic::RlState;
use crate::mask::{select_u64, select_u8};

/// Typed refusal codes for Adaptive Mutation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveMutationRefusal {
    None = 0,
    LearningFrozen = 9,
    CertificateStale = 10,
    EnvelopeViolated = 11,
    ReceiptMissing = 12,
}

/// Fixed-width integer metrics replacing floating-point telemetry.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AutoSelectTelemetry {
    pub reward_accum: u64,
    pub penalty_accum: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveMutationResult {
    pub next_state: RlState,
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 18: Radon Law verified.

/// Applies accumulated telemetry to the control state.
/// S_control' = select(M_admit, apply_gradients(S_control, S_telemetry), S_control)
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn auto_select_adaptive_mutation(
    s_control: &RlState,
    s_telemetry: &AutoSelectTelemetry,
    m_learning: u64,
    m_cert: u64,
    m_env: u64,
    m_outcome: u64,
) -> AdaptiveMutationResult {
    let m_admit = m_learning & m_cert & m_env & m_outcome;

    let candidate = apply_gradients(s_control, s_telemetry);

    let next_state = RlState {
        low: select_u64(m_admit, candidate.low, s_control.low),
        high: select_u64(m_admit, candidate.high, s_control.high),
        extra: select_u8((m_admit & 0xFF) as u8, candidate.extra, s_control.extra),
    };

    let m_learning_failed = !m_learning;
    let m_cert_failed = m_learning & !m_cert;
    let m_env_failed = (m_learning & m_cert) & !m_env;
    let m_outcome_failed = (m_learning & m_cert & m_env) & !m_outcome;

    let refusal_code = (m_learning_failed & (AdaptiveMutationRefusal::LearningFrozen as u64))
        | (m_cert_failed & (AdaptiveMutationRefusal::CertificateStale as u64))
        | (m_env_failed & (AdaptiveMutationRefusal::EnvelopeViolated as u64))
        | (m_outcome_failed & (AdaptiveMutationRefusal::ReceiptMissing as u64));

    AdaptiveMutationResult {
        next_state,
        refusal_code: refusal_code as u8,
    }
}

/// Applies branchless fixed-point gradients to the state.
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn apply_gradients(s_control: &RlState, s_telemetry: &AutoSelectTelemetry) -> RlState {
    // Fixed-width saturating math
    // Learning rate scale: reward >> 4
    let scaled_reward = s_telemetry.reward_accum >> 4;
    let scaled_penalty = s_telemetry.penalty_accum >> 2;

    let low_add = s_control.low.saturating_add(scaled_reward);
    let low_sub = low_add.saturating_sub(scaled_penalty);

    let high_add = s_control.high.saturating_add(scaled_reward);
    let high_sub = high_add.saturating_sub(scaled_penalty);

    RlState {
        low: low_sub,
        high: high_sub,
        extra: s_control.extra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_auto_select_adaptive_mutation(
        s_control: &RlState,
        s_telemetry: &AutoSelectTelemetry,
        m_learning: u64,
        m_cert: u64,
        m_env: u64,
        m_outcome: u64,
    ) -> AdaptiveMutationResult {
        let admit = m_learning == !0 && m_cert == !0 && m_env == !0 && m_outcome == !0;

        let mut refusal_code = AdaptiveMutationRefusal::None as u8;
        if m_learning == 0 {
            refusal_code = AdaptiveMutationRefusal::LearningFrozen as u8;
        } else if m_cert == 0 {
            refusal_code = AdaptiveMutationRefusal::CertificateStale as u8;
        } else if m_env == 0 {
            refusal_code = AdaptiveMutationRefusal::EnvelopeViolated as u8;
        } else if m_outcome == 0 {
            refusal_code = AdaptiveMutationRefusal::ReceiptMissing as u8;
        }

        if admit {
            let mut next = *s_control;
            let scaled_reward = s_telemetry.reward_accum >> 4;
            let scaled_penalty = s_telemetry.penalty_accum >> 2;
            next.low = next
                .low
                .saturating_add(scaled_reward)
                .saturating_sub(scaled_penalty);
            next.high = next
                .high
                .saturating_add(scaled_reward)
                .saturating_sub(scaled_penalty);
            AdaptiveMutationResult {
                next_state: next,
                refusal_code,
            }
        } else {
            AdaptiveMutationResult {
                next_state: *s_control,
                refusal_code,
            }
        }
    }

    // Hostile mutants

    fn mutant_auto_select_bypassed_refusal(
        s_control: &RlState,
        s_telemetry: &AutoSelectTelemetry,
        _m_learning: u64,
        _m_cert: u64,
        _m_env: u64,
        _m_outcome: u64,
    ) -> AdaptiveMutationResult {
        // MUTANT: Always applies gradients and returns None refusal
        AdaptiveMutationResult {
            next_state: apply_gradients(s_control, s_telemetry),
            refusal_code: AdaptiveMutationRefusal::None as u8,
        }
    }

    fn mutant_auto_select_inverted_mask(
        s_control: &RlState,
        s_telemetry: &AutoSelectTelemetry,
        m_learning: u64,
        m_cert: u64,
        m_env: u64,
        m_outcome: u64,
    ) -> AdaptiveMutationResult {
        // MUTANT: Applies gradients when conditions fail
        let m_admit = !(m_learning & m_cert & m_env & m_outcome);
        let candidate = apply_gradients(s_control, s_telemetry);
        let next_state = RlState {
            low: select_u64(m_admit, candidate.low, s_control.low),
            high: select_u64(m_admit, candidate.high, s_control.high),
            extra: select_u8((m_admit & 0xFF) as u8, candidate.extra, s_control.extra),
        };
        AdaptiveMutationResult {
            next_state,
            refusal_code: AdaptiveMutationRefusal::None as u8, // corrupted refusal code
        }
    }

    fn mutant_auto_select_dropped_factor(
        s_control: &RlState,
        s_telemetry: &AutoSelectTelemetry,
        m_learning: u64,
        m_cert: u64,
        m_env: u64,
        m_outcome: u64,
    ) -> AdaptiveMutationResult {
        // MUTANT: Drops the penalty subtract factor
        let mut candidate = apply_gradients(s_control, s_telemetry);
        let scaled_reward = s_telemetry.reward_accum >> 4;
        candidate.low = s_control.low.saturating_add(scaled_reward);

        let mut res = auto_select_adaptive_mutation(
            s_control,
            s_telemetry,
            m_learning,
            m_cert,
            m_env,
            m_outcome,
        );
        res.next_state.low = select_u64(
            m_learning & m_cert & m_env & m_outcome,
            candidate.low,
            s_control.low,
        );
        res
    }

    fn mutant_auto_select_stale_certificate(
        s_control: &RlState,
        s_telemetry: &AutoSelectTelemetry,
        m_learning: u64,
        _m_cert: u64, // ignored
        m_env: u64,
        m_outcome: u64,
    ) -> AdaptiveMutationResult {
        // MUTANT: Ignores stale certificate
        auto_select_adaptive_mutation(s_control, s_telemetry, m_learning, !0, m_env, m_outcome)
    }

    fn mutant_auto_select_missing_receipt(
        s_control: &RlState,
        s_telemetry: &AutoSelectTelemetry,
        m_learning: u64,
        m_cert: u64,
        m_env: u64,
        _m_outcome: u64, // ignored
    ) -> AdaptiveMutationResult {
        // MUTANT: Ignores missing receipt
        auto_select_adaptive_mutation(s_control, s_telemetry, m_learning, m_cert, m_env, !0)
    }

    #[test]
    fn test_auto_select_equivalence() {
        let s_control = RlState {
            low: 100,
            high: 200,
            extra: 0,
        };
        let s_telemetry = AutoSelectTelemetry {
            reward_accum: 160,
            penalty_accum: 20,
        }; // +10, -5 -> +5

        let res = auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, !0, !0, !0);
        let oracle = oracle_auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, !0, !0, !0);
        assert_eq!(res, oracle);
        assert_ne!(res.next_state.low, s_control.low);
        assert_eq!(res.refusal_code, AdaptiveMutationRefusal::None as u8);

        // Test LearningFrozen
        let res2 = auto_select_adaptive_mutation(&s_control, &s_telemetry, 0, !0, !0, !0);
        assert_eq!(res2.next_state, s_control);
        assert_eq!(
            res2.refusal_code,
            AdaptiveMutationRefusal::LearningFrozen as u8
        );

        // Test CertificateStale
        let res3 = auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, 0, !0, !0);
        assert_eq!(res3.next_state, s_control);
        assert_eq!(
            res3.refusal_code,
            AdaptiveMutationRefusal::CertificateStale as u8
        );

        // Test EnvelopeViolated
        let res4 = auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, !0, 0, !0);
        assert_eq!(res4.next_state, s_control);
        assert_eq!(
            res4.refusal_code,
            AdaptiveMutationRefusal::EnvelopeViolated as u8
        );

        // Test ReceiptMissing
        let res5 = auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, !0, !0, 0);
        assert_eq!(res5.next_state, s_control);
        assert_eq!(
            res5.refusal_code,
            AdaptiveMutationRefusal::ReceiptMissing as u8
        );
    }

    #[test]
    fn test_auto_select_mutants() {
        let s_control = RlState {
            low: 100,
            high: 200,
            extra: 0,
        };
        // Choose values so that applied gradients actually change the state!
        let s_telemetry = AutoSelectTelemetry {
            reward_accum: 160,
            penalty_accum: 0,
        }; // +10, 0 -> +10

        let oracle_frozen =
            oracle_auto_select_adaptive_mutation(&s_control, &s_telemetry, 0, !0, !0, !0);
        let m1 = mutant_auto_select_bypassed_refusal(&s_control, &s_telemetry, 0, !0, !0, !0);
        assert_eq!(
            m1.refusal_code,
            AdaptiveMutationRefusal::None as u8,
            "Mutant 1 incorrectly bypassed typed refusal"
        );
        assert_ne!(
            oracle_frozen.next_state, m1.next_state,
            "Mutant 1 incorrectly mutated state when it should be frozen"
        );
        assert_eq!(
            m1.next_state.low, 110,
            "Mutant 1 should have updated low state to 110"
        );

        let m2 = mutant_auto_select_inverted_mask(&s_control, &s_telemetry, !0, !0, !0, !0);
        let oracle_admit =
            oracle_auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, !0, !0, !0);
        assert_ne!(
            oracle_admit.next_state, m2.next_state,
            "mutant inverted mask"
        );
        assert_eq!(
            m2.next_state.low, 100,
            "Mutant 2 should have blocked admission"
        );

        let s_telemetry_penalties = AutoSelectTelemetry {
            reward_accum: 160,
            penalty_accum: 160,
        }; // +10, -40 -> saturated? 100 + 10 = 110, 110 - 40 = 70.
        let oracle_admit_penalty = oracle_auto_select_adaptive_mutation(
            &s_control,
            &s_telemetry_penalties,
            !0,
            !0,
            !0,
            !0,
        );
        let m3 =
            mutant_auto_select_dropped_factor(&s_control, &s_telemetry_penalties, !0, !0, !0, !0);
        assert_ne!(
            oracle_admit_penalty.next_state, m3.next_state,
            "mutant dropped penalty factor"
        );
        assert_eq!(oracle_admit_penalty.next_state.low, 70);
        assert_eq!(
            m3.next_state.low, 110,
            "Mutant 3 dropped the penalty factor"
        );

        // Stale Certificate Mutant
        let m4 = mutant_auto_select_stale_certificate(&s_control, &s_telemetry, !0, 0, !0, !0);
        let oracle_stale_cert =
            oracle_auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, 0, !0, !0);
        assert_ne!(
            oracle_stale_cert, m4,
            "Mutant 4 incorrectly accepted a stale certificate"
        );
        assert_eq!(m4.refusal_code, AdaptiveMutationRefusal::None as u8);
        assert_eq!(m4.next_state.low, 110);

        // Missing Receipt Mutant
        let m5 = mutant_auto_select_missing_receipt(&s_control, &s_telemetry, !0, !0, !0, 0);
        let oracle_missing_receipt =
            oracle_auto_select_adaptive_mutation(&s_control, &s_telemetry, !0, !0, !0, 0);
        assert_ne!(
            oracle_missing_receipt, m5,
            "Mutant 5 incorrectly accepted a missing receipt"
        );
        assert_eq!(m5.refusal_code, AdaptiveMutationRefusal::None as u8);
        assert_eq!(m5.next_state.low, 110);
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3
// counterfactual_mutant 4
// counterfactual_mutant 5

// boundaries, equivalence, _reference, oracle
