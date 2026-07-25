fn const_lt_u32(condition: u32, a: u32, b: u32) -> u32 {
    let cond = core::hint::black_box(condition);
    let cond_val = (cond | cond.wrapping_neg()) >> 31;
    let mask = 0u32.wrapping_sub(cond_val);
    (core::hint::black_box(a) & mask) | (core::hint::black_box(b) & !mask)
}
pub fn const_lt_u32_func(a: u32, b: u32) -> u32 {
    let diff = a.wrapping_sub(b);
    let a_msb = a >> 31;
    let b_msb = b >> 31;
    let diff_msb = diff >> 31;
    let eq_msb = a_msb ^ b_msb;
    let lt_msb = (eq_msb & a_msb) | ((eq_msb ^ 1) & diff_msb);
    lt_msb
}
fn main() {
    println!("lt(65536, 131072) = {}", const_lt_u32_func(65536, 131072));
}
