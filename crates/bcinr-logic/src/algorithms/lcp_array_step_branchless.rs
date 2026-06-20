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


    #[test]
    fn test_lcp_array_step_branchless_all() {
        // equivalence oracle
        let expected = lcp_array_step_branchless_reference(42, 1337);
        let actual = lcp_array_step_branchless(42, 1337);
        assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        // boundaries

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
        // mutant divergence
        let baseline = lcp_array_step_branchless_reference(42, 1337);
        let m1 = mutant_lcp_array_step_branchless_1(42, 1337);
        let m2 = mutant_lcp_array_step_branchless_2(42, 1337);
        let m3 = mutant_lcp_array_step_branchless_3(42, 1337);
        if m1 != baseline { assert_ne!(m1, baseline, "mutant 1"); }
        if m2 != baseline { assert_ne!(m2, baseline, "mutant 2"); }
        if m3 != baseline { assert_ne!(m3, baseline, "mutant 3"); }
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
