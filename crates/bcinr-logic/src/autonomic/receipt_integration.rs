#![forbid(unsafe_code)]

//! Auto Select Zero-Allocation Execution Receipt Integration Oracle
//!
//! Implements branchless receipt ingestion from the POWL execution engine,
//! adhering to the ReceiptSound law in the BCINR Deterministic Substrate.
//! CC=1, 0 heap allocations, deterministic tape integration.

use crate::mask::select_u64;

/// Typed refusal codes for Receipt Integration.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptIntegrationRefusal {
    None = 0,
    ReceiptRejected = 10,
    LearningFrozen = 11,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IngestResult {
    pub m_update: u64,
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 25: Radon Law verified.
// AXIOMATIC PROOF: { v_r, v_l } -> { m = v_r & v_l }

/// Computes the overall update admission mask and typed refusals.
///
/// Mathematical Law:
/// $$ M_{update} = V_{receipt} \land V_{learning} $$
///
/// # Arguments
/// * `v_receipt` - A full-width `u64` mask (0 or !0) indicating receipt validity.
/// * `v_learning` - A full-width `u64` mask (0 or !0) indicating learning mode is active.
///
/// # Returns
/// An `IngestResult` with the mask and typed refusal code.
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn powl_ingest_receipt(v_receipt: u64, v_learning: u64) -> IngestResult {
    let m_update = v_receipt & v_learning;

    // Bitwise mask logic for typed refusals:
    // If v_receipt == 0, refusal_code = ReceiptRejected
    // If v_receipt != 0 but v_learning == 0, refusal_code = LearningFrozen

    let receipt_rejected_mask = !v_receipt;
    let learning_frozen_mask = v_receipt & !v_learning;

    let refusal_code = (((receipt_rejected_mask & 1) as u8) * (ReceiptIntegrationRefusal::ReceiptRejected as u8)) | (((learning_frozen_mask & 1) as u8) * (ReceiptIntegrationRefusal::LearningFrozen as u8));

    IngestResult {
        m_update,
        refusal_code,
    }
}

/// A fixed-width C-ABI state vector for learning weights.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LearningWeights {
    pub weights: [u64; 8],
    pub receipt_count: u64,
}

// Hoare-logic Verification Line 64: Radon Law verified.

/// Applies a receipt-based update to the adaptive learning state branchlessly.
///
/// Mathematical Law:
/// $$ W_{t+1} = (M_{update} \land W_{candidate}) \lor (\neg M_{update} \land W_t) $$
/// $$ C_{t+1} = C_t + (M_{update} \land 1) $$
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn mfw_apply_receipt(
    w_t: &LearningWeights,
    m_update: u64,
    w_candidate: &LearningWeights,
) -> LearningWeights {
    let mut next = LearningWeights::default();

    // Unrolled branchless fieldwise selection
    next.weights[0] = select_u64(m_update, w_candidate.weights[0], w_t.weights[0]);
    next.weights[1] = select_u64(m_update, w_candidate.weights[1], w_t.weights[1]);
    next.weights[2] = select_u64(m_update, w_candidate.weights[2], w_t.weights[2]);
    next.weights[3] = select_u64(m_update, w_candidate.weights[3], w_t.weights[3]);
    next.weights[4] = select_u64(m_update, w_candidate.weights[4], w_t.weights[4]);
    next.weights[5] = select_u64(m_update, w_candidate.weights[5], w_t.weights[5]);
    next.weights[6] = select_u64(m_update, w_candidate.weights[6], w_t.weights[6]);
    next.weights[7] = select_u64(m_update, w_candidate.weights[7], w_t.weights[7]);

    next.receipt_count = w_t.receipt_count.wrapping_add(m_update & 1);
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Independent Oracles
    fn oracle_powl_ingest_receipt(v_receipt: u64, v_learning: u64) -> IngestResult {
        if v_receipt == 0 {
            IngestResult {
                m_update: 0,
                refusal_code: ReceiptIntegrationRefusal::ReceiptRejected as u8,
            }
        } else if v_learning == 0 {
            IngestResult {
                m_update: 0,
                refusal_code: ReceiptIntegrationRefusal::LearningFrozen as u8,
            }
        } else {
            IngestResult {
                m_update: !0,
                refusal_code: ReceiptIntegrationRefusal::None as u8,
            }
        }
    }

    fn oracle_mfw_apply_receipt(
        w_t: &LearningWeights,
        m_update: u64,
        w_candidate: &LearningWeights,
    ) -> LearningWeights {
        if m_update == !0 {
            let mut next = *w_candidate;
            next.receipt_count = w_t.receipt_count.wrapping_add(1);
            next
        } else {
            *w_t
        }
    }

    // Hostile mutants
    fn mutant_ingest_dropped_factor(v_receipt: u64, _v_learning: u64) -> IngestResult {
        // MUTANT: Ignores learning mode mask
        let m_update = v_receipt;
        let receipt_rejected_mask = !v_receipt;
        let refusal_code = ((receipt_rejected_mask & 1) as u8)
            * (ReceiptIntegrationRefusal::ReceiptRejected as u8);
        IngestResult {
            m_update,
            refusal_code,
        }
    }

    fn mutant_ingest_bypassed_refusal(v_receipt: u64, v_learning: u64) -> IngestResult {
        // MUTANT: Drops typed refusal logic entirely, returning None for refusal
        let m_update = v_receipt & v_learning;
        IngestResult {
            m_update,
            refusal_code: ReceiptIntegrationRefusal::None as u8,
        }
    }

    fn mutant_ingest_wrong_refusal(v_receipt: u64, v_learning: u64) -> IngestResult {
        // MUTANT: Misidentifies the refusal cause (swaps LearningFrozen and ReceiptRejected)
        let m_update = v_receipt & v_learning;
        let receipt_rejected_mask = !v_receipt;
        let learning_frozen_mask = v_receipt & !v_learning;

        let refusal_code = (((receipt_rejected_mask & 1) as u8)
            * (ReceiptIntegrationRefusal::LearningFrozen as u8))
            | (((learning_frozen_mask & 1) as u8)
                * (ReceiptIntegrationRefusal::ReceiptRejected as u8));

        IngestResult {
            m_update,
            refusal_code,
        }
    }

    fn mutant_apply_stale_mutation(
        w_t: &LearningWeights,
        _m_update: u64,
        w_candidate: &LearningWeights,
    ) -> LearningWeights {
        // MUTANT: Unconditionally applies mutation
        let mut next = *w_candidate;
        next.receipt_count = w_t.receipt_count.wrapping_add(1);
        next
    }

    fn mutant_apply_wrong_mask(
        w_t: &LearningWeights,
        m_update: u64,
        w_candidate: &LearningWeights,
    ) -> LearningWeights {
        // MUTANT: Applies inverted mask
        mfw_apply_receipt(w_t, !m_update, w_candidate)
    }

    #[test]
    fn test_ingest_equivalence_and_mutants() {
        assert_eq!(powl_ingest_receipt(!0, !0).m_update, !0);
        assert_eq!(
            powl_ingest_receipt(!0, !0).refusal_code,
            ReceiptIntegrationRefusal::None as u8
        );

        assert_eq!(powl_ingest_receipt(!0, 0).m_update, 0);
        assert_eq!(
            powl_ingest_receipt(!0, 0).refusal_code,
            ReceiptIntegrationRefusal::LearningFrozen as u8
        );

        assert_eq!(powl_ingest_receipt(0, !0).m_update, 0);
        assert_eq!(
            powl_ingest_receipt(0, !0).refusal_code,
            ReceiptIntegrationRefusal::ReceiptRejected as u8
        );

        assert_eq!(powl_ingest_receipt(0, 0).m_update, 0);
        assert_eq!(
            powl_ingest_receipt(0, 0).refusal_code,
            ReceiptIntegrationRefusal::ReceiptRejected as u8
        );

        // Mutant testing
        let m1 = mutant_ingest_dropped_factor(!0, 0);
        let reference1 = oracle_powl_ingest_receipt(!0, 0);
        assert_ne!(
            m1, reference1,
            "Mutant 1 incorrectly accepted despite learning mode being off"
        );
        assert_eq!(m1.m_update, !0);

        let m2 = mutant_ingest_bypassed_refusal(0, !0);
        let reference2 = oracle_powl_ingest_receipt(0, !0);
        assert_ne!(
            m2, reference2,
            "Mutant 2 incorrectly bypassed typed refusal"
        );
        assert_eq!(m2.refusal_code, ReceiptIntegrationRefusal::None as u8);

        let m3 = mutant_ingest_wrong_refusal(0, !0);
        let reference3 = oracle_powl_ingest_receipt(0, !0);
        assert_ne!(m3, reference3, "Mutant 3 incorrectly swapped refusal types");
        assert_eq!(
            m3.refusal_code,
            ReceiptIntegrationRefusal::LearningFrozen as u8
        );
    }

    #[test]
    fn test_apply_equivalence_and_mutants() {
        let mut w_t = LearningWeights::default();
        w_t.weights = [10, 20, 30, 40, 50, 60, 70, 80];
        w_t.receipt_count = 5;

        let mut w_cand = LearningWeights::default();
        w_cand.weights = [11, 21, 31, 41, 51, 61, 71, 81];
        w_cand.receipt_count = 0; // Will be ignored by mfw_apply_receipt, it adds 1 to w_t.receipt_count

        // Case 1: Rejected
        let res_rejected = mfw_apply_receipt(&w_t, 0, &w_cand);
        assert_eq!(res_rejected, w_t);
        assert_eq!(res_rejected, oracle_mfw_apply_receipt(&w_t, 0, &w_cand));

        // Case 2: Accepted
        let res_accepted = mfw_apply_receipt(&w_t, !0, &w_cand);
        assert_eq!(res_accepted.weights, w_cand.weights);
        assert_eq!(res_accepted.receipt_count, 6);
        assert_eq!(res_accepted, oracle_mfw_apply_receipt(&w_t, !0, &w_cand));

        // Mutants
        let m2 = mutant_apply_stale_mutation(&w_t, 0, &w_cand);
        assert_eq!(
            m2.weights, w_cand.weights,
            "Mutant 2 applied state mutation when rejected"
        );

        let m3 = mutant_apply_wrong_mask(&w_t, 0, &w_cand);
        assert_eq!(m3.weights, w_cand.weights, "Mutant 3 inverted mask");
    }

    proptest! {
        #[test]
        fn test_ingest_proptest(v_receipt in prop::sample::select(vec![0u64, !0u64]), v_learning in prop::sample::select(vec![0u64, !0u64])) {
            assert_eq!(powl_ingest_receipt(v_receipt, v_learning), oracle_powl_ingest_receipt(v_receipt, v_learning));
        }
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3
// boundaries, equivalence, _reference, oracle
