// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
pub fn hazard_pointer_retire(val: u64, aux: u64) -> u64 {
    // Mask pointer (val) with an epoch tag (aux) to safely retire.
    let ptr_mask = 0x0000_FFFF_FFFF_F000u64;
    let epoch_mask = 0xFFFF_0000_0000_0FFFu64;
    (val & ptr_mask) | (aux & epoch_mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reference_hazard(val: u64, aux: u64) -> u64 {
        let ptr_mask = 0x0000_FFFF_FFFF_F000u64;
        let epoch_mask = 0xFFFF_0000_0000_0FFFu64;
        (val & ptr_mask) | (aux & epoch_mask)
    }

    proptest! {
        #[test]
        fn test_hazard_fuzz(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(hazard_pointer_retire(val, aux), reference_hazard(val, aux));
        }
    }
}
