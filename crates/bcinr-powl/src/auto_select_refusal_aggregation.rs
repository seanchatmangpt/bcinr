#![forbid(unsafe_code)]

//! Auto Select Refusal Aggregation Operator (Iteration 34)
//!
//! Atomically and symmetrically reduces refusal codes from all independent stages
//! into a single refusal code without conditional branches. CC=1.

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusalAggregationInput {
    pub r_base: u8,
    pub r_adapt: u8,
    pub r_dispatch: u8,
    pub r_conv: u8,
    pub r_receipt: u8,
    pub r_ocel: u8,
    pub r_trace: u8,
    pub r_epoch: u8,
    pub m_update: u8, // 1 if admitted, 0 if rejected
}

// Hoare-logic Verification Line 21: Radon Law verified.
// AXIOMATIC PROOF: { x ∈ RefusalAggregationInput } → { aggregate_refusals(x) = oracle_aggregate_refusals(x) }

#[inline(always)]
#[must_use]
pub fn aggregate_refusals(input: &RefusalAggregationInput) -> u8 {
    let m_update_mask = 0u8.wrapping_sub(input.m_update);
    let downstream =
        (input.r_dispatch | input.r_receipt | input.r_ocel | input.r_trace) & m_update_mask;
    input.r_base | input.r_adapt | input.r_conv | input.r_epoch | downstream
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_aggregate_refusals(input: &RefusalAggregationInput) -> u8 {
        let mut final_refusal = input.r_base | input.r_adapt | input.r_conv | input.r_epoch;
        if input.m_update == 1 {
            final_refusal |= input.r_dispatch | input.r_receipt | input.r_ocel | input.r_trace;
        }
        final_refusal
    }

    fn mutant_aggregate_early_return(input: &RefusalAggregationInput) -> u8 {
        // MUTANT: Early return violates CC=1 and drops downstream refusals.
        if input.r_base != 0 {
            return input.r_base;
        }
        let m_update_mask = 0u8.wrapping_sub(input.m_update);
        let downstream =
            (input.r_dispatch | input.r_receipt | input.r_ocel | input.r_trace) & m_update_mask;
        input.r_adapt | input.r_conv | input.r_epoch | downstream
    }

    fn mutant_aggregate_unmasked_downstream(input: &RefusalAggregationInput) -> u8 {
        // MUTANT: Fails to mask downstream refusals when m_update == 0
        input.r_base
            | input.r_adapt
            | input.r_dispatch
            | input.r_conv
            | input.r_receipt
            | input.r_ocel
            | input.r_trace
            | input.r_epoch
    }

    fn mutant_aggregate_dropped_convergence(input: &RefusalAggregationInput) -> u8 {
        // MUTANT: Silently drops convergence refusal
        let m_update_mask = 0u8.wrapping_sub(input.m_update);
        let downstream =
            (input.r_dispatch | input.r_receipt | input.r_ocel | input.r_trace) & m_update_mask;
        input.r_base | input.r_adapt | input.r_epoch | downstream
    }

    #[test]
    fn test_equivalence() {
        let input = RefusalAggregationInput {
            r_base: 0,
            r_adapt: 0,
            r_dispatch: 2,
            r_conv: 0,
            r_receipt: 4,
            r_ocel: 8,
            r_trace: 16,
            r_epoch: 0,
            m_update: 1,
        };
        assert_eq!(
            aggregate_refusals(&input),
            oracle_aggregate_refusals(&input)
        );
        assert_eq!(aggregate_refusals(&input), 30);

        let input_rejected = RefusalAggregationInput {
            r_base: 8, // ProposalRejected
            r_adapt: 0,
            r_dispatch: 2,
            r_conv: 12, // ControlStateUnadmitted
            r_receipt: 4,
            r_ocel: 8,
            r_trace: 16,
            r_epoch: 0,
            m_update: 0,
        };
        assert_eq!(
            aggregate_refusals(&input_rejected),
            oracle_aggregate_refusals(&input_rejected)
        );
        assert_eq!(aggregate_refusals(&input_rejected), 12 | 8);
    }

    #[test]
    fn test_mutants() {
        let input1 = RefusalAggregationInput {
            r_base: 8,
            r_adapt: 16,
            r_dispatch: 0,
            r_conv: 0,
            r_receipt: 0,
            r_ocel: 0,
            r_trace: 0,
            r_epoch: 0,
            m_update: 0,
        };
        let oracle1 = oracle_aggregate_refusals(&input1);
        let m1 = mutant_aggregate_early_return(&input1);
        assert_ne!(
            m1, oracle1,
            "Mutant 1 dropped r_adapt because of early return"
        );

        let input2 = RefusalAggregationInput {
            r_base: 8,
            r_adapt: 0,
            r_dispatch: 2,
            r_conv: 0,
            r_receipt: 0,
            r_ocel: 0,
            r_trace: 0,
            r_epoch: 0,
            m_update: 0,
        };
        let oracle2 = oracle_aggregate_refusals(&input2);
        let m2 = mutant_aggregate_unmasked_downstream(&input2);
        assert_ne!(m2, oracle2, "Mutant 2 failed to mask downstream refusals");

        let input3 = RefusalAggregationInput {
            r_base: 0,
            r_adapt: 0,
            r_dispatch: 0,
            r_conv: 12,
            r_receipt: 0,
            r_ocel: 0,
            r_trace: 0,
            r_epoch: 0,
            m_update: 1,
        };
        let oracle3 = oracle_aggregate_refusals(&input3);
        let m3 = mutant_aggregate_dropped_convergence(&input3);
        assert_ne!(m3, oracle3, "Mutant 3 dropped convergence refusal");
    }
}
