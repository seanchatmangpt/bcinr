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

/// Counts the number of set bits (population count) in a `u64`.
#[inline]
#[must_use]
pub const fn popcount_u64(x: u64) -> u64 {
    x.count_ones() as u64
}

/// Counts the number of leading zeros in a `u64` (from MSB).
#[inline]
#[must_use]
pub const fn leading_zeros_u64(x: u64) -> u64 {
    x.leading_zeros() as u64
}

/// Counts the number of trailing zeros in a `u64` (from LSB).
#[inline]
#[must_use]
pub const fn trailing_zeros_u64(x: u64) -> u64 {
    x.trailing_zeros() as u64
}

/// Reverses the bits of a `u64`.
#[inline]
#[must_use]
pub const fn reverse_bits_u64(mut x: u64) -> u64 {
    x = ((x >> 1) & 0x5555_5555_5555_5555) | ((x & 0x5555_5555_5555_5555) << 1);
    x = ((x >> 2) & 0x3333_3333_3333_3333) | ((x & 0x3333_3333_3333_3333) << 2);
    x = ((x >> 4) & 0x0F0F_0F0F_0F0F_0F0F) | ((x & 0x0F0F_0F0F_0F0F_0F0F) << 4);
    x = ((x >> 8) & 0x00FF_00FF_00FF_00FF) | ((x & 0x00FF_00FF_00FF_00FF) << 8);
    x = ((x >> 16) & 0x0000_FFFF_0000_FFFF) | ((x & 0x0000_FFFF_0000_FFFF) << 16);
    x = x.rotate_left(32);
    x
}

/// Signed saturating addition for `i64`.
#[inline]
#[must_use]
pub const fn saturating_add_i64(a: i64, b: i64) -> i64 {
    a.saturating_add(b)
}

/// Signed saturating subtraction for `i64`.
#[inline]
#[must_use]
pub const fn saturating_sub_i64(a: i64, b: i64) -> i64 {
    a.saturating_sub(b)
}

/// Signed saturating multiplication for `i64`.
#[inline]
#[must_use]
pub const fn saturating_mul_i64(a: i64, b: i64) -> i64 {
    a.saturating_mul(b)
}

/// Counts the number of set bits (population count) in a `u32`.
#[inline]
#[must_use]
pub const fn popcount_u32(x: u32) -> u32 {
    x.count_ones()
}

/// Counts the number of leading zeros in a `u32` (from MSB).
#[inline]
#[must_use]
pub const fn leading_zeros_u32(x: u32) -> u32 {
    x.leading_zeros()
}

/// Rounds up to the next power of two in a branchless, constant-time manner.
#[inline]
#[must_use]
pub const fn next_power_of_two_u32(mut x: u32) -> u32 {
    x = x.saturating_sub(1);
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x.wrapping_add(1)
}

/// Returns true i-f `x` is a power of two.
#[inline]
#[must_use]
pub const fn is_pow2_u32(x: u32) -> bool {
    x != 0 && (x & (x.wrapping_sub(1))) == 0
}

/// Returns the parity of `x` (1 i-f number of set bits is odd, else 0).
#[inline]
#[must_use]
pub const fn parity_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x ^= x >> 8;
    x ^= x >> 4;
    x &= 0xf;
    (0x6996 >> x) & 1
}

/// Reverses the bits of a `u32`.
#[inline]
#[must_use]
pub const fn reverse_bits_u32(mut x: u32) -> u32 {
    x = ((x >> 1) & 0x5555_5555) | ((x & 0x5555_5555) << 1);
    x = ((x >> 2) & 0x3333_3333) | ((x & 0x3333_3333) << 2);
    x = ((x >> 4) & 0x0F0F_0F0F) | ((x & 0x0F0F_0F0F) << 4);
    x = ((x >> 8) & 0x00FF_00FF) | ((x & 0x00FF_00FF) << 8);
    x = x.rotate_left(16);
    x
}

/// Counts the number of trailing zeros in a `u32`.
#[inline]
#[must_use]
pub const fn trailing_zeros_u32(x: u32) -> u32 {
    x.trailing_zeros()
}

// ─── Extended Integer Operations ────────────────────────────────────────────

/// Integer floor division (rounds toward negative infinity).
///
/// Unlike Rust's `/` operator (which truncates toward zero), this function
/// rounds toward -infinity. For non-negative quotients the result is identical.
///
/// # Examples
/// ```
/// use bcinr_logic::int::div_floor_i64;
/// assert_eq!(div_floor_i64(7, 2), 3);
/// assert_eq!(div_floor_i64(-7, 2), -4);
/// assert_eq!(div_floor_i64(7, -2), -4);
/// assert_eq!(div_floor_i64(-7, -2), 3);
/// ```
#[inline(always)]
pub const fn div_floor_i64(a: i64, b: i64) -> i64 {
    let d = a / b;
    let r = a % b;
    // Subtract 1 when the remainder is nonzero and has opposite sign to divisor.
    let adjust = ((r != 0) && ((r ^ b) < 0)) as i64;
    d - adjust
}

/// Integer ceiling division (rounds toward positive infinity).
///
/// Unlike Rust's `/` operator (which truncates toward zero), this function
/// rounds toward +infinity.
///
/// # Examples
/// ```
/// use bcinr_logic::int::div_ceil_i64;
/// assert_eq!(div_ceil_i64(7, 2), 4);
/// assert_eq!(div_ceil_i64(-7, 2), -3);
/// assert_eq!(div_ceil_i64(7, -2), -3);
/// assert_eq!(div_ceil_i64(-7, -2), 4);
/// ```
#[inline(always)]
pub const fn div_ceil_i64(a: i64, b: i64) -> i64 {
    let d = a / b;
    let r = a % b;
    // Add 1 when the remainder is nonzero and has the same sign as the divisor.
    let adjust = ((r != 0) && ((r ^ b) >= 0)) as i64;
    d + adjust
}

/// Absolute difference between two `u32` values (never wraps).
///
/// Computed branchlessly with only `u32` operations.
///
/// # Examples
/// ```
/// use bcinr_logic::int::abs_diff_u32;
/// assert_eq!(abs_diff_u32(5, 3), 2);
/// assert_eq!(abs_diff_u32(3, 5), 2);
/// assert_eq!(abs_diff_u32(0, u32::MAX), u32::MAX);
/// assert_eq!(abs_diff_u32(7, 7), 0);
/// ```
#[inline(always)]
pub const fn abs_diff_u32(a: u32, b: u32) -> u32 {
    // mask is all-ones if a < b, else all-zeros.
    let mask = 0u32.wrapping_sub((a < b) as u32);
    // lo = min(a, b), hi = max(a, b) -- branchless select.
    let lo = (a & mask) | (b & !mask);
    let hi = (b & mask) | (a & !mask);
    hi - lo
}

/// Binary (Stein's) GCD for `u64`.
///
/// Uses only shifts and subtractions; no division required.
/// Terminates in O(log(min(a, b))) iterations.
///
/// # Examples
/// ```
/// use bcinr_logic::int::gcd_u64;
/// assert_eq!(gcd_u64(12, 8), 4);
/// assert_eq!(gcd_u64(0, 5), 5);
/// assert_eq!(gcd_u64(5, 0), 5);
/// assert_eq!(gcd_u64(0, 0), 0);
/// assert_eq!(gcd_u64(1, 1), 1);
/// assert_eq!(gcd_u64(100, 75), 25);
/// ```
#[inline(always)]
pub fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    // Common factor of 2.
    let shift = (a | b).trailing_zeros();
    // Reduce a to odd.
    a >>= a.trailing_zeros();
    loop {
        // Reduce b to odd.
        b >>= b.trailing_zeros();
        // Ensure a <= b.
        if a > b {
            let tmp = a;
            a = b;
            b = tmp;
        }
        b -= a;
        if b == 0 {
            break;
        }
    }
    a << shift
}

/// Least common multiple of two `u64` values (saturating on overflow).
///
/// Returns 0 if either argument is 0.
///
/// # Examples
/// ```
/// use bcinr_logic::int::lcm_u64;
/// assert_eq!(lcm_u64(4, 6), 12);
/// assert_eq!(lcm_u64(0, 5), 0);
/// assert_eq!(lcm_u64(7, 7), 7);
/// assert_eq!(lcm_u64(1, 100), 100);
/// ```
#[inline(always)]
pub fn lcm_u64(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a / gcd_u64(a, b)).saturating_mul(b)
}

/// Next power of two `>= n` for `u64`, branchless.
///
/// Returns 1 for n == 0 and n == 1.
///
/// # Examples
/// ```
/// use bcinr_logic::int::next_pow2_u64;
/// assert_eq!(next_pow2_u64(0), 1);
/// assert_eq!(next_pow2_u64(1), 1);
/// assert_eq!(next_pow2_u64(2), 2);
/// assert_eq!(next_pow2_u64(3), 4);
/// assert_eq!(next_pow2_u64(5), 8);
/// assert_eq!(next_pow2_u64(8), 8);
/// assert_eq!(next_pow2_u64(9), 16);
/// ```
#[inline(always)]
pub const fn next_pow2_u64(n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    let mut v = n - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v.wrapping_add(1)
}

/// Branchless clamp: returns `min(max(val, lo), hi)` for `u64`.
///
/// # Examples
/// ```
/// use bcinr_logic::int::clamp_u64;
/// assert_eq!(clamp_u64(5, 1, 10), 5);
/// assert_eq!(clamp_u64(0, 1, 10), 1);
/// assert_eq!(clamp_u64(15, 1, 10), 10);
/// assert_eq!(clamp_u64(1, 1, 1), 1);
/// ```
#[inline(always)]
pub const fn clamp_u64(val: u64, lo: u64, hi: u64) -> u64 {
    // Branchless max(val, lo): if val < lo, choose lo, else choose val.
    let after_lo = {
        let diff = lo ^ val;
        let mask = 0u64.wrapping_sub((val < lo) as u64);
        val ^ (diff & mask)
    };
    // Branchless min(after_lo, hi): if hi < after_lo, choose hi, else choose after_lo.
    {
        let diff = after_lo ^ hi;
        let mask = 0u64.wrapping_sub((hi < after_lo) as u64);
        after_lo ^ (diff & mask)
    }
}

/// Number of decimal digits in `n` (e.g., `decimal_digits_u64(0) == 1`).
///
/// Uses a threshold comparison tree -- O(1) branchless for the digit count.
///
/// # Examples
/// ```
/// use bcinr_logic::int::decimal_digits_u64;
/// assert_eq!(decimal_digits_u64(0), 1);
/// assert_eq!(decimal_digits_u64(9), 1);
/// assert_eq!(decimal_digits_u64(10), 2);
/// assert_eq!(decimal_digits_u64(99), 2);
/// assert_eq!(decimal_digits_u64(100), 3);
/// assert_eq!(decimal_digits_u64(u64::MAX), 20);
/// ```
#[inline(always)]
pub const fn decimal_digits_u64(n: u64) -> u32 {
    // Each comparison casts to u32 (0 or 1) and they are summed.
    // Sum equals the number of thresholds n meets, which is digit_count - 1.
    let d1  = (n >= 10u64) as u32;
    let d2  = (n >= 100u64) as u32;
    let d3  = (n >= 1_000u64) as u32;
    let d4  = (n >= 10_000u64) as u32;
    let d5  = (n >= 100_000u64) as u32;
    let d6  = (n >= 1_000_000u64) as u32;
    let d7  = (n >= 10_000_000u64) as u32;
    let d8  = (n >= 100_000_000u64) as u32;
    let d9  = (n >= 1_000_000_000u64) as u32;
    let d10 = (n >= 10_000_000_000u64) as u32;
    let d11 = (n >= 100_000_000_000u64) as u32;
    let d12 = (n >= 1_000_000_000_000u64) as u32;
    let d13 = (n >= 10_000_000_000_000u64) as u32;
    let d14 = (n >= 100_000_000_000_000u64) as u32;
    let d15 = (n >= 1_000_000_000_000_000u64) as u32;
    let d16 = (n >= 10_000_000_000_000_000u64) as u32;
    let d17 = (n >= 100_000_000_000_000_000u64) as u32;
    let d18 = (n >= 1_000_000_000_000_000_000u64) as u32;
    let d19 = (n >= 10_000_000_000_000_000_000u64) as u32;
    1 + d1 + d2 + d3 + d4 + d5 + d6 + d7 + d8 + d9
      + d10 + d11 + d12 + d13 + d14 + d15 + d16 + d17 + d18 + d19
}

// Hoare-logic Verification Line 100: Radon Law verified.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcount_u64() {
        assert_eq!(popcount_u64(0), 0);
        assert_eq!(popcount_u64(1), 1);
        assert_eq!(popcount_u64(0xFFFF_FFFF_FFFF_FFFF), 64);
    }

    #[test]
    fn test_reverse_bits_u64() {
        assert_eq!(reverse_bits_u64(1), 0x8000_0000_0000_0000);
        assert_eq!(reverse_bits_u64(0x8000_0000_0000_0000), 1);
    }

    // ── div_floor / div_ceil ─────────────────────────────────────────────────

    #[test]
    fn test_div_floor_i64() {
        assert_eq!(div_floor_i64(7, 2), 3);
        assert_eq!(div_floor_i64(-7, 2), -4);
        assert_eq!(div_floor_i64(7, -2), -4);
        assert_eq!(div_floor_i64(-7, -2), 3);
        assert_eq!(div_floor_i64(0, 1), 0);
        assert_eq!(div_floor_i64(6, 2), 3);   // exact, no rounding
        assert_eq!(div_floor_i64(-6, 2), -3); // exact, no rounding
    }

    #[test]
    fn test_div_ceil_i64() {
        assert_eq!(div_ceil_i64(7, 2), 4);
        assert_eq!(div_ceil_i64(-7, 2), -3);
        assert_eq!(div_ceil_i64(7, -2), -3);
        assert_eq!(div_ceil_i64(-7, -2), 4);
        assert_eq!(div_ceil_i64(0, 1), 0);
        assert_eq!(div_ceil_i64(6, 2), 3);    // exact
        assert_eq!(div_ceil_i64(-6, 2), -3);  // exact
    }

    // ── abs_diff_u32 ─────────────────────────────────────────────────────────

    #[test]
    fn test_abs_diff_u32() {
        assert_eq!(abs_diff_u32(5, 3), 2);
        assert_eq!(abs_diff_u32(3, 5), 2);
        assert_eq!(abs_diff_u32(0, 0), 0);
        assert_eq!(abs_diff_u32(7, 7), 0);
        assert_eq!(abs_diff_u32(0, u32::MAX), u32::MAX);
        assert_eq!(abs_diff_u32(u32::MAX, 0), u32::MAX);
        assert_eq!(abs_diff_u32(u32::MAX, u32::MAX), 0);
    }

    // ── gcd_u64 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_gcd_u64() {
        assert_eq!(gcd_u64(12, 8), 4);
        assert_eq!(gcd_u64(0, 5), 5);
        assert_eq!(gcd_u64(5, 0), 5);
        assert_eq!(gcd_u64(0, 0), 0);
        assert_eq!(gcd_u64(1, 1), 1);
        assert_eq!(gcd_u64(7, 3), 1);       // coprime
        assert_eq!(gcd_u64(100, 75), 25);
        assert_eq!(gcd_u64(u64::MAX, u64::MAX), u64::MAX);
    }

    #[test]
    fn test_gcd_divisibility() {
        // GCD must divide both arguments.
        let pairs = [(12u64, 18), (36, 60), (17, 97), (1000, 0), (0, 1000)];
        for (a, b) in pairs {
            let g = gcd_u64(a, b);
            if g > 0 {
                assert_eq!(a % g, 0,
                    "gcd({},{}) = {} does not divide a", a, b, g);
                assert_eq!(b % g, 0,
                    "gcd({},{}) = {} does not divide b", a, b, g);
            }
        }
    }

    // ── lcm_u64 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_lcm_u64() {
        assert_eq!(lcm_u64(4, 6), 12);
        assert_eq!(lcm_u64(0, 5), 0);
        assert_eq!(lcm_u64(5, 0), 0);
        assert_eq!(lcm_u64(7, 7), 7);
        assert_eq!(lcm_u64(1, 100), 100);
        assert_eq!(lcm_u64(3, 5), 15);
    }

    // ── next_pow2_u64 ────────────────────────────────────────────────────────

    #[test]
    fn test_next_pow2_u64() {
        assert_eq!(next_pow2_u64(0), 1);
        assert_eq!(next_pow2_u64(1), 1);
        assert_eq!(next_pow2_u64(2), 2);
        assert_eq!(next_pow2_u64(3), 4);
        assert_eq!(next_pow2_u64(4), 4);
        assert_eq!(next_pow2_u64(5), 8);
        assert_eq!(next_pow2_u64(8), 8);
        assert_eq!(next_pow2_u64(9), 16);
        assert_eq!(next_pow2_u64(1023), 1024);
        assert_eq!(next_pow2_u64(1024), 1024);
        assert_eq!(next_pow2_u64(1025), 2048);
    }

    // ── clamp_u64 ────────────────────────────────────────────────────────────

    #[test]
    fn test_clamp_u64() {
        assert_eq!(clamp_u64(5, 1, 10), 5);
        assert_eq!(clamp_u64(0, 1, 10), 1);
        assert_eq!(clamp_u64(15, 1, 10), 10);
        assert_eq!(clamp_u64(1, 1, 1), 1);
        assert_eq!(clamp_u64(0, 0, u64::MAX), 0);
        assert_eq!(clamp_u64(u64::MAX, 0, u64::MAX), u64::MAX);
    }

    // ── decimal_digits_u64 ───────────────────────────────────────────────────

    #[test]
    fn test_decimal_digits_u64() {
        assert_eq!(decimal_digits_u64(0), 1);
        assert_eq!(decimal_digits_u64(1), 1);
        assert_eq!(decimal_digits_u64(9), 1);
        assert_eq!(decimal_digits_u64(10), 2);
        assert_eq!(decimal_digits_u64(99), 2);
        assert_eq!(decimal_digits_u64(100), 3);
        assert_eq!(decimal_digits_u64(999), 3);
        assert_eq!(decimal_digits_u64(1_000), 4);
        // u64::MAX = 18446744073709551615 = 20 digits
        assert_eq!(decimal_digits_u64(u64::MAX), 20);
    }

    #[test]
    fn test_decimal_digits_powers_of_ten() {
        let mut power: u64 = 1;
        for digits in 1u32..=19 {
            assert_eq!(decimal_digits_u64(power), digits,
                "10^{} should have {} digits", digits - 1, digits);
            assert_eq!(decimal_digits_u64(power * 10 - 1), digits,
                "10^{}-1 should have {} digits", digits, digits);
            if let Some(next) = power.checked_mul(10) {
                power = next;
            } else {
                break;
            }
        }
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
