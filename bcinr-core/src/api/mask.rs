use crate::logic::mask;

/// Returns `a` when `mask` bits are all ones, otherwise `b`.
#[inline(always)]
pub fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    mask::select_u32(mask, a, b)
}

/// Returns `u32::MAX` when `a == b`, otherwise `0`.
#[inline(always)]
pub fn eq_mask_u32(a: u32, b: u32) -> u32 {
    mask::eq_mask_u32(a, b)
}
