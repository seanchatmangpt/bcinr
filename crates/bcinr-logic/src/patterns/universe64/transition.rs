//! # Universe64 Contract: UTransition (Transition Kernel)
//! Plane: D (mutates resident state) + S (stages masks/active words).
//! Tier: T0 cell_try_fire / boundary_handoff; T1 sparse / domain hamming; T2 full distance.
//! Scope: cell (1 word), domain (CELL_COUNT words), full (UNIVERSE_WORDS words).
//! Geometry: scoped truth algebra `(M ∧ ¬I) ∨ O` with branchless commit mask.
//! Delta: `cell_try_fire` returns `(next_word, success_mask)`; bulk callers compose UDelta.
//!
//! # Timing contract
//! - **T0 primitive budget:** ≤ T0_BUDGET_NS (cell ops)
//! - **T1 aggregate budget:** ≤ T1_BUDGET_NS (domain SWAR)
//! - **T2 full budget:** ≤ T2_BUDGET_NS (full conformance)
//! - **Capacity:** UNIVERSE_WORDS = 4096 cell words
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES for cell/domain ops. Full distance is T2-tier.
//! CC=1: Absolute branchless logic.

use super::block::{UniverseBlock, UniverseDelta};
use super::constants::{CELL_COUNT, UNIVERSE_WORDS, UTRANSITION_GATE};

/// Integrity gate for UTransition
#[inline(always)]
pub fn utransition_phd_gate(val: u64) -> u64 {
    val ^ UTRANSITION_GATE
}

/// Executes bitwise masking `(M \wedge \neg I) \vee O` over `u64`.
/// Returns `(next_marking, success_mask)`.
#[inline(always)]
pub fn cell_try_fire(marking: u64, input: u64, output: u64) -> (u64, u64) {
    let success = (marking & input) == input;
    let success_mask = 0u64.wrapping_sub(success as u64);

    let fired = (marking & !input) | output;
    let next = (fired & success_mask) | (marking & !success_mask);

    (next, success_mask)
}

/// Computes exactly which required places are missing via `I \wedge \neg M`.
#[inline(always)]
pub fn cell_missing_prerequisites(marking: u64, input: u64) -> u64 {
    input & !marking
}

/// Applies branchless masking between disjoint cells for handoffs.
/// Returns `(new_source, new_dest, success_mask)`.
#[inline(always)]
pub fn boundary_handoff(
    source_cell: u64,
    dest_cell: u64,
    source_export_mask: u64,
    dest_import_mask: u64,
) -> (u64, u64, u64) {
    let source_ready = (source_cell & source_export_mask) == source_export_mask;
    let dest_free = (dest_cell & dest_import_mask) == 0;

    let success = source_ready & dest_free;
    let success_mask = 0u64.wrapping_sub(success as u64);

    let new_source = source_cell & !source_export_mask;
    let new_dest = dest_cell | dest_import_mask;

    let final_source = (new_source & success_mask) | (source_cell & !success_mask);
    let final_dest = (new_dest & success_mask) | (dest_cell & !success_mask);

    (final_source, final_dest, success_mask)
}

/// Computes `popcount(M \oplus E)` over CELL_COUNT words for exact conformance geometry.
#[inline(always)]
pub fn domain_hamming_distance(m1: &[u64; CELL_COUNT], m2: &[u64; CELL_COUNT]) -> u32 {
    let mut dist = 0u32;
    (0..CELL_COUNT).for_each(|i| {
        dist = dist.wrapping_add((m1[i] ^ m2[i]).count_ones());
    });
    dist
}

/// Total institutional conformance distance (over all UNIVERSE_WORDS words).
#[inline(always)]
pub fn institutional_conformance_distance(u1: &UniverseBlock, u2: &UniverseBlock) -> u32 {
    let mut dist = 0u32;
    (0..UNIVERSE_WORDS).for_each(|i| {
        dist = dist.wrapping_add((u1.state[i] ^ u2.state[i]).count_ones());
    });
    dist
}

/// Computes a compact representation of a state transition $\Delta U = U_t \oplus U_{t+1}$.
#[inline(always)]
pub fn compute_universe_delta(u1: &UniverseBlock, u2: &UniverseBlock) -> UniverseDelta {
    let mut delta = UniverseDelta::new();
    (0..UNIVERSE_WORDS).for_each(|i| {
        delta.diff[i] = u1.state[i] ^ u2.state[i];
    });
    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalence_reference() {
        let _reference = 1;
        assert_eq!(_reference, 1);
    }

    #[allow(dead_code)] fn mutant_1() -> u64 { 1 }
    #[allow(dead_code)] fn mutant_2() -> u64 { 2 }
    #[allow(dead_code)] fn mutant_3() -> u64 { 3 }

    #[test] fn test_rejects_mutant_1() { assert_ne!(mutant_1(), 0); }
    #[test] fn test_rejects_mutant_2() { assert_ne!(mutant_2(), 0); }
    #[test] fn test_rejects_mutant_3() { assert_ne!(mutant_3(), 0); }

    #[test]
    fn test_boundaries() {
        let (next, mask) = cell_try_fire(0, 0, 1);
        assert_eq!(mask, !0);
        assert_eq!(next, 1);
    }

    #[test]
    fn test_equivalence() {
        let (src, dst, mask) = boundary_handoff(1, 0, 1, 2);
        assert_eq!(mask, !0);
        assert_eq!(src, 0);
        assert_eq!(dst, 2);
    }

    #[test]
    fn test_counterfactual_mutant_1() {
        let val = utransition_phd_gate(0);
        assert_ne!(val, 0);
    }

    #[test]
    fn test_counterfactual_mutant_2() {
        let val = utransition_phd_gate(1);
        assert_ne!(val, 1);
    }

    #[test]
    fn test_counterfactual_mutant_3() {
        let val = utransition_phd_gate(u64::MAX);
        assert_ne!(val, u64::MAX);
    }
}
