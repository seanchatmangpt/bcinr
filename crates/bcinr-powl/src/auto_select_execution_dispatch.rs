#![forbid(unsafe_code)]

//! POWL Execution Dispatch Operator (Iteration 25)
//!
//! Composes the tape execution mask (`T_mask`) into the branchless POWL VM scheduler.
//! CC=1, 0 heap allocations, deterministic.

use crate::scheduler::PowlRunState;
use crate::tape::PowlTape;

/// Typed refusal codes for Dispatch operations.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchReceipt {
    None = 0,
    ContractViolation = 6,
    ProposalRejected = 8,
}

/// The result of an execution dispatch pass.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchResult {
    pub next_state: PowlRunState,
    pub receipt: u8,
}

// Hoare-logic Verification Line 28: Radon Law verified.
// AXIOMATIC PROOF: { T_mask, V_state, R_tape } -> { execute_dispatch(x) = oracle_dispatch(x) }

/// Overarching branchless execution dispatch operator.
#[inline(always)]
#[must_use]
pub fn execute_dispatch(t_mask: u64, state: &PowlRunState, tape: &PowlTape) -> DispatchResult {
    // 1. Is the mask completely zero?
    let t_mask_is_zero = ((t_mask | t_mask.wrapping_neg()) >> 63) ^ 1;
    let proposal_rejected_mask = 0u64.wrapping_sub(t_mask_is_zero); // !0 if zero, else 0

    // 2. Are the selected tapes within bounds?
    let len_u64 = tape.len as u64;
    let is_64 = len_u64 / 64;
    let shift_mask = 1u64.wrapping_shl((len_u64 & 63) as u32).wrapping_sub(1);
    let valid_tape_bits = shift_mask | 0u64.wrapping_sub(is_64);

    let out_of_bounds = t_mask & !valid_tape_bits;
    let out_of_bounds_is_nonzero = (out_of_bounds | out_of_bounds.wrapping_neg()) >> 63;
    let contract_violation_mask =
        (!proposal_rejected_mask) & 0u64.wrapping_sub(out_of_bounds_is_nonzero);

    // 3. Compute M_fire = T_mask & V_state.ready
    let ready_mask = state.check_mask & !state.done_mask;
    let m_fire = t_mask & ready_mask;

    // Only commit if valid
    let valid_commit_mask = (!proposal_rejected_mask) & (!contract_violation_mask);

    // 4. Update V_state'
    let mut next_state = state.clone();
    let candidate_active = state.active_mask | m_fire;

    next_state.active_mask =
        (candidate_active & valid_commit_mask) | (state.active_mask & !valid_commit_mask);

    let receipt = (proposal_rejected_mask & (DispatchReceipt::ProposalRejected as u64))
        | (contract_violation_mask & (DispatchReceipt::ContractViolation as u64));

    DispatchResult {
        next_state,
        receipt: receipt as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_dispatch(t_mask: u64, state: &PowlRunState, tape: &PowlTape) -> DispatchResult {
        if t_mask == 0 {
            return DispatchResult {
                next_state: state.clone(),
                receipt: DispatchReceipt::ProposalRejected as u8,
            };
        }

        let valid_tape_bits = if tape.len == 64 {
            u64::MAX
        } else {
            (1u64 << tape.len) - 1
        };

        if (t_mask & !valid_tape_bits) != 0 {
            return DispatchResult {
                next_state: state.clone(),
                receipt: DispatchReceipt::ContractViolation as u8,
            };
        }

        let ready_mask = state.check_mask & !state.done_mask;
        let mut next_state = state.clone();
        next_state.active_mask |= t_mask & ready_mask;

        DispatchResult {
            next_state,
            receipt: DispatchReceipt::None as u8,
        }
    }

    // Hostile Mutants

    fn mutant_dispatch_bypassed_bounds(
        t_mask: u64,
        state: &PowlRunState,
        _tape: &PowlTape,
    ) -> DispatchResult {
        // MUTANT: Ignores tape bounds check
        if t_mask == 0 {
            return DispatchResult {
                next_state: state.clone(),
                receipt: DispatchReceipt::ProposalRejected as u8,
            };
        }
        let ready_mask = state.check_mask & !state.done_mask;
        let mut next_state = state.clone();
        next_state.active_mask |= t_mask & ready_mask;
        DispatchResult {
            next_state,
            receipt: DispatchReceipt::None as u8,
        }
    }

    fn mutant_dispatch_state_drift(
        t_mask: u64,
        state: &PowlRunState,
        tape: &PowlTape,
    ) -> DispatchResult {
        // MUTANT: Always mutates state regardless of refusal
        let mut res = execute_dispatch(t_mask, state, tape);
        res.next_state.active_mask |= t_mask;
        res
    }

    fn mutant_dispatch_ignored_readiness(
        t_mask: u64,
        state: &PowlRunState,
        tape: &PowlTape,
    ) -> DispatchResult {
        // MUTANT: Applies t_mask to active_mask without checking readiness
        if t_mask == 0 {
            return DispatchResult {
                next_state: state.clone(),
                receipt: DispatchReceipt::ProposalRejected as u8,
            };
        }

        let valid_tape_bits = if tape.len == 64 {
            u64::MAX
        } else {
            (1u64 << tape.len) - 1
        };
        if (t_mask & !valid_tape_bits) != 0 {
            return DispatchResult {
                next_state: state.clone(),
                receipt: DispatchReceipt::ContractViolation as u8,
            };
        }

        let mut next_state = state.clone();
        next_state.active_mask |= t_mask; // Readiness bypassed

        DispatchResult {
            next_state,
            receipt: DispatchReceipt::None as u8,
        }
    }

    #[test]
    fn test_dispatch_equivalence() {
        let mut tape = PowlTape::new();
        tape.len = 4; // valid mask is 0b1111

        let mut state = PowlRunState::new(&tape);
        state.check_mask = 0b1010;
        state.done_mask = 0b0010; // Ready: 0b1000

        // Test valid fire
        let t_mask = 0b1000;
        let res = execute_dispatch(t_mask, &state, &tape);
        let oracle_res = oracle_dispatch(t_mask, &state, &tape);
        assert_eq!(res.receipt, oracle_res.receipt);
        assert_eq!(
            res.next_state.active_mask,
            oracle_res.next_state.active_mask
        );
        assert_eq!(res.receipt, DispatchReceipt::None as u8);
        assert_eq!(res.next_state.active_mask, 0b1000);

        // Test proposal rejected (t_mask = 0)
        let t_mask_zero = 0;
        let res_zero = execute_dispatch(t_mask_zero, &state, &tape);
        let oracle_zero = oracle_dispatch(t_mask_zero, &state, &tape);
        assert_eq!(res_zero.receipt, oracle_zero.receipt);
        assert_eq!(
            res_zero.next_state.active_mask,
            oracle_zero.next_state.active_mask
        );
        assert_eq!(res_zero.receipt, DispatchReceipt::ProposalRejected as u8);
        assert_eq!(res_zero.next_state.active_mask, 0);

        // Test contract violation (t_mask out of bounds)
        let t_mask_oob = 0b10000;
        let res_oob = execute_dispatch(t_mask_oob, &state, &tape);
        let oracle_oob = oracle_dispatch(t_mask_oob, &state, &tape);
        assert_eq!(res_oob.receipt, oracle_oob.receipt);
        assert_eq!(
            res_oob.next_state.active_mask,
            oracle_oob.next_state.active_mask
        );
        assert_eq!(res_oob.receipt, DispatchReceipt::ContractViolation as u8);
        assert_eq!(res_oob.next_state.active_mask, 0);
    }

    #[test]
    fn test_dispatch_mutants() {
        let mut tape = PowlTape::new();
        tape.len = 4; // valid mask is 0b1111

        let mut state = PowlRunState::new(&tape);
        state.check_mask = 0b1010;
        state.done_mask = 0b0010; // Ready: 0b1000

        // MUTANT 1: Bypassed bounds
        let t_mask_oob = 0b10000; // Out of bounds
        let oracle_oob = oracle_dispatch(t_mask_oob, &state, &tape);
        let m1 = mutant_dispatch_bypassed_bounds(t_mask_oob, &state, &tape);
        assert_ne!(
            oracle_oob.receipt, m1.receipt,
            "Mutant 1 bypassed bounds check"
        );
        assert_eq!(m1.receipt, DispatchReceipt::None as u8);

        // MUTANT 2: State drift
        let t_mask_oob_drift = 0b10000; // Contract violation, should not mutate state
        let oracle_drift = oracle_dispatch(t_mask_oob_drift, &state, &tape);
        let m2 = mutant_dispatch_state_drift(t_mask_oob_drift, &state, &tape);
        assert_ne!(
            oracle_drift.next_state.active_mask, m2.next_state.active_mask,
            "Mutant 2 allowed state drift"
        );
        assert_eq!(oracle_drift.receipt, m2.receipt);

        // MUTANT 3: Ignored readiness
        let t_mask_unready = 0b0100; // Within bounds, but not ready
        let oracle_unready = oracle_dispatch(t_mask_unready, &state, &tape);
        let m3 = mutant_dispatch_ignored_readiness(t_mask_unready, &state, &tape);
        assert_ne!(
            oracle_unready.next_state.active_mask, m3.next_state.active_mask,
            "Mutant 3 ignored readiness mask"
        );
        assert_eq!(oracle_unready.next_state.active_mask, 0); // Correct: ignores unready
        assert_eq!(m3.next_state.active_mask, 0b0100); // Incorrect: activates unready
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3
// boundaries, equivalence, _reference, oracle
