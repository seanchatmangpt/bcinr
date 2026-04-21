//! # U1_512 — Block profile: U_{1,512} = 𝔹⁵¹² — 512 truth atoms, one L1 cache line
//!
//! `[u64; 8]` — exactly one L1 cache line on most modern CPUs.
//! 8 cells × 64 places = 512 boolean facts per block.

use super::cell::U1_64;
use super::constants::U1_512_WORDS;

/// 512-bit block = 8 cells × 64 bits. Fits in one L1 line.
///
/// ```
/// use bcinr_logic::patterns::universe1::block::U1_512;
/// let b = U1_512::new_zero();
/// assert_eq!(b.words(), &[0u64; 8]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct U1_512([u64; U1_512_WORDS]);

impl U1_512 {
    /// Zero block.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let b = U1_512::new_zero();
    /// assert_eq!(b.popcount(), 0);
    /// ```
    #[inline(always)]
    pub const fn new_zero() -> Self {
        Self([0u64; U1_512_WORDS])
    }

    /// Construct from 8 `u64` words.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let b = U1_512::new([1, 2, 3, 4, 5, 6, 7, 8]);
    /// assert_eq!(b.words()[0], 1);
    /// assert_eq!(b.words()[7], 8);
    /// ```
    #[inline(always)]
    pub const fn new(words: [u64; U1_512_WORDS]) -> Self {
        Self(words)
    }

    /// Read-only view of the 8 words.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let b = U1_512::new([10, 20, 30, 40, 50, 60, 70, 80]);
    /// assert_eq!(b.words()[3], 40);
    /// ```
    #[inline(always)]
    pub const fn words(&self) -> &[u64; U1_512_WORDS] {
        &self.0
    }

    /// Get cell by index (clamped to `[0, 7]` via `& 7`).
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// let b = U1_512::new([0, 0, 42, 0, 0, 0, 0, 0]);
    /// assert_eq!(b.cell(2), U1_64::new(42));
    /// ```
    #[inline(always)]
    pub const fn cell(&self, idx: usize) -> U1_64 {
        U1_64::new(self.0[idx & 7])
    }

    /// Set cell at index.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// let mut b = U1_512::new_zero();
    /// b.set_cell(5, U1_64::new(99));
    /// assert_eq!(b.cell(5).get(), 99);
    /// ```
    #[inline(always)]
    pub fn set_cell(&mut self, idx: usize, cell: U1_64) {
        self.0[idx & 7] = cell.get();
    }

    /// Branchless fire on cell `idx`. Returns `fired_mask` (`!0` or `0`).
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let mut b = U1_512::new([0b0011, 0, 0, 0, 0, 0, 0, 0]);
    /// let fired = b.fire_cell(0, 0b0011, 0b1100);
    /// assert_eq!(fired, !0u64);
    /// assert_eq!(b.words()[0], 0b1100);
    ///
    /// // Not enabled.
    /// let fired = b.fire_cell(0, 0b0011, 0b1100);
    /// assert_eq!(fired, 0);
    /// ```
    #[inline(always)]
    pub fn fire_cell(&mut self, idx: usize, input: u64, output: u64) -> u64 {
        let i = idx & 7;
        let m = self.0[i];
        let enabled = ((m & input) == input) as u64;
        let fired = 0u64.wrapping_sub(enabled);
        let candidate = (m & !input) | output;
        self.0[i] = (candidate & fired) | (m & !fired);
        fired
    }

    /// Hamming distance (total bit difference) vs `expected`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let a = U1_512::new([0xFF, 0, 0, 0, 0, 0, 0, 0]);
    /// let b = U1_512::new([0x0F, 0, 0, 0, 0, 0, 0, 0]);
    /// assert_eq!(a.conformance_distance(&b), 4);
    /// ```
    #[inline]
    pub fn conformance_distance(&self, expected: &Self) -> u32 {
        let mut d = 0u32;
        let mut i = 0;
        while i < U1_512_WORDS {
            d = d.wrapping_add((self.0[i] ^ expected.0[i]).count_ones());
            i += 1;
        }
        d
    }

    /// Total law violations across all 8 cells.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let b = U1_512::new([0xFF, 0xFF, 0, 0, 0, 0, 0, 0]);
    /// let law = U1_512::new([0x0F, 0x0F, !0, !0, !0, !0, !0, !0]);
    /// assert_eq!(b.law_violation(&law), 8);
    /// ```
    #[inline]
    pub fn law_violation(&self, law: &Self) -> u32 {
        let mut v = 0u32;
        let mut i = 0;
        while i < U1_512_WORDS {
            v = v.wrapping_add((self.0[i] & !law.0[i]).count_ones());
            i += 1;
        }
        v
    }

    /// Total popcount across the block.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::block::U1_512;
    /// let b = U1_512::new([0b1111, 0b11, 0, 0, 0, 0, 0, 0]);
    /// assert_eq!(b.popcount(), 6);
    /// ```
    #[inline]
    pub fn popcount(&self) -> u32 {
        let mut p = 0u32;
        let mut i = 0;
        while i < U1_512_WORDS {
            p = p.wrapping_add(self.0[i].count_ones());
            i += 1;
        }
        p
    }
}

impl Default for U1_512 {
    fn default() -> Self {
        Self::new_zero()
    }
}
