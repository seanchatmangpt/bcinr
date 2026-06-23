// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
pub fn xoroshiro128_plus(val: u64, aux: u64) -> u64 {
    // Computes the next state of s0.
    let s0 = val;
    let s1 = aux ^ s0;
    s0.rotate_left(24) ^ s1 ^ (s1 << 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reference_xoroshiro(val: u64, aux: u64) -> u64 {
        let s0 = val;
        let s1 = aux ^ s0;
        s0.rotate_left(24) ^ s1 ^ (s1 << 16)
    }

    proptest! {
        #[test]
        fn test_xoroshiro_fuzz(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(xoroshiro128_plus(val, aux), reference_xoroshiro(val, aux));
        }
    }
}
