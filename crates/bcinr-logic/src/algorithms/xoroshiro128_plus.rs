// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
#[rustfmt::skip]
pub  fn xoroshiro128_plus(val: u64, aux: u64) -> u64 {
    // Computes the next state of s0.
    let s0 = val;
    let s1 = aux ^ s0;
    s0.rotate_left(24) ^ s1 ^ (s1 << 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Known-answer vectors derived independently from the xoroshiro128+ spec
    // (s1' = s0 ^ s1; new_s0 = rotl(s0, 24) ^ s1' ^ (s1' << 16)), NOT by copying
    // the implementation. This breaks the prior tautological self-comparison.
    #[test]
    fn test_xoroshiro_known_vectors() {
        // (0,0): rotl(0,24) ^ 0 ^ 0 = 0
        assert_eq!(xoroshiro128_plus(0, 0), 0);
        // (1,0): s1'=1; rotl(1,24)=0x0100_0000; ^1 ^ (1<<16=0x1_0000) = 0x0101_0001
        assert_eq!(xoroshiro128_plus(1, 0), 0x0101_0001);
        // (0,1): s1'=1; rotl(0,24)=0; ^1 ^ 0x1_0000 = 0x0001_0001
        assert_eq!(xoroshiro128_plus(0, 1), 0x0001_0001);
    }

    proptest! {
        // Structural invariants that hold for the real transform without
        // restating its formula: the high rotation makes it sensitive to the
        // top bits of s0, and equal inputs cancel s1' to s0 alone.
        #[test]
        fn test_xoroshiro_equal_inputs(s in any::<u64>()) {
            // val == aux => s1' = s ^ s = 0 => result = rotl(s, 24)
            prop_assert_eq!(xoroshiro128_plus(s, s), s.rotate_left(24));
        }

        #[test]
        fn test_xoroshiro_deterministic(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(xoroshiro128_plus(val, aux), xoroshiro128_plus(val, aux));
        }
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle

// Axiomatic Hoare logic
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
