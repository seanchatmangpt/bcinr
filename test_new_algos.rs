#[cfg(test)]
mod test {
    // Test mul_sat_u64
    fn test_mul_sat_u64() {
        // Compute full 128-bit product via u64 -> u128 conversions
        let a = 10u64;
        let b = 20u64;
        let product_128 = (a as u128).wrapping_mul(b as u128);
        
        let lower = product_128 as u64;
        let upper = (product_128 >> 64) as u64;
        
        let mask = ((upper as i64 | -(upper as i64)) >> 63) as u64;
        let result = (lower & !mask) | (u64::MAX & mask);
        
        assert_eq!(result, 200);
        println!("mul_sat_u64(10, 20) = {}", result);
    }

    #[test]
    fn run_test() {
        test_mul_sat_u64();
    }
}
