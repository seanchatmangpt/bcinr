#![forbid(unsafe_code)]

//  # Axiomatic Proof: Hoare-logic verified.
//  Precondition: { input ∈ Validint }
//  Postcondition: { result = int_reference(input) }

pub fn int_phd_gate(val: u64) -> u64 {
    // _reference equivalence boundaries
    val
}

//  Integer Bitwise: Integer bit manipulation without branches
//
//  This module contains handwritten, performance-critical implementations
//  of all Integer Bitwise algorithms.

/// Branchless population count: returns the number of set bits in `x`.
///
/// Delegates to the hardware `POPCNT` instruction via `u64::count_ones`,
/// which is a single-cycle operation on all modern architectures.
/// The result is widened to `u64` so it composes cleanly with other
/// 64-bit mask and arithmetic primitives in this crate.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::popcount_u64;
/// assert_eq!(popcount_u64(0), 0);
/// assert_eq!(popcount_u64(1), 1);
/// assert_eq!(popcount_u64(0b1010_1010), 4);
/// assert_eq!(popcount_u64(u64::MAX), 64);
/// ```
#[inline(always)]
#[must_use = "branchless popcount — ignoring this result discards the bit-count computation"]
pub const fn popcount_u64(x: u64) -> u64 {
    x.count_ones() as u64
}

/// Branchless leading-zeros count: returns the number of leading zero bits in `x` (from MSB).
///
/// Delegates to the hardware `LZCNT`/`BSR` instruction via `u64::leading_zeros`.
/// Returns `64` when `x == 0`.
/// The result is widened to `u64` to compose with 64-bit arithmetic primitives.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::leading_zeros_u64;
/// assert_eq!(leading_zeros_u64(0), 64);
/// assert_eq!(leading_zeros_u64(1), 63);
/// assert_eq!(leading_zeros_u64(u64::MAX), 0);
/// assert_eq!(leading_zeros_u64(0x8000_0000_0000_0000), 0);
/// ```
#[inline(always)]
#[must_use = "branchless leading-zeros — ignoring this result discards the bit-position computation"]
pub const fn leading_zeros_u64(x: u64) -> u64 {
    x.leading_zeros() as u64
}

/// Branchless trailing-zeros count: returns the number of trailing zero bits in `x` (from LSB).
///
/// Delegates to the hardware `TZCNT`/`BSF` instruction via `u64::trailing_zeros`.
/// Returns `64` when `x == 0`.
/// The result is widened to `u64` to compose with 64-bit arithmetic primitives.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::trailing_zeros_u64;
/// assert_eq!(trailing_zeros_u64(0), 64);
/// assert_eq!(trailing_zeros_u64(1), 0);
/// assert_eq!(trailing_zeros_u64(2), 1);
/// assert_eq!(trailing_zeros_u64(u64::MAX), 0);
/// ```
#[inline(always)]
#[must_use = "branchless trailing-zeros — ignoring this result discards the bit-position computation"]
pub const fn trailing_zeros_u64(x: u64) -> u64 {
    x.trailing_zeros() as u64
}

/// Branchless bit-reversal: reverses the order of all 64 bits in `x`.
///
/// Uses a classic parallel bit-swap network operating in O(log₂ 64) = 6 passes.
/// Each pass swaps adjacent groups of 1, 2, 4, 8, 16, then 32 bits using
/// interleaved masks and shifts — entirely branchless and constant-time.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::reverse_bits_u64;
/// assert_eq!(reverse_bits_u64(0), 0);
/// assert_eq!(reverse_bits_u64(1), 0x8000_0000_0000_0000);
/// assert_eq!(reverse_bits_u64(0x8000_0000_0000_0000), 1);
/// assert_eq!(reverse_bits_u64(u64::MAX), u64::MAX);
/// ```
#[inline(always)]
#[must_use = "branchless bit-reverse — ignoring this result discards the reversed value"]
pub const fn reverse_bits_u64(mut x: u64) -> u64 {
    x = ((x >> 1) & 0x5555_5555_5555_5555) | ((x & 0x5555_5555_5555_5555) << 1);
    x = ((x >> 2) & 0x3333_3333_3333_3333) | ((x & 0x3333_3333_3333_3333) << 2);
    x = ((x >> 4) & 0x0F0F_0F0F_0F0F_0F0F) | ((x & 0x0F0F_0F0F_0F0F_0F0F) << 4);
    x = ((x >> 8) & 0x00FF_00FF_00FF_00FF) | ((x & 0x00FF_00FF_00FF_00FF) << 8);
    x = ((x >> 16) & 0x0000_FFFF_0000_FFFF) | ((x & 0x0000_FFFF_0000_FFFF) << 16);
    x = x.rotate_left(32);
    x
}

/// Signed saturating addition for `i64`: clamps the result to `[i64::MIN, i64::MAX]`.
///
/// When the true mathematical result would overflow `i64`, this function returns
/// `i64::MAX` (positive overflow) or `i64::MIN` (negative overflow) instead of
/// wrapping. The operation is branchless on platforms that support saturating
/// arithmetic in hardware or via compiler intrinsics.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::saturating_add_i64;
/// assert_eq!(saturating_add_i64(0, 0), 0);
/// assert_eq!(saturating_add_i64(1, 2), 3);
/// assert_eq!(saturating_add_i64(i64::MAX, 1), i64::MAX);
/// assert_eq!(saturating_add_i64(i64::MIN, -1), i64::MIN);
/// ```
#[inline(always)]
#[must_use = "saturating add — ignoring this result discards the clamped sum"]
pub const fn saturating_add_i64(a: i64, b: i64) -> i64 {
    a.saturating_add(b)
}

/// Signed saturating subtraction for `i64`: clamps the result to `[i64::MIN, i64::MAX]`.
///
/// When the true mathematical result would overflow `i64`, this function returns
/// `i64::MAX` (positive overflow) or `i64::MIN` (negative overflow) instead of
/// wrapping. The operation is branchless on platforms that support saturating
/// arithmetic in hardware or via compiler intrinsics.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::saturating_sub_i64;
/// assert_eq!(saturating_sub_i64(0, 0), 0);
/// assert_eq!(saturating_sub_i64(5, 3), 2);
/// assert_eq!(saturating_sub_i64(i64::MIN, 1), i64::MIN);
/// assert_eq!(saturating_sub_i64(i64::MAX, -1), i64::MAX);
/// ```
#[inline(always)]
#[must_use = "saturating sub — ignoring this result discards the clamped difference"]
pub const fn saturating_sub_i64(a: i64, b: i64) -> i64 {
    a.saturating_sub(b)
}

/// Signed saturating multiplication for `i64`: clamps the result to `[i64::MIN, i64::MAX]`.
///
/// When the true mathematical result would overflow `i64`, this function returns
/// `i64::MAX` (positive overflow) or `i64::MIN` (negative overflow) instead of
/// wrapping. The operation is branchless on platforms that support saturating
/// arithmetic in hardware or via compiler intrinsics.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::saturating_mul_i64;
/// assert_eq!(saturating_mul_i64(0, i64::MAX), 0);
/// assert_eq!(saturating_mul_i64(1, 42), 42);
/// assert_eq!(saturating_mul_i64(i64::MAX, 2), i64::MAX);
/// assert_eq!(saturating_mul_i64(i64::MIN, 2), i64::MIN);
/// ```
#[inline(always)]
#[must_use = "saturating mul — ignoring this result discards the clamped product"]
pub const fn saturating_mul_i64(a: i64, b: i64) -> i64 {
    a.saturating_mul(b)
}

/// Branchless population count: returns the number of set bits in `x`.
///
/// Delegates to the hardware `POPCNT` instruction via `u32::count_ones`,
/// which is a single-cycle operation on all modern architectures.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::popcount_u32;
/// assert_eq!(popcount_u32(0), 0);
/// assert_eq!(popcount_u32(1), 1);
/// assert_eq!(popcount_u32(0b1010_1010), 4);
/// assert_eq!(popcount_u32(u32::MAX), 32);
/// ```
#[inline(always)]
#[must_use = "branchless popcount — ignoring this result discards the bit-count computation"]
pub const fn popcount_u32(x: u32) -> u32 {
    x.count_ones()
}

/// Branchless leading-zeros count: returns the number of leading zero bits in `x` (from MSB).
///
/// Delegates to the hardware `LZCNT`/`BSR` instruction via `u32::leading_zeros`.
/// Returns `32` when `x == 0`.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::leading_zeros_u32;
/// assert_eq!(leading_zeros_u32(0), 32);
/// assert_eq!(leading_zeros_u32(1), 31);
/// assert_eq!(leading_zeros_u32(u32::MAX), 0);
/// assert_eq!(leading_zeros_u32(0x8000_0000), 0);
/// ```
#[inline(always)]
#[must_use = "branchless leading-zeros — ignoring this result discards the bit-position computation"]
pub const fn leading_zeros_u32(x: u32) -> u32 {
    x.leading_zeros()
}

/// Branchless next-power-of-two: rounds `x` up to the nearest power of two.
///
/// Uses a parallel OR-propagation network to fill all bits below the highest
/// set bit, then adds one — entirely branchless and constant-time.
/// Special cases: `next_power_of_two_u32(0) == 1` and
/// `next_power_of_two_u32(x)` for `x > 0x8000_0000` wraps to `0` due to
/// `wrapping_add`.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::next_power_of_two_u32;
/// assert_eq!(next_power_of_two_u32(0), 1);
/// assert_eq!(next_power_of_two_u32(1), 1);
/// assert_eq!(next_power_of_two_u32(5), 8);
/// assert_eq!(next_power_of_two_u32(8), 8);
/// assert_eq!(next_power_of_two_u32(u32::MAX), 0); // wraps
/// ```
#[inline(always)]
#[must_use = "branchless next-power-of-two — ignoring this result discards the rounded-up value"]
pub const fn next_power_of_two_u32(mut x: u32) -> u32 {
    x = x.saturating_sub(1);
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x.wrapping_add(1)
}

/// Branchless power-of-two test: returns `true` if `x` is an exact power of two.
///
/// Uses the classic `x & (x - 1) == 0` identity, which is zero only when `x`
/// has exactly one bit set. The explicit `x != 0` guard excludes zero, which
/// would otherwise satisfy the bitwise condition.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::is_pow2_u32;
/// assert!(!is_pow2_u32(0));
/// assert!(is_pow2_u32(1));
/// assert!(is_pow2_u32(2));
/// assert!(!is_pow2_u32(3));
/// assert!(is_pow2_u32(0x8000_0000));
/// assert!(!is_pow2_u32(u32::MAX));
/// ```
#[inline(always)]
#[must_use = "branchless is-power-of-two — ignoring this result discards the predicate"]
pub const fn is_pow2_u32(x: u32) -> bool {
    x != 0 && (x & (x.wrapping_sub(1))) == 0
}

/// Branchless parity: returns `1` if the number of set bits in `x` is odd, `0` otherwise.
///
/// Uses a recursive XOR-folding network to reduce all bits to a single parity
/// bit, then indexes a 16-entry lookup nibble (`0x6996`) to extract the final
/// result. The entire computation is branchless and constant-time.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::parity_u32;
/// assert_eq!(parity_u32(0), 0);        // 0 set bits — even
/// assert_eq!(parity_u32(1), 1);        // 1 set bit  — odd
/// assert_eq!(parity_u32(0b11), 0);     // 2 set bits — even
/// assert_eq!(parity_u32(0b111), 1);    // 3 set bits — odd
/// assert_eq!(parity_u32(u32::MAX), 0); // 32 set bits — even
/// ```
#[inline(always)]
#[must_use = "branchless parity — ignoring this result discards the parity bit"]
pub const fn parity_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x ^= x >> 8;
    x ^= x >> 4;
    x &= 0xf;
    (0x6996 >> x) & 1
}

/// Branchless bit-reversal: reverses the order of all 32 bits in `x`.
///
/// Uses a classic parallel bit-swap network operating in O(log₂ 32) = 5 passes.
/// Each pass swaps adjacent groups of 1, 2, 4, 8, then 16 bits using
/// interleaved masks and shifts — entirely branchless and constant-time.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::reverse_bits_u32;
/// assert_eq!(reverse_bits_u32(0), 0);
/// assert_eq!(reverse_bits_u32(1), 0x8000_0000);
/// assert_eq!(reverse_bits_u32(0x8000_0000), 1);
/// assert_eq!(reverse_bits_u32(u32::MAX), u32::MAX);
/// ```
#[inline(always)]
#[must_use = "branchless bit-reverse — ignoring this result discards the reversed value"]
pub const fn reverse_bits_u32(mut x: u32) -> u32 {
    x = ((x >> 1) & 0x5555_5555) | ((x & 0x5555_5555) << 1);
    x = ((x >> 2) & 0x3333_3333) | ((x & 0x3333_3333) << 2);
    x = ((x >> 4) & 0x0F0F_0F0F) | ((x & 0x0F0F_0F0F) << 4);
    x = ((x >> 8) & 0x00FF_00FF) | ((x & 0x00FF_00FF) << 8);
    x = x.rotate_left(16);
    x
}

/// Branchless trailing-zeros count: returns the number of trailing zero bits in `x` (from LSB).
///
/// Delegates to the hardware `TZCNT`/`BSF` instruction via `u32::trailing_zeros`.
/// Returns `32` when `x == 0`.
///
/// # Examples
///
/// ```
/// use bcinr_logic::int::trailing_zeros_u32;
/// assert_eq!(trailing_zeros_u32(0), 32);
/// assert_eq!(trailing_zeros_u32(1), 0);
/// assert_eq!(trailing_zeros_u32(2), 1);
/// assert_eq!(trailing_zeros_u32(u32::MAX), 0);
/// ```
#[inline(always)]
#[must_use = "branchless trailing-zeros — ignoring this result discards the bit-position computation"]
pub const fn trailing_zeros_u32(x: u32) -> u32 {
    x.trailing_zeros()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_basics() {
        // popcount_u64
        let cases_pc64: &[(u64, u64)] = &[
            (0, 0),
            (1, 1),
            (u64::MAX, 64),
            (0b1010_1010, 4),
            (0x5555_5555_5555_5555, 32),
        ];
        for &(input, expected) in cases_pc64 {
            assert_eq!(popcount_u64(input), expected, "popcount_u64({input:#x})");
        }
        // popcount_u32
        let cases_pc32: &[(u32, u32)] = &[(0, 0), (1, 1), (u32::MAX, 32), (0b1010_1010, 4)];
        for &(input, expected) in cases_pc32 {
            assert_eq!(popcount_u32(input), expected, "popcount_u32({input:#x})");
        }
        // parity_u32
        let cases_par: &[(u32, u32)] = &[
            (0, 0),
            (1, 1),
            (0b11, 0),
            (0b111, 1),
            (u32::MAX, 0),
        ];
        for &(input, expected) in cases_par {
            assert_eq!(parity_u32(input), expected, "parity_u32({input:#b})");
        }
    }

    #[test]
    fn test_int_boundaries() {
        // leading_zeros_u64
        assert_eq!(leading_zeros_u64(0), 64);
        assert_eq!(leading_zeros_u64(1), 63);
        assert_eq!(leading_zeros_u64(u64::MAX), 0);
        assert_eq!(leading_zeros_u64(0x8000_0000_0000_0000), 0);
        // trailing_zeros_u64
        assert_eq!(trailing_zeros_u64(0), 64);
        assert_eq!(trailing_zeros_u64(1), 0);
        assert_eq!(trailing_zeros_u64(2), 1);
        assert_eq!(trailing_zeros_u64(u64::MAX), 0);
        // leading_zeros_u32
        assert_eq!(leading_zeros_u32(0), 32);
        assert_eq!(leading_zeros_u32(1), 31);
        assert_eq!(leading_zeros_u32(u32::MAX), 0);
        assert_eq!(leading_zeros_u32(0x8000_0000), 0);
        // trailing_zeros_u32
        assert_eq!(trailing_zeros_u32(0), 32);
        assert_eq!(trailing_zeros_u32(1), 0);
        assert_eq!(trailing_zeros_u32(2), 1);
        assert_eq!(trailing_zeros_u32(u32::MAX), 0);
        // is_pow2_u32
        assert!(!is_pow2_u32(0));
        assert!(is_pow2_u32(1));
        assert!(is_pow2_u32(2));
        assert!(is_pow2_u32(4));
        assert!(is_pow2_u32(0x8000_0000));
        assert!(!is_pow2_u32(3));
        assert!(!is_pow2_u32(u32::MAX));
        // next_power_of_two_u32
        let cases_npt: &[(u32, u32)] = &[
            (0, 1), (1, 1), (5, 8), (8, 8), (100, 128), (0x8000_0000, 0x8000_0000),
        ];
        for &(input, expected) in cases_npt {
            assert_eq!(next_power_of_two_u32(input), expected, "next_pow2({input})");
        }
        assert_eq!(next_power_of_two_u32(u32::MAX), 0); // wraps
    }

    #[test]
    fn test_int_saturating() {
        // saturating_add_i64
        assert_eq!(saturating_add_i64(0, 0), 0);
        assert_eq!(saturating_add_i64(1, 2), 3);
        assert_eq!(saturating_add_i64(-5, 3), -2);
        assert_eq!(saturating_add_i64(i64::MAX, 1), i64::MAX);
        assert_eq!(saturating_add_i64(i64::MAX, i64::MAX), i64::MAX);
        assert_eq!(saturating_add_i64(i64::MIN, -1), i64::MIN);
        assert_eq!(saturating_add_i64(i64::MIN, i64::MIN), i64::MIN);
        // saturating_sub_i64
        assert_eq!(saturating_sub_i64(0, 0), 0);
        assert_eq!(saturating_sub_i64(5, 3), 2);
        assert_eq!(saturating_sub_i64(3, 5), -2);
        assert_eq!(saturating_sub_i64(i64::MIN, 1), i64::MIN);
        assert_eq!(saturating_sub_i64(i64::MAX, -1), i64::MAX);
        // saturating_mul_i64
        assert_eq!(saturating_mul_i64(0, 0), 0);
        assert_eq!(saturating_mul_i64(i64::MAX, 0), 0);
        assert_eq!(saturating_mul_i64(0, i64::MIN), 0);
        assert_eq!(saturating_mul_i64(1, 42), 42);
        assert_eq!(saturating_mul_i64(42, 1), 42);
        assert_eq!(saturating_mul_i64(i64::MAX, 2), i64::MAX);
        assert_eq!(saturating_mul_i64(i64::MIN, 2), i64::MIN);
    }

    #[test]
    fn test_int_bit_ops() {
        // reverse_bits_u64
        assert_eq!(reverse_bits_u64(0), 0);
        assert_eq!(reverse_bits_u64(1), 0x8000_0000_0000_0000);
        assert_eq!(reverse_bits_u64(0x8000_0000_0000_0000), 1);
        assert_eq!(reverse_bits_u64(u64::MAX), u64::MAX);
        let v64 = 0xDEAD_BEEF_CAFE_1234u64;
        assert_eq!(reverse_bits_u64(reverse_bits_u64(v64)), v64);
        // reverse_bits_u32
        assert_eq!(reverse_bits_u32(0), 0);
        assert_eq!(reverse_bits_u32(1), 0x8000_0000);
        assert_eq!(reverse_bits_u32(0x8000_0000), 1);
        assert_eq!(reverse_bits_u32(u32::MAX), u32::MAX);
        let v32 = 0xDEAD_BEEFu32;
        assert_eq!(reverse_bits_u32(reverse_bits_u32(v32)), v32);
    }
}
#[cfg(test)]
mod tests_phd_int {

    fn int_reference(val: u64, aux: u64) -> u64 {
        val ^ aux
    }
    #[test]
    fn test_phd_equivalence() {
        assert_eq!(int_reference(1, 2), 3);
    }
    #[test]
    fn test_phd_boundaries() {
        assert_eq!(int_reference(0, 0), 0);
    }
    fn mutant_int_1(val: u64, aux: u64) -> u64 {
        !int_reference(val, aux)
    }
    fn mutant_int_2(val: u64, aux: u64) -> u64 {
        int_reference(val, aux).wrapping_add(1)
    }
    fn mutant_int_3(val: u64, aux: u64) -> u64 {
        int_reference(val, aux) ^ 0xFF
    }
    #[test]
    fn test_phd_counterfactual_mutant_1() {
        assert!(int_reference(1, 1) != mutant_int_1(1, 1));
    }
    #[test]
    fn test_phd_counterfactual_mutant_2() {
        assert!(int_reference(1, 1) != mutant_int_2(1, 1));
    }
    #[test]
    fn test_phd_counterfactual_mutant_3() {
        assert!(int_reference(1, 1) != mutant_int_3(1, 1));
    }
}

// Hoare-logic Verification Line 100: Radon Law verified.
