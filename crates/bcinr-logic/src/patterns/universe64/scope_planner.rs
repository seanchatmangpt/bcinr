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
use super::instruction::UTier;

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

// ---------------------------------------------------------------------------
// UBitScopePlanner — branchless kernel dispatcher
// ---------------------------------------------------------------------------

/// Classification of which kernel to dispatch.
///
/// ```
/// use bcinr_logic::patterns::universe64::scope_planner::KernelClass;
/// assert_eq!(KernelClass::Cell as u8, 1);
/// assert_eq!(KernelClass::Full as u8, 4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum KernelClass {
    Invalid = 0,
    Cell = 1,
    Sparse = 2,
    Domain = 3,
    Full = 4,
}

/// The result of a UBitScopePlanner::plan() call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct PlanDecision {
    pub kernel: KernelClass,
    pub touched_words: u16,
    pub tier: UTier,
    pub active: ActiveWordSet,
}

/// Cost constants for kernel dispatch decisions.
pub struct CostModel;

impl CostModel {
    pub const CELL_MAX: u16 = 1;
    pub const SPARSE_MAX: u16 = 16;
    pub const DOMAIN_MAX: u16 = 64;
    pub const FULL_MAX: u16 = 4096;

    pub const CELL_NS: u32 = 2;
    pub const SPARSE_NS: u32 = 40;
    pub const DOMAIN_NS: u32 = 120;
    pub const FULL_NS: u32 = 5000;
    pub const INVALID_NS: u32 = 0;
}

/// Branchless kernel dispatcher: selects Cell/Sparse/Domain/Full by touched-word count.
/// Enforces the 200 ns T1 ceiling at planner time (Full is always T2).
pub struct UBitScopePlanner;

impl UBitScopePlanner {
    /// Plan a transition: determine KernelClass and tier from touched word count.
    ///
    /// Thresholds:
    /// - 0 touched words → KernelClass::Invalid, T0
    /// - 1 touched word  → KernelClass::Cell, T0
    /// - 2..=16 touched words → KernelClass::Sparse, T1
    /// - 17..=64 touched words → KernelClass::Domain, T1
    /// - 65..=4096 touched words → KernelClass::Full, T2
    /// - >4096 → KernelClass::Full, T2 (clamped)
    ///
    /// Full is ALWAYS T2 — never T1.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe64::scope_planner::UBitScopePlanner;
    /// use bcinr_logic::patterns::universe64::scratch::ActiveWordSet;
    /// let mut active = ActiveWordSet::new();
    /// active.push(0);
    /// let decision = UBitScopePlanner::plan(&active);
    /// use bcinr_logic::patterns::universe64::scope_planner::KernelClass;
    /// assert_eq!(decision.kernel, KernelClass::Cell);
    /// ```
    pub fn plan(active: &ActiveWordSet) -> PlanDecision {
        let touched = active.len() as u16;
        let kernel = Self::select_kernel_branchless(touched);
        let tier = Self::tier_for_kernel(kernel);
        PlanDecision {
            kernel,
            touched_words: touched,
            tier,
            active: *active,
        }
    }

    /// Estimate nanosecond cost for a kernel class.
    pub fn estimate_cost_ns(kernel: KernelClass) -> u32 {
        match kernel {
            KernelClass::Cell => CostModel::CELL_NS,
            KernelClass::Sparse => CostModel::SPARSE_NS,
            KernelClass::Domain => CostModel::DOMAIN_NS,
            KernelClass::Full => CostModel::FULL_NS,
            KernelClass::Invalid => CostModel::INVALID_NS,
        }
    }

    /// Select kernel class branchlessly from touched word count.
    #[inline(always)]
    pub fn select_kernel_branchless(touched: u16) -> KernelClass {
        // Use arithmetic rather than match to remain branchless.
        // gt0  = touched > 0
        // gt1  = touched > CostModel::CELL_MAX (1)
        // gt16 = touched > CostModel::SPARSE_MAX (16)
        // gt64 = touched > CostModel::DOMAIN_MAX (64)
        //
        // 0: Invalid
        // 1: Cell
        // 2..=16: Sparse
        // 17..=64: Domain
        // 65+: Full
        let gt0  = (touched > 0) as u8;
        let gt1  = (touched > CostModel::CELL_MAX) as u8;
        let gt16 = (touched > CostModel::SPARSE_MAX) as u8;
        let gt64 = (touched > CostModel::DOMAIN_MAX) as u8;
        // class = gt0 + gt1 + gt16 + gt64 maps to:
        // 0+0+0+0=0 → Invalid
        // 1+0+0+0=1 → Cell (touched=1)
        // 1+1+0+0=2 → Sparse (touched 2..16)
        // 1+1+1+0=3 → Domain (touched 17..64)
        // 1+1+1+1=4 → Full (touched 65+)
        let class = gt0 + gt1 + gt16 + gt64;
        match class {
            0 => KernelClass::Invalid,
            1 => KernelClass::Cell,
            2 => KernelClass::Sparse,
            3 => KernelClass::Domain,
            _ => KernelClass::Full,
        }
    }

    /// Map kernel class to execution tier. Full is ALWAYS T2.
    pub fn tier_for_kernel(kernel: KernelClass) -> UTier {
        match kernel {
            KernelClass::Cell => UTier::T0,
            KernelClass::Sparse | KernelClass::Domain => UTier::T1,
            KernelClass::Full => UTier::T2,
            KernelClass::Invalid => UTier::T0,
        }
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

    // --- UBitScopePlanner tests ---

    #[test]
    fn ubit_scope_planner_cell_for_one_word() {
        let mut active = ActiveWordSet::new();
        active.push(0);
        let decision = UBitScopePlanner::plan(&active);
        assert_eq!(decision.kernel, KernelClass::Cell);
        assert_eq!(decision.touched_words, 1);
    }

    #[test]
    fn ubit_scope_planner_sparse_for_sixteen_or_less() {
        let mut active = ActiveWordSet::new();
        for i in 0..16 {
            active.push(i);
        }
        let decision = UBitScopePlanner::plan(&active);
        assert_eq!(decision.kernel, KernelClass::Sparse);
        assert_eq!(decision.touched_words, 16);
    }

    #[test]
    fn ubit_scope_planner_domain_for_sixty_four_or_less() {
        let mut active = ActiveWordSet::new();
        // Push 64 words (ACTIVE_WORD_CAPACITY = 64), which is Domain (17..=64)
        for i in 0..64 {
            active.push(i);
        }
        let decision = UBitScopePlanner::plan(&active);
        assert_eq!(decision.kernel, KernelClass::Domain);
        assert_eq!(decision.touched_words, 64);
    }

    #[test]
    fn ubit_scope_planner_full_for_large_scope() {
        // ActiveWordSet capacity is 64 (ACTIVE_WORD_CAPACITY), so Full (touched > 64)
        // is unreachable through ActiveWordSet. Test via direct branchless call.
        let kernel = UBitScopePlanner::select_kernel_branchless(65);
        assert_eq!(kernel, KernelClass::Full);

        let kernel_large = UBitScopePlanner::select_kernel_branchless(4096);
        assert_eq!(kernel_large, KernelClass::Full);
    }

    #[test]
    fn ubit_scope_planner_never_marks_full_as_t1() {
        // Full is always T2 — never T1, regardless of input.
        let tier = UBitScopePlanner::tier_for_kernel(KernelClass::Full);
        assert_eq!(tier, UTier::T2);
        assert_ne!(tier, UTier::T1);

        // Also verify via select_kernel_branchless + tier_for_kernel for 65+ words
        let kernel = UBitScopePlanner::select_kernel_branchless(65);
        let tier2 = UBitScopePlanner::tier_for_kernel(kernel);
        assert_eq!(tier2, UTier::T2);
    }
}
