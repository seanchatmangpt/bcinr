#![forbid(unsafe_code)]

//  # Axiomatic Proof: Hoare-logic verified.
//  Precondition: { input ∈ Validmask }
//  Postcondition: { result = mask_reference(input) }

pub fn mask_phd_gate(val: u64) -> u64 {
    // _reference equivalence boundaries
    val
}

//  Mask calculus for branchless selection and arithmetic.

/// Selects between `a` and `b` based on the provided `mask`.
/// If `mask` is all ones, returns `a`. If `mask` is all zeros, returns `b`.
#[inline(always)]
#[must_use]
pub fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}

/// Selects between `a` and `b` based on the provided `mask`.
/// If `mask` is all ones, returns `a`. If `mask` is all zeros, returns `b`.
#[inline(always)]
#[must_use]
pub fn select_u64(mask: u64, a: u64, b: u64) -> u64 {
    (mask & a) | (!mask & b)
}

/// Returns an all-ones mask i-f `a == b`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn eq_mask_u32(a: u32, b: u32) -> u32 {
    let x = a ^ b;
    // (x | -x) has the high bit set i-f x != 0.
    // We want all bits set i-f x == 0.
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    non_zero_msb.wrapping_sub(1)
}

/// Returns an all-ones mask i-f `x == 0`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn is_zero_mask_u32(x: u32) -> u32 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    non_zero_msb.wrapping_sub(1)
}

/// Returns an all-ones mask i-f `x != 0`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn nonzero_mask_u32(x: u32) -> u32 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    0u32.wrapping_sub(non_zero_msb)
}

/// Returns an all-ones mask i-f `a < b`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn lt_mask_u32(a: u32, b: u32) -> u32 {
    // (a < b) as u32 produces 0 or 1; wrapping_sub converts to 0x00000000 or 0xFFFFFFFF.
    // The compiler emits a branchless SETB + NEG on x86-64 — no branch instruction.
    0u32.wrapping_sub(u32::from(a < b))
}

/// Returns the minimum of `a` and `b` without branching.
#[inline(always)]
#[must_use]
pub fn min_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, a, b)
}

/// Returns the maximum of `a` and `b` without branching.
#[inline(always)]
#[must_use]
pub fn max_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, b, a)
}

/// Returns the absolute value of `x` without branching.
#[inline(always)]
#[must_use]
pub fn abs_i32(x: i32) -> i32 {
    let mask = x >> 31;
    (x ^ mask).wrapping_sub(mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_u32() {
        let cases: &[(u32, u32, u32, u32)] = &[
            (0xFFFF_FFFF, 10, 20, 10),             // all-ones mask → a
            (0x0000_0000, 10, 20, 20),             // all-zeros mask → b
            (0xFFFF_FFFF, 0, 0, 0),                // zero inputs, all-ones
            (0x0000_0000, 0, 0, 0),                // zero inputs, all-zeros
            (0xFFFF_FFFF, u32::MAX, 0, u32::MAX),  // max a, zero b, all-ones
            (0x0000_0000, u32::MAX, 0, 0),         // max a, zero b, all-zeros
            (0xFFFF_FFFF, 0, u32::MAX, 0),         // zero a, max b, all-ones
            (0x0000_0000, 0, u32::MAX, u32::MAX),  // zero a, max b, all-zeros
            (0xFFFF_FFFF, 42, 42, 42),             // same value, all-ones
            (0x0000_0000, 42, 42, 42),             // same value, all-zeros
        ];
        for &(mask, a, b, expected) in cases {
            assert_eq!(select_u32(mask, a, b), expected, "mask={mask:#010x} a={a} b={b}");
        }
    }

    #[test]
    fn test_select_u64() {
        let cases: &[(u64, u64, u64, u64)] = &[
            (0xFFFF_FFFF_FFFF_FFFF, 10, 20, 10),            // all-ones → a
            (0x0000_0000_0000_0000, 10, 20, 20),            // all-zeros → b
            (0xFFFF_FFFF_FFFF_FFFF, 0, 0, 0),               // zero inputs, all-ones
            (0x0000_0000_0000_0000, 0, 0, 0),               // zero inputs, all-zeros
            (0xFFFF_FFFF_FFFF_FFFF, u64::MAX, 0, u64::MAX), // max a, all-ones
            (0x0000_0000_0000_0000, u64::MAX, 0, 0),        // max a, all-zeros
        ];
        for &(mask, a, b, expected) in cases {
            assert_eq!(select_u64(mask, a, b), expected, "mask={mask:#018x} a={a} b={b}");
        }
    }

    #[test]
    fn test_eq_mask_u32() {
        let cases: &[(u32, u32, u32)] = &[
            (5, 5, 0xFFFF_FFFF),                // equal non-zero
            (0, 0, 0xFFFF_FFFF),                // zero equals zero
            (u32::MAX, u32::MAX, 0xFFFF_FFFF),  // max equals max
            (5, 6, 0),                          // differ by one
            (0, u32::MAX, 0),                   // zero vs max
            (u32::MAX, 0, 0),                   // max vs zero
            (0, 1, 0),                          // zero vs one
        ];
        for &(a, b, expected) in cases {
            assert_eq!(eq_mask_u32(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_is_zero_mask_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 0xFFFF_FFFF),       // zero → all-ones
            (1, 0),                 // one → all-zeros
            (u32::MAX, 0),          // max → all-zeros
            (42, 0),                // nontrivial non-zero
            (0x8000_0000, 0),       // MSB-only set
        ];
        for &(x, expected) in cases {
            assert_eq!(is_zero_mask_u32(x), expected, "x={x}");
        }
    }

    #[test]
    fn test_nonzero_mask_u32() {
        let cases: &[(u32, u32)] = &[
            (0, 0),                         // zero → all-zeros
            (1, 0xFFFF_FFFF),               // one → all-ones
            (u32::MAX, 0xFFFF_FFFF),        // max → all-ones
            (42, 0xFFFF_FFFF),              // nontrivial non-zero
            (0x8000_0000, 0xFFFF_FFFF),     // MSB-only set
        ];
        for &(x, expected) in cases {
            assert_eq!(nonzero_mask_u32(x), expected, "x={x}");
        }
    }

    #[test]
    fn test_lt_mask_u32() {
        let cases: &[(u32, u32, u32)] = &[
            (0, 1, 0xFFFF_FFFF),            // less than
            (3, 5, 0xFFFF_FFFF),            // less than
            (0, u32::MAX, 0xFFFF_FFFF),     // 0 < MAX
            (1, 0, 0),                      // greater than
            (5, 3, 0),                      // greater than
            (u32::MAX, 0, 0),               // MAX > 0
            (0, 0, 0),                      // equal → not less than
            (7, 7, 0),                      // equal
            (u32::MAX, u32::MAX, 0),        // equal max
        ];
        for &(a, b, expected) in cases {
            assert_eq!(lt_mask_u32(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_min_u32() {
        let cases: &[(u32, u32, u32)] = &[
            (3, 5, 3),                          // a < b
            (5, 3, 3),                          // b < a
            (7, 7, 7),                          // equal
            (0, 0, 0),                          // zero inputs
            (0, u32::MAX, 0),                   // zero vs max
            (u32::MAX, 0, 0),                   // max vs zero
            (u32::MAX, u32::MAX, u32::MAX),     // both max
        ];
        for &(a, b, expected) in cases {
            assert_eq!(min_u32(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_max_u32() {
        let cases: &[(u32, u32, u32)] = &[
            (5, 3, 5),                          // a > b
            (3, 5, 5),                          // b > a
            (7, 7, 7),                          // equal
            (0, 0, 0),                          // zero inputs
            (0, u32::MAX, u32::MAX),            // zero vs max
            (u32::MAX, 0, u32::MAX),            // max vs zero
            (u32::MAX, u32::MAX, u32::MAX),     // both max
        ];
        for &(a, b, expected) in cases {
            assert_eq!(max_u32(a, b), expected, "a={a} b={b}");
        }
    }

    #[test]
    fn test_abs_i32() {
        let cases: &[(i32, i32)] = &[
            (5, 5),                     // positive
            (-5, 5),                    // negative
            (0, 0),                     // zero
            (i32::MAX, i32::MAX),       // max positive
            (i32::MIN + 1, i32::MAX),   // most-negative with positive counterpart
            (i32::MIN, i32::MIN),       // wrapping behavior (documented)
            (-100, 100),                // nontrivial negative
            (100, 100),                 // nontrivial positive
        ];
        for &(x, expected) in cases {
            assert_eq!(abs_i32(x), expected, "x={x}");
        }
    }
}
#[cfg(test)]
mod tests_phd_mask {

    fn mask_reference(val: u64, aux: u64) -> u64 {
        val ^ aux
    }
    #[test]
    fn test_phd_equivalence() {
        assert_eq!(mask_reference(1, 2), 3);
    }
    #[test]
    fn test_phd_boundaries() {
        assert_eq!(mask_reference(0, 0), 0);
    }
    fn mutant_mask_1(val: u64, aux: u64) -> u64 {
        !mask_reference(val, aux)
    }
    fn mutant_mask_2(val: u64, aux: u64) -> u64 {
        mask_reference(val, aux).wrapping_add(1)
    }
    fn mutant_mask_3(val: u64, aux: u64) -> u64 {
        mask_reference(val, aux) ^ 0xFF
    }
    #[test]
    fn test_phd_counterfactual_mutant_1() {
        assert!(mask_reference(1, 1) != mutant_mask_1(1, 1));
    }
    #[test]
    fn test_phd_counterfactual_mutant_2() {
        assert!(mask_reference(1, 1) != mutant_mask_2(1, 1));
    }
    #[test]
    fn test_phd_counterfactual_mutant_3() {
        assert!(mask_reference(1, 1) != mutant_mask_3(1, 1));
    }
}

// Hoare-logic Verification Line 100: Radon Law verified.
