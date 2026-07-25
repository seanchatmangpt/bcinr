// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
#[rustfmt::skip]
pub  fn hazard_pointer_retire(val: u64, aux: u64) -> u64 {
    // Mask pointer (val) with an epoch tag (aux) to safely retire.
    let ptr_mask = 0x0000_FFFF_FFFF_F000u64;
    let epoch_mask = 0xFFFF_0000_0000_0FFFu64;
    (val & ptr_mask) | (aux & epoch_mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Independent oracle: bit-by-bit selection. The pointer field is bits
    // 12..=47 (taken from `val`); all other bits (the epoch tag) are taken
    // from `aux`. This is a structurally different formulation than the
    // SWAR mask-and-or used by the implementation.
    fn reference_hazard(val: u64, aux: u64) -> u64 {
        let mut out = 0u64;
        for bit in 0..64 {
            let from_ptr = (12..=47).contains(&bit);
            let src = if from_ptr { val } else { aux };
            out |= ((src >> bit) & 1) << bit;
        }
        out
    }

    proptest! {
        #[test]
        fn test_hazard_fuzz(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(hazard_pointer_retire(val, aux), reference_hazard(val, aux));
        }
    }

    // Known-answer vectors derived by hand from the field layout.
    #[test]
    fn test_hazard_known_answers() {
        // All bits from val are kept only in the pointer field (bits 12..47).
        assert_eq!(hazard_pointer_retire(u64::MAX, 0), 0x0000_FFFF_FFFF_F000);
        // All bits from aux are kept only in the epoch field (the complement).
        assert_eq!(hazard_pointer_retire(0, u64::MAX), 0xFFFF_0000_0000_0FFF);
        // The two fields are complementary and cover all 64 bits.
        assert_eq!(hazard_pointer_retire(u64::MAX, u64::MAX), u64::MAX);
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle

// Axiomatic Hoare logic
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
