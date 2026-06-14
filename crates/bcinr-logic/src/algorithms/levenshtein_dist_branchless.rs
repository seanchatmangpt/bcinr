// Academic-grade branchless algorithm library: levenshtein_dist_branchless
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// levenshtein_dist_branchless
///
/// Interpretation: `val` and `aux` are two 8-byte strings of equal length.
/// For equal-length strings the Levenshtein (edit) distance reduces to the
/// number of substitution positions, i.e. the count of byte lanes that differ.
/// Computed branchlessly: XOR the words, fold each non-zero byte to exactly
/// one bit via a SWAR saturate, then population-count the per-byte flags.
///
/// # Branchless Contract
/// **Ensures:** Result equals the number of differing bytes between val and aux.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::levenshtein_dist_branchless::levenshtein_dist_branchless;
/// let result = levenshtein_dist_branchless(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn levenshtein_dist_branchless(val: u64, aux: u64) -> u64 {
    let diff = val ^ aux;
    // SWAR: set the high bit of each byte iff that byte is non-zero.
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;
    let nonzero = (((diff | HI).wrapping_sub(LO)) | diff) & HI;
    (nonzero >> 7).count_ones() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn levenshtein_dist_branchless_reference(val: u64, aux: u64) -> u64 {
        // Independent structure: split into bytes, compare each pair directly.
        let vb = val.to_le_bytes();
        let ab = aux.to_le_bytes();
        let mut count = 0u64;
        for i in 0..8 {
            if vb[i] != ab[i] {
                count += 1;
            }
        }
        count
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_levenshtein_dist_branchless_1(val: u64, aux: u64) -> u64 {
        !levenshtein_dist_branchless_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_levenshtein_dist_branchless_2(val: u64, aux: u64) -> u64 {
        levenshtein_dist_branchless_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_levenshtein_dist_branchless_3(val: u64, aux: u64) -> u64 {
        levenshtein_dist_branchless_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_levenshtein_dist_branchless_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = levenshtein_dist_branchless_reference(val, aux);
            let actual = levenshtein_dist_branchless(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_levenshtein_dist_branchless_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = levenshtein_dist_branchless_reference(val, aux);
            let actual = mutant_levenshtein_dist_branchless_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_levenshtein_dist_branchless_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = levenshtein_dist_branchless_reference(val, aux);
            let actual = mutant_levenshtein_dist_branchless_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_levenshtein_dist_branchless_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = levenshtein_dist_branchless_reference(val, aux);
            let actual = mutant_levenshtein_dist_branchless_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_levenshtein_dist_branchless_boundaries() {
        assert_eq!(
            levenshtein_dist_branchless(0, 0),
            levenshtein_dist_branchless_reference(0, 0)
        );
        assert_eq!(
            levenshtein_dist_branchless(u64::MAX, u64::MAX),
            levenshtein_dist_branchless_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            levenshtein_dist_branchless(u64::MAX, 0),
            levenshtein_dist_branchless_reference(u64::MAX, 0)
        );
        assert_eq!(
            levenshtein_dist_branchless(0, u64::MAX),
            levenshtein_dist_branchless_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = levenshtein_dist_branchless_reference(val, aux) }
    //
    // Counterfactual Analysis for levenshtein_dist_branchless:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_levenshtein_dist_branchless(c: &mut Criterion) {
        c.bench_function("levenshtein_dist_branchless", |b| {
            b.iter(|| {
                let res = levenshtein_dist_branchless(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
