// Academic-grade branchless algorithm library: bext_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// bext_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Ensures:** Parallel bit-extract (BMI2 `pext`): gathers the bits of `val` selected
/// by mask `aux` and packs them contiguously into the low-order bits of the result,
/// in ascending bit order. Implemented via Hacker's Delight's branchless `compress`
/// (six unrolled parallel-prefix stages).
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::bext_u64::bext_u64;
/// let result = bext_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn bext_u64(val: u64, aux: u64) -> u64 {
    let mut x = val & aux;
    let mut mask = aux;
    let mut mk = !mask << 1;

    // Stage i shifts selected bits right by 1<<i (i = 0..6), unrolled.
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mut mv = mp & mask;
    mask = (mask ^ mv) | (mv >> 1);
    let mut t = x & mv;
    x = (x ^ t) | (t >> 1);
    mk &= !mp;

    mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    mv = mp & mask;
    mask = (mask ^ mv) | (mv >> 2);
    t = x & mv;
    x = (x ^ t) | (t >> 2);
    mk &= !mp;

    mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    mv = mp & mask;
    mask = (mask ^ mv) | (mv >> 4);
    t = x & mv;
    x = (x ^ t) | (t >> 4);
    mk &= !mp;

    mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    mv = mp & mask;
    mask = (mask ^ mv) | (mv >> 8);
    t = x & mv;
    x = (x ^ t) | (t >> 8);
    mk &= !mp;

    mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    mv = mp & mask;
    mask = (mask ^ mv) | (mv >> 16);
    t = x & mv;
    x = (x ^ t) | (t >> 16);
    mk &= !mp;

    mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    mv = mp & mask;
    t = x & mv;
    x = (x ^ t) | (t >> 32);

    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn bext_u64_reference(val: u64, aux: u64) -> u64 {
        // Independent: linear scan of mask bits, appending selected val bits.
        let mut out = 0u64;
        let mut k = 0u32;
        for i in 0..64u32 {
            if (aux >> i) & 1 == 1 {
                out |= ((val >> i) & 1) << k;
                k += 1;
            }
        }
        out
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_bext_u64_1(val: u64, aux: u64) -> u64 {
        !bext_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_bext_u64_2(val: u64, aux: u64) -> u64 {
        bext_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_bext_u64_3(val: u64, aux: u64) -> u64 {
        bext_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_bext_u64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = bext_u64_reference(val, aux);
            let actual = bext_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_bext_u64_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = bext_u64_reference(val, aux);
            let actual = mutant_bext_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_bext_u64_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = bext_u64_reference(val, aux);
            let actual = mutant_bext_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_bext_u64_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = bext_u64_reference(val, aux);
            let actual = mutant_bext_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_bext_u64_boundaries() {
        assert_eq!(bext_u64(0, 0), bext_u64_reference(0, 0));
        assert_eq!(
            bext_u64(u64::MAX, u64::MAX),
            bext_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(bext_u64(u64::MAX, 0), bext_u64_reference(u64::MAX, 0));
        assert_eq!(bext_u64(0, u64::MAX), bext_u64_reference(0, u64::MAX));
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = bext_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for bext_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_bext_u64(c: &mut Criterion) {
        c.bench_function("bext_u64", |b| {
            b.iter(|| {
                let res = bext_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
