//! # Universe64 Contract: ReadyMask (Per-Transition Ready Bit Scheduler)
//! Plane: S — scratch-resident readiness state.
//! Tier: T0 per-bit update; T1 schedule scan.
//! Scope: one u64 bitmask per 64 transition slots; bounded.
//! Geometry: bit i set in ReadyMask = transition i may be scheduled.
//! Delta: reads UDelta to drive readiness updates; no new deltas emitted.
//!
//! # Timing contract
//! - **T0 set/clear/test:** ≤ T0_BUDGET_NS
//! - **T1 scan for ready set:** ≤ T1_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Bitwise operations on u64 words; branchless scan.
//! CC=1: Absolute branchless logic.

use super::scratch::UDelta;
use super::index_plane::{MAX_TRANSITIONS, MAX_TRANSITIONS_PER_BIT, BitTransitionIndex};
use super::block::UniverseBlock;
use super::admission::cell_admit;

/// Number of u64 words in the ready mask (covers MAX_TRANSITIONS bits).
pub const READY_MASK_WORDS: usize = MAX_TRANSITIONS / 64;

/// Per-transition ready bit scheduler.
/// Bit i set → transition i is ready to fire.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReadyMask {
    words: [u64; READY_MASK_WORDS],
}

impl ReadyMask {
    pub const fn new() -> Self {
        Self { words: [0u64; READY_MASK_WORDS] }
    }

    /// Set ready bit for transition `id`.
    #[inline(always)]
    pub fn set_ready(&mut self, id: usize) {
        let word = (id / 64) & (READY_MASK_WORDS - 1);
        let bit = id & 63;
        self.words[word] |= 1u64 << bit;
    }

    /// Clear ready bit for transition `id`.
    #[inline(always)]
    pub fn clear_ready(&mut self, id: usize) {
        let word = (id / 64) & (READY_MASK_WORDS - 1);
        let bit = id & 63;
        self.words[word] &= !(1u64 << bit);
    }

    /// Test if transition `id` is ready (branchless).
    #[inline(always)]
    pub fn is_ready(&self, id: usize) -> bool {
        let word = (id / 64) & (READY_MASK_WORDS - 1);
        let bit = id & 63;
        (self.words[word] >> bit) & 1 != 0
    }

    /// Count ready transitions.
    #[inline]
    pub fn ready_count(&self) -> u32 {
        let mut n = 0u32;
        for w in &self.words {
            n = n.wrapping_add(w.count_ones());
        }
        n
    }

    /// Fill `out` with ready transition ids (up to `out.len()`). Returns count filled.
    pub fn collect_ready(&self, out: &mut [u16]) -> usize {
        let mut n = 0usize;
        'outer: for w in 0..READY_MASK_WORDS {
            let mut word = self.words[w];
            while word != 0 {
                let bit = word.trailing_zeros() as usize;
                let tid = w * 64 + bit;
                if n < out.len() { out[n] = tid as u16; n += 1; } else { break 'outer; }
                word &= word.wrapping_sub(1);
            }
        }
        n
    }

    /// Reset all bits to zero.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.words = [0u64; READY_MASK_WORDS];
    }

    /// Update readiness for all transitions sensitive to a changed bit.
    /// For each transition in `bit_transition`, recheck prerequisites in `block`.
    pub fn update_on_delta(
        &mut self,
        delta: &UDelta,
        bit_index: &BitTransitionIndex,
        block: &UniverseBlock,
        prereqs: &[u64; MAX_TRANSITIONS],
    ) {
        let widx = delta.word_idx as usize;
        // Probe all 64 bit positions in the changed word.
        let changed_bits = delta.changed_mask();
        let mut bits = changed_bits;
        let mut tid_buf = [0u16; MAX_TRANSITIONS_PER_BIT];
        while bits != 0 {
            let bit = bits.trailing_zeros();
            let n = bit_index.lookup(widx, bit, &mut tid_buf);
            for &tid_raw in &tid_buf[..n] {
                let tid = tid_raw as usize;
                let prereq = prereqs[tid & (MAX_TRANSITIONS - 1)];
                let satisfied = cell_admit(block.state[widx], prereq) == 0;
                let bit_pos = tid & 63;
                let word_slot = (tid / 64) & (READY_MASK_WORDS - 1);
                // Branchless set/clear.
                let m = 1u64 << bit_pos;
                let set_mask = satisfied as u64 * m;
                let clr_mask = !satisfied as u64 * m;
                self.words[word_slot] = (self.words[word_slot] | set_mask) & !clr_mask;
            }
            bits &= bits.wrapping_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_test() {
        let mut rm = ReadyMask::new();
        rm.set_ready(5);
        assert!(rm.is_ready(5));
        assert!(!rm.is_ready(6));
    }

    #[test]
    fn test_clear_ready() {
        let mut rm = ReadyMask::new();
        rm.set_ready(10);
        rm.clear_ready(10);
        assert!(!rm.is_ready(10));
    }

    #[test]
    fn test_ready_count() {
        let mut rm = ReadyMask::new();
        rm.set_ready(0);
        rm.set_ready(1);
        rm.set_ready(63);
        assert_eq!(rm.ready_count(), 3);
    }

    #[test]
    fn test_collect_ready() {
        let mut rm = ReadyMask::new();
        rm.set_ready(2);
        rm.set_ready(5);
        rm.set_ready(100);
        let mut out = [0u16; 16];
        let n = rm.collect_ready(&mut out);
        assert_eq!(n, 3);
        assert_eq!(&out[..3], &[2, 5, 100]);
    }

    #[test]
    fn test_reset() {
        let mut rm = ReadyMask::new();
        rm.set_ready(7);
        rm.reset();
        assert_eq!(rm.ready_count(), 0);
    }
}
