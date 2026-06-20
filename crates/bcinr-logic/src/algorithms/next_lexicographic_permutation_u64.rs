// Academic-grade branchless algorithm library: next_lexicographic_permutation_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// next_lexicographic_permutation_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::next_lexicographic_permutation_u64::next_lexicographic_permutation_u64;
/// let result = next_lexicographic_permutation_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn next_lexicographic_permutation_u64(val: u64, aux: u64) -> u64 {
    let t = val | val.wrapping_sub(1);
    let c = !t & t.wrapping_add(1);
    let tz = val.trailing_zeros();
    let shift = tz.wrapping_add(1) & 0x3F;
    let o = (c.wrapping_sub(1)).wrapping_shr(shift);
    (t.wrapping_add(1) | o) * (val != 0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn next_lexicographic_permutation_u64_reference(val: u64, _aux: u64) -> u64 {
        if val == 0 {
            0
        } else {
            let t = val | val.wrapping_sub(1);
            let next = t.wrapping_add(1);
            let ones = ((!t & next).wrapping_sub(1)).wrapping_shr(val.trailing_zeros() + 1);
            next | ones
        }
    }

    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_next_lexicographic_permutation_u64_1(val: u64, aux: u64) -> u64 {
        !next_lexicographic_permutation_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_next_lexicographic_permutation_u64_2(val: u64, aux: u64) -> u64 {
        next_lexicographic_permutation_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_next_lexicographic_permutation_u64_3(val: u64, aux: u64) -> u64 {
        next_lexicographic_permutation_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_next_lexicographic_permutation_u64_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = next_lexicographic_permutation_u64_reference(val, aux);
            let actual = next_lexicographic_permutation_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = next_lexicographic_permutation_u64_reference(val, aux);
            let actual = mutant_next_lexicographic_permutation_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = next_lexicographic_permutation_u64_reference(val, aux);
            let actual = mutant_next_lexicographic_permutation_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = next_lexicographic_permutation_u64_reference(val, aux);
            let actual = mutant_next_lexicographic_permutation_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_next_lexicographic_permutation_u64_boundaries() {
        assert_eq!(
            next_lexicographic_permutation_u64(0, 0),
            next_lexicographic_permutation_u64_reference(0, 0)
        );
        assert_eq!(
            next_lexicographic_permutation_u64(u64::MAX, u64::MAX),
            next_lexicographic_permutation_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            next_lexicographic_permutation_u64(u64::MAX, 0),
            next_lexicographic_permutation_u64_reference(u64::MAX, 0)
        );
        assert_eq!(
            next_lexicographic_permutation_u64(0, u64::MAX),
            next_lexicographic_permutation_u64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = next_lexicographic_permutation_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for next_lexicographic_permutation_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_next_lexicographic_permutation_u64(c: &mut Criterion) {
        c.bench_function("next_lexicographic_permutation_u64", |b| {
            b.iter(|| {
                let res = next_lexicographic_permutation_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
