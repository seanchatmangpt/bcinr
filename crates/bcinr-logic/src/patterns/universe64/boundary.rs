//! # Universe64 Contract: BoundaryKernel (Cross-Domain Motion)
//! Plane: D — source domain reads; D — dest domain writes.
//! Tier: T1 (bounded active words per side).
//! Scope: sparse source + sparse dest; ACTIVE_WORD_CAPACITY per side.
//! Geometry: handoff = copy source_words to dest_words under fired_mask.
//! Delta: emits UDelta per dest word touched.
//!
//! # Timing contract
//! - **T1 handoff budget:** ≤ T1_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Bounded-loop bitwise copy with fired_mask gating.
//! CC=1: Absolute branchless logic.

use super::constants::{MAX_WORD_INDEX, ACTIVE_WORD_CAPACITY};
use super::block::UniverseBlock;
use super::scratch::{UDelta, UScope};
use super::admission::missing_prerequisites;

/// Diagnostic from a boundary check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BoundaryStatus {
    /// Handoff completed (all prerequisites met on source side).
    Ok = 0,
    /// Source prerequisites missing — handoff blocked.
    PrereqMissing = 1,
    /// Boundary is empty (no words to transfer).
    Empty = 2,
}

/// Result of boundary execution: deltas emitted and status.
#[derive(Clone, Copy, Debug)]
pub struct BoundaryResult {
    pub status: BoundaryStatus,
    /// Number of UDelta records written into `out_deltas`.
    pub delta_count: u8,
    /// Missing-prerequisite bitmask (OR of all unmet prereqs).
    pub missing_mask: u64,
}

/// Describe one side of a boundary (source or dest) as word_idx + prereq_mask pairs.
#[derive(Clone, Copy, Debug)]
pub struct BoundarySide {
    pub count: u8,
    pub word_idxs: [u16; ACTIVE_WORD_CAPACITY],
    pub prereq_masks: [u64; ACTIVE_WORD_CAPACITY],
}

impl BoundarySide {
    pub const fn empty() -> Self {
        Self { count: 0, word_idxs: [0u16; ACTIVE_WORD_CAPACITY], prereq_masks: [0u64; ACTIVE_WORD_CAPACITY] }
    }

    pub fn push(&mut self, word_idx: usize, prereq: u64) {
        let admit = (self.count as usize) < ACTIVE_WORD_CAPACITY;
        let slot = (self.count as usize) & (ACTIVE_WORD_CAPACITY - 1);
        self.word_idxs[slot] = ((word_idx & MAX_WORD_INDEX) as u16) * admit as u16;
        self.prereq_masks[slot] = prereq * admit as u64;
        self.count = self.count.wrapping_add(admit as u8);
    }
}

// ---------------------------------------------------------------------------
// BoundaryKernel
// ---------------------------------------------------------------------------

/// Stateless boundary execution kernel.
/// Transfers truth values from source words to dest words under fired_mask.
pub struct BoundaryKernel;

impl BoundaryKernel {
    /// Check source prerequisites; return OR of all missing prereq bits.
    #[inline]
    pub fn check_prerequisites(
        block: &UniverseBlock,
        source: &BoundarySide,
    ) -> u64 {
        let mut missing = 0u64;
        for i in 0..source.count as usize {
            let widx = source.word_idxs[i] as usize & MAX_WORD_INDEX;
            missing |= missing_prerequisites(block.state[widx], source.prereq_masks[i]);
        }
        missing
    }

    /// Execute handoff: for each dest word, copy the corresponding source word
    /// gated by `fired_mask`. Emits UDelta per word changed.
    /// `out_deltas` must have capacity >= dest.count.
    pub fn execute(
        block: &mut UniverseBlock,
        source: &BoundarySide,
        dest: &BoundarySide,
        fired_mask: u64,
        instr_id: u32,
        out_deltas: &mut [UDelta; ACTIVE_WORD_CAPACITY],
    ) -> BoundaryResult {
        if dest.count == 0 {
            return BoundaryResult { status: BoundaryStatus::Empty, delta_count: 0, missing_mask: 0 };
        }

        let missing = Self::check_prerequisites(block, source);
        // Branchless: if missing != 0, fired_mask becomes 0 (no writes).
        let gate = (missing == 0) as u64 * fired_mask;

        let n = dest.count as usize;
        let m = source.count as usize;
        let mut delta_count = 0u8;

        for i in 0..n {
            let dwidx = dest.word_idxs[i] as usize & MAX_WORD_INDEX;
            // Source word: use same index if parallel, else first source word.
            let src_val = if i < m {
                block.state[source.word_idxs[i] as usize & MAX_WORD_INDEX]
            } else {
                block.state[source.word_idxs[0] as usize & MAX_WORD_INDEX]
            };

            let before = block.state[dwidx];
            // Branchless write: new = (src_val & gate) | (before & !gate)
            let new_val = (src_val & gate) | (before & !gate);
            block.state[dwidx] = new_val;

            out_deltas[delta_count as usize] = UDelta::new(
                dwidx, UScope::Sparse, instr_id, before, new_val, gate,
            );
            delta_count = delta_count.wrapping_add(1);
        }

        let status = match missing {
            0 => BoundaryStatus::Ok,
            _ => BoundaryStatus::PrereqMissing,
        };

        BoundaryResult { status, delta_count, missing_mask: missing }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source() -> BoundarySide {
        let mut s = BoundarySide::empty();
        s.push(0, 0); // word 0, no prereq
        s
    }

    fn make_dest() -> BoundarySide {
        let mut d = BoundarySide::empty();
        d.push(1, 0); // word 1, no prereq
        d
    }

    #[test]
    fn test_boundary_handoff_ok() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0xABCD_EF01;
        let src = make_source();
        let dst = make_dest();
        let mut deltas = [UDelta::new(0, UScope::Cell, 0, 0, 0, 0); ACTIVE_WORD_CAPACITY];
        let result = BoundaryKernel::execute(&mut block, &src, &dst, !0, 1, &mut deltas);
        assert_eq!(result.status, BoundaryStatus::Ok);
        assert_eq!(block.state[1], 0xABCD_EF01);
        assert_eq!(result.delta_count, 1);
    }

    #[test]
    fn test_boundary_prereq_missing() {
        let block = UniverseBlock::new(); // all zeros
        let mut src = BoundarySide::empty();
        src.push(0, 0xFF); // word 0, prereq = all bits set
        let dst = make_dest();
        // block.state[0] = 0, but prereq = 0xFF → missing
        let mut block2 = block;
        let mut deltas = [UDelta::new(0, UScope::Cell, 0, 0, 0, 0); ACTIVE_WORD_CAPACITY];
        let result = BoundaryKernel::execute(&mut block2, &src, &dst, !0, 1, &mut deltas);
        assert_eq!(result.status, BoundaryStatus::PrereqMissing);
        assert_ne!(result.missing_mask, 0);
    }

    #[test]
    fn test_check_prerequisites_all_met() {
        let mut block = UniverseBlock::new();
        block.state[5] = 0xFF;
        let mut src = BoundarySide::empty();
        src.push(5, 0x0F);
        assert_eq!(BoundaryKernel::check_prerequisites(&block, &src), 0);
    }
}
