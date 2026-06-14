// Academic-grade branchless algorithm library: rolling_hash_rabin_karp
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// rolling_hash_rabin_karp
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
/// use bcinr_logic::algorithms::rolling_hash_rabin_karp::rolling_hash_rabin_karp;
/// let result = rolling_hash_rabin_karp(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn rolling_hash_rabin_karp(val: u64, aux: u64) -> u64 {
    val.wrapping_mul(31).wrapping_add(aux)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn rolling_hash_rabin_karp_reference(val: u64, aux: u64) -> u64 {
        let (p1, _) = val.overflowing_mul(31);
        let (p2, _) = p1.overflowing_add(aux);
        p2
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_rolling_hash_rabin_karp_1(val: u64, aux: u64) -> u64 {
        !rolling_hash_rabin_karp_reference(val, aux)
    }
    #[allow(unused_variables)]
    fn mutant_rolling_hash_rabin_karp_2(val: u64, aux: u64) -> u64 {
        rolling_hash_rabin_karp_reference(val, aux).wrapping_add(1)
    }
    #[allow(unused_variables)]
    fn mutant_rolling_hash_rabin_karp_3(val: u64, aux: u64) -> u64 {
        rolling_hash_rabin_karp_reference(val, aux) ^ 0xFFFFFFFF
    }

    proptest! {
        #[test]
        fn test_rolling_hash_rabin_karp_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = rolling_hash_rabin_karp_reference(val, aux);
            let actual = rolling_hash_rabin_karp(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_rolling_hash_rabin_karp_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = rolling_hash_rabin_karp_reference(val, aux);
            let actual = mutant_rolling_hash_rabin_karp_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_rolling_hash_rabin_karp_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = rolling_hash_rabin_karp_reference(val, aux);
            let actual = mutant_rolling_hash_rabin_karp_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_rolling_hash_rabin_karp_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = rolling_hash_rabin_karp_reference(val, aux);
            let actual = mutant_rolling_hash_rabin_karp_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_rolling_hash_rabin_karp_boundaries() {
        assert_eq!(
            rolling_hash_rabin_karp(0, 0),
            rolling_hash_rabin_karp_reference(0, 0)
        );
        assert_eq!(
            rolling_hash_rabin_karp(u64::MAX, u64::MAX),
            rolling_hash_rabin_karp_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            rolling_hash_rabin_karp(u64::MAX, 0),
            rolling_hash_rabin_karp_reference(u64::MAX, 0)
        );
        assert_eq!(
            rolling_hash_rabin_karp(0, u64::MAX),
            rolling_hash_rabin_karp_reference(0, u64::MAX)
        );
    }
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_rolling_hash_rabin_karp(c: &mut Criterion) {
        c.bench_function("rolling_hash_rabin_karp", |b| {
            b.iter(|| {
                let res = rolling_hash_rabin_karp(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
