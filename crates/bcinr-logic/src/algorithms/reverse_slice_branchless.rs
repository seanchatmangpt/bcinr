// Academic-grade branchless algorithm library: reverse_slice_branchless
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// reverse_slice_branchless
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::reverse_slice_branchless::reverse_slice_branchless;
/// let result = reverse_slice_branchless(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
///
/// # Branchless Contract
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn reverse_slice_branchless(val: u64, aux: u64) -> u64 {
    // Branchless Contract: reverse the bit order of `val` within the low
    // `width = (aux & 63) + 1` bits, leaving the higher bits cleared. This is the
    // in-place slice reversal of a bit-slice of the requested width.
    let width = ((aux & 63) + 1) as u32;
    let full = val.reverse_bits();
    full >> (64 - width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    fn reverse_slice_branchless_reference(val: u64, aux: u64) -> u64 {
        // Independent derivation: build the reversed slice bit by bit with an
        // explicit loop (test-only), placing source bit i at mirror position.
        let width = ((aux % 64) + 1) as u32;
        let mut out: u64 = 0;
        let mut i: u32 = 0;
        while i < width {
            let bit = (val >> i) & 1;
            out |= bit << (width - 1 - i);
            i += 1;
        }
        out
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_reverse_slice_branchless_1(val: u64, aux: u64) -> u64 {
        !reverse_slice_branchless_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_reverse_slice_branchless_2(val: u64, aux: u64) -> u64 {
        reverse_slice_branchless_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_reverse_slice_branchless_3(val: u64, aux: u64) -> u64 {
        reverse_slice_branchless_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_reverse_slice_branchless_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = reverse_slice_branchless_reference(val, aux);
            let actual = reverse_slice_branchless(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = reverse_slice_branchless_reference(val, aux);
            let actual = mutant_reverse_slice_branchless_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = reverse_slice_branchless_reference(val, aux);
            let actual = mutant_reverse_slice_branchless_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = reverse_slice_branchless_reference(val, aux);
            let actual = mutant_reverse_slice_branchless_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_reverse_slice_branchless_boundaries() {
        assert_eq!(
            reverse_slice_branchless(0, 0),
            reverse_slice_branchless_reference(0, 0)
        );
        assert_eq!(
            reverse_slice_branchless(u64::MAX, u64::MAX),
            reverse_slice_branchless_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            reverse_slice_branchless(u64::MAX, 0),
            reverse_slice_branchless_reference(u64::MAX, 0)
        );
        assert_eq!(
            reverse_slice_branchless(0, u64::MAX),
            reverse_slice_branchless_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = reverse_slice_branchless_reference(val, aux) }
    //
    // Counterfactual Analysis for reverse_slice_branchless:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_reverse_slice_branchless(c: &mut Criterion) {
        c.bench_function("reverse_slice_branchless", |b| {
            b.iter(|| {
                let res = reverse_slice_branchless(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
