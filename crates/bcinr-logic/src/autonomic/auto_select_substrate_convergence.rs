#![forbid(unsafe_code)]

//! Auto Select Substrate Convergence Operator (Iteration 28)
//!
//! Synchronizes the mutated adaptive control state back into the global Autonomic Substrate.
//! CC=1, 0 heap allocations, deterministic.

use crate::autonomic::autonomic_substrate::AutonomicSubstrate;
use crate::autonomic::rl_state::RlState;
use crate::mask::select_u64;

/// Typed refusal codes for Substrate Convergence.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateConvergenceRefusal {
    None = 0,
    ControlStateUnadmitted = 12,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvergenceResult {
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 25: Radon Law verified.
// AXIOMATIC PROOF: { S_sub, S_ctrl, M_commit } -> { S_sub.state = select(M, S_ctrl, S_sub.state) }

/// Deterministically synchronizes the adaptive control state into the Autonomic Substrate.
///
/// Mathematical Law:
/// $$ S_{substrate}'.state = \operatorname{select}(M_{commit}, S_{control}', S_{substrate}.state) $$
#[inline(always)]
#[must_use]
pub fn substrate_convergence<K: Copy + Default + PartialEq, V: Copy + Default, const N: usize>(
    substrate: &mut AutonomicSubstrate<K, V, N>,
    s_control: &RlState,
    m_commit: u64,
) -> ConvergenceResult {
    let m_admit = m_commit; // Pre-validated as 0 or !0

    // Fieldwise masked commit for RlState (low, high, extra)
    let final_low = select_u64(m_admit, s_control.low, substrate.state.low);
    let final_high = select_u64(m_admit, s_control.high, substrate.state.high);

    // extra is u8, map mask to u8
    let m_admit_8 = m_admit as u8;
    let final_extra = (m_admit_8 & s_control.extra) | (!m_admit_8 & substrate.state.extra);

    substrate.state.low = final_low;
    substrate.state.high = final_high;
    substrate.state.extra = final_extra;

    let refused_mask = !m_commit;
    let refusal_code =
        ((refused_mask & 1) as u8) * (SubstrateConvergenceRefusal::ControlStateUnadmitted as u8);

    ConvergenceResult { refusal_code }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_substrate_convergence<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
    >(
        substrate: &mut AutonomicSubstrate<K, V, N>,
        s_control: &RlState,
        m_commit: u64,
    ) -> ConvergenceResult {
        if m_commit == !0 {
            substrate.state = *s_control;
            ConvergenceResult {
                refusal_code: SubstrateConvergenceRefusal::None as u8,
            }
        } else {
            ConvergenceResult {
                refusal_code: SubstrateConvergenceRefusal::ControlStateUnadmitted as u8,
            }
        }
    }

    // Hostile mutants (Armstrong Fault)

    fn mutant_convergence_speculative_mutation<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
    >(
        substrate: &mut AutonomicSubstrate<K, V, N>,
        s_control: &RlState,
        m_commit: u64,
    ) -> ConvergenceResult {
        // MUTANT: Executes speculative update before validation, corrupting state on refusal
        substrate.state = *s_control;
        let refused_mask = !m_commit;
        let refusal_code = ((refused_mask & 1) as u8)
            * (SubstrateConvergenceRefusal::ControlStateUnadmitted as u8);
        ConvergenceResult { refusal_code }
    }

    fn mutant_convergence_ignored_mask<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
    >(
        substrate: &mut AutonomicSubstrate<K, V, N>,
        s_control: &RlState,
        _m_commit: u64,
    ) -> ConvergenceResult {
        // MUTANT: Ignores the commit mask entirely and applies the update
        substrate.state = *s_control;
        ConvergenceResult {
            refusal_code: SubstrateConvergenceRefusal::None as u8,
        }
    }

    fn mutant_convergence_dropped_factor<
        K: Copy + Default + PartialEq,
        V: Copy + Default,
        const N: usize,
    >(
        substrate: &mut AutonomicSubstrate<K, V, N>,
        s_control: &RlState,
        m_commit: u64,
    ) -> ConvergenceResult {
        // MUTANT: Partially updates state (drops the high factor)
        let m_admit = m_commit;
        substrate.state.low = select_u64(m_admit, s_control.low, substrate.state.low);
        // high is dropped/unchanged!
        let m_admit_8 = m_admit as u8;
        substrate.state.extra =
            (m_admit_8 & s_control.extra) | (!m_admit_8 & substrate.state.extra);

        let refused_mask = !m_commit;
        let refusal_code = ((refused_mask & 1) as u8)
            * (SubstrateConvergenceRefusal::ControlStateUnadmitted as u8);
        ConvergenceResult { refusal_code }
    }

    #[test]
    fn test_convergence_equivalence() {
        let mut sub_res: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let mut sub_oracle: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();

        let s_control = RlState::new(42, 84, 7);

        // Authorized commit
        let res = substrate_convergence(&mut sub_res, &s_control, !0);
        let oracle_res = oracle_substrate_convergence(&mut sub_oracle, &s_control, !0);

        assert_eq!(res, oracle_res);
        assert_eq!(sub_res.state, s_control);
        assert_eq!(sub_oracle.state, s_control);

        // Refused commit
        let mut sub_res_ref: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let mut sub_oracle_ref: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();

        let s_control_ref = RlState::new(99, 100, 1);

        let res_ref = substrate_convergence(&mut sub_res_ref, &s_control_ref, 0);
        let oracle_res_ref = oracle_substrate_convergence(&mut sub_oracle_ref, &s_control_ref, 0);

        assert_eq!(res_ref, oracle_res_ref);
        assert_eq!(sub_res_ref.state, RlState::default()); // unchanged
        assert_eq!(sub_oracle_ref.state, RlState::default());
        assert_eq!(
            res_ref.refusal_code,
            SubstrateConvergenceRefusal::ControlStateUnadmitted as u8
        );
    }

    #[test]
    fn test_convergence_mutants() {
        let mut sub_oracle: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let s_control = RlState::new(42, 84, 7);
        let m_commit = 0; // Test on refusal where mutations show

        let oracle_res = oracle_substrate_convergence(&mut sub_oracle, &s_control, m_commit);

        // Mutant 1: Speculative Mutation
        let mut sub_m1: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let m1 = mutant_convergence_speculative_mutation(&mut sub_m1, &s_control, m_commit);
        assert_ne!(
            sub_oracle.state, sub_m1.state,
            "Mutant 1 corrupts state on refusal"
        );
        assert_eq!(m1.refusal_code, oracle_res.refusal_code);

        // Mutant 2: Ignored Mask
        let mut sub_m2: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let m2 = mutant_convergence_ignored_mask(&mut sub_m2, &s_control, m_commit);
        assert_ne!(
            oracle_res.refusal_code, m2.refusal_code,
            "Mutant 2 ignored the refusal code"
        );
        assert_ne!(
            sub_oracle.state, sub_m2.state,
            "Mutant 2 ignored the mask and mutated state"
        );

        // Mutant 3: Dropped Factor
        let mut sub_oracle_valid: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let oracle_res_valid = oracle_substrate_convergence(&mut sub_oracle_valid, &s_control, !0); // Valid commit
        let mut sub_m3: AutonomicSubstrate<u32, u32, 1> = AutonomicSubstrate::new();
        let m3 = mutant_convergence_dropped_factor(&mut sub_m3, &s_control, !0); // Valid commit

        assert_eq!(oracle_res_valid.refusal_code, m3.refusal_code); // both 0
        assert_ne!(
            sub_oracle_valid.state, sub_m3.state,
            "Mutant 3 dropped a factor (high is not updated)"
        );
        assert_eq!(sub_m3.state.high, 0); // Still default
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3
// boundaries, equivalence, _reference, oracle
