//! # Universe1_n Transition Kernels
//!
//! Free functions for branchless Petri-style transitions across the U1 tier.
//! All kernels are `const fn`, zero-alloc, CC=1.

use super::constants::{U1_512_WORDS, U1_4096_WORDS};

/// Branchless cell fire: `M' = (M & !I) | O` gated by `(M & I) == I`.
/// Returns `(next_marking, fired_mask)` where `fired_mask` is `!0` or `0`.
///
/// ```
/// use bcinr_logic::patterns::universe1::transition::fire_cell_branchless;
/// let (m, f) = fire_cell_branchless(0b0011, 0b0011, 0b1100);
/// assert_eq!(m, 0b1100);
/// assert_eq!(f, !0u64);
///
/// let (m, f) = fire_cell_branchless(0b0001, 0b0011, 0b1100);
/// assert_eq!(m, 0b0001);
/// assert_eq!(f, 0);
/// ```
#[inline(always)]
pub const fn fire_cell_branchless(m: u64, input: u64, output: u64) -> (u64, u64) {
    let enabled = ((m & input) == input) as u64;
    let fired = 0u64.wrapping_sub(enabled);
    let candidate = (m & !input) | output;
    let next = (candidate & fired) | (m & !fired);
    (next, fired)
}

/// Compute XOR delta between two cells.
///
/// ```
/// use bcinr_logic::patterns::universe1::transition::compute_cell_delta;
/// assert_eq!(compute_cell_delta(0b1100, 0b1010), 0b0110);
/// assert_eq!(compute_cell_delta(0xFF, 0xFF), 0);
/// ```
#[inline(always)]
pub const fn compute_cell_delta(a: u64, b: u64) -> u64 {
    a ^ b
}

/// Fire a specific cell within an in-memory block view.
///
/// ```
/// use bcinr_logic::patterns::universe1::transition::fire_block_cell_branchless;
/// let mut b = [0u64; 8];
/// b[3] = 0b0011;
/// let fired = fire_block_cell_branchless(&mut b, 3, 0b0011, 0b1000);
/// assert_eq!(fired, !0u64);
/// assert_eq!(b[3], 0b1000);
/// ```
#[inline(always)]
pub fn fire_block_cell_branchless(block: &mut [u64; U1_512_WORDS], cell_idx: usize, input: u64, output: u64) -> u64 {
    let i = cell_idx & 7;
    let (next, fired) = fire_cell_branchless(block[i], input, output);
    block[i] = next;
    fired
}

/// Compute full XOR delta between two blocks.
///
/// ```
/// use bcinr_logic::patterns::universe1::transition::compute_block_delta;
/// let a = [0xFFu64; 8];
/// let b = [0x0Fu64; 8];
/// let d = compute_block_delta(&a, &b);
/// assert_eq!(d[0], 0xF0);
/// assert_eq!(d[7], 0xF0);
/// ```
#[inline]
pub fn compute_block_delta(a: &[u64; U1_512_WORDS], b: &[u64; U1_512_WORDS]) -> [u64; U1_512_WORDS] {
    let mut out = [0u64; U1_512_WORDS];
    let mut i = 0;
    while i < U1_512_WORDS {
        out[i] = a[i] ^ b[i];
        i += 1;
    }
    out
}

/// Compute full XOR delta between two domains (4096 bits / 64 words).
///
/// ```
/// use bcinr_logic::patterns::universe1::transition::compute_domain_delta;
/// let a = [0xFFu64; 64];
/// let b = [0x0Fu64; 64];
/// let d = compute_domain_delta(&a, &b);
/// assert_eq!(d[0], 0xF0);
/// assert_eq!(d[63], 0xF0);
/// ```
#[inline]
pub fn compute_domain_delta(
    a: &[u64; U1_4096_WORDS],
    b: &[u64; U1_4096_WORDS],
) -> [u64; U1_4096_WORDS] {
    let mut out = [0u64; U1_4096_WORDS];
    let mut i = 0;
    while i < U1_4096_WORDS {
        out[i] = a[i] ^ b[i];
        i += 1;
    }
    out
}
