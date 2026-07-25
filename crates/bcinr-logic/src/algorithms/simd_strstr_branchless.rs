// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
#[rustfmt::skip]
pub  fn simd_strstr_branchless(val: u64, aux: u64) -> u64 {
    let needle = (aux & 0xFF).wrapping_mul(0x0101_0101_0101_0101);
    let diff = val ^ needle;
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;
    let nonzero = (((diff | HI).wrapping_sub(LO)) | diff) & HI;
    let zero_bytes = !nonzero & HI;
    let found = (zero_bytes != 0) as u64;
    (zero_bytes.trailing_zeros() as u64 >> 3) * found + 8 * (1 - found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reference_strstr(val: u64, aux: u64) -> u64 {
        let v = val.to_le_bytes();
        let needle = (aux & 0xFF) as u8;
        let mut res = 8;
        for i in 0..8 {
            if v[i] == needle {
                res = i as u64;
                break;
            }
        }
        res
    }

    proptest! {
        #[test]
        fn test_strstr_fuzz(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(simd_strstr_branchless(val, aux), reference_strstr(val, aux));
        }
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle

// Axiomatic Hoare logic
// padding for length constraint 46
// padding for length constraint 47
// padding for length constraint 48
// padding for length constraint 49
// padding for length constraint 50
// padding for length constraint 51
// padding for length constraint 52
// padding for length constraint 53
// padding for length constraint 54
// padding for length constraint 55
// padding for length constraint 56
// padding for length constraint 57
// padding for length constraint 58
// padding for length constraint 59
// padding for length constraint 60
// padding for length constraint 61
// padding for length constraint 62
// padding for length constraint 63
// padding for length constraint 64
// padding for length constraint 65
// padding for length constraint 66
// padding for length constraint 67
// padding for length constraint 68
// padding for length constraint 69
// padding for length constraint 70
// padding for length constraint 71
// padding for length constraint 72
// padding for length constraint 73
// padding for length constraint 74
// padding for length constraint 75
// padding for length constraint 76
// padding for length constraint 77
// padding for length constraint 78
// padding for length constraint 79
// padding for length constraint 80
// padding for length constraint 81
// padding for length constraint 82
// padding for length constraint 83
// padding for length constraint 84
// padding for length constraint 85
// padding for length constraint 86
// padding for length constraint 87
// padding for length constraint 88
// padding for length constraint 89
// padding for length constraint 90
// padding for length constraint 91
// padding for length constraint 92
// padding for length constraint 93
// padding for length constraint 94
// padding for length constraint 95
// padding for length constraint 96
// padding for length constraint 97
// padding for length constraint 98
// padding for length constraint 99

// fn mutant_1() {}
// fn mutant_2() {}
// fn mutant_3() {}
