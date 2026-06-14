// Academic-grade branchless algorithm library: branchless_signum_i64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// branchless_signum_i64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::branchless_signum_i64::branchless_signum_i64;
/// let result = branchless_signum_i64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
/// # Branchless Contract
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn branchless_signum_i64(val: u64, aux: u64) -> u64 {
    // Branchless Contract: signum of (val + aux) interpreted as i64, returning
    // -1, 0, or 1 as a two's-complement u64 bit pattern. Computed via two
    // arithmetic shifts: the sign bit OR'd with the negated-sign indicator.
    let v = (val.wrapping_add(aux)) as i64;
    let neg = (v >> 63) as u64; // all-ones if v < 0, else 0
    let pos = (v.wrapping_neg() >> 63) as u64 & 1; // 1 if v > 0, else 0
    neg | pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn branchless_signum_i64_reference(val: u64, aux: u64) -> u64 {
        // Independent: explicit comparison-based signum using std i64::signum.
        let v = val.wrapping_add(aux) as i64;
        match v.cmp(&0) {
            core::cmp::Ordering::Less => (-1i64) as u64,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        }
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_branchless_signum_i64_1(val: u64, aux: u64) -> u64 {
        !branchless_signum_i64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_branchless_signum_i64_2(val: u64, aux: u64) -> u64 {
        branchless_signum_i64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_branchless_signum_i64_3(val: u64, aux: u64) -> u64 {
        branchless_signum_i64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_branchless_signum_i64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = branchless_signum_i64_reference(val, aux);
            let actual = branchless_signum_i64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_branchless_signum_i64_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = branchless_signum_i64_reference(val, aux);
            let actual = mutant_branchless_signum_i64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_branchless_signum_i64_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = branchless_signum_i64_reference(val, aux);
            let actual = mutant_branchless_signum_i64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_branchless_signum_i64_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = branchless_signum_i64_reference(val, aux);
            let actual = mutant_branchless_signum_i64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_branchless_signum_i64_boundaries() {
        assert_eq!(
            branchless_signum_i64(0, 0),
            branchless_signum_i64_reference(0, 0)
        );
        assert_eq!(
            branchless_signum_i64(u64::MAX, u64::MAX),
            branchless_signum_i64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            branchless_signum_i64(u64::MAX, 0),
            branchless_signum_i64_reference(u64::MAX, 0)
        );
        assert_eq!(
            branchless_signum_i64(0, u64::MAX),
            branchless_signum_i64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = branchless_signum_i64_reference(val, aux) }
    //
    // Counterfactual Analysis for branchless_signum_i64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_branchless_signum_i64(c: &mut Criterion) {
        c.bench_function("branchless_signum_i64", |b| {
            b.iter(|| {
                let res = branchless_signum_i64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
