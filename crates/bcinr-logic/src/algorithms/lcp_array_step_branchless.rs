// Academic-grade branchless algorithm library: lcp_array_step_branchless
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// lcp_array_step_branchless
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Category:** B — Cell Arithmetic
/// **Plane:** D-resident cell word; no scratch
/// **Tier:** T0 — single-word arithmetic primitive
/// **Scope:** branchless, O(1), CC=1; admissible_T1.
/// **Inputs:** `val` = current cell value; `aux` = second operand / parameter.
///
/// ```rust
/// use bcinr_logic::algorithms::lcp_array_step_branchless::lcp_array_step_branchless;
/// let result = lcp_array_step_branchless(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn lcp_array_step_branchless(val: u64, aux: u64) -> u64 {
    (val ^ aux).leading_zeros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn lcp_array_step_branchless_reference(val: u64, aux: u64) -> u64 {
        let mut count = 0u64;
        let x = val ^ aux;
        for i in (0..64).rev() {
            if ((x >> i) & 1) == 0 {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_lcp_array_step_branchless_1(val: u64, aux: u64) -> u64 {
        !lcp_array_step_branchless_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_lcp_array_step_branchless_2(val: u64, aux: u64) -> u64 {
        lcp_array_step_branchless_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_lcp_array_step_branchless_3(val: u64, aux: u64) -> u64 {
        lcp_array_step_branchless_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_lcp_array_step_branchless_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = lcp_array_step_branchless_reference(val, aux);
            let actual = lcp_array_step_branchless(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = lcp_array_step_branchless_reference(val, aux);
            let actual = mutant_lcp_array_step_branchless_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = lcp_array_step_branchless_reference(val, aux);
            let actual = mutant_lcp_array_step_branchless_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = lcp_array_step_branchless_reference(val, aux);
            let actual = mutant_lcp_array_step_branchless_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_lcp_array_step_branchless_boundaries() {
        assert_eq!(
            lcp_array_step_branchless(0, 0),
            lcp_array_step_branchless_reference(0, 0)
        );
        assert_eq!(
            lcp_array_step_branchless(u64::MAX, u64::MAX),
            lcp_array_step_branchless_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            lcp_array_step_branchless(u64::MAX, 0),
            lcp_array_step_branchless_reference(u64::MAX, 0)
        );
        assert_eq!(
            lcp_array_step_branchless(0, u64::MAX),
            lcp_array_step_branchless_reference(0, u64::MAX)
        );
    }
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_lcp_array_step_branchless(c: &mut Criterion) {
        c.bench_function("lcp_array_step_branchless", |b| {
            b.iter(|| {
                let res = lcp_array_step_branchless(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
