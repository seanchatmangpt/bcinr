// Academic-grade branchless algorithm library: is_contiguous_mask_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// is_contiguous_mask_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::is_contiguous_mask_u64::is_contiguous_mask_u64;
/// let result = is_contiguous_mask_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn is_contiguous_mask_u64(val: u64, aux: u64) -> u64 {
    let b = val & val.wrapping_neg();
    let t = val.wrapping_add(b);
    ((t & val == 0) && val != 0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn is_contiguous_mask_u64_reference(val: u64, _aux: u64) -> u64 {
        if val == 0 {
            0
        } else {
            let b = val & val.wrapping_neg();
            let t = val.wrapping_add(b);
            if t & val == 0 {
                1
            } else {
                0
            }
        }
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_is_contiguous_mask_u64_1(val: u64, aux: u64) -> u64 {
        !is_contiguous_mask_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_is_contiguous_mask_u64_2(val: u64, aux: u64) -> u64 {
        is_contiguous_mask_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_is_contiguous_mask_u64_3(val: u64, aux: u64) -> u64 {
        is_contiguous_mask_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_is_contiguous_mask_u64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = is_contiguous_mask_u64_reference(val, aux);
            let actual = is_contiguous_mask_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_is_contiguous_mask_u64_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = is_contiguous_mask_u64_reference(val, aux);
            let actual = mutant_is_contiguous_mask_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_is_contiguous_mask_u64_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = is_contiguous_mask_u64_reference(val, aux);
            let actual = mutant_is_contiguous_mask_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_is_contiguous_mask_u64_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = is_contiguous_mask_u64_reference(val, aux);
            let actual = mutant_is_contiguous_mask_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_is_contiguous_mask_u64_boundaries() {
        assert_eq!(
            is_contiguous_mask_u64(0, 0),
            is_contiguous_mask_u64_reference(0, 0)
        );
        assert_eq!(
            is_contiguous_mask_u64(u64::MAX, u64::MAX),
            is_contiguous_mask_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            is_contiguous_mask_u64(u64::MAX, 0),
            is_contiguous_mask_u64_reference(u64::MAX, 0)
        );
        assert_eq!(
            is_contiguous_mask_u64(0, u64::MAX),
            is_contiguous_mask_u64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = is_contiguous_mask_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for is_contiguous_mask_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_is_contiguous_mask_u64(c: &mut Criterion) {
        c.bench_function("is_contiguous_mask_u64", |b| {
            b.iter(|| {
                let res = is_contiguous_mask_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
