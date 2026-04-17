use bcinr_logic::::mask;

/// Returns `a` when `mask` bits are all ones, otherwise `b`.
#[inline(always)]
pub fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    mask::select_u32(mask, a, b)
}

/// Returns `a` when `mask` bits are all ones, otherwise `b`.
#[inline(always)]
pub fn select_u64(mask: u64, a: u64, b: u64) -> u64 {
    mask::select_u64(mask, a, b)
}

/// Returns `u32::MAX` when `a == b`, otherwise `0`.
#[inline(always)]
pub fn eq_mask_u32(a: u32, b: u32) -> u32 {
    mask::eq_mask_u32(a, b)
}

/// Returns an all-ones mask if `x == 0`, otherwise all-zeros.
#[inline(always)]
pub fn is_zero_mask_u32(x: u32) -> u32 {
    mask::is_zero_mask_u32(x)
}

/// Returns an all-ones mask if `x != 0`, otherwise all-zeros.
#[inline(always)]
pub fn nonzero_mask_u32(x: u32) -> u32 {
    mask::nonzero_mask_u32(x)
}

/// Returns an all-ones mask if `a < b`, otherwise all-zeros.
#[inline(always)]
pub fn lt_mask_u32(a: u32, b: u32) -> u32 {
    mask::lt_mask_u32(a, b)
}

/// Returns the minimum of `a` and `b` without branching.
#[inline(always)]
pub fn min_u32(a: u32, b: u32) -> u32 {
    mask::min_u32(a, b)
}

/// Returns the maximum of `a` and `b` without branching.
#[inline(always)]
pub fn max_u32(a: u32, b: u32) -> u32 {
    mask::max_u32(a, b)
}

/// Returns the absolute value of `x` without branching.
#[inline(always)]
pub fn abs_i32(x: i32) -> i32 {
    mask::abs_i32(x)
}
