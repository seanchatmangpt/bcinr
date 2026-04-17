//! Bitset Algebra: Bitset operations: rank, select, set, clear
//!
//! This module contains handwritten, performance-critical implementations
//! of all Bitset Algebra algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/bitset.rs`.
//!
//! Bootstrap-once (skip_existing): manually edit this file to add implementations.
//! When you `unrdf sync`:
//! 1. First run: Creates this file with stubs
//! 2. Subsequent runs: Leaves this file untouched
//! 3. API wrappers regenerate automatically
//!
//! # Intrinsic Availability
//!
//! - `x86_64` with SSE4.2: Uses `_popcnt64` (hardware popcount)
//! - Other targets: Fallback to Brian Kernighan algorithm (counts set bits, O(k) where k = popcount)
//! - All bit positions valid for [0, 63]

/// Set bit at position in u64 value.
///
/// Sets the bit at the given position to 1. Position must be in [0, 63].
///
/// # Arguments
/// * `x` - The u64 value to modify
/// * `pos` - The bit position [0, 63]
///
/// # Returns
/// The value with bit at position `pos` set to 1
///
/// # Panics
/// Panics if `pos >= 64`
///
/// # Examples
/// ```
/// use bcinr_core::logic::bitset::set_bit_u64;
/// assert_eq!(set_bit_u64(0, 0), 1);
/// assert_eq!(set_bit_u64(0, 63), 0x8000000000000000);
/// assert_eq!(set_bit_u64(1, 0), 1); // already set
/// ```
#[inline(always)]
pub(crate) fn set_bit_u64(x: u64, pos: usize) -> u64 {
    debug_assert!(pos < 64, "bit position {} out of range [0, 63]", pos);
    // SAFETY: debug_assert checks bounds; branchless bit operation
    x | (1u64 << pos)
}

/// Clear bit at position in u64 value.
///
/// Sets the bit at the given position to 0. Position must be in [0, 63].
///
/// # Arguments
/// * `x` - The u64 value to modify
/// * `pos` - The bit position [0, 63]
///
/// # Returns
/// The value with bit at position `pos` set to 0
///
/// # Panics
/// Panics if `pos >= 64`
///
/// # Examples
/// ```
/// use bcinr_core::logic::bitset::clear_bit_u64;
/// assert_eq!(clear_bit_u64(u64::MAX, 0), 0xFFFFFFFFFFFFFFFE);
/// assert_eq!(clear_bit_u64(u64::MAX, 63), 0x7FFFFFFFFFFFFFFF);
/// assert_eq!(clear_bit_u64(0, 0), 0); // already clear
/// ```
#[inline(always)]
pub(crate) fn clear_bit_u64(x: u64, pos: usize) -> u64 {
    debug_assert!(pos < 64, "bit position {} out of range [0, 63]", pos);
    // SAFETY: debug_assert checks bounds; branchless bit operation
    x & !(1u64 << pos)
}

/// Count set bits (population count) up to and including position.
///
/// Returns the number of 1-bits in positions [0, pos] (inclusive).
/// Uses hardware popcount on x86_64 via SSE4.2; falls back to Brian Kernighan
/// algorithm on other targets.
///
/// # Arguments
/// * `x` - The u64 value to count
/// * `pos` - The upper limit (inclusive), [0, 63]
///
/// # Returns
/// Number of set bits in [0, pos]
///
/// # Panics
/// Panics if `pos >= 64`
///
/// # Examples
/// ```
/// use bcinr_core::logic::bitset::rank_u64;
/// assert_eq!(rank_u64(0b1010, 3), 2); // bits 1,3 set
/// assert_eq!(rank_u64(0b0001, 0), 1); // bit 0 set
/// assert_eq!(rank_u64(0, 63), 0); // no bits set
/// assert_eq!(rank_u64(u64::MAX, 63), 64); // all bits set
/// ```
#[inline(always)]
pub(crate) fn rank_u64(x: u64, pos: usize) -> usize {
    debug_assert!(pos < 64, "bit position {} out of range [0, 63]", pos);

    // Create mask for bits [0, pos] inclusive
    let mask = if pos == 63 {
        u64::MAX
    } else {
        (1u64 << (pos + 1)) - 1
    };
    let masked = x & mask;

    // Use hardware popcount on x86_64 with SSE4.2
    #[cfg(all(target_arch = "x86_64", target_feature = "popcnt"))]
    {
        // SAFETY: popcnt is available per target_feature attribute
        unsafe { core::arch::x86_64::_popcnt64(masked as i64) as usize }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "popcnt")))]
    {
        // Fallback: Brian Kernighan algorithm
        // Counts only set bits: O(k) where k = popcount
        let mut count = 0;
        let mut val = masked;
        while val != 0 {
            val &= val - 1; // Clear the lowest set bit
            count += 1;
        }
        count
    }
}

/// Find the position of the nth set bit (0-indexed).
///
/// Returns the bit position of the nth 1-bit in the value (0-indexed).
/// Example: if x = 0b01010, select_bit_u64(x, 0) returns 1 (first set bit),
/// select_bit_u64(x, 1) returns 3 (second set bit).
///
/// Uses binary search paired with rank to locate the exact position.
///
/// # Arguments
/// * `x` - The u64 value to search
/// * `nth` - The 0-indexed position of the set bit to find [0, popcount(x) - 1]
///
/// # Returns
/// The bit position [0, 63] of the nth set bit
///
/// # Panics
/// Panics if `nth >= popcount(x)` (nth set bit does not exist)
///
/// # Examples
/// ```
/// use bcinr_core::logic::bitset::select_bit_u64;
/// assert_eq!(select_bit_u64(0b1010, 0), 1); // first set bit at position 1
/// assert_eq!(select_bit_u64(0b1010, 1), 3); // second set bit at position 3
/// assert_eq!(select_bit_u64(1, 0), 0); // single bit at position 0
/// assert_eq!(select_bit_u64(0x8000000000000000, 0), 63); // single bit at position 63
/// ```
#[inline]
pub(crate) fn select_bit_u64(x: u64, nth: usize) -> usize {
    // Count total set bits; return early if nth is out of range
    #[cfg(all(target_arch = "x86_64", target_feature = "popcnt"))]
    let total_bits = unsafe { core::arch::x86_64::_popcnt64(x as i64) as usize };

    #[cfg(not(all(target_arch = "x86_64", target_feature = "popcnt")))]
    let total_bits = {
        let mut count = 0;
        let mut val = x;
        while val != 0 {
            val &= val - 1;
            count += 1;
        }
        count
    };

    debug_assert!(
        nth < total_bits,
        "nth set bit {} does not exist (only {} set bits)",
        nth,
        total_bits
    );

    // Binary search for the position of the nth set bit
    // Invariant: left is a position whose rank <= nth + 1
    //            right is a position whose rank > nth + 1
    let mut left = 0usize;
    let mut right = 64usize;

    while left < right {
        let mid = (left + right) / 2;
        let count_at_mid = if mid == 0 { 0 } else { rank_u64(x, mid - 1) };

        if count_at_mid <= nth {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    // left is now the first position where rank(x, pos) > nth
    // Verify this position has a set bit
    let pos = left - 1;
    debug_assert!(pos < 64, "select position {} out of range", pos);
    debug_assert!(
        (x >> pos) & 1 == 1,
        "position {} should have a set bit",
        pos
    );

    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== set_bit_u64 tests ==========
    #[test]
    fn test_set_bit_u64_position_0() {
        assert_eq!(set_bit_u64(0, 0), 1);
    }

    #[test]
    fn test_set_bit_u64_position_63() {
        assert_eq!(set_bit_u64(0, 63), 0x8000000000000000);
    }

    #[test]
    fn test_set_bit_u64_already_set() {
        assert_eq!(set_bit_u64(1, 0), 1);
        assert_eq!(set_bit_u64(0xFFFFFFFFFFFFFFFF, 0), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_set_bit_u64_all_zeros() {
        for pos in 0..64 {
            let result = set_bit_u64(0, pos);
            assert_eq!(result, 1u64 << pos);
        }
    }

    #[test]
    fn test_set_bit_u64_sparse_pattern() {
        let x = 0b0000_0001; // bit 0 set
        assert_eq!(set_bit_u64(x, 3), 0b0000_1001);
        assert_eq!(set_bit_u64(x, 7), 0b1000_0001);
    }

    // ========== clear_bit_u64 tests ==========
    #[test]
    fn test_clear_bit_u64_position_0() {
        assert_eq!(clear_bit_u64(1, 0), 0);
    }

    #[test]
    fn test_clear_bit_u64_position_63() {
        assert_eq!(clear_bit_u64(0x8000000000000000, 63), 0);
    }

    #[test]
    fn test_clear_bit_u64_already_clear() {
        assert_eq!(clear_bit_u64(0, 0), 0);
        assert_eq!(clear_bit_u64(0, 63), 0);
    }

    #[test]
    fn test_clear_bit_u64_all_ones() {
        let all_ones = 0xFFFFFFFFFFFFFFFF;
        for pos in 0..64 {
            let result = clear_bit_u64(all_ones, pos);
            assert_eq!(result, all_ones ^ (1u64 << pos));
        }
    }

    #[test]
    fn test_clear_bit_u64_dense_pattern() {
        let x = 0xFF; // bits 0-7 set
        assert_eq!(clear_bit_u64(x, 0), 0xFE);
        assert_eq!(clear_bit_u64(x, 7), 0x7F);
        assert_eq!(clear_bit_u64(x, 4), 0xEF);
    }

    // ========== rank_u64 tests ==========
    #[test]
    fn test_rank_u64_no_bits_set() {
        assert_eq!(rank_u64(0, 0), 0);
        assert_eq!(rank_u64(0, 32), 0);
        assert_eq!(rank_u64(0, 63), 0);
    }

    #[test]
    fn test_rank_u64_all_bits_set() {
        assert_eq!(rank_u64(0xFFFFFFFFFFFFFFFF, 0), 1);
        assert_eq!(rank_u64(0xFFFFFFFFFFFFFFFF, 32), 33);
        assert_eq!(rank_u64(0xFFFFFFFFFFFFFFFF, 63), 64);
    }

    #[test]
    fn test_rank_u64_single_bit_position_0() {
        assert_eq!(rank_u64(0b0001, 0), 1);
        assert_eq!(rank_u64(0b0001, 1), 1);
        assert_eq!(rank_u64(0b0001, 63), 1);
    }

    #[test]
    fn test_rank_u64_single_bit_position_63() {
        let bit_63 = 0x8000000000000000;
        assert_eq!(rank_u64(bit_63, 0), 0);
        assert_eq!(rank_u64(bit_63, 32), 0);
        assert_eq!(rank_u64(bit_63, 62), 0);
        assert_eq!(rank_u64(bit_63, 63), 1);
    }

    #[test]
    fn test_rank_u64_sparse_pattern() {
        // bits 1, 3, 5 set
        let x = 0b101010;
        assert_eq!(rank_u64(x, 0), 0);
        assert_eq!(rank_u64(x, 1), 1);
        assert_eq!(rank_u64(x, 2), 1);
        assert_eq!(rank_u64(x, 3), 2);
        assert_eq!(rank_u64(x, 4), 2);
        assert_eq!(rank_u64(x, 5), 3);
        assert_eq!(rank_u64(x, 63), 3);
    }

    #[test]
    fn test_rank_u64_dense_pattern() {
        let x = 0xFF; // bits 0-7 set
        assert_eq!(rank_u64(x, 0), 1);
        assert_eq!(rank_u64(x, 3), 4);
        assert_eq!(rank_u64(x, 7), 8);
        assert_eq!(rank_u64(x, 8), 8);
        assert_eq!(rank_u64(x, 63), 8);
    }

    // ========== select_bit_u64 tests ==========
    #[test]
    fn test_select_bit_u64_single_bit_position_0() {
        assert_eq!(select_bit_u64(1, 0), 0);
    }

    #[test]
    fn test_select_bit_u64_single_bit_position_63() {
        assert_eq!(select_bit_u64(0x8000000000000000, 0), 63);
    }

    #[test]
    fn test_select_bit_u64_two_bits() {
        let x = 0b1010; // bits 1 and 3 set
        assert_eq!(select_bit_u64(x, 0), 1);
        assert_eq!(select_bit_u64(x, 1), 3);
    }

    #[test]
    fn test_select_bit_u64_sparse_pattern() {
        let x = 0b100100_0001; // bits 0, 5, 8 set
        assert_eq!(select_bit_u64(x, 0), 0);
        assert_eq!(select_bit_u64(x, 1), 5);
        assert_eq!(select_bit_u64(x, 2), 8);
    }

    #[test]
    fn test_select_bit_u64_dense_pattern() {
        let x = 0xFF; // bits 0-7 set
        for i in 0..8 {
            assert_eq!(select_bit_u64(x, i), i);
        }
    }

    #[test]
    fn test_select_bit_u64_all_bits_set() {
        let x = 0xFFFFFFFFFFFFFFFF;
        for i in 0..64 {
            assert_eq!(select_bit_u64(x, i), i);
        }
    }

    #[test]
    #[should_panic(expected = "nth set bit")]
    fn test_select_bit_u64_out_of_range() {
        let x = 0b1111; // 4 bits set (nth valid: 0-3)
        select_bit_u64(x, 4); // should panic
    }

    #[test]
    #[should_panic(expected = "nth set bit")]
    fn test_select_bit_u64_no_bits_set() {
        select_bit_u64(0, 0); // should panic (no bits set)
    }

    // ========== Integration tests: round-trip rank ↔ select ==========
    #[test]
    fn test_rank_select_roundtrip() {
        let x = 0b1010_1010_1010_1010;
        for i in 0..8 {
            let pos = select_bit_u64(x, i);
            let rank = rank_u64(x, pos);
            assert_eq!(rank, i + 1, "rank(x, select(x, {})) should be {}", i, i + 1);
        }
    }

    #[test]
    fn test_select_rank_composition() {
        let x = 0xDEADBEEFCAFEBABE;
        #[cfg(all(target_arch = "x86_64", target_feature = "popcnt"))]
        let popcount = unsafe { core::arch::x86_64::_popcnt64(x as i64) as usize };
        #[cfg(not(all(target_arch = "x86_64", target_feature = "popcnt")))]
        let popcount = {
            let mut count = 0;
            let mut val = x;
            while val != 0 {
                val &= val - 1;
                count += 1;
            }
            count
        };

        for i in 0..popcount {
            let pos = select_bit_u64(x, i);
            assert!(
                (x >> pos) & 1 == 1,
                "position {} should have a set bit",
                pos
            );
        }
    }

    // ========== Set/clear composition tests ==========
    #[test]
    fn test_set_then_clear() {
        let x = 0;
        let y = set_bit_u64(x, 5);
        assert_eq!(y, 1 << 5);
        let z = clear_bit_u64(y, 5);
        assert_eq!(z, 0);
    }

    #[test]
    fn test_clear_then_set() {
        let x = 0xFFFFFFFFFFFFFFFF;
        let y = clear_bit_u64(x, 10);
        assert_eq!(y & (1 << 10), 0);
        let z = set_bit_u64(y, 10);
        assert_eq!(z, 0xFFFFFFFFFFFFFFFF);
    }
}

#[cfg(all(test, feature = "bench"))]
mod benches {
    use super::*;

    #[bench]
    fn bench_set_bit_u64(b: &mut test::Bencher) {
        b.iter(|| {
            let mut x = 0u64;
            for pos in 0..64 {
                x = set_bit_u64(x, pos);
            }
            test::black_box(x)
        });
    }

    #[bench]
    fn bench_clear_bit_u64(b: &mut test::Bencher) {
        b.iter(|| {
            let mut x = 0xFFFFFFFFFFFFFFFF;
            for pos in 0..64 {
                x = clear_bit_u64(x, pos);
            }
            test::black_box(x)
        });
    }

    #[bench]
    fn bench_rank_u64_sparse(b: &mut test::Bencher) {
        let x = 0x00FF00FF00FF00FF;
        b.iter(|| {
            let mut sum = 0usize;
            for pos in 0..64 {
                sum = sum.wrapping_add(rank_u64(x, pos));
            }
            test::black_box(sum)
        });
    }

    #[bench]
    fn bench_rank_u64_dense(b: &mut test::Bencher) {
        let x = 0xFFFFFFFFFFFFFFFF;
        b.iter(|| {
            let mut sum = 0usize;
            for pos in 0..64 {
                sum = sum.wrapping_add(rank_u64(x, pos));
            }
            test::black_box(sum)
        });
    }

    #[bench]
    fn bench_select_bit_u64_sparse(b: &mut test::Bencher) {
        let x = 0x0000000000000FFF; // 12 bits set
        b.iter(|| {
            let mut sum = 0usize;
            for i in 0..12 {
                sum = sum.wrapping_add(select_bit_u64(x, i));
            }
            test::black_box(sum)
        });
    }

    #[bench]
    fn bench_select_bit_u64_dense(b: &mut test::Bencher) {
        let x = 0xFFFFFFFFFFFFFFFF;
        b.iter(|| {
            let mut sum = 0usize;
            for i in 0..64 {
                sum = sum.wrapping_add(select_bit_u64(x, i));
            }
            test::black_box(sum)
        });
    }
}
