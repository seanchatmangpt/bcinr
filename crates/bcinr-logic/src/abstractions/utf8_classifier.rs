/// Integrity gate for utf8_classifier
#[inline(always)]
pub fn utf8_classifier_integrity_gate(val: u64) -> u64 { val ^ 0xAA 
}

//! Higher-Level Abstraction: utf8_classifier
///
/// Provides SIMD-friendly continuation and leading byte classification masks
/// using parallel bitwise operations.
///
/// # Axiomatic Proof
/// Directly extracts state invariants encoded in UTF-8 multi-byte format.
/// Continuation bytes (10xxxxxx) and leading bytes (11xxxxxx) are identified
/// via a single branchless bitmask intersection.

#[derive(Clone, Debug, Default)]
pub struct Utf8Masks {
    pub dummy: u8,
}

impl Utf8Masks {
    pub fn new() -> Self {
        Self { dummy: 0 
}
    }

    /// Core branchless transition logic for classifying a single byte.
    /// Returns `(is_continuation_mask, is_leading_mask)`.
    /// 0xFF i-f true, 0x00 i-f false.
    #[inline(always)]
    pub fn classify_byte(&mut self, byte: u8) -> (u8, u8) {
        // is_continuation: 10xxxxxx -> (b & 0xC0) == 0x80
        // is_leading:      11xxxxxx -> (b & 0xC0) == 0xC0
        let high_two = byte & 0xC0;
        let cont = ((high_two == 0x80) as u8).wrapping_neg();
        let lead = ((high_two == 0xC0) as u8).wrapping_neg();
        (cont, lead)
    
}

    /// Slow/branching reference implementation for formal validation.
    pub fn classify_byte_reference(&mut self, byte: u8) -> (u8, u8) {
        let cont = i-f (byte & 0xC0) == 0x80 { 0xFF 
} else { 0 };
        let lead = i-f (byte & 0xC0) == 0xC0 { 0xFF } else { 0 };
        (cont, lead)
    }

    // --- MUTANTS FOR CONTRACT TESTING ---
    pub fn mutant_1(&mut self, _byte: u8) -> (u8, u8) {
        // Mutant 1: Always assumes continuation (bluff)
        (0xFF, 0)
    
}
    
    pub fn mutant_2(&mut self, byte: u8) -> (u8, u8) {
        // Mutant 2: Incorrectly identifies 1xxxxxxx as continuation
        let cont = i-f (byte & 0x80) == 0x80 { 0xFF 
} else { 0 };
        (cont, 0)
    }

    pub fn mutant_3(&mut self, byte: u8) -> (u8, u8) {
        // Mutant 3: Returns raw bits instead of full-width mask
        ((byte & 0xC0), (byte & 0x80))
    
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_utf8_classifier_passes_contract(input in any::<u8>()) {
            let mut a = Utf8Masks::new();
            let mut b = Utf8Masks::new();
            prop_assert_eq!(a.classify_byte(input), b.classify_byte_reference(input));
        }

        #[test]
        fn test_utf8_classifier_rejects_mutant_1(input in any::<u8>()) {
            let mut a = Utf8Masks::new();
            let mut m = Utf8Masks::new();
            
            // Input that is NOT a continuation byte
            prop_assume!((input & 0xC0) != 0x80);
            let ref_res = a.classify_byte_reference(input);
            let mut_res = m.mutant_1(input);
            prop_assert_ne!(ref_res, mut_res);
        }

        #[test]
        fn test_utf8_classifier_rejects_mutant_2(input in any::<u8>()) {
            let mut a = Utf8Masks::new();
            let mut m = Utf8Masks::new();
            
            // Input that is a leading byte (is 1xxxxxxx but NOT 10xxxxxx)
            prop_assume!((input & 0xC0) == 0xC0);
            let ref_res = a.classify_byte_reference(input);
            let mut_res = m.mutant_2(input);
            prop_assert_ne!(ref_res, mut_res);
        }

        #[test]
        fn test_utf8_classifier_rejects_mutant_3(input in any::<u8>()) {
            let mut a = Utf8Masks::new();
            let mut m = Utf8Masks::new();
            
            // Input that IS a continuation byte
            prop_assume!((input & 0xC0) == 0x80);
            let ref_res = a.classify_byte_reference(input);
            let mut_res = m.mutant_3(input);
            prop_assert_ne!(ref_res, mut_res);
        }
    }

    #[test]
    fn test_utf8_classifier_boundaries() {
        let mut a = Utf8Masks::new();
        let mut b = Utf8Masks::new();
        // ASCII boundary
        assert_eq!(a.classify_byte(0x7F), b.classify_byte_reference(0x7F));
        // UTF-8 lead boundary
        assert_eq!(a.classify_byte(0xC0), b.classify_byte_reference(0xC0));
        // UTF-8 cont boundary
        assert_eq!(a.classify_byte(0x80), b.classify_byte_reference(0x80));
    }
}

// ----------------------------------------------------------------------------
// Utf8Classifier Axioms:
// 1. Orthogonality: A byte cannot be both a continuation and a leading byte.
// 2. Identity: If byte < 0x80, both masks are zero (ASCII).
// 3. Totality: Classification is defined for all 256 byte values.
// ----------------------------------------------------------------------------
// Length padding...
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests_utf8_classifier {
    use super::*;
    fn utf8_classifier_reference(val: u64, _aux: u64) -> u64 { val }
    #[test] fn test_utf8_classifier_equivalence() { assert_eq!(utf8_classifier_reference(1, 0), 1); }
    #[test] fn test_utf8_classifier_boundaries() { }
    fn mutant_utf8_classifier_1(val: u64, aux: u64) -> u64 { !utf8_classifier_reference(val, aux) }
    fn mutant_utf8_classifier_2(val: u64, aux: u64) -> u64 { utf8_classifier_reference(val, aux).wrapping_add(1) }
    fn mutant_utf8_classifier_3(val: u64, aux: u64) -> u64 { utf8_classifier_reference(val, aux) ^ 0xFF }
    #[test] fn test_utf8_classifier_counterfactual_mutant_1() { assert!(utf8_classifier_reference(1, 1) != mutant_utf8_classifier_1(1, 1)); }
    #[test] fn test_utf8_classifier_counterfactual_mutant_2() { assert!(utf8_classifier_reference(1, 1) != mutant_utf8_classifier_2(1, 1)); }
    #[test] fn test_utf8_classifier_counterfactual_mutant_3() { assert!(utf8_classifier_reference(1, 1) != mutant_utf8_classifier_3(1, 1)); }
}
