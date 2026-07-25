#![forbid(unsafe_code)]

//! Auto Select and POWL Execution Tape Integration Oracle
//!
//! Implements the deterministic mathematical bridge between the `mfw-auto-select`
//! hot path and the `bcinr-powl` token execution tape scheduler.
//!
//! Complies with the BCINR Deterministic Substrate Constitution.
//! CC=1, 0 heap allocations, deterministic tape integration.

use crate::scheduler::PowlRunState;
#[cfg(test)]
use bcinr_logic::autonomic::canonical_mass::AutoSelectRefusal;
use bcinr_logic::autonomic::canonical_mass::AutoSelectResult;

/// Bridges the auto select outcome into a POWL tape fire mask.
///
/// Converts a branchless `AutoSelectResult` into a 64-bit mask suitable for
/// directly transitioning a token onto the `bcinr-powl` execution tape.
///
/// Mathematical Law:
/// $$ T_{mask} = V_{valid} \land (1 \ll S_{out}) $$
#[inline(always)]
#[must_use]
pub fn powl_bridge_select(result: &AutoSelectResult) -> u64 {
    // Map is_ok (0 or 1) to a full 64-bit mask (0 or !0) using two's complement underflow.
    let is_ok = result.is_ok as u64;
    let valid_mask = 0u64.wrapping_sub(is_ok);

    // Compute the bit shift. The domain of tool_id in auto select is [0, 7].
    // Bitwise AND with 63 strictly bounds the shift and prevents any panic paths.
    let bounded_shift = (result.tool_id as u64) & 63;
    let shift_mask = 1u64 << bounded_shift;

    // Compute the output token mask using algebraic selection.
    valid_mask & shift_mask
}

/// Admits a POWL selection directly into the execution run state.
///
/// Deterministically commits the $T_{mask}$ to the `active_mask` without any
/// conditional branching or heap allocation.
///
/// Mathematical Law:
/// $$ active\_mask_{t+1} = active\_mask_t \lor T_{mask} $$
#[inline(always)]
pub fn powl_admit_selection(state: &mut PowlRunState, t_mask: u64) {
    state.active_mask |= t_mask;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::PowlRunState;
    use crate::tape::PowlTape;
    use proptest::prelude::*;

    /// Independent oracle for `powl_bridge_select`
    fn oracle_powl_bridge_select(result: &AutoSelectResult) -> u64 {
        if result.is_ok == 1 {
            1u64.wrapping_shl(result.tool_id as u32)
        } else {
            0
        }
    }

    // Hostile mutants

    fn mutant_bridge_select_bypassed_refusal(result: &AutoSelectResult) -> u64 {
        // MUTANT: Ignores is_ok refusal entirely
        let bounded_shift = (result.tool_id as u64) & 63;
        1u64 << bounded_shift
    }

    fn mutant_bridge_select_incorrect_mask(result: &AutoSelectResult) -> u64 {
        // MUTANT: Flips is_ok mapping logic
        let valid_mask = 0u64.wrapping_sub((result.is_ok == 0) as u64);
        let bounded_shift = (result.tool_id as u64) & 63;
        valid_mask & (1u64 << bounded_shift)
    }

    fn mutant_admit_stale_state(state: &mut PowlRunState, t_mask: u64) {
        // MUTANT: Ignores state mutation on admission
        let _ = t_mask;
        state.active_mask |= 0;
    }

    #[test]
    fn test_powl_bridge_select_equivalence() {
        let valid_result = AutoSelectResult {
            is_ok: 1,
            tool_id: 3,
            refusal_code: AutoSelectRefusal::None as u8,
        };
        let t_mask = powl_bridge_select(&valid_result);
        assert_eq!(t_mask, 1u64 << 3);
        assert_eq!(t_mask, oracle_powl_bridge_select(&valid_result));

        let refused_result = AutoSelectResult {
            is_ok: 0,
            tool_id: 0,
            refusal_code: AutoSelectRefusal::ControlStateUnadmitted as u8,
        };
        let t_mask_refused = powl_bridge_select(&refused_result);
        assert_eq!(t_mask_refused, 0);
        assert_eq!(t_mask_refused, oracle_powl_bridge_select(&refused_result));
    }

    #[test]
    fn test_powl_bridge_select_mutants() {
        let refused_result = AutoSelectResult {
            is_ok: 0,
            tool_id: 2,
            refusal_code: AutoSelectRefusal::ControlStateUnadmitted as u8,
        };

        let reference = oracle_powl_bridge_select(&refused_result);
        assert_eq!(reference, 0);

        let m1 = mutant_bridge_select_bypassed_refusal(&refused_result);
        assert_eq!(
            m1, 4,
            "Mutant 1 incorrectly bypasses refusal and emits a mask"
        );

        let m2 = mutant_bridge_select_incorrect_mask(&refused_result);
        assert_eq!(m2, 4, "Mutant 2 inversely maps refusal and emits a mask");
    }

    #[test]
    fn test_powl_admit_selection_mutant() {
        let tape = PowlTape::default();
        let mut state = PowlRunState::new(&tape);

        let t_mask = 1u64 << 3;

        // Oracle expectation
        let mut expected_state = state.clone();
        expected_state.active_mask = t_mask;

        // Hostile mutant test
        let mut m_state = state.clone();
        mutant_admit_stale_state(&mut m_state, t_mask);
        assert_eq!(
            m_state.active_mask, 0,
            "Mutant should not update state properly"
        );
        assert_ne!(m_state.active_mask, expected_state.active_mask);

        // Actual function
        powl_admit_selection(&mut state, t_mask);
        assert_eq!(state.active_mask, expected_state.active_mask);
    }

    proptest! {
        #[test]
        fn test_powl_bridge_select_proptest(is_ok in 0u8..=1, tool_id in 0u8..=7) {
            let result = AutoSelectResult {
                is_ok,
                tool_id,
                refusal_code: AutoSelectRefusal::None as u8,
            };

            let t_mask = powl_bridge_select(&result);
            let expected = oracle_powl_bridge_select(&result);

            assert_eq!(t_mask, expected);
            if is_ok == 1 {
                assert_eq!(t_mask.count_ones(), 1);
                assert_eq!(t_mask, 1u64 << tool_id);
            } else {
                assert_eq!(t_mask, 0);
            }
        }
    }
}
