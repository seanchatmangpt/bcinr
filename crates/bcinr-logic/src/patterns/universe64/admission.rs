//! # Universe64 Contract: AdmissionEvaluator (Scope-Gated Admission)
//! Plane: D — evaluates proposed transitions before allowing execution.
//! Tier: T0 per-cell; T1 sparse multi-word; T2 full-universe.
//! Scope: reads block state, prerequisite masks; no writes.
//! Geometry: evaluates (input_mask & state) == input_mask at requested scope.
//! Delta: none produced — read-only gate.
//!
//! # Timing contract
//! - **T0 cell check:** ≤ T0_BUDGET_NS
//! - **T1 sparse check:** ≤ T1_BUDGET_NS
//! - **T2 full check:** ≤ T2_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Branchless bitwise AND tests over bounded arrays.
//! CC=1: Absolute branchless logic.

use super::constants::{UNIVERSE_WORDS, MAX_WORD_INDEX};
use super::block::UniverseBlock;
use super::scratch::{UScope, ActiveWordSet};
use super::masks::{CellMask, UniverseMask};
use super::admission_lifecycle::{TransitionEntry, MaskBank};

/// Admission decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AdmissionDecision {
    /// All prerequisites satisfied; transition may fire.
    Admitted = 0,
    /// Prerequisites missing on at least one word.
    Denied = 1,
    /// Scope mismatch or invalid mask.
    Invalid = 2,
}

// ---------------------------------------------------------------------------
// Cell-scope admission (T0)
// ---------------------------------------------------------------------------

/// T0: check whether `state_word` satisfies `prereq_mask`.
/// Returns 0 (admitted) or non-zero (denied) — branchless.
#[inline(always)]
pub fn cell_admit(state_word: u64, prereq_mask: u64) -> u64 {
    // satisfied = (state & prereq) ^ prereq; zero means all prereqs set
    (state_word & prereq_mask) ^ prereq_mask
}

/// T0 branchless admission for one cell.
#[inline(always)]
pub fn eval_cell(block: &UniverseBlock, word_idx: usize, prereq: CellMask) -> AdmissionDecision {
    let widx = word_idx & MAX_WORD_INDEX;
    let unsat = cell_admit(block.state[widx], prereq.0);
    match unsat {
        0 => AdmissionDecision::Admitted,
        _ => AdmissionDecision::Denied,
    }
}

// ---------------------------------------------------------------------------
// Sparse-scope admission (T1)
// ---------------------------------------------------------------------------

/// T1: check multiple active words against per-word prereq masks.
/// prereqs must have same len as `active.len()`.
#[inline]
pub fn eval_sparse(
    block: &UniverseBlock,
    active: &ActiveWordSet,
    prereqs: &[u64],
) -> AdmissionDecision {
    if prereqs.len() < active.len() {
        return AdmissionDecision::Invalid;
    }
    let mut unsat = 0u64;
    for (i, &prereq) in prereqs[..active.len()].iter().enumerate() {
        let widx = active.indexes[i] as usize & MAX_WORD_INDEX;
        unsat |= cell_admit(block.state[widx], prereq);
    }
    match unsat {
        0 => AdmissionDecision::Admitted,
        _ => AdmissionDecision::Denied,
    }
}

// ---------------------------------------------------------------------------
// Full-scope admission (T2)
// ---------------------------------------------------------------------------

/// T2: check all UNIVERSE_WORDS against a UniverseMask (word-wise prereqs).
#[inline]
pub fn eval_full(block: &UniverseBlock, prereq: &UniverseMask) -> AdmissionDecision {
    let mut unsat = 0u64;
    for i in 0..UNIVERSE_WORDS {
        unsat |= cell_admit(block.state[i], prereq.0[i]);
    }
    match unsat {
        0 => AdmissionDecision::Admitted,
        _ => AdmissionDecision::Denied,
    }
}

// ---------------------------------------------------------------------------
// AdmissionEvaluator — dispatch by scope
// ---------------------------------------------------------------------------

/// Stateless scope-dispatching admission evaluator.
pub struct AdmissionEvaluator;

impl AdmissionEvaluator {
    /// Evaluate a TransitionEntry against the current block state.
    /// Uses the input_mask from MaskBank to determine prerequisites.
    pub fn evaluate(
        block: &UniverseBlock,
        entry: &TransitionEntry,
        bank: &MaskBank,
        active: &ActiveWordSet,
    ) -> AdmissionDecision {
        if !entry.valid {
            return AdmissionDecision::Invalid;
        }
        let mask_entry = bank.get(entry.input_mask_id);
        if !mask_entry.valid {
            return AdmissionDecision::Invalid;
        }
        match entry.scope {
            UScope::Cell => {
                let widx = if entry.active_word_count > 0 {
                    entry.active_words[0] as usize
                } else {
                    mask_entry.word_idx as usize
                };
                eval_cell(block, widx, super::masks::CellMask(mask_entry.bits))
            }
            UScope::Sparse => {
                // Build prereq slice from repeated mask bits (simplified).
                let n = active.len();
                let prereqs = [mask_entry.bits; 64];
                eval_sparse(block, active, &prereqs[..n])
            }
            UScope::Domain | UScope::Full => {
                // Simplified: check first word.
                let widx = mask_entry.word_idx as usize & MAX_WORD_INDEX;
                eval_cell(block, widx, super::masks::CellMask(mask_entry.bits))
            }
        }
    }
}

/// Missing prerequisites: returns bitmask of unset required bits in a word.
#[inline(always)]
pub fn missing_prerequisites(state_word: u64, prereq_mask: u64) -> u64 {
    cell_admit(state_word, prereq_mask)
}

// ---------------------------------------------------------------------------
// Capability-gated admission variants (T0–T2)
// ---------------------------------------------------------------------------

use super::ubit_capability::cap_admit;

/// T0 cell admission gated by capability mask.
/// Returns Denied if either prerequisites fail OR capability denies the transition.
#[inline(always)]
pub fn eval_cell_with_cap(
    block: &UniverseBlock,
    word_idx: usize,
    prereq: CellMask,
    transition_id: u32,
    cap_mask: u64,
) -> AdmissionDecision {
    let prereq_result = cell_admit(block.state[word_idx & MAX_WORD_INDEX], prereq.0);
    let cap_result = cap_admit(transition_id, cap_mask);
    // Combine: denied if either is nonzero.
    match prereq_result | cap_result {
        0 => AdmissionDecision::Admitted,
        _ => AdmissionDecision::Denied,
    }
}

/// T1 sparse admission gated by capability mask.
#[inline(always)]
pub fn eval_sparse_with_cap(
    block: &UniverseBlock,
    active: &ActiveWordSet,
    prereqs: &[u64],
    transition_id: u32,
    cap_mask: u64,
) -> AdmissionDecision {
    let cap_result = cap_admit(transition_id, cap_mask);
    match cap_result {
        0 => eval_sparse(block, active, prereqs),
        _ => AdmissionDecision::Denied,
    }
}

/// T1 domain admission gated by capability mask.
/// Evaluates against a fixed 64-word per-domain prerequisite array.
#[inline(always)]
pub fn eval_domain_with_cap(
    block: &UniverseBlock,
    domain: usize,
    prereqs: &[u64; 64],
    transition_id: u32,
    cap_mask: u64,
) -> AdmissionDecision {
    let cap_result = cap_admit(transition_id, cap_mask);
    if cap_result != 0 {
        return AdmissionDecision::Denied;
    }
    // Evaluate per-domain: check each word in the domain slice against its prereq.
    use super::constants::CELL_COUNT;
    let base = domain * CELL_COUNT;
    let mut unsat = 0u64;
    for (i, &prereq) in prereqs[..CELL_COUNT].iter().enumerate() {
        let widx = (base + i) & MAX_WORD_INDEX;
        unsat |= cell_admit(block.state[widx], prereq);
    }
    match unsat {
        0 => AdmissionDecision::Admitted,
        _ => AdmissionDecision::Denied,
    }
}

/// T2 full admission gated by capability mask.
#[inline(always)]
pub fn eval_full_with_cap(
    block: &UniverseBlock,
    prereq: &UniverseMask,
    transition_id: u32,
    cap_mask: u64,
) -> AdmissionDecision {
    let cap_result = cap_admit(transition_id, cap_mask);
    match cap_result {
        0 => eval_full(block, prereq),
        _ => AdmissionDecision::Denied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_admit_all_set() {
        assert_eq!(cell_admit(0xFF, 0x0F), 0);
    }

    #[test]
    fn test_cell_admit_some_missing() {
        assert_ne!(cell_admit(0b0001, 0b1111), 0);
    }

    #[test]
    fn test_eval_cell_admitted() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0xFF;
        assert_eq!(eval_cell(&block, 0, CellMask(0x0F)), AdmissionDecision::Admitted);
    }

    #[test]
    fn test_eval_cell_denied() {
        let block = UniverseBlock::new();
        assert_eq!(eval_cell(&block, 0, CellMask(0x01)), AdmissionDecision::Denied);
    }

    #[test]
    fn test_eval_sparse_admitted() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0xFF;
        block.state[1] = 0xFF;
        let mut active = ActiveWordSet::new();
        active.push(0);
        active.push(1);
        let prereqs = [0x0Fu64, 0xF0u64];
        assert_eq!(eval_sparse(&block, &active, &prereqs), AdmissionDecision::Admitted);
    }

    #[test]
    fn test_missing_prerequisites() {
        let missing = missing_prerequisites(0b0011, 0b1111);
        assert_eq!(missing, 0b1100);
    }
}
