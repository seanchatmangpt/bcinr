// SAFETY_LEVEL: no unsafe code permitted in algorithm modules
#[no_mangle]
pub fn jaro_winkler_branchless(val: u64, aux: u64) -> u64 {
    let v = val.to_le_bytes();
    let a = aux.to_le_bytes();
    let mut matches = 0u64;
    
    let mut i = 0;
    while i < 8 {
        let mut matched = 0u64;
        let mut j = 0;
        while j < 8 {
            let dist = if i > j { i - j } else { j - i };
            let in_window = (dist <= 3) as u64;
            let eq = (v[i] == a[j]) as u64;
            matched |= in_window & eq;
            j += 1;
        }
        matches += matched;
        i += 1;
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reference_jaro(val: u64, aux: u64) -> u64 {
        let v = val.to_le_bytes();
        let a = aux.to_le_bytes();
        let mut matches = 0u64;
        for i in 0..8 {
            let mut matched = false;
            for j in 0..8 {
                let dist = if i > j { i - j } else { j - i };
                if dist <= 3 && v[i] == a[j] {
                    matched = true;
                }
            }
            if matched { matches += 1; }
        }
        matches
    }

    proptest! {
        #[test]
        fn test_jaro_fuzz(val in any::<u64>(), aux in any::<u64>()) {
            prop_assert_eq!(jaro_winkler_branchless(val, aux), reference_jaro(val, aux));
        }
    }
}
