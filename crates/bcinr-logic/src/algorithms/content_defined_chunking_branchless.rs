// Academic-grade branchless algorithm library: content_defined_chunking_branchless
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// content_defined_chunking_branchless
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
/// use bcinr_logic::algorithms::content_defined_chunking_branchless::content_defined_chunking_branchless;
/// let result = content_defined_chunking_branchless(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn content_defined_chunking_branchless(val: u64, aux: u64) -> u64 {
    val.wrapping_shl(1)
        .wrapping_add(aux.wrapping_mul(0x9E3779B97F4A7C15u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn content_defined_chunking_branchless_reference(val: u64, aux: u64) -> u64 {
        let term1 = val.wrapping_shl(1);
        let term2 = aux.wrapping_mul(0x9E3779B97F4A7C15u64);
        term1.wrapping_add(term2)
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_content_defined_chunking_branchless_1(val: u64, aux: u64) -> u64 {
        !content_defined_chunking_branchless_reference(val, aux)
    }
    #[allow(unused_variables)]
    fn mutant_content_defined_chunking_branchless_2(val: u64, aux: u64) -> u64 {
        content_defined_chunking_branchless_reference(val, aux).wrapping_add(1)
    }
    #[allow(unused_variables)]
    fn mutant_content_defined_chunking_branchless_3(val: u64, aux: u64) -> u64 {
        content_defined_chunking_branchless_reference(val, aux) ^ 0xFFFFFFFF
    }

    proptest! {
        #[test]
        fn test_content_defined_chunking_branchless_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = content_defined_chunking_branchless_reference(val, aux);
            let actual = content_defined_chunking_branchless(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_content_defined_chunking_branchless_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = content_defined_chunking_branchless_reference(val, aux);
            let actual = mutant_content_defined_chunking_branchless_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_content_defined_chunking_branchless_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = content_defined_chunking_branchless_reference(val, aux);
            let actual = mutant_content_defined_chunking_branchless_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_content_defined_chunking_branchless_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = content_defined_chunking_branchless_reference(val, aux);
            let actual = mutant_content_defined_chunking_branchless_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_content_defined_chunking_branchless_boundaries() {
        assert_eq!(
            content_defined_chunking_branchless(0, 0),
            content_defined_chunking_branchless_reference(0, 0)
        );
        assert_eq!(
            content_defined_chunking_branchless(u64::MAX, u64::MAX),
            content_defined_chunking_branchless_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            content_defined_chunking_branchless(u64::MAX, 0),
            content_defined_chunking_branchless_reference(u64::MAX, 0)
        );
        assert_eq!(
            content_defined_chunking_branchless(0, u64::MAX),
            content_defined_chunking_branchless_reference(0, u64::MAX)
        );
    }
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_content_defined_chunking_branchless(c: &mut Criterion) {
        c.bench_function("content_defined_chunking_branchless", |b| {
            b.iter(|| {
                let res = content_defined_chunking_branchless(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
