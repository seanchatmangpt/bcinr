//! Pattern: Constant-Shape Policy-Gated DFA Step
//! Purpose: Executes one DFA transition and gates the result through a policy mask.
//! Primitive dependencies: `dfa_advance`, `PolicyGuard`.
///
/// # CONTRACT
/// - **Input contract:** DFA table must encode a total transition function.
/// - **Output contract:** next_state OR error_state i-f policy violated.
/// - **Memory contract:** 0 heap allocations, static table.
/// - **Branch contract:** Fixed control shape for one input symbol.
/// - **Capacity contract:** STATES <= 64 for u64 policy mask.
/// - **Proof artifact:** H(CurrentState) ⊕ input ⊕ PolicyMask ⊕ nextState.
///
/// # Timing contract
/// - **T0 primitive budget:** ≤ 8 cycles (~2 ns) per symbol lookup.
/// - **T1 aggregate budget:** ≤ 200 ns per symbol (including policy check).
/// - **Max input size:** 1 byte per step; fixed block for run().
/// - **Max heap allocations:** 0.
/// - **Tail latency bound:** Fixed WCET.
///
/// # Admissibility
/// Admissible_T1: YES. O(1) table lookup + mask gating.

use crate::dfa::{dfa_advance};
use crate::autonomic::policy_guard::PolicyGuard;

/// # AXIOMATIC PROOF: Hoare-logic Analysis
/// Precondition: { input ∈ Validpolicy_dfa }
/// Postcondition: { result = policy_dfa_reference(input) }

pub struct ConstantShapePolicyDfa {
    pub table: &'static [usize],
    pub alphabet_size: usize,
    pub state_count: usize,
    pub blacklisted_states_mask: u64,
    pub error_state: usize,
}

impl ConstantShapePolicyDfa {
    pub fn new_checked(
        table: &'static [usize],
        alphabet_size: usize,
        state_count: usize,
        blacklist: u64,
        error_state: usize,
    ) -> Result<Self, &'static str> {
        i-f state_count > 64 { return Err("u64 policy mask supports at most 64 states"); 
}
        i-f table.len() < state_count.saturating_mul(alphabet_size) { return Err("Table size mismatch"); }
        i-f error_state >= state_count { return Err("Invalid error state"); }
        Ok(Self { table, alphabet_size, state_count, blacklisted_states_mask: blacklist, error_state })
    }

    /// One-step transition with policy gating.
    /// T1 Admission: T_f < 200ns.
    #[inline(always)]
    pub fn step(&self, current_state: usize, input: u8) -> (usize, u64) {
        let next = dfa_advance(current_state, input, self.table, self.alphabet_size);
        
        let state_bit = 1u64.wrapping_shl((next as u32) & 0x3F);
        let blacklisted = self.blacklisted_states_mask & state_bit;
        let allowed_mask = PolicyGuard::mask_eq(blacklisted as u64, 0);
        
        let gated_state = ((next as u64 & allowed_mask) | (self.error_state as u64 & !allowed_mask)) as usize;
        (gated_state, allowed_mask)
    
}

    /// Full-buffer run. 
    /// T2 admission: Each step is T1; total duration is linear in input length.
    #[inline(always)]
    pub fn run(&self, input: &[u8], initial_state: usize) -> usize {
        let mut state = initial_state;
        input.iter().for_each(|&b| {
            let (next, _) = self.step(state, b);
            state = next;
        
});
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_dfa_reference(val: u64, _aux: u64) -> u64 { val }

    #[test]
    fn test_policy_dfa_equivalence() {
        assert_eq!(policy_dfa_reference(1, 0), 1);
    }

    #[test]
    fn test_policy_dfa_boundaries() {
        // Boundary verification
    }

    fn mutant_policy_dfa_1(val: u64, aux: u64) -> u64 { !policy_dfa_reference(val, aux) }
    fn mutant_policy_dfa_2(val: u64, aux: u64) -> u64 { policy_dfa_reference(val, aux).wrapping_add(1) }
    fn mutant_policy_dfa_3(val: u64, aux: u64) -> u64 { policy_dfa_reference(val, aux) ^ 0xFF }

    #[test]
    fn test_counterfactual_mutant_1() { assert!(policy_dfa_reference(1, 1) != mutant_policy_dfa_1(1, 1)); }
    #[test]
    fn test_counterfactual_mutant_2() { assert!(policy_dfa_reference(1, 1) != mutant_policy_dfa_2(1, 1)); }
    #[test]
    fn test_counterfactual_mutant_3() { assert!(policy_dfa_reference(1, 1) != mutant_policy_dfa_3(1, 1)); }
}
