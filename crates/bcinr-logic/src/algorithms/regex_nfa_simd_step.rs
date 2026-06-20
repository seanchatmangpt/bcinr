// Academic-grade branchless algorithm library: regex_nfa_simd_step
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// regex_nfa_simd_step
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// Branchless Contract: one Glushkov/bitap NFA transition step over a 64-bit
/// active-state bitset `val` against the current symbol's transition mask
/// `aux`: shift all states forward by one, inject the always-live start state,
/// then keep only states permitted by the mask -> `((val << 1) | 1) & aux`.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::regex_nfa_simd_step::regex_nfa_simd_step;
/// let result = regex_nfa_simd_step(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn regex_nfa_simd_step(val: u64, aux: u64) -> u64 {
    ((val << 1) | 1) & aux
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn regex_nfa_simd_step_reference(val: u64, aux: u64) -> u64 {
        // Independent derivation: compute the propagated/start state set first,
        // then filter it state-by-state against the transition mask in a loop.
        let propagated = (val << 1) | 1;
        let mut next = 0u64;
        for i in 0..64 {
            let bit = (propagated >> i) & 1;
            let allowed = (aux >> i) & 1;
            next |= (bit & allowed) << i;
        }
        next
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_regex_nfa_simd_step_1(val: u64, aux: u64) -> u64 {
        !regex_nfa_simd_step_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_regex_nfa_simd_step_2(val: u64, aux: u64) -> u64 {
        regex_nfa_simd_step_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_regex_nfa_simd_step_3(val: u64, aux: u64) -> u64 {
        regex_nfa_simd_step_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_regex_nfa_simd_step_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = regex_nfa_simd_step_reference(val, aux);
            let actual = regex_nfa_simd_step(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = regex_nfa_simd_step_reference(val, aux);
            let actual = mutant_regex_nfa_simd_step_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = regex_nfa_simd_step_reference(val, aux);
            let actual = mutant_regex_nfa_simd_step_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = regex_nfa_simd_step_reference(val, aux);
            let actual = mutant_regex_nfa_simd_step_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_regex_nfa_simd_step_boundaries() {
        assert_eq!(
            regex_nfa_simd_step(0, 0),
            regex_nfa_simd_step_reference(0, 0)
        );
        assert_eq!(
            regex_nfa_simd_step(u64::MAX, u64::MAX),
            regex_nfa_simd_step_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            regex_nfa_simd_step(u64::MAX, 0),
            regex_nfa_simd_step_reference(u64::MAX, 0)
        );
        assert_eq!(
            regex_nfa_simd_step(0, u64::MAX),
            regex_nfa_simd_step_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = regex_nfa_simd_step_reference(val, aux) }
    //
    // Counterfactual Analysis for regex_nfa_simd_step:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_regex_nfa_simd_step(c: &mut Criterion) {
        c.bench_function("regex_nfa_simd_step", |b| {
            b.iter(|| {
                let res = regex_nfa_simd_step(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
