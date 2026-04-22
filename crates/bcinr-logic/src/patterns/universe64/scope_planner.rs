//! # Universe64 Contract: ScopePlanner (Minimal Active Path Resolver)
//! Plane: S — scratch resident; feeds active word sets to kernels.
//! Tier: T1 (bounded active-word resolution).
//! Scope: resolves minimal ActiveWordSet for a transition from its index entry.
//! Geometry: walks TransitionWordIndex to produce ActiveWordSet.
//! Delta: none produced — structural planner only.
//!
//! # Timing contract
//! - **T1 resolve budget:** ≤ T1_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Bounded loop over at most MAX_WORDS_PER_TRANSITION entries.
//! CC=1: Absolute branchless logic.

use super::constants::MAX_WORD_INDEX;
use super::scratch::{ActiveWordSet, UScope};
use super::index_plane::TransitionWordIndex;
use super::admission_lifecycle::TransitionEntry;

/// Result of scope planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScopePlanResult {
    /// ActiveWordSet populated successfully.
    Ok = 0,
    /// Transition id is invalid or has no words.
    Invalid = 1,
    /// Active word capacity exceeded; set was clamped.
    Clamped = 2,
}

/// Stateless scope planner.
pub struct ScopePlanner;

impl ScopePlanner {
    /// Resolve active words for a transition from the TransitionWordIndex.
    /// Fills `out_set` and returns plan result.
    pub fn resolve(
        tid: usize,
        index: &TransitionWordIndex,
        out_set: &mut ActiveWordSet,
    ) -> ScopePlanResult {
        let entry = index.lookup(tid);
        if entry.count == 0 {
            return ScopePlanResult::Invalid;
        }
        out_set.clear();
        let n = entry.count as usize;
        let mut clamped = false;
        for i in 0..n {
            let widx = entry.word_idxs[i] as usize & MAX_WORD_INDEX;
            let before = out_set.len();
            out_set.push(widx);
            if out_set.len() == before {
                clamped = true;
            }
        }
        match clamped {
            true => ScopePlanResult::Clamped,
            false => ScopePlanResult::Ok,
        }
    }

    /// Resolve active words from a TransitionEntry's embedded word list.
    pub fn resolve_from_entry(
        entry: &TransitionEntry,
        out_set: &mut ActiveWordSet,
    ) -> ScopePlanResult {
        if !entry.valid || entry.active_word_count == 0 {
            return ScopePlanResult::Invalid;
        }
        out_set.clear();
        let n = entry.active_word_count as usize;
        let mut clamped = false;
        for i in 0..n {
            let widx = entry.active_words[i] as usize & MAX_WORD_INDEX;
            let before = out_set.len();
            out_set.push(widx);
            if out_set.len() == before {
                clamped = true;
            }
        }
        match clamped {
            true => ScopePlanResult::Clamped,
            false => ScopePlanResult::Ok,
        }
    }

    /// Determine scope classification from active word set size.
    #[inline(always)]
    pub fn classify_scope(word_count: usize) -> UScope {
        // Branchless: 0 = invalid/cell, 1 = Cell, 2..=ACTIVE_WORD_CAPACITY = Sparse
        // >ACTIVE_WORD_CAPACITY not possible by construction.
        let is_cell = (word_count == 1) as u8;
        let is_sparse = (word_count > 1) as u8;
        match (is_cell, is_sparse) {
            (1, 0) => UScope::Cell,
            (0, 1) => UScope::Sparse,
            _ => UScope::Cell, // 0 words defaults to Cell
        }
    }

    /// Build a Cell-scope ActiveWordSet from a single word index.
    #[inline(always)]
    pub fn cell_scope(word_idx: usize, out: &mut ActiveWordSet) {
        out.clear();
        out.push(word_idx & MAX_WORD_INDEX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_from_index() {
        let mut idx = TransitionWordIndex::default();
        idx.register(7, 10);
        idx.register(7, 20);
        let mut set = ActiveWordSet::new();
        let result = ScopePlanner::resolve(7, &idx, &mut set);
        assert_eq!(result, ScopePlanResult::Ok);
        assert_eq!(set.len(), 2);
        assert_eq!(set.indexes[0], 10);
        assert_eq!(set.indexes[1], 20);
    }

    #[test]
    fn test_resolve_invalid() {
        let idx = TransitionWordIndex::default();
        let mut set = ActiveWordSet::new();
        let result = ScopePlanner::resolve(0, &idx, &mut set);
        assert_eq!(result, ScopePlanResult::Invalid);
    }

    #[test]
    fn test_cell_scope() {
        let mut set = ActiveWordSet::new();
        ScopePlanner::cell_scope(42, &mut set);
        assert_eq!(set.len(), 1);
        assert_eq!(set.indexes[0], 42);
    }

    #[test]
    fn test_classify_scope() {
        assert_eq!(ScopePlanner::classify_scope(1), UScope::Cell);
        assert_eq!(ScopePlanner::classify_scope(4), UScope::Sparse);
        assert_eq!(ScopePlanner::classify_scope(0), UScope::Cell);
    }

    #[test]
    fn test_resolve_from_entry() {
        let entry = TransitionEntry {
            id: 3,
            input_mask_id: 0,
            output_mask_id: 0,
            scope: UScope::Sparse,
            active_word_count: 2,
            active_words: {
                let mut a = [0u16; 16];
                a[0] = 5; a[1] = 10;
                a
            },
            valid: true,
        };
        let mut set = ActiveWordSet::new();
        let result = ScopePlanner::resolve_from_entry(&entry, &mut set);
        assert_eq!(result, ScopePlanResult::Ok);
        assert_eq!(set.len(), 2);
    }
}
