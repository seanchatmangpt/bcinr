//! # Universe64 Contract: LawKernel (Invariant Enforcement)
//! Plane: D — data plane conformance probe.
//! Tier: T0 per-word check; T1 domain sweep; T2 full scan.
//! Scope: stateless; reads UniverseBlock words and a law mask.
//! Geometry: violation = U & !Law; zero violation = conformant.
//! Delta: none produced — read-only enforcement probe.
//!
//! # Timing contract
//! - **T0 per-word:** ≤ T0_BUDGET_NS
//! - **T1 domain-sweep:** ≤ T1_BUDGET_NS (64 words)
//! - **T2 full-scan:** ≤ T2_BUDGET_NS (4096 words)
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. SWAR bitwise scan over bounded arrays.
//! CC=1: Absolute branchless logic.

use super::constants::{UNIVERSE_WORDS, CELL_COUNT};
use super::block::UniverseBlock;

/// Integrity gate for LawKernel.
#[inline(always)]
pub fn ulaw_phd_gate(val: u64) -> u64 {
    val ^ 0xF01AF01AF01AF01Au64
}

// ---------------------------------------------------------------------------
// Per-word violation probe (T0)
// ---------------------------------------------------------------------------

/// Branchless single-word law check.
/// violation_mask = state_word & !law_word
/// Returns 0 if the word is law-conformant.
#[inline(always)]
pub fn word_violation(state_word: u64, law_word: u64) -> u64 {
    state_word & !law_word
}

// ---------------------------------------------------------------------------
// CellLaw — single-word law constraint
// ---------------------------------------------------------------------------

/// Law constraint over one cell word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellLaw {
    /// Bitmask of places that are ALLOWED to be set (law = allowed bits).
    pub allowed: u64,
}

impl CellLaw {
    /// Check state_word against this cell law.
    #[inline(always)]
    pub fn violation(&self, state_word: u64) -> u64 {
        word_violation(state_word, self.allowed)
    }

    /// True if no violation.
    #[inline(always)]
    pub fn is_conformant(&self, state_word: u64) -> bool {
        self.violation(state_word) == 0
    }
}

// ---------------------------------------------------------------------------
// DomainLaw — 64-word domain law
// ---------------------------------------------------------------------------

/// Law constraint over all 64 cells in one domain.
#[derive(Clone, Copy, Debug)]
pub struct DomainLaw {
    /// Allowed bitmask per cell in this domain.
    pub allowed: [u64; CELL_COUNT],
}

impl DomainLaw {
    pub const fn new_permissive() -> Self {
        Self { allowed: [u64::MAX; CELL_COUNT] }
    }

    /// Count words with at least one violation in this domain.
    #[inline]
    pub fn violation_count(&self, block: &UniverseBlock, domain: usize) -> u32 {
        let base = (domain & 63) * CELL_COUNT;
        let mut count = 0u32;
        for i in 0..CELL_COUNT {
            let v = word_violation(block.state[base + i], self.allowed[i]);
            count = count.wrapping_add((v != 0) as u32);
        }
        count
    }

    /// Branchless OR of all violation words — non-zero means at least one violation.
    #[inline]
    pub fn any_violation(&self, block: &UniverseBlock, domain: usize) -> u64 {
        let base = (domain & 63) * CELL_COUNT;
        let mut acc = 0u64;
        for i in 0..CELL_COUNT {
            acc |= word_violation(block.state[base + i], self.allowed[i]);
        }
        acc
    }
}

// ---------------------------------------------------------------------------
// UniverseLaw — full 4096-word law
// ---------------------------------------------------------------------------

/// Full-universe law: one `allowed` bitmask per word.
pub struct UniverseLaw {
    pub allowed: [u64; UNIVERSE_WORDS],
}

impl UniverseLaw {
    pub fn new_permissive() -> Self {
        Self { allowed: [u64::MAX; UNIVERSE_WORDS] }
    }

    /// Populate `out[i]` with violation words (state[i] & !allowed[i]).
    /// Returns total number of non-zero violation words.
    #[inline]
    pub fn scan_violations(&self, block: &UniverseBlock, out: &mut [u64; UNIVERSE_WORDS]) -> u32 {
        let mut count = 0u32;
        for i in 0..UNIVERSE_WORDS {
            let v = word_violation(block.state[i], self.allowed[i]);
            out[i] = v;
            count = count.wrapping_add((v != 0) as u32);
        }
        count
    }

    /// Branchless popcount across all violation words.
    #[inline]
    pub fn total_violating_bits(&self, block: &UniverseBlock) -> u32 {
        let mut total = 0u32;
        for i in 0..UNIVERSE_WORDS {
            total = total.wrapping_add(
                word_violation(block.state[i], self.allowed[i]).count_ones()
            );
        }
        total
    }
}

// ---------------------------------------------------------------------------
// LawKernel — composite law evaluation engine
// ---------------------------------------------------------------------------

/// Stateless law enforcement engine.
/// All methods are branchless and tier-bounded.
pub struct LawKernel;

impl LawKernel {
    /// T0: check a single word.
    #[inline(always)]
    pub fn check_word(state_word: u64, allowed: u64) -> u64 {
        word_violation(state_word, allowed)
    }

    /// T1: check all 64 words in a domain; return OR of all violations.
    #[inline]
    pub fn check_domain(block: &UniverseBlock, domain: usize, law: &DomainLaw) -> u64 {
        law.any_violation(block, domain)
    }

    /// T2: full scan; return count of violating words.
    #[inline]
    pub fn check_universe(block: &UniverseBlock, law: &UniverseLaw) -> u32 {
        let mut count = 0u32;
        for i in 0..UNIVERSE_WORDS {
            count = count.wrapping_add(
                (word_violation(block.state[i], law.allowed[i]) != 0) as u32
            );
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_violation_none() {
        assert_eq!(word_violation(0b1010, 0b1111), 0);
    }

    #[test]
    fn test_word_violation_some() {
        assert_eq!(word_violation(0b1111, 0b0011), 0b1100);
    }

    #[test]
    fn test_cell_law_conformant() {
        let law = CellLaw { allowed: 0xFF };
        assert!(law.is_conformant(0x0F));
        assert!(!law.is_conformant(0x100));
    }

    #[test]
    fn test_domain_law_any_violation() {
        let block = UniverseBlock::new();
        let law = DomainLaw::new_permissive();
        // All-zero block with all-ones law → no violation.
        assert_eq!(law.any_violation(&block, 0), 0);
    }

    #[test]
    fn test_universe_law_scan() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0b1100;
        let mut law = UniverseLaw::new_permissive();
        law.allowed[0] = 0b0011; // bits 2 and 3 are forbidden
        let mut out = [0u64; UNIVERSE_WORDS];
        let count = law.scan_violations(&block, &mut out);
        assert_eq!(count, 1);
        assert_eq!(out[0], 0b1100);
    }

    #[test]
    fn test_law_kernel_check_word() {
        let v = LawKernel::check_word(0xFF, 0x0F);
        assert_eq!(v, 0xF0);
    }
}
