//! # Universe64 Contract: DriftKernel (Expected-State Tracking)
//! Plane: D — compares live block against expected model.
//! Tier: T0 per-word; T1 domain; T2 full scan.
//! Scope: stateless; reads block + expected model.
//! Geometry: drift = XOR(live, expected); severity = popcount(drift).
//! Delta: none produced — read-only observer.
//!
//! # Timing contract
//! - **T0 per-word:** ≤ T0_BUDGET_NS
//! - **T1 domain sweep:** ≤ T1_BUDGET_NS
//! - **T2 full scan:** ≤ T2_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Branchless XOR + popcount over bounded arrays.
//! CC=1: Absolute branchless logic.

use super::constants::{UNIVERSE_WORDS, CELL_COUNT};
use super::block::UniverseBlock;

/// Drift severity for a single word.
#[inline(always)]
pub fn word_drift(live: u64, expected: u64) -> u64 {
    live ^ expected
}

/// Popcount of drift bits in one word.
#[inline(always)]
pub fn word_drift_bits(live: u64, expected: u64) -> u32 {
    word_drift(live, expected).count_ones()
}

/// Expected model snapshot (32 KiB, matches UniverseBlock layout).
#[derive(Clone, Copy)]
pub struct ExpectedModel {
    pub words: [u64; UNIVERSE_WORDS],
}

impl ExpectedModel {
    /// All-zero expected state (everything off = default expected).
    pub const fn all_zero() -> Self {
        Self { words: [0u64; UNIVERSE_WORDS] }
    }

    /// All-one expected state (everything on).
    pub const fn all_ones() -> Self {
        Self { words: [u64::MAX; UNIVERSE_WORDS] }
    }

    /// Clone from UniverseBlock.
    pub fn snapshot(block: &UniverseBlock) -> Self {
        Self { words: block.state }
    }
}

/// Drift measurement result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriftReport {
    /// Total drifted bits (sum of popcounts across all words).
    pub total_bits: u32,
    /// Number of words with any drift.
    pub drifted_words: u32,
    /// Word index of maximum drift (tie: first).
    pub max_drift_word: u16,
    /// Number of drifted bits in that word.
    pub max_drift_bits: u32,
}

/// Stateless drift analysis kernel.
pub struct DriftKernel;

impl DriftKernel {
    /// T0: per-word drift probe.
    #[inline(always)]
    pub fn probe_word(live: u64, expected: u64) -> u64 {
        word_drift(live, expected)
    }

    /// T1: domain drift scan (64 words).
    #[inline]
    pub fn scan_domain(block: &UniverseBlock, model: &ExpectedModel, domain: usize) -> u32 {
        let base = (domain & 63) * CELL_COUNT;
        let mut bits = 0u32;
        for i in 0..CELL_COUNT {
            bits = bits.wrapping_add(word_drift_bits(block.state[base + i], model.words[base + i]));
        }
        bits
    }

    /// T2: full universe drift scan.
    pub fn scan_universe(block: &UniverseBlock, model: &ExpectedModel) -> DriftReport {
        let mut total_bits = 0u32;
        let mut drifted_words = 0u32;
        let mut max_drift_word = 0u16;
        let mut max_drift_bits = 0u32;

        for i in 0..UNIVERSE_WORDS {
            let d = word_drift_bits(block.state[i], model.words[i]);
            total_bits = total_bits.wrapping_add(d);
            drifted_words = drifted_words.wrapping_add((d != 0) as u32);
            // Branchless max tracking.
            let is_new_max = (d > max_drift_bits) as u32;
            max_drift_bits = max_drift_bits * (1 - is_new_max) + d * is_new_max;
            max_drift_word = (max_drift_word as u32 * (1 - is_new_max) + i as u32 * is_new_max) as u16;
        }

        DriftReport { total_bits, drifted_words, max_drift_word, max_drift_bits }
    }

    /// Incremental update: given one delta, adjust running drift.
    /// `expected` is the expected value at word_idx.
    #[inline(always)]
    pub fn update_drift(
        prev_live: u64,
        new_live: u64,
        expected: u64,
        running_bits: &mut u32,
    ) {
        let old_d = word_drift_bits(prev_live, expected);
        let new_d = word_drift_bits(new_live, expected);
        *running_bits = running_bits.wrapping_add(new_d).wrapping_sub(old_d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_drift_identical() {
        assert_eq!(word_drift(0xABCD, 0xABCD), 0);
    }

    #[test]
    fn test_word_drift_all_diff() {
        assert_eq!(word_drift(0u64, u64::MAX), u64::MAX);
    }

    #[test]
    fn test_scan_universe_no_drift() {
        let block = UniverseBlock::new();
        let model = ExpectedModel::all_zero();
        let report = DriftKernel::scan_universe(&block, &model);
        assert_eq!(report.total_bits, 0);
        assert_eq!(report.drifted_words, 0);
    }

    #[test]
    fn test_scan_universe_with_drift() {
        let mut block = UniverseBlock::new();
        block.state[10] = 0b1111; // 4 bits drifted from 0
        let model = ExpectedModel::all_zero();
        let report = DriftKernel::scan_universe(&block, &model);
        assert_eq!(report.total_bits, 4);
        assert_eq!(report.drifted_words, 1);
        assert_eq!(report.max_drift_word, 10);
        assert_eq!(report.max_drift_bits, 4);
    }

    #[test]
    fn test_update_drift_incremental() {
        let mut running = 0u32;
        DriftKernel::update_drift(0, 0b1010, 0, &mut running);
        assert_eq!(running, 2);
        DriftKernel::update_drift(0b1010, 0, 0, &mut running);
        assert_eq!(running, 0);
    }

    #[test]
    fn test_snapshot_and_compare() {
        let mut block = UniverseBlock::new();
        let model = ExpectedModel::snapshot(&block);
        block.state[5] = 0xDEAD;
        let report = DriftKernel::scan_universe(&block, &model);
        assert_eq!(report.drifted_words, 1);
    }
}
