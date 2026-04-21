//! # U1_4096 — Domain profile: U_{1,4096} = 𝔹⁴⁰⁹⁶ — 4096 truth atoms, half of a 4 KiB page
//!
//! `[u64; 64]` — half of a 4 KiB page. 64 blocks × 64 cells × 8 places.
//! One U1_4096 fits entirely within L1 data cache on all modern CPUs.

use super::constants::{U1_512_WORDS, U1_4096_WORDS};

/// 4096-bit domain = 64 `u64` words. Fits in 8 L1 cache lines.
///
/// ```
/// use bcinr_logic::patterns::universe1::domain::U1_4096;
/// let d = U1_4096::new_zero();
/// assert_eq!(d.words().len(), 64);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct U1_4096([u64; U1_4096_WORDS]);

impl U1_4096 {
    /// Zero domain.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// assert_eq!(U1_4096::new_zero().popcount(), 0);
    /// ```
    #[inline(always)]
    pub const fn new_zero() -> Self {
        Self([0u64; U1_4096_WORDS])
    }

    /// Construct from 64 words.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let d = U1_4096::new([1u64; 64]);
    /// assert_eq!(d.words()[0], 1);
    /// assert_eq!(d.words()[63], 1);
    /// ```
    #[inline(always)]
    pub const fn new(words: [u64; U1_4096_WORDS]) -> Self {
        Self(words)
    }

    /// View of all 64 words.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let d = U1_4096::new_zero();
    /// assert_eq!(d.words()[7], 0);
    /// ```
    #[inline(always)]
    pub const fn words(&self) -> &[u64; U1_4096_WORDS] {
        &self.0
    }

    /// Get a single word by index (clamped to `[0, 63]`).
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let mut w = [0u64; 64];
    /// w[17] = 0xABCD;
    /// let d = U1_4096::new(w);
    /// assert_eq!(d.word(17), 0xABCD);
    /// ```
    #[inline(always)]
    pub const fn word(&self, idx: usize) -> u64 {
        self.0[idx & 63]
    }

    /// Set a word by index.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let mut d = U1_4096::new_zero();
    /// d.set_word(5, 0xDEAD);
    /// assert_eq!(d.word(5), 0xDEAD);
    /// ```
    #[inline(always)]
    pub fn set_word(&mut self, idx: usize, v: u64) {
        self.0[idx & 63] = v;
    }

    /// View of one block (8 consecutive words) starting at block index `b`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let d = U1_4096::new([1u64; 64]);
    /// let slice = d.block_view(0);
    /// assert_eq!(slice.len(), 8);
    /// assert_eq!(slice[0], 1);
    /// ```
    #[inline(always)]
    pub fn block_view(&self, block_idx: usize) -> &[u64] {
        let b = (block_idx & 7) * U1_512_WORDS;
        &self.0[b..b + U1_512_WORDS]
    }

    /// Branchless fire on a specific cell within a block.
    /// Returns fired mask.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let mut d = U1_4096::new_zero();
    /// d.set_word(0, 0b0011);
    /// let fired = d.fire_block_cell(0, 0, 0b0011, 0b1000);
    /// assert_eq!(fired, !0u64);
    /// assert_eq!(d.word(0), 0b1000);
    /// ```
    #[inline]
    pub fn fire_block_cell(&mut self, block_idx: usize, cell_idx: usize, input: u64, output: u64) -> u64 {
        let w = (block_idx & 7) * U1_512_WORDS + (cell_idx & 7);
        let m = self.0[w];
        let enabled = ((m & input) == input) as u64;
        let fired = 0u64.wrapping_sub(enabled);
        let candidate = (m & !input) | output;
        self.0[w] = (candidate & fired) | (m & !fired);
        fired
    }

    /// Hamming distance vs expected.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let a = U1_4096::new([0xFFu64; 64]);
    /// let b = U1_4096::new_zero();
    /// assert_eq!(a.conformance_distance(&b), 64 * 8);
    /// ```
    #[inline]
    pub fn conformance_distance(&self, expected: &Self) -> u32 {
        let mut d = 0u32;
        let mut i = 0;
        while i < U1_4096_WORDS {
            d = d.wrapping_add((self.0[i] ^ expected.0[i]).count_ones());
            i += 1;
        }
        d
    }

    /// Full XOR delta vs another domain.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let a = U1_4096::new([0xFFu64; 64]);
    /// let b = U1_4096::new([0x0Fu64; 64]);
    /// let diff = a.delta(&b);
    /// assert_eq!(diff[0], 0xF0);
    /// assert_eq!(diff[63], 0xF0);
    /// ```
    #[inline]
    pub fn delta(&self, other: &Self) -> [u64; U1_4096_WORDS] {
        let mut out = [0u64; U1_4096_WORDS];
        let mut i = 0;
        while i < U1_4096_WORDS {
            out[i] = self.0[i] ^ other.0[i];
            i += 1;
        }
        out
    }

    /// Total popcount across the domain.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// let d = U1_4096::new([0xFu64; 64]);
    /// assert_eq!(d.popcount(), 64 * 4);
    /// ```
    #[inline]
    pub fn popcount(&self) -> u32 {
        let mut p = 0u32;
        let mut i = 0;
        while i < U1_4096_WORDS {
            p = p.wrapping_add(self.0[i].count_ones());
            i += 1;
        }
        p
    }

    /// Size in bytes of one U1_4096 (always 512).
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::domain::U1_4096;
    /// assert_eq!(U1_4096::size_bytes(), 512);
    /// ```
    #[inline(always)]
    pub const fn size_bytes() -> usize {
        U1_4096_WORDS * 8
    }
}

impl Default for U1_4096 {
    fn default() -> Self {
        Self::new_zero()
    }
}
