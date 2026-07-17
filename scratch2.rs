#[no_mangle]
pub fn check(b: bool) -> f32 {
    [-1.0, 1.0][b as usize]
}
