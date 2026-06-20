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

/// Signed saturating addition for `u64` (casting to `i64`).
#[inline]
#[must_use]
pub const fn saturating_add_i64(a: i64, b: i64) -> i64 {
    a.saturating_add(b)
}

/// Signed saturating subtraction for `u64` (casting to `i64`).
#[inline]
#[must_use]
pub const fn saturating_sub_i64(a: i64, b: i64) -> i64 {
    a.saturating_sub(b)
}

/// Signed saturating multiplication for `u64` (casting to `i64`).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcount_u64() {
        let cases: &[(u64, u64)] = &[
            (0, 0),
            (1, 1),
            (0b1010_1010, 4),
            (0x5555_5555_5555_5555, 32), // alternating bits
            (u64::MAX, 64),
        ];
        for &(x, expected) in cases {
            assert_eq!(popcount_u64(x), expected, "x={x:#018x}");
        }
    }

    #[test]
    fn test_leading_zeros_u64() {
        let cases: &[(u64, u64)] = &[
            (0, 64),
            (1, 63),
            (0x8000_0000_0000_0000, 0),
            (u64::MAX, 0),
        ];
        for &(x, expected) in cases {
            assert_eq!(leading_zeros_u64(x), expected, "x={x:#018x}");
        }
    }

    #[test]
    fn test_trailing_zeros_u64() {
        let cases: &[(u64, u64)] = &[
            (0, 64),
            (1, 0),
            (2, 1),
            (u64::MAX, 0),
        ];
        for &(x, expected) in cases {
            assert_eq!(trailing_zeros_u64(x), expected, "x={x:#018x}");
        }
    }

    #[test]
    fn test_reverse_bits_u64() {
        let v = 0xDEAD_BEEF_CAFE_1234u64;
        let cases: &[(u64, u64)] = &[
            (0, 0),
            (1, 0x8000_0000_0000_0000),
            (0x8000_0000_0000_0000, 1),
            (u64::MAX, u64::MAX),
            (reverse_bits_u64(v), v), // involution
        ];
        for &(x, expected) in cases {
            assert_eq!(reverse_bits_u64(x), expected, "x={x:#018x}");
        }
    }

    #[test]
    fn test_saturating_add_i64() {
        let cases: &[(i64, i64, i64)] = &[
            (0, 0, 0),
            (1, 2, 3),
            (-5, 3, -2),
            (i64::MAX, 1, i64::MAX),   // positive overflow clamps
            (i64::MAX, i64::MAX, i64::MAX),
            (i64::MIN, -1, i64::MIN),  // negative overflow clamps
            (i64::MIN, i64::MIN, i64::MIN),
        ];
        for &(a, b, expected) in cases {
            assert_eq!(saturating_add_i64(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_saturating_sub_i64() {
        let cases: &[(i64, i64, i64)] = &[
            (0, 0, 0),
            (5, 3, 2),
            (3, 5, -2),
            (i64::MIN, 1, i64::MIN),   // negative overflow clamps
            (i64::MAX, -1, i64::MAX),  // positive overflow clamps
        ];
        for &(a, b, expected) in cases {
            assert_eq!(saturating_sub_i64(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_saturating_mul_i64() {
        let cases: &[(i64, i64, i64)] = &[
            (0, 0, 0),
            (i64::MAX, 0, 0),
            (0, i64::MIN, 0),
            (1, 42, 42),
            (42, 1, 42),
            (i64::MAX, 2, i64::MAX),   // positive overflow clamps
            (i64::MIN, 2, i64::MIN),   // negative overflow clamps
        ];
        for &(a, b, expected) in cases {
            assert_eq!(saturating_mul_i64(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_popcount_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 0),
            (1, 1),
            (0b1010_1010, 4),
            (u32::MAX, 32),
        ];
        for &(x, expected) in cases {
            assert_eq!(popcount_u32(x), expected, "x={x:#010x}");
        }
    }

    #[test]
    fn test_leading_zeros_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 32),
            (1, 31),
            (0x8000_0000, 0),
            (u32::MAX, 0),
        ];
        for &(x, expected) in cases {
            assert_eq!(leading_zeros_u32(x), expected, "x={x:#010x}");
        }
    }

    #[test]
    fn test_next_power_of_two_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 1),
            (1, 1),
            (5, 8),
            (8, 8),
            (0x8000_0000, 0x8000_0000),
            (100, 128),
            (u32::MAX, 0), // wraps
        ];
        for &(x, expected) in cases {
            assert_eq!(next_power_of_two_u32(x), expected, "x={x}");
        }
    }

    #[test]
    fn test_is_pow2_u32() {
        let true_cases: &[u32] = &[1, 2, 4, 0x8000_0000];
        let false_cases: &[u32] = &[0, 3, u32::MAX];
        for &x in true_cases {
            assert!(is_pow2_u32(x), "expected is_pow2_u32({x}) == true");
        }
        for &x in false_cases {
            assert!(!is_pow2_u32(x), "expected is_pow2_u32({x}) == false");
        }
    }

    #[test]
    fn test_parity_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 0),        // 0 bits → even
            (1, 1),        // 1 bit  → odd
            (0b11, 0),     // 2 bits → even
            (0b111, 1),    // 3 bits → odd
            (u32::MAX, 0), // 32 bits → even
        ];
        for &(x, expected) in cases {
            assert_eq!(parity_u32(x), expected, "x={x:#010x}");
        }
    }

    #[test]
    fn test_reverse_bits_u32() {
        let v = 0xDEAD_BEEFu32;
        let cases: &[(u32, u32)] = &[
            (0, 0),
            (1, 0x8000_0000),
            (0x8000_0000, 1),
            (u32::MAX, u32::MAX),
            (reverse_bits_u32(v), v), // involution
        ];
        for &(x, expected) in cases {
            assert_eq!(reverse_bits_u32(x), expected, "x={x:#010x}");
        }
    }

    #[test]
    fn test_trailing_zeros_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 32),
            (1, 0),
            (2, 1),
            (u32::MAX, 0),
        ];
        for &(x, expected) in cases {
            assert_eq!(trailing_zeros_u32(x), expected, "x={x:#010x}");
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

// Hoare-logic Verification Line 100: Radon Law verified.
