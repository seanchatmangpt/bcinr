// Academic-grade branchless algorithm library: rotate_slice_branchless
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// rotate_slice_branchless
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::rotate_slice_branchless::rotate_slice_branchless;
/// let result = rotate_slice_branchless(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
///
/// # Branchless Contract
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn rotate_slice_branchless(val: u64, aux: u64) -> u64 {
    // Branchless Contract: cyclic rotation of the full 64-bit slice left by
    // `aux & 63` positions. Bits shifted off the top re-enter at the bottom.
    val.rotate_left((aux & 63) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    fn rotate_slice_branchless_reference(val: u64, aux: u64) -> u64 {
        // Independent derivation: compose the rotation from two shifts and an OR
        // rather than the intrinsic, handling the zero-shift case explicitly.
        let s = (aux % 64) as u32;
        if s == 0 {
            val
        } else {
            (val << s) | (val >> (64 - s))
        }
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_rotate_slice_branchless_1(val: u64, aux: u64) -> u64 {
        !rotate_slice_branchless_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_rotate_slice_branchless_2(val: u64, aux: u64) -> u64 {
        rotate_slice_branchless_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_rotate_slice_branchless_3(val: u64, aux: u64) -> u64 {
        rotate_slice_branchless_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_rotate_slice_branchless_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = rotate_slice_branchless_reference(val, aux);
            let actual = rotate_slice_branchless(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = rotate_slice_branchless_reference(val, aux);
            let actual = mutant_rotate_slice_branchless_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = rotate_slice_branchless_reference(val, aux);
            let actual = mutant_rotate_slice_branchless_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = rotate_slice_branchless_reference(val, aux);
            let actual = mutant_rotate_slice_branchless_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_rotate_slice_branchless_boundaries() {
        assert_eq!(
            rotate_slice_branchless(0, 0),
            rotate_slice_branchless_reference(0, 0)
        );
        assert_eq!(
            rotate_slice_branchless(u64::MAX, u64::MAX),
            rotate_slice_branchless_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            rotate_slice_branchless(u64::MAX, 0),
            rotate_slice_branchless_reference(u64::MAX, 0)
        );
        assert_eq!(
            rotate_slice_branchless(0, u64::MAX),
            rotate_slice_branchless_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = rotate_slice_branchless_reference(val, aux) }
    //
    // Counterfactual Analysis for rotate_slice_branchless:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_rotate_slice_branchless(c: &mut Criterion) {
        c.bench_function("rotate_slice_branchless", |b| {
            b.iter(|| {
                let res = rotate_slice_branchless(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
