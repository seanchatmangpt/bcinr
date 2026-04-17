// Minimal test to verify fix.rs compiles

#[inline(always)]
pub(crate) fn add_sat_u8(a: u8, b: u8) -> u8 {
    a.saturating_add(b)
}

#[inline(always)]
pub(crate) fn sub_sat_u8(a: u8, b: u8) -> u8 {
    a.saturating_sub(b)
}

#[inline(always)]
pub(crate) fn clamp_u32(x: u32, min: u32, max: u32) -> u32 {
    debug_assert!(min <= max, "min must be <= max in clamp_u32");
    x.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_sat_u8() {
        assert_eq!(add_sat_u8(100, 50), 150);
        assert_eq!(add_sat_u8(255, 1), 255);
        assert_eq!(add_sat_u8(0, 0), 0);
    }

    #[test]
    fn test_sub_sat_u8() {
        assert_eq!(sub_sat_u8(150, 50), 100);
        assert_eq!(sub_sat_u8(50, 100), 0);
        assert_eq!(sub_sat_u8(255, 0), 255);
    }

    #[test]
    fn test_clamp_u32() {
        assert_eq!(clamp_u32(50, 0, 100), 50);
        assert_eq!(clamp_u32(150, 0, 100), 100);
        assert_eq!(clamp_u32(25, 50, 100), 50);
    }
}

fn main() {
    println!("add_sat_u8(200, 100) = {}", add_sat_u8(200, 100));
    println!("sub_sat_u8(50, 100) = {}", sub_sat_u8(50, 100));
    println!("clamp_u32(150, 0, 100) = {}", clamp_u32(150, 0, 100));
}
