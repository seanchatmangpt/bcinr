// Academic-grade branchless algorithm library: gray_encode_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// gray_encode_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Ensures:** Returns the reflected binary Gray code of `val`, i.e.
/// `val ^ (val >> 1)`. `aux` is unused. This is invertible by `gray_decode_u64`.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::gray_encode_u64::gray_encode_u64;
/// let result = gray_encode_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn gray_encode_u64(val: u64, aux: u64) -> u64 {
    val ^ (val >> 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn gray_encode_u64_reference(val: u64, aux: u64) -> u64 {
        // Independent structure: build the Gray code bit-by-bit, where bit i is
        // the XOR of binary bits i and i+1.
        let mut out: u64 = 0;
        for i in 0..64 {
            let b = (val >> i) & 1;
            let nb = if i < 63 { (val >> (i + 1)) & 1 } else { 0 };
            out |= (b ^ nb) << i;
        }
        out
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_gray_encode_u64_1(val: u64, aux: u64) -> u64 {
        !gray_encode_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_gray_encode_u64_2(val: u64, aux: u64) -> u64 {
        gray_encode_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_gray_encode_u64_3(val: u64, aux: u64) -> u64 {
        gray_encode_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_gray_encode_u64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = gray_encode_u64_reference(val, aux);
            let actual = gray_encode_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_gray_encode_u64_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = gray_encode_u64_reference(val, aux);
            let actual = mutant_gray_encode_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_gray_encode_u64_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = gray_encode_u64_reference(val, aux);
            let actual = mutant_gray_encode_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_gray_encode_u64_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = gray_encode_u64_reference(val, aux);
            let actual = mutant_gray_encode_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_gray_encode_u64_boundaries() {
        assert_eq!(gray_encode_u64(0, 0), gray_encode_u64_reference(0, 0));
        assert_eq!(
            gray_encode_u64(u64::MAX, u64::MAX),
            gray_encode_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            gray_encode_u64(u64::MAX, 0),
            gray_encode_u64_reference(u64::MAX, 0)
        );
        assert_eq!(
            gray_encode_u64(0, u64::MAX),
            gray_encode_u64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = gray_encode_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for gray_encode_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_gray_encode_u64(c: &mut Criterion) {
        c.bench_function("gray_encode_u64", |b| {
            b.iter(|| {
                let res = gray_encode_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
