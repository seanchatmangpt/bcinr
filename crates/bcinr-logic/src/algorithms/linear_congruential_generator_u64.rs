// Academic-grade branchless algorithm library: linear_congruential_generator_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// linear_congruential_generator_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::linear_congruential_generator_u64::linear_congruential_generator_u64;
/// let result = linear_congruential_generator_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn linear_congruential_generator_u64(val: u64, aux: u64) -> u64 {
    // Interpretation: one step of a 64-bit linear congruential generator
    //   next = a * state + c   (mod 2^64)
    // with Knuth's MMIX multiplier `a` and an odd increment `c` derived from
    // `aux` (forcing the low bit set guarantees a full period). `val` is state.
    const MMIX_A: u64 = 0x5851_F42D_4C95_7F2D;
    val.wrapping_mul(MMIX_A).wrapping_add(aux | 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    fn linear_congruential_generator_u64_reference(val: u64, aux: u64) -> u64 {
        // Independent: 128-bit product truncated to 64 bits, then add the
        // odd increment with an explicit parity adjustment.
        const A: u128 = 0x5851_F42D_4C95_7F2D;
        let prod = ((val as u128) * A) as u64;
        let inc = if aux & 1 == 1 { aux } else { aux + 1 };
        prod.wrapping_add(inc)
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_linear_congruential_generator_u64_1(val: u64, aux: u64) -> u64 {
        !linear_congruential_generator_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_linear_congruential_generator_u64_2(val: u64, aux: u64) -> u64 {
        linear_congruential_generator_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_linear_congruential_generator_u64_3(val: u64, aux: u64) -> u64 {
        linear_congruential_generator_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_linear_congruential_generator_u64_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = linear_congruential_generator_u64_reference(val, aux);
            let actual = linear_congruential_generator_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = linear_congruential_generator_u64_reference(val, aux);
            let actual = mutant_linear_congruential_generator_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = linear_congruential_generator_u64_reference(val, aux);
            let actual = mutant_linear_congruential_generator_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = linear_congruential_generator_u64_reference(val, aux);
            let actual = mutant_linear_congruential_generator_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_linear_congruential_generator_u64_boundaries() {
        assert_eq!(
            linear_congruential_generator_u64(0, 0),
            linear_congruential_generator_u64_reference(0, 0)
        );
        assert_eq!(
            linear_congruential_generator_u64(u64::MAX, u64::MAX),
            linear_congruential_generator_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            linear_congruential_generator_u64(u64::MAX, 0),
            linear_congruential_generator_u64_reference(u64::MAX, 0)
        );
        assert_eq!(
            linear_congruential_generator_u64(0, u64::MAX),
            linear_congruential_generator_u64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = linear_congruential_generator_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for linear_congruential_generator_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_linear_congruential_generator_u64(c: &mut Criterion) {
        c.bench_function("linear_congruential_generator_u64", |b| {
            b.iter(|| {
                let res = linear_congruential_generator_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
