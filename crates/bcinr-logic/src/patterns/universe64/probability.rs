//! # Universe64 Contract: ProbabilityKernel (Branchless Histogram)
//! Plane: S — scratch resident histogram; updated from delta stream.
//! Tier: T0 per-sample update; T1 domain histogram.
//! Scope: bounded histogram array; branchless bucket selection.
//! Geometry: bucket = (val >> shift) & mask; count[bucket]++.
//! Delta: none produced — observer only.
//!
//! # Timing contract
//! - **T0 per-sample:** ≤ T0_BUDGET_NS
//! - **T1 domain sweep:** ≤ T1_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Fixed-size array; branchless index arithmetic.
//! CC=1: Absolute branchless logic.

use super::constants::{UNIVERSE_WORDS, CELL_COUNT};
use super::block::UniverseBlock;
use super::scratch::UDelta;

/// Number of histogram buckets (power of 2, ≤ 64).
pub const HISTOGRAM_BUCKETS: usize = 64;

/// Branchless histogram over u64 sample values.
/// Bucket = count_ones(val) (Hamming weight histogram, 0..=64).
#[derive(Clone, Copy, Debug)]
pub struct PopcountHistogram {
    /// counts[b] = number of samples with popcount == b.
    pub counts: [u32; 65],
    /// Total samples accumulated.
    pub total: u64,
}

impl PopcountHistogram {
    pub const fn new() -> Self {
        Self { counts: [0u32; 65], total: 0 }
    }

    /// Accumulate one sample (branchless bucket select = popcount).
    #[inline(always)]
    pub fn observe(&mut self, val: u64) {
        let bucket = val.count_ones() as usize;
        self.counts[bucket] = self.counts[bucket].wrapping_add(1);
        self.total = self.total.wrapping_add(1);
    }

    /// Observe all words in a block domain.
    #[inline]
    pub fn observe_domain(&mut self, block: &UniverseBlock, domain: usize) {
        let base = (domain & 63) * CELL_COUNT;
        for i in 0..CELL_COUNT {
            self.observe(block.state[base + i]);
        }
    }

    /// Observe a delta's changed bits.
    #[inline(always)]
    pub fn observe_delta_change(&mut self, delta: &UDelta) {
        self.observe(delta.changed_mask());
    }

    /// Mean popcount (× 1000, integer).
    pub fn mean_x1000(&self) -> u64 {
        if self.total == 0 { return 0; }
        let mut sum = 0u64;
        for b in 0..65usize {
            sum = sum.wrapping_add(b as u64 * self.counts[b] as u64);
        }
        (sum * 1000) / self.total
    }

    /// Reset histogram.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.counts = [0u32; 65];
        self.total = 0;
    }
}

impl Default for PopcountHistogram {
    fn default() -> Self { Self::new() }
}

/// Branchless fixed-width histogram (HISTOGRAM_BUCKETS buckets).
/// Bucket = (val >> shift) & (HISTOGRAM_BUCKETS - 1).
#[derive(Clone, Copy, Debug)]
pub struct FixedHistogram {
    pub counts: [u32; HISTOGRAM_BUCKETS],
    pub shift: u32,
    pub total: u64,
}

impl FixedHistogram {
    pub const fn new(shift: u32) -> Self {
        Self { counts: [0u32; HISTOGRAM_BUCKETS], shift, total: 0 }
    }

    #[inline(always)]
    pub fn observe(&mut self, val: u64) {
        let bucket = ((val >> self.shift) as usize) & (HISTOGRAM_BUCKETS - 1);
        self.counts[bucket] = self.counts[bucket].wrapping_add(1);
        self.total = self.total.wrapping_add(1);
    }

    #[inline]
    pub fn observe_block(&mut self, block: &UniverseBlock) {
        for i in 0..UNIVERSE_WORDS {
            self.observe(block.state[i]);
        }
    }

    /// Bucket with the highest count (branchless scan).
    pub fn mode_bucket(&self) -> usize {
        let mut max_count = 0u32;
        let mut max_idx = 0usize;
        for i in 0..HISTOGRAM_BUCKETS {
            let is_max = (self.counts[i] > max_count) as u32;
            max_count = max_count * (1 - is_max) + self.counts[i] * is_max;
            max_idx = max_idx * (1 - is_max as usize) + i * is_max as usize;
        }
        max_idx
    }

    pub fn reset(&mut self) {
        self.counts = [0u32; HISTOGRAM_BUCKETS];
        self.total = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::scratch::UScope;

    #[test]
    fn test_popcount_histogram_observe() {
        let mut h = PopcountHistogram::new();
        h.observe(0b1111); // popcount = 4
        h.observe(0b0001); // popcount = 1
        assert_eq!(h.counts[4], 1);
        assert_eq!(h.counts[1], 1);
        assert_eq!(h.total, 2);
    }

    #[test]
    fn test_popcount_histogram_mean() {
        let mut h = PopcountHistogram::new();
        h.observe(0b1111); // 4
        h.observe(0b0000); // 0
        // mean = 2000 × 1/1000
        assert_eq!(h.mean_x1000(), 2000);
    }

    #[test]
    fn test_fixed_histogram_observe() {
        let mut h = FixedHistogram::new(58); // top 6 bits → bucket
        h.observe(0u64);
        h.observe(u64::MAX);
        assert_eq!(h.counts[0], 1);
        assert_eq!(h.counts[63], 1);
    }

    #[test]
    fn test_fixed_histogram_mode() {
        let mut h = FixedHistogram::new(0);
        for _ in 0..10 { h.observe(5); } // bucket 5
        for _ in 0..3  { h.observe(1); }
        assert_eq!(h.mode_bucket(), 5);
    }

    #[test]
    fn test_observe_delta_change() {
        let mut h = PopcountHistogram::new();
        let delta = UDelta::new(0, UScope::Cell, 0, 0b0000, 0b1111, !0);
        h.observe_delta_change(&delta);
        assert_eq!(h.counts[4], 1);
    }
}
