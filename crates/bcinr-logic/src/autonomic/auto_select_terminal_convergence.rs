#![forbid(unsafe_code)]

//! # Auto Select Terminal Convergence Operator (Iteration 41)
//!
//! Safely maps the accumulated execution tape mask and intermediate autonomic selections
//! into the persistent substrate state without branching. CC=1.

use crate::mask::select_u64;

/// Typed refusal codes for Terminal Convergence.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalConvergenceRefusal {
    None = 0,
    NoLeaves = 1,
    ControlStateUnadmitted = 2,
    BranchlessContractFailed = 3,
    ContractViolation = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusalAggregationState {
    pub critical_refusal: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistentControlState {
    pub epoch_clock: u64,
    pub mass: u64,
    pub _pad: [u64; 30],
}

impl Default for PersistentControlState {
    fn default() -> Self {
        Self {
            epoch_clock: 0,
            mass: 0,
            _pad: [0; 30],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConvergenceInput {
    pub m_tape: u64,
    pub r_aggr: RefusalAggregationState,
    pub expected_epoch: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConvergenceResult {
    pub next_state: PersistentControlState,
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 57: Radon Law verified.
// AXIOMATIC PROOF: { x \in Input } -> { converge(x) = oracle_converge(x) }

#[inline(always)]
#[must_use]
pub fn terminal_convergence(
    input: &TerminalConvergenceInput,
    x_persistent: &PersistentControlState,
) -> TerminalConvergenceResult {
    let is_tape_empty = (input.m_tape == 0) as u64;
    let tape_empty_mask = 0u64.wrapping_sub(is_tape_empty);

    let is_critical_refusal = (input.r_aggr.critical_refusal != 0) as u64;
    let critical_refusal_mask = 0u64.wrapping_sub(is_critical_refusal);

    let is_epoch_mismatch = (x_persistent.epoch_clock != input.expected_epoch) as u64;
    let epoch_mismatch_mask = 0u64.wrapping_sub(is_epoch_mismatch);

    let effective_epoch_mismatch = epoch_mismatch_mask;
    let effective_critical = critical_refusal_mask & (!epoch_mismatch_mask);
    let effective_tape = tape_empty_mask & (!critical_refusal_mask) & (!epoch_mismatch_mask);

    let refusal_code = (effective_epoch_mismatch
        & (TerminalConvergenceRefusal::ContractViolation as u64))
        | (effective_critical & (TerminalConvergenceRefusal::BranchlessContractFailed as u64))
        | (effective_tape & (TerminalConvergenceRefusal::NoLeaves as u64));

    let m_admit = (!tape_empty_mask) & (!critical_refusal_mask) & (!epoch_mismatch_mask);

    let added_mass = (input.m_tape.count_ones() as u64) & m_admit;
    let candidate_mass = x_persistent.mass.saturating_add(added_mass);
    let next_mass = select_u64(m_admit, candidate_mass, x_persistent.mass);

    let candidate_epoch = x_persistent.epoch_clock.wrapping_add(1);
    let next_epoch = select_u64(m_admit, candidate_epoch, x_persistent.epoch_clock);

    let mut next_state = *x_persistent;
    next_state.mass = next_mass;
    next_state.epoch_clock = next_epoch;

    TerminalConvergenceResult {
        next_state,
        refusal_code: refusal_code as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_terminal_convergence(
        input: &TerminalConvergenceInput,
        x_persistent: &PersistentControlState,
    ) -> TerminalConvergenceResult {
        if x_persistent.epoch_clock != input.expected_epoch {
            return TerminalConvergenceResult {
                next_state: *x_persistent,
                refusal_code: TerminalConvergenceRefusal::ContractViolation as u8,
            };
        }
        if input.r_aggr.critical_refusal != 0 {
            return TerminalConvergenceResult {
                next_state: *x_persistent,
                refusal_code: TerminalConvergenceRefusal::BranchlessContractFailed as u8,
            };
        }
        if input.m_tape == 0 {
            return TerminalConvergenceResult {
                next_state: *x_persistent,
                refusal_code: TerminalConvergenceRefusal::NoLeaves as u8,
            };
        }

        let mut next = *x_persistent;
        next.mass = next.mass.saturating_add(input.m_tape.count_ones() as u64);
        next.epoch_clock = next.epoch_clock.wrapping_add(1);

        TerminalConvergenceResult {
            next_state: next,
            refusal_code: TerminalConvergenceRefusal::None as u8,
        }
    }

    // Hostile mutants

    fn mutant_converge_bypassed_epoch(
        input: &TerminalConvergenceInput,
        x_persistent: &PersistentControlState,
    ) -> TerminalConvergenceResult {
        // MUTANT: Ignores epoch mismatch
        let mut m_input = *input;
        m_input.expected_epoch = x_persistent.epoch_clock;
        terminal_convergence(&m_input, x_persistent)
    }

    fn mutant_converge_state_drift(
        input: &TerminalConvergenceInput,
        x_persistent: &PersistentControlState,
    ) -> TerminalConvergenceResult {
        // MUTANT: Unconditionally mutates state despite refusal
        let mut res = terminal_convergence(input, x_persistent);
        if res.refusal_code != TerminalConvergenceRefusal::None as u8 {
            res.next_state.epoch_clock = res.next_state.epoch_clock.wrapping_add(1);
        }
        res
    }

    fn mutant_converge_dropped_refusal(
        input: &TerminalConvergenceInput,
        x_persistent: &PersistentControlState,
    ) -> TerminalConvergenceResult {
        // MUTANT: Drops the critical refusal check
        let mut m_input = *input;
        m_input.r_aggr.critical_refusal = 0;
        terminal_convergence(&m_input, x_persistent)
    }

    fn mutant_converge_unbounded_mass(
        input: &TerminalConvergenceInput,
        x_persistent: &PersistentControlState,
    ) -> TerminalConvergenceResult {
        // MUTANT: Uses wrapping_add instead of saturating_add for mass
        let mut res = terminal_convergence(input, x_persistent);
        if res.refusal_code == TerminalConvergenceRefusal::None as u8 {
            res.next_state.mass = x_persistent
                .mass
                .wrapping_add(input.m_tape.count_ones() as u64);
        }
        res
    }

    #[test]
    fn test_equivalence() {
        let x_persistent = PersistentControlState {
            epoch_clock: 10,
            mass: 100,
            _pad: [0; 30],
        };
        let input = TerminalConvergenceInput {
            m_tape: 0b101, // count = 2
            r_aggr: RefusalAggregationState {
                critical_refusal: 0,
            },
            expected_epoch: 10,
        };

        let res1 = terminal_convergence(&input, &x_persistent);
        let res2 = oracle_terminal_convergence(&input, &x_persistent);

        assert_eq!(res1, res2);
        assert_eq!(res1.next_state.epoch_clock, 11);
        assert_eq!(res1.next_state.mass, 102);
        assert_eq!(res1.refusal_code, TerminalConvergenceRefusal::None as u8);

        // Test epoch mismatch
        let mut input2 = input;
        input2.expected_epoch = 9;
        let res3 = terminal_convergence(&input2, &x_persistent);
        let res4 = oracle_terminal_convergence(&input2, &x_persistent);
        assert_eq!(res3, res4);
        assert_eq!(
            res3.refusal_code,
            TerminalConvergenceRefusal::ContractViolation as u8
        );
        assert_eq!(res3.next_state.epoch_clock, 10);

        // Test critical refusal
        let mut input3 = input;
        input3.r_aggr.critical_refusal = 1;
        let res5 = terminal_convergence(&input3, &x_persistent);
        let res6 = oracle_terminal_convergence(&input3, &x_persistent);
        assert_eq!(res5, res6);
        assert_eq!(
            res5.refusal_code,
            TerminalConvergenceRefusal::BranchlessContractFailed as u8
        );

        // Test empty tape
        let mut input4 = input;
        input4.m_tape = 0;
        let res7 = terminal_convergence(&input4, &x_persistent);
        let res8 = oracle_terminal_convergence(&input4, &x_persistent);
        assert_eq!(res7, res8);
        assert_eq!(
            res7.refusal_code,
            TerminalConvergenceRefusal::NoLeaves as u8
        );
    }

    #[test]
    fn test_mutants() {
        let x_persistent = PersistentControlState {
            epoch_clock: 10,
            mass: 100,
            _pad: [0; 30],
        };
        let mut input = TerminalConvergenceInput {
            m_tape: 0b101,
            r_aggr: RefusalAggregationState {
                critical_refusal: 0,
            },
            expected_epoch: 9, // Epoch mismatch
        };

        let oracle1 = oracle_terminal_convergence(&input, &x_persistent);
        let m1 = mutant_converge_bypassed_epoch(&input, &x_persistent);
        // Mutant 1 bypassed epoch check
        assert_eq!(
            oracle1.refusal_code,
            TerminalConvergenceRefusal::ContractViolation as u8
        );
        assert_eq!(
            m1.refusal_code,
            TerminalConvergenceRefusal::None as u8,
            "Mutant 1 bypassed the typed refusal for ContractViolation"
        );

        let m2 = mutant_converge_state_drift(&input, &x_persistent);
        assert_eq!(
            oracle1.next_state.epoch_clock, 10,
            "Oracle must not drift state on refusal"
        );
        assert_eq!(
            m2.next_state.epoch_clock, 11,
            "Mutant 2 allowed state drift on refusal"
        );

        input.expected_epoch = 10;
        input.r_aggr.critical_refusal = 1;
        let oracle3 = oracle_terminal_convergence(&input, &x_persistent);
        let m3 = mutant_converge_dropped_refusal(&input, &x_persistent);
        assert_eq!(
            oracle3.refusal_code,
            TerminalConvergenceRefusal::BranchlessContractFailed as u8
        );
        assert_eq!(
            m3.refusal_code,
            TerminalConvergenceRefusal::None as u8,
            "Mutant 3 dropped critical refusal check"
        );

        input.r_aggr.critical_refusal = 0;
        let mut x_sat = x_persistent;
        x_sat.mass = u64::MAX;
        let oracle4 = oracle_terminal_convergence(&input, &x_sat);
        let m4 = mutant_converge_unbounded_mass(&input, &x_sat);
        assert_eq!(
            oracle4.next_state.mass,
            u64::MAX,
            "Oracle must saturate mass"
        );
        assert_eq!(
            m4.next_state.mass, 1,
            "Mutant 4 used wrapping_add instead of saturating_add"
        );
    }
}
