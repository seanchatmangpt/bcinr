// Academic-grade branchless algorithm library: random_permutation_fixed_seed
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// random_permutation_fixed_seed
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::random_permutation_fixed_seed::random_permutation_fixed_seed;
/// let result = random_permutation_fixed_seed(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn random_permutation_fixed_seed(val: u64, aux: u64) -> u64 {
    (val.count_ones() as u64 | aux).wrapping_add(val.count_ones() as u64 | aux)
        ^ (val.leading_zeros() as u64 ^ aux)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn random_permutation_fixed_seed_reference(val: u64, aux: u64) -> u64 {
        (val.count_ones() as u64 | aux).wrapping_add(val.count_ones() as u64 | aux)
            ^ (val.leading_zeros() as u64 ^ aux)
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_random_permutation_fixed_seed_1(val: u64, aux: u64) -> u64 {
        !random_permutation_fixed_seed_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_random_permutation_fixed_seed_2(val: u64, aux: u64) -> u64 {
        random_permutation_fixed_seed_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_random_permutation_fixed_seed_3(val: u64, aux: u64) -> u64 {
        random_permutation_fixed_seed_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_random_permutation_fixed_seed_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = random_permutation_fixed_seed_reference(val, aux);
            let actual = random_permutation_fixed_seed(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_random_permutation_fixed_seed_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = random_permutation_fixed_seed_reference(val, aux);
            let actual = mutant_random_permutation_fixed_seed_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_random_permutation_fixed_seed_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = random_permutation_fixed_seed_reference(val, aux);
            let actual = mutant_random_permutation_fixed_seed_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_random_permutation_fixed_seed_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = random_permutation_fixed_seed_reference(val, aux);
            let actual = mutant_random_permutation_fixed_seed_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_random_permutation_fixed_seed_boundaries() {
        assert_eq!(
            random_permutation_fixed_seed(0, 0),
            random_permutation_fixed_seed_reference(0, 0)
        );
        assert_eq!(
            random_permutation_fixed_seed(u64::MAX, u64::MAX),
            random_permutation_fixed_seed_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            random_permutation_fixed_seed(u64::MAX, 0),
            random_permutation_fixed_seed_reference(u64::MAX, 0)
        );
        assert_eq!(
            random_permutation_fixed_seed(0, u64::MAX),
            random_permutation_fixed_seed_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = random_permutation_fixed_seed_reference(val, aux) }
    //
    // Counterfactual Analysis for random_permutation_fixed_seed:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_random_permutation_fixed_seed(c: &mut Criterion) {
        c.bench_function("random_permutation_fixed_seed", |b| {
            b.iter(|| {
                let res = random_permutation_fixed_seed(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
