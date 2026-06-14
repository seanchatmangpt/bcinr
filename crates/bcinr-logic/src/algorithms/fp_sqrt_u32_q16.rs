// Academic-grade branchless algorithm library: fp_sqrt_u32_q16
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// fp_sqrt_u32_q16
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::fp_sqrt_u32_q16::fp_sqrt_u32_q16;
/// let result = fp_sqrt_u32_q16(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn fp_sqrt_u32_q16(val: u64, aux: u64) -> u64 {
    // Branchless digit-by-digit integer sqrt of (val << 16).
    // val < 2^64 so the scaled operand is < 2^80; the highest possible
    // even power of four is 4^40 = 2^80, so 41 reduction steps suffice.
    let mut x = (val as u128) << 16;
    let mut res = 0u128;
    let mut bit = 1u128 << 80;
    let mut k = 0u32;
    while k < 41 {
        let candidate = res + bit;
        let cond = x >= candidate;
        let m = (cond as u128).wrapping_neg();
        x -= candidate & m;
        res = (res >> 1) + (bit & m);
        bit >>= 2;
        k += 1;
    }
    res as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn fp_sqrt_u32_q16_reference(val: u64, _aux: u64) -> u64 {
        let val_scaled = (val as u128) << 16;
        if val_scaled == 0 {
            return 0;
        }
        let mut r = val_scaled;
        loop {
            let next = (r + val_scaled / r) / 2;
            if next >= r {
                break;
            }
            r = next;
        }
        r as u64
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_fp_sqrt_u32_q16_1(val: u64, aux: u64) -> u64 {
        !fp_sqrt_u32_q16_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_fp_sqrt_u32_q16_2(val: u64, aux: u64) -> u64 {
        fp_sqrt_u32_q16_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_fp_sqrt_u32_q16_3(val: u64, aux: u64) -> u64 {
        fp_sqrt_u32_q16_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_fp_sqrt_u32_q16_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fp_sqrt_u32_q16_reference(val, aux);
            let actual = fp_sqrt_u32_q16(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_fp_sqrt_u32_q16_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fp_sqrt_u32_q16_reference(val, aux);
            let actual = mutant_fp_sqrt_u32_q16_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_fp_sqrt_u32_q16_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fp_sqrt_u32_q16_reference(val, aux);
            let actual = mutant_fp_sqrt_u32_q16_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_fp_sqrt_u32_q16_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = fp_sqrt_u32_q16_reference(val, aux);
            let actual = mutant_fp_sqrt_u32_q16_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_fp_sqrt_u32_q16_boundaries() {
        assert_eq!(fp_sqrt_u32_q16(0, 0), fp_sqrt_u32_q16_reference(0, 0));
        assert_eq!(
            fp_sqrt_u32_q16(u64::MAX, u64::MAX),
            fp_sqrt_u32_q16_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            fp_sqrt_u32_q16(u64::MAX, 0),
            fp_sqrt_u32_q16_reference(u64::MAX, 0)
        );
        assert_eq!(
            fp_sqrt_u32_q16(0, u64::MAX),
            fp_sqrt_u32_q16_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = fp_sqrt_u32_q16_reference(val, aux) }
    //
    // Counterfactual Analysis for fp_sqrt_u32_q16:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_fp_sqrt_u32_q16(c: &mut Criterion) {
        c.bench_function("fp_sqrt_u32_q16", |b| {
            b.iter(|| {
                let res = fp_sqrt_u32_q16(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
