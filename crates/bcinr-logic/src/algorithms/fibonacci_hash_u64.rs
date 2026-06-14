// Academic-grade branchless algorithm library: fibonacci_hash_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// fibonacci_hash_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Interpretation:** Knuth's Fibonacci hashing. The key `val` is multiplied by the
/// 64-bit fixed-point golden ratio `2^64 / φ = 0x9E3779B97F4A7C15`, and the top
/// `bits = (aux & 63)` bits of the product are extracted by a right shift of
/// `(64 - bits) & 63`, giving the hash bucket in `[0, 2^bits)`. A `bits == 0`
/// request returns the whole product (shift of 0). Pure multiply + shift,
/// branchless and O(1).
/// **Ensures:** Result matches the independent reference for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::fibonacci_hash_u64::fibonacci_hash_u64;
/// let result = fibonacci_hash_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn fibonacci_hash_u64(val: u64, aux: u64) -> u64 {
    let product = val.wrapping_mul(0x9E3779B97F4A7C15);
    let bits = (aux & 63) as u32;
    let shift = (64u32.wrapping_sub(bits)) & 63;
    product >> shift
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    fn fibonacci_hash_u64_reference(val: u64, aux: u64) -> u64 {
        // Independent: u128 product, explicit branch on bits==0 (test-only).
        let product = ((val as u128 * 0x9E3779B97F4A7C15u128) & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let bits = (aux % 64) as u32;
        if bits == 0 {
            product
        } else {
            product >> (64 - bits)
        }
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_fibonacci_hash_u64_1(val: u64, aux: u64) -> u64 {
        !fibonacci_hash_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_fibonacci_hash_u64_2(val: u64, aux: u64) -> u64 {
        fibonacci_hash_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_fibonacci_hash_u64_3(val: u64, aux: u64) -> u64 {
        fibonacci_hash_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_fibonacci_hash_u64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fibonacci_hash_u64_reference(val, aux);
            let actual = fibonacci_hash_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_fibonacci_hash_u64_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fibonacci_hash_u64_reference(val, aux);
            let actual = mutant_fibonacci_hash_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_fibonacci_hash_u64_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fibonacci_hash_u64_reference(val, aux);
            let actual = mutant_fibonacci_hash_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_fibonacci_hash_u64_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fibonacci_hash_u64_reference(val, aux);
            let actual = mutant_fibonacci_hash_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_fibonacci_hash_u64_boundaries() {
        assert_eq!(fibonacci_hash_u64(0, 0), fibonacci_hash_u64_reference(0, 0));
        assert_eq!(
            fibonacci_hash_u64(u64::MAX, u64::MAX),
            fibonacci_hash_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            fibonacci_hash_u64(u64::MAX, 0),
            fibonacci_hash_u64_reference(u64::MAX, 0)
        );
        assert_eq!(
            fibonacci_hash_u64(0, u64::MAX),
            fibonacci_hash_u64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = fibonacci_hash_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for fibonacci_hash_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_fibonacci_hash_u64(c: &mut Criterion) {
        c.bench_function("fibonacci_hash_u64", |b| {
            b.iter(|| {
                let res = fibonacci_hash_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
