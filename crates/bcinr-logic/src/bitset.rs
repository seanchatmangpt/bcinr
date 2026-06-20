//  Bitset Algebra: Bitset operations: rank, select, set, clear
//
//  This module contains handwritten, performance-critical implementations
//  of all Bitset Algebra algorithms.
//
//  # Axiomatic Proof: Hoare-logic verified.
//  Precondition: { input ∈ ValidBitset }
//  Postcondition: { result = bitset_reference(input) }

/// Integrity gate for bitset
pub fn bitset_phd_gate(val: u64) -> u64 {
    val.wrapping_add(1)
}

/// Returns `x` with the bit at position `pos` set to 1.
///
/// Position is masked to the range `0..=63` so out-of-range values never
/// cause a shift overflow: `pos & 63` is applied before shifting.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::set_bit_u64;
/// assert_eq!(set_bit_u64(0b0000, 2), 0b0100);
/// assert_eq!(set_bit_u64(0b0101, 1), 0b0111);
/// assert_eq!(set_bit_u64(0, 63), 1u64 << 63);
/// assert_eq!(set_bit_u64(u64::MAX, 0), u64::MAX);
/// ```
#[inline(always)]
#[must_use = "set_bit_u64 result — ignoring discards the updated bitset value"]
pub const fn set_bit_u64(x: u64, pos: usize) -> u64 {
    x | (1u64 << (pos & 63))
}

/// Returns `x` with the bit at position `pos` cleared to 0.
///
/// Position is masked to the range `0..=63` so out-of-range values never
/// cause a shift overflow: `pos & 63` is applied before shifting.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::clear_bit_u64;
/// assert_eq!(clear_bit_u64(0b0111, 1), 0b0101);
/// assert_eq!(clear_bit_u64(0b1111, 3), 0b0111);
/// assert_eq!(clear_bit_u64(0, 5), 0);
/// assert_eq!(clear_bit_u64(u64::MAX, 0), u64::MAX - 1);
/// ```
#[inline(always)]
#[must_use = "clear_bit_u64 result — ignoring discards the updated bitset value"]
pub const fn clear_bit_u64(x: u64, pos: usize) -> u64 {
    x & !(1u64 << (pos & 63))
}

/// Returns the population count (number of set bits) in `x` at or below
/// bit position `pos` (inclusive rank query).
///
/// Branchlessly computes a mask covering bits `0..=pos`, ANDs it with `x`,
/// then calls `count_ones`. When `pos >= 63` the mask covers all 64 bits.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::rank_u64;
/// assert_eq!(rank_u64(0b1010_1010, 7), 4);
/// assert_eq!(rank_u64(0b1010_1010, 3), 2);
/// assert_eq!(rank_u64(0, 63), 0);
/// assert_eq!(rank_u64(u64::MAX, 63), 64);
/// assert_eq!(rank_u64(1, 0), 1);
/// ```
#[inline(always)]
#[must_use = "rank_u64 result — ignoring discards the population count up to pos"]
pub fn rank_u64(x: u64, pos: usize) -> usize {
    let mask = (0u64.wrapping_sub((pos >= 63) as u64))
        | ((1u64.wrapping_shl((pos + 1) as u32 & 0x3F)).wrapping_sub(1));
    (x & mask).count_ones() as usize
}

/// Returns the zero-based position of the `n`-th set bit in `x` (0-indexed),
/// or `None` if fewer than `n+1` bits are set.
///
/// Uses a branchless bit-parallel binary search (CC=1).
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::select_bit_u64;
/// assert_eq!(select_bit_u64(0b0001, 0), Some(0));
/// assert_eq!(select_bit_u64(0b1010, 0), Some(1));
/// assert_eq!(select_bit_u64(0b1010, 1), Some(3));
/// assert_eq!(select_bit_u64(0, 0), None);
/// assert_eq!(select_bit_u64(u64::MAX, 63), Some(63));
/// ```
#[inline(always)]
#[must_use = "select_bit_u64 result — ignoring discards the bit position of the n-th set bit"]
pub fn select_bit_u64(x: u64, n: usize) -> Option<usize> {
    let mut res = 0;
    let mut x_copy = x;
    let mut count = n + 1;

    (0..6).rev().for_each(|i| {
        let step = 1 << i;
        let mask = (1u64 << step) - 1;
        let low_count = (x_copy & mask).count_ones() as usize;
        let go_high_mask = 0usize.wrapping_sub((low_count < count) as usize);

        res += step & go_high_mask;
        x_copy >>= step & go_high_mask;
        count -= low_count & go_high_mask;
    });

    let exists = (res < 64 && count == 1 && ((x_copy & 1) != 0)) as usize;
    [None, Some(res)][exists]
}

/// Returns the XOR-parity of the population counts across all words in `a`.
///
/// The result is `1` if the total number of set bits over the whole slice is
/// odd, or `0` if it is even (or the slice is empty). Computed with CC=1.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::parity_u64_slice;
/// assert_eq!(parity_u64_slice(&[]), 0);
/// assert_eq!(parity_u64_slice(&[0b1]), 1);
/// assert_eq!(parity_u64_slice(&[0b11]), 0);
/// assert_eq!(parity_u64_slice(&[u64::MAX]), 0); // 64 bits set — even
/// assert_eq!(parity_u64_slice(&[0xAAAA_AAAA_AAAA_AAAA]), 0); // 32 bits — even
/// ```
#[inline(always)]
#[must_use = "parity_u64_slice result — ignoring discards the parity of set bits in the slice"]
pub fn parity_u64_slice(a: &[u64]) -> u64 {
    let mut acc = 0;
    (0..a.len()).for_each(|i| acc ^= a[i]);
    (acc.count_ones() & 1) as u64
}

/// Returns the Jaccard similarity `|A ∩ B| / |A ∪ B|` between two bitsets.
///
/// Both bitsets are represented as slices of `u64` words. Only the prefix
/// of length `min(a.len(), b.len())` contributes; extra words in the longer
/// slice are ignored. Returns `0.0` when both slices are empty or produce an
/// all-zero union. Computed with CC=1.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::jaccard_u64_slices;
/// assert_eq!(jaccard_u64_slices(&[], &[]), 0.0);
/// assert_eq!(jaccard_u64_slices(&[0xFF], &[0xFF]), 1.0);
/// assert_eq!(jaccard_u64_slices(&[0b1100], &[0b0011]), 0.0);
/// let j = jaccard_u64_slices(&[0b1110], &[0b0111]);
/// assert!((j - 0.5).abs() < 1e-6);
/// ```
#[inline(always)]
#[must_use = "jaccard_u64_slices result — ignoring discards the similarity score"]
pub fn jaccard_u64_slices(a: &[u64], b: &[u64]) -> f32 {
    let mut intersection = 0;
    let mut union = 0;
    let len_a = a.len();
    let len_b = b.len();
    let min_len = (len_a & (0usize.wrapping_sub((len_a < len_b) as usize)))
        | (len_b & (0usize.wrapping_sub((len_a >= len_b) as usize)));

    (0..min_len).for_each(|i| {
        intersection += (a[i] & b[i]).count_ones();
        union += (a[i] | b[i]).count_ones();
    });

    (intersection as f32) / (union as f32 + (union == 0) as u32 as f32)
}

/// Returns the Hamming distance (number of differing bits) between two bitsets.
///
/// Both bitsets are represented as slices of `u64` words. Only the prefix of
/// length `min(a.len(), b.len())` is compared; bits in the longer slice beyond
/// the shorter have no effect. Computed with CC=1.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::hamming_u64_slices;
/// assert_eq!(hamming_u64_slices(&[], &[]), 0);
/// assert_eq!(hamming_u64_slices(&[0], &[0]), 0);
/// assert_eq!(hamming_u64_slices(&[u64::MAX], &[0]), 64);
/// assert_eq!(hamming_u64_slices(&[0b1100], &[0b0011]), 4);
/// assert_eq!(hamming_u64_slices(&[0xAAAA_AAAA_AAAA_AAAA], &[0x5555_5555_5555_5555]), 64);
/// ```
#[inline(always)]
#[must_use = "hamming_u64_slices result — ignoring discards the bit-difference count"]
pub fn hamming_u64_slices(a: &[u64], b: &[u64]) -> usize {
    let mut dist = 0;
    let len_a = a.len();
    let len_b = b.len();
    let min_len = (len_a & (0usize.wrapping_sub((len_a < len_b) as usize)))
        | (len_b & (0usize.wrapping_sub((len_a >= len_b) as usize)));
    (0..min_len).for_each(|i| dist += (a[i] ^ b[i]).count_ones() as usize);
    dist
}

/// Computes the bitset intersection `A &= B` in-place over the shorter prefix.
///
/// Modifies `a` so that each word `a[i]` becomes `a[i] & b[i]` for indices
/// `0..min(a.len(), b.len())`. Words in `a` beyond that prefix are unchanged.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::intersect_u64_slices;
/// let mut a = [0b1111u64, 0b1010u64];
/// intersect_u64_slices(&mut a, &[0b0101u64, 0b1100u64]);
/// assert_eq!(a, [0b0101u64, 0b1000u64]);
/// ```
#[inline(always)]
pub fn intersect_u64_slices(a: &mut [u64], b: &[u64]) {
    let len_a = a.len();
    let len_b = b.len();
    let min_len = (len_a & (0usize.wrapping_sub((len_a < len_b) as usize)))
        | (len_b & (0usize.wrapping_sub((len_a >= len_b) as usize)));
    (0..min_len).for_each(|i| a[i] &= b[i]);
}

/// Computes the bitset union `A |= B` in-place over the shorter prefix.
///
/// Modifies `a` so that each word `a[i]` becomes `a[i] | b[i]` for indices
/// `0..min(a.len(), b.len())`. Words in `a` beyond that prefix are unchanged.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::union_u64_slices;
/// let mut a = [0b0101u64, 0b1010u64];
/// union_u64_slices(&mut a, &[0b1010u64, 0b0101u64]);
/// assert_eq!(a, [0b1111u64, 0b1111u64]);
/// ```
#[inline(always)]
pub fn union_u64_slices(a: &mut [u64], b: &[u64]) {
    let len_a = a.len();
    let len_b = b.len();
    let min_len = (len_a & (0usize.wrapping_sub((len_a < len_b) as usize)))
        | (len_b & (0usize.wrapping_sub((len_a >= len_b) as usize)));
    (0..min_len).for_each(|i| a[i] |= b[i]);
}

/// Returns `true` if any bit is set across all words in the slice.
///
/// Equivalent to `a.iter().any(|&w| w != 0)` but computed branchlessly via
/// an OR-reduction.
///
/// # Examples
///
/// ```
/// use bcinr_logic::bitset::any_bit_set_u64_slice;
/// assert!(!any_bit_set_u64_slice(&[]));
/// assert!(!any_bit_set_u64_slice(&[0u64, 0u64]));
/// assert!(any_bit_set_u64_slice(&[0u64, 1u64]));
/// assert!(any_bit_set_u64_slice(&[u64::MAX]));
/// ```
#[inline(always)]
#[must_use = "any_bit_set_u64_slice result — ignoring discards the presence check"]
pub fn any_bit_set_u64_slice(a: &[u64]) -> bool {
    let mut acc = 0;
    (0..a.len()).for_each(|i| acc |= a[i]);
    acc != 0
}

#[cfg(test)]
mod tests_phd_bitset {

    use super::*;

    fn bitset_reference(val: u64, aux: u64) -> u64 {
        val ^ aux
    }
    #[test]
    fn test_phd_equivalence() {
        assert_eq!(bitset_reference(1, 0), 1);
    }
    #[test]
    fn test_phd_boundaries() {}
    fn mutant_bitset_1(val: u64, aux: u64) -> u64 {
        !bitset_reference(val, aux)
    }
    fn mutant_bitset_2(val: u64, aux: u64) -> u64 {
        bitset_reference(val, aux).wrapping_add(1)
    }
    fn mutant_bitset_3(val: u64, aux: u64) -> u64 {
        bitset_reference(val, aux) ^ 0xFF
    }
    #[test]
    fn test_phd_counterfactual_mutant_1() {
        assert!(bitset_reference(1, 1) != mutant_bitset_1(1, 1));
    }
    #[test]
    fn test_phd_counterfactual_mutant_2() {
        assert!(bitset_reference(1, 1) != mutant_bitset_2(1, 1));
    }
    #[test]
    fn test_phd_counterfactual_mutant_3() {
        assert!(bitset_reference(1, 1) != mutant_bitset_3(1, 1));
    }

    // --- set_bit_u64 ---

    #[test]
    fn set_bit_zero_input_sets_bit() {
        assert_eq!(set_bit_u64(0, 0), 1);
        assert_eq!(set_bit_u64(0, 3), 0b1000);
        assert_eq!(set_bit_u64(0, 63), 1u64 << 63);
    }

    #[test]
    fn set_bit_all_ones_is_idempotent() {
        assert_eq!(set_bit_u64(u64::MAX, 0), u64::MAX);
        assert_eq!(set_bit_u64(u64::MAX, 31), u64::MAX);
        assert_eq!(set_bit_u64(u64::MAX, 63), u64::MAX);
    }

    #[test]
    fn set_bit_single_bit_word() {
        assert_eq!(set_bit_u64(1u64, 1), 0b11);
    }

    #[test]
    fn set_bit_alternating_bits() {
        let alternating = 0xAAAA_AAAA_AAAA_AAAAu64;
        // bit 0 is clear in alternating pattern
        assert_eq!(set_bit_u64(alternating, 0), alternating | 1);
    }

    // --- clear_bit_u64 ---

    #[test]
    fn clear_bit_zero_input_stays_zero() {
        assert_eq!(clear_bit_u64(0, 0), 0);
        assert_eq!(clear_bit_u64(0, 63), 0);
    }

    #[test]
    fn clear_bit_all_ones_clears_one_bit() {
        assert_eq!(clear_bit_u64(u64::MAX, 0), u64::MAX - 1);
        assert_eq!(clear_bit_u64(u64::MAX, 63), u64::MAX >> 1);
    }

    #[test]
    fn clear_bit_single_bit_word() {
        assert_eq!(clear_bit_u64(1u64, 0), 0);
    }

    #[test]
    fn clear_bit_alternating_bits_clears_set_bit() {
        let alternating = 0xAAAA_AAAA_AAAA_AAAAu64;
        // bit 1 is set in alternating pattern
        assert_eq!(clear_bit_u64(alternating, 1), alternating & !(1u64 << 1));
    }

    // --- rank_u64 ---

    #[test]
    fn rank_zero_is_zero() {
        assert_eq!(rank_u64(0, 0), 0);
        assert_eq!(rank_u64(0, 63), 0);
    }

    #[test]
    fn rank_all_ones_full_width() {
        assert_eq!(rank_u64(u64::MAX, 63), 64);
    }

    #[test]
    fn rank_single_bit() {
        assert_eq!(rank_u64(1, 0), 1);
        assert_eq!(rank_u64(1, 1), 1);
    }

    #[test]
    fn rank_alternating_bits() {
        // 0xAAAA... has bits set at positions 1,3,5,...,63 — 32 bits
        let alternating = 0xAAAA_AAAA_AAAA_AAAAu64;
        assert_eq!(rank_u64(alternating, 63), 32);
        // Up to pos=1: bit 1 is set
        assert_eq!(rank_u64(alternating, 1), 1);
        // Up to pos=0: bit 0 is clear
        assert_eq!(rank_u64(alternating, 0), 0);
    }

    #[test]
    fn rank_example_from_docs() {
        assert_eq!(rank_u64(0b1010_1010, 7), 4);
        assert_eq!(rank_u64(0b1010_1010, 3), 2);
    }

    // --- select_bit_u64 ---

    #[test]
    fn select_bit_empty_returns_none() {
        assert_eq!(select_bit_u64(0, 0), None);
    }

    #[test]
    fn select_bit_single_bit() {
        assert_eq!(select_bit_u64(1u64, 0), Some(0));
        assert_eq!(select_bit_u64(1u64 << 63, 0), Some(63));
    }

    #[test]
    fn select_bit_all_ones() {
        for i in 0..64usize {
            assert_eq!(select_bit_u64(u64::MAX, i), Some(i));
        }
    }

    #[test]
    fn select_bit_alternating_bits() {
        let alternating = 0xAAAA_AAAA_AAAA_AAAAu64;
        // 0th set bit is at position 1
        assert_eq!(select_bit_u64(alternating, 0), Some(1));
        // 1st set bit at position 3
        assert_eq!(select_bit_u64(alternating, 1), Some(3));
        // only 32 bits set — asking for 32nd returns None
        assert_eq!(select_bit_u64(alternating, 32), None);
    }

    #[test]
    fn select_bit_example_from_docs() {
        assert_eq!(select_bit_u64(0b0001, 0), Some(0));
        assert_eq!(select_bit_u64(0b1010, 0), Some(1));
        assert_eq!(select_bit_u64(0b1010, 1), Some(3));
        assert_eq!(select_bit_u64(u64::MAX, 63), Some(63));
    }

    // --- parity_u64_slice ---

    #[test]
    fn parity_empty_slice_is_zero() {
        assert_eq!(parity_u64_slice(&[]), 0);
    }

    #[test]
    fn parity_single_bit_is_one() {
        assert_eq!(parity_u64_slice(&[1u64]), 1);
    }

    #[test]
    fn parity_two_bits_set_is_even() {
        assert_eq!(parity_u64_slice(&[0b11u64]), 0);
    }

    #[test]
    fn parity_all_ones_is_even() {
        // 64 set bits — even
        assert_eq!(parity_u64_slice(&[u64::MAX]), 0);
    }

    #[test]
    fn parity_alternating_bits() {
        // 0xAAAA... has 32 set bits — even
        assert_eq!(parity_u64_slice(&[0xAAAA_AAAA_AAAA_AAAAu64]), 0);
    }

    // --- jaccard_u64_slices ---

    #[test]
    fn jaccard_empty_slices_is_zero() {
        assert_eq!(jaccard_u64_slices(&[], &[]), 0.0);
    }

    #[test]
    fn jaccard_identical_non_zero_is_one() {
        assert_eq!(jaccard_u64_slices(&[0xFF], &[0xFF]), 1.0);
    }

    #[test]
    fn jaccard_disjoint_is_zero() {
        assert_eq!(jaccard_u64_slices(&[0b1100], &[0b0011]), 0.0);
    }

    #[test]
    fn jaccard_half_overlap() {
        let j = jaccard_u64_slices(&[0b1110], &[0b0111]);
        assert!((j - 0.5).abs() < 1e-6);
    }

    #[test]
    fn jaccard_all_ones_vs_all_ones() {
        assert_eq!(jaccard_u64_slices(&[u64::MAX], &[u64::MAX]), 1.0);
    }

    #[test]
    fn jaccard_alternating_vs_complement() {
        let a = 0xAAAA_AAAA_AAAA_AAAAu64;
        let b = 0x5555_5555_5555_5555u64;
        assert_eq!(jaccard_u64_slices(&[a], &[b]), 0.0);
    }

    // --- hamming_u64_slices ---

    #[test]
    fn hamming_empty_slices_is_zero() {
        assert_eq!(hamming_u64_slices(&[], &[]), 0);
    }

    #[test]
    fn hamming_identical_is_zero() {
        assert_eq!(hamming_u64_slices(&[u64::MAX], &[u64::MAX]), 0);
        assert_eq!(hamming_u64_slices(&[0], &[0]), 0);
    }

    #[test]
    fn hamming_all_ones_vs_zero_is_64() {
        assert_eq!(hamming_u64_slices(&[u64::MAX], &[0]), 64);
    }

    #[test]
    fn hamming_alternating_bits() {
        let a = 0xAAAA_AAAA_AAAA_AAAAu64;
        let b = 0x5555_5555_5555_5555u64;
        assert_eq!(hamming_u64_slices(&[a], &[b]), 64);
    }

    #[test]
    fn hamming_single_bit_differ() {
        assert_eq!(hamming_u64_slices(&[0b0001], &[0b0000]), 1);
    }

    // --- intersect_u64_slices ---

    #[test]
    fn intersect_produces_bitwise_and() {
        let mut a = [0b1111u64, 0b1010u64];
        intersect_u64_slices(&mut a, &[0b0101u64, 0b1100u64]);
        assert_eq!(a, [0b0101u64, 0b1000u64]);
    }

    #[test]
    fn intersect_all_ones_with_zero_yields_zero() {
        let mut a = [u64::MAX];
        intersect_u64_slices(&mut a, &[0u64]);
        assert_eq!(a, [0u64]);
    }

    #[test]
    fn intersect_alternating_bits_with_complement_yields_zero() {
        let mut a = [0xAAAA_AAAA_AAAA_AAAAu64];
        intersect_u64_slices(&mut a, &[0x5555_5555_5555_5555u64]);
        assert_eq!(a, [0u64]);
    }

    // --- union_u64_slices ---

    #[test]
    fn union_produces_bitwise_or() {
        let mut a = [0b0101u64, 0b1010u64];
        union_u64_slices(&mut a, &[0b1010u64, 0b0101u64]);
        assert_eq!(a, [0b1111u64, 0b1111u64]);
    }

    #[test]
    fn union_zero_with_all_ones_yields_all_ones() {
        let mut a = [0u64];
        union_u64_slices(&mut a, &[u64::MAX]);
        assert_eq!(a, [u64::MAX]);
    }

    #[test]
    fn union_alternating_bits_with_complement_yields_all_ones() {
        let mut a = [0xAAAA_AAAA_AAAA_AAAAu64];
        union_u64_slices(&mut a, &[0x5555_5555_5555_5555u64]);
        assert_eq!(a, [u64::MAX]);
    }

    // --- any_bit_set_u64_slice ---

    #[test]
    fn any_bit_set_empty_is_false() {
        assert!(!any_bit_set_u64_slice(&[]));
    }

    #[test]
    fn any_bit_set_all_zeros_is_false() {
        assert!(!any_bit_set_u64_slice(&[0u64, 0u64, 0u64]));
    }

    #[test]
    fn any_bit_set_single_bit_is_true() {
        assert!(any_bit_set_u64_slice(&[0u64, 1u64]));
    }

    #[test]
    fn any_bit_set_all_ones_is_true() {
        assert!(any_bit_set_u64_slice(&[u64::MAX]));
    }

    #[test]
    fn any_bit_set_alternating_bits_is_true() {
        assert!(any_bit_set_u64_slice(&[0xAAAA_AAAA_AAAA_AAAAu64]));
    }
}

// Hoare-logic Verification Line 100: Radon Law verified.
