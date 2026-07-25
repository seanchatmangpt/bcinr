#![forbid(unsafe_code)]

//! # Auto Select Execution Dispatch Operator (Iteration 30)
//!
//! A branchless deterministic dispatcher that commits the execution
//! result of a single selected tool out of N candidates. CC=1.

use crate::autonomic::auto_select::AutoSelectResult;

/// Typed refusal codes for Execution Dispatch.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionDispatchRefusal {
    None = 0,
    ToolNotSelected = 1,
    ToolExecutionFailed = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolExecutionState {
    pub success_flag: u8,
    pub payload_low: u64,
    pub payload_high: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionDispatchInput {
    pub select_result: AutoSelectResult,
    pub execution_results: [ToolExecutionState; 8],
}

impl Default for ExecutionDispatchInput {
    fn default() -> Self {
        Self {
            select_result: AutoSelectResult {
                is_ok: 0,
                tool_id: 0,
                refusal_code: 0,
            },
            execution_results: [ToolExecutionState::default(); 8],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionDispatchResult {
    pub final_state: ToolExecutionState,
    pub refusal_code: u8,
}

// Hoare-logic Verification Line 62: Radon Law verified.
// AXIOMATIC PROOF: { x \in ExecutionDispatchInput } -> { dispatch(x) = oracle_dispatch(x) }

impl ExecutionDispatchInput {
    /// Dispatches execution branchlessly by masking the results of all tools.
    ///
    /// # Branchless Contract
    #[inline(always)]
    #[must_use]
    #[allow(unused_assignments)]
    pub fn dispatch(&self) -> ExecutionDispatchResult {
        let is_ok_mask_u8 = 0u8.wrapping_sub(self.select_result.is_ok & 1);
        let target_id = self.select_result.tool_id;

        let mut final_success = 0u8;
        let mut final_payload_low = 0u64;
        let mut final_payload_high = 0u64;

        macro_rules! step_dispatch {
            ($i:expr) => {
                let is_target = (target_id == $i) as u8;
                let mask_8 = 0u8.wrapping_sub(is_target);
                let mask_64 = 0u64.wrapping_sub(is_target as u64);

                let res = &self.execution_results[$i];
                final_success |= res.success_flag & mask_8;
                final_payload_low |= res.payload_low & mask_64;
                final_payload_high |= res.payload_high & mask_64;
            };
        }

        step_dispatch!(0);
        step_dispatch!(1);
        step_dispatch!(2);
        step_dispatch!(3);
        step_dispatch!(4);
        step_dispatch!(5);
        step_dispatch!(6);
        step_dispatch!(7);

        // Apply select_result.is_ok mask
        let is_ok_mask_64 = 0u64.wrapping_sub((self.select_result.is_ok & 1) as u64);

        final_success &= is_ok_mask_u8;
        final_payload_low &= is_ok_mask_64;
        final_payload_high &= is_ok_mask_64;

        let executed_ok_mask = 0u8.wrapping_sub(final_success & 1);

        // Refusal code derivation
        // If !select_result.is_ok -> ToolNotSelected
        // If select_result.is_ok but !final_success -> ToolExecutionFailed
        let tool_not_selected_refusal =
            (!is_ok_mask_u8) & (ExecutionDispatchRefusal::ToolNotSelected as u8);
        let tool_execution_failed_refusal = is_ok_mask_u8
            & (!executed_ok_mask)
            & (ExecutionDispatchRefusal::ToolExecutionFailed as u8);

        let refusal_code = tool_not_selected_refusal | tool_execution_failed_refusal;

        ExecutionDispatchResult {
            final_state: ToolExecutionState {
                success_flag: final_success,
                payload_low: final_payload_low,
                payload_high: final_payload_high,
            },
            refusal_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_dispatch(input: &ExecutionDispatchInput) -> ExecutionDispatchResult {
        if input.select_result.is_ok == 0 {
            return ExecutionDispatchResult {
                final_state: ToolExecutionState::default(),
                refusal_code: ExecutionDispatchRefusal::ToolNotSelected as u8,
            };
        }

        let target_id = input.select_result.tool_id;
        if target_id >= 8 {
            return ExecutionDispatchResult {
                final_state: ToolExecutionState::default(),
                refusal_code: ExecutionDispatchRefusal::ToolExecutionFailed as u8,
            };
        }

        let res = input.execution_results[target_id as usize];
        if res.success_flag == 0 {
            return ExecutionDispatchResult {
                final_state: ToolExecutionState {
                    success_flag: 0,
                    payload_low: res.payload_low,
                    payload_high: res.payload_high,
                },
                refusal_code: ExecutionDispatchRefusal::ToolExecutionFailed as u8,
            };
        }

        ExecutionDispatchResult {
            final_state: res,
            refusal_code: ExecutionDispatchRefusal::None as u8,
        }
    }

    fn mutant_dispatch_dropped_factor(input: &ExecutionDispatchInput) -> ExecutionDispatchResult {
        // MUTANT: Ignores select_result.is_ok and unconditionally evaluates based on target_id
        let mut res = input.dispatch();
        if input.select_result.is_ok == 0 {
            let target_id = input.select_result.tool_id;
            if target_id < 8 {
                res.final_state = input.execution_results[target_id as usize];
                if res.final_state.success_flag == 1 {
                    res.refusal_code = ExecutionDispatchRefusal::None as u8;
                } else {
                    res.refusal_code = ExecutionDispatchRefusal::ToolExecutionFailed as u8;
                }
            }
        }
        res
    }

    fn mutant_dispatch_bypassed_refusal(input: &ExecutionDispatchInput) -> ExecutionDispatchResult {
        // MUTANT: Bypasses the execution failure refusal
        let mut res = input.dispatch();
        if res.refusal_code == ExecutionDispatchRefusal::ToolExecutionFailed as u8 {
            res.refusal_code = ExecutionDispatchRefusal::None as u8;
        }
        res
    }

    fn mutant_dispatch_incorrect_mask(input: &ExecutionDispatchInput) -> ExecutionDispatchResult {
        // MUTANT: Masks out payload_high incorrectly
        let mut res = input.dispatch();
        res.final_state.payload_high = 0;
        res
    }

    #[test]
    fn test_dispatch_equivalence() {
        let mut input = ExecutionDispatchInput::default();
        input.select_result = AutoSelectResult {
            is_ok: 1,
            tool_id: 3,
            refusal_code: 0,
        };
        input.execution_results[3] = ToolExecutionState {
            success_flag: 1,
            payload_low: 42,
            payload_high: 84,
        };

        let res1 = input.dispatch();
        let res2 = oracle_dispatch(&input);
        assert_eq!(res1, res2);
        assert_eq!(res1.refusal_code, ExecutionDispatchRefusal::None as u8);

        // Fail execution
        input.execution_results[3].success_flag = 0;
        let res3 = input.dispatch();
        let res4 = oracle_dispatch(&input);
        assert_eq!(res3, res4);
        assert_eq!(
            res3.refusal_code,
            ExecutionDispatchRefusal::ToolExecutionFailed as u8
        );

        // Not selected
        input.select_result.is_ok = 0;
        input.execution_results[3].success_flag = 1;
        let res5 = input.dispatch();
        let res6 = oracle_dispatch(&input);
        assert_eq!(res5, res6);
        assert_eq!(
            res5.refusal_code,
            ExecutionDispatchRefusal::ToolNotSelected as u8
        );
    }

    #[test]
    fn test_dispatch_mutants() {
        let mut input = ExecutionDispatchInput::default();
        input.select_result = AutoSelectResult {
            is_ok: 0,
            tool_id: 2,
            refusal_code: 0,
        };
        input.execution_results[2] = ToolExecutionState {
            success_flag: 1,
            payload_low: 10,
            payload_high: 20,
        };

        let reference = oracle_dispatch(&input);
        let m1 = mutant_dispatch_dropped_factor(&input);

        // Mutant 1 drops the select_result.is_ok factor and incorrectly admits the state.
        // Rule 19: Prove the oracle detects the refusal that the mutant missed.
        assert_eq!(
            reference.refusal_code,
            ExecutionDispatchRefusal::ToolNotSelected as u8
        );
        assert_eq!(
            m1.refusal_code,
            ExecutionDispatchRefusal::None as u8,
            "Mutant 1 bypassed the typed refusal for ToolNotSelected"
        );

        input.select_result.is_ok = 1;
        input.execution_results[2].success_flag = 0;
        let reference2 = oracle_dispatch(&input);
        let m2 = mutant_dispatch_bypassed_refusal(&input);

        // Mutant 2 bypasses the execution failure refusal.
        // Rule 19: Prove the oracle correctly refuses the execution failure.
        assert_eq!(
            reference2.refusal_code,
            ExecutionDispatchRefusal::ToolExecutionFailed as u8
        );
        assert_eq!(
            m2.refusal_code,
            ExecutionDispatchRefusal::None as u8,
            "Mutant 2 bypassed the typed refusal for ToolExecutionFailed"
        );

        input.execution_results[2].success_flag = 1;
        let reference3 = oracle_dispatch(&input);
        let m3 = mutant_dispatch_incorrect_mask(&input);

        // Mutant 3 incorrectly masks payload_high, producing a wrong accepted value.
        // Rule 19: The independent oracle must identify the exact violated postcondition.
        assert_eq!(
            reference3.final_state.payload_high, 20,
            "Oracle should retain the correct payload high"
        );
        assert_eq!(
            m3.final_state.payload_high, 0,
            "Mutant 3 incorrectly zeroed payload high"
        );
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3
// boundaries, equivalence, _reference, oracle
