#![forbid(unsafe_code)]
//! Mask calculus for branchless selection and arithmetic.

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

/// Returns an all-ones mask if `a == b`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn eq_mask_u32(a: u32, b: u32) -> u32 {
    let x = a ^ b;
    // (x | -x) has the high bit set if x != 0.
    // We want all bits set if x == 0.
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    non_zero_msb.wrapping_sub(1)
}

/// Returns an all-ones mask if `x == 0`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn is_zero_mask_u32(x: u32) -> u32 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    non_zero_msb.wrapping_sub(1)
}

/// Returns an all-ones mask if `x != 0`, otherwise all-zeros.
#[inline(always)]
#[must_use]
pub fn nonzero_mask_u32(x: u32) -> u32 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    0u32.wrapping_sub(non_zero_msb)
}

/// Returns an all-ones mask if `a < b`, otherwise all-zeros.
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
    fn test_lt_mask_less_than() {
        assert_eq!(lt_mask_u32(0, 1), 0xFFFF_FFFF);
        assert_eq!(lt_mask_u32(3, 5), 0xFFFF_FFFF);
        assert_eq!(lt_mask_u32(0, u32::MAX), 0xFFFF_FFFF);
    }

    #[test]
    fn test_lt_mask_greater_than() {
        assert_eq!(lt_mask_u32(1, 0), 0);
        assert_eq!(lt_mask_u32(5, 3), 0);
        assert_eq!(lt_mask_u32(u32::MAX, 0), 0);
    }

    #[test]
    fn test_lt_mask_equal() {
        assert_eq!(lt_mask_u32(0, 0), 0);
        assert_eq!(lt_mask_u32(7, 7), 0);
        assert_eq!(lt_mask_u32(u32::MAX, u32::MAX), 0);
    }

    #[test]
    fn test_min_u32() {
        assert_eq!(min_u32(5, 3), 3);
        assert_eq!(min_u32(3, 5), 3);
        assert_eq!(min_u32(7, 7), 7);
        assert_eq!(min_u32(0, u32::MAX), 0);
        assert_eq!(min_u32(u32::MAX, 0), 0);
    }

    #[test]
    fn test_max_u32() {
        assert_eq!(max_u32(5, 3), 5);
        assert_eq!(max_u32(3, 5), 5);
        assert_eq!(max_u32(7, 7), 7);
        assert_eq!(max_u32(0, u32::MAX), u32::MAX);
        assert_eq!(max_u32(u32::MAX, 0), u32::MAX);
    }

    #[test]
    fn test_select_u32() {
        assert_eq!(select_u32(0xFFFF_FFFF, 10, 20), 10);
        assert_eq!(select_u32(0, 10, 20), 20);
    }

    #[test]
    fn test_eq_mask_u32() {
        assert_eq!(eq_mask_u32(5, 5), 0xFFFF_FFFF);
        assert_eq!(eq_mask_u32(5, 6), 0);
        assert_eq!(eq_mask_u32(0, 0), 0xFFFF_FFFF);
    }

    #[test]
    fn test_is_zero_mask_u32() {
        assert_eq!(is_zero_mask_u32(0), 0xFFFF_FFFF);
        assert_eq!(is_zero_mask_u32(1), 0);
        assert_eq!(is_zero_mask_u32(u32::MAX), 0);
    }

    #[test]
    fn test_nonzero_mask_u32() {
        assert_eq!(nonzero_mask_u32(0), 0);
        assert_eq!(nonzero_mask_u32(1), 0xFFFF_FFFF);
        assert_eq!(nonzero_mask_u32(u32::MAX), 0xFFFF_FFFF);
    }

    #[test]
    fn test_abs_i32() {
        assert_eq!(abs_i32(5), 5);
        assert_eq!(abs_i32(-5), 5);
        assert_eq!(abs_i32(0), 0);
        assert_eq!(abs_i32(i32::MIN + 1), i32::MAX);
    }
}
