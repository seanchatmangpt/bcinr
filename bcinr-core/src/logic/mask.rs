pub(crate) fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}

pub(crate) fn eq_mask_u32(a: u32, b: u32) -> u32 {
    let x = a ^ b;
    if x == 0 { !0 } else { 0 }
}
