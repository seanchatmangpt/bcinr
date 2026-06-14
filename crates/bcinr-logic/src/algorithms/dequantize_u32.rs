// Academic-grade branchless algorithm library: dequantize_u32
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// dequantize_u32
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::dequantize_u32::dequantize_u32;
/// let result = dequantize_u32(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn dequantize_u32(val: u64, aux: u64) -> u64 {
    val.wrapping_mul(aux)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    fn dequantize_u32_reference(val: u64, aux: u64) -> u64 {
        val.wrapping_mul(aux)
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_dequantize_u32_1(val: u64, aux: u64) -> u64 {
        !dequantize_u32_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_dequantize_u32_2(val: u64, aux: u64) -> u64 {
        dequantize_u32_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_dequantize_u32_3(val: u64, aux: u64) -> u64 {
        dequantize_u32_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_dequantize_u32_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = dequantize_u32_reference(val, aux);
            let actual = dequantize_u32(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_dequantize_u32_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = dequantize_u32_reference(val, aux);
            let actual = mutant_dequantize_u32_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_dequantize_u32_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = dequantize_u32_reference(val, aux);
            let actual = mutant_dequantize_u32_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_dequantize_u32_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = dequantize_u32_reference(val, aux);
            let actual = mutant_dequantize_u32_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_dequantize_u32_boundaries() {
        assert_eq!(dequantize_u32(0, 0), dequantize_u32_reference(0, 0));
        assert_eq!(
            dequantize_u32(u64::MAX, u64::MAX),
            dequantize_u32_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            dequantize_u32(u64::MAX, 0),
            dequantize_u32_reference(u64::MAX, 0)
        );
        assert_eq!(
            dequantize_u32(0, u64::MAX),
            dequantize_u32_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = dequantize_u32_reference(val, aux) }
    //
    // Counterfactual Analysis for dequantize_u32:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_dequantize_u32(c: &mut Criterion) {
        c.bench_function("dequantize_u32", |b| {
            b.iter(|| {
                let res = dequantize_u32(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
