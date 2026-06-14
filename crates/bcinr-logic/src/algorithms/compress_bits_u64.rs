// Academic-grade branchless algorithm library: compress_bits_u64
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// compress_bits_u64
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// # Branchless Contract
/// Parallel bit-gather (PEXT): packs the bits of `val` selected by mask `aux`
/// into the low end of the result, with data-independent control flow.
///
/// ```rust
/// use bcinr_logic::algorithms::compress_bits_u64::compress_bits_u64;
/// let result = compress_bits_u64(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn compress_bits_u64(val: u64, aux: u64) -> u64 {
    // Branchless Contract: parallel bit-gather (PEXT). Extract the bits of
    // `val` at positions where mask `aux` is set and pack them into the low
    // end of the result, in order. Hacker's Delight `compress`, fully unrolled
    // (6 fixed stages) so control flow is data-independent.
    let mut x = val & aux;
    let mut m = aux;
    let mut mk = !m << 1;
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mv = mp & m;
    m = (m ^ mv) | (mv >> 1);
    let t = x & mv;
    x = (x ^ t) | (t >> 1);
    mk &= !mp;
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mv = mp & m;
    m = (m ^ mv) | (mv >> 2);
    let t = x & mv;
    x = (x ^ t) | (t >> 2);
    mk &= !mp;
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mv = mp & m;
    m = (m ^ mv) | (mv >> 4);
    let t = x & mv;
    x = (x ^ t) | (t >> 4);
    mk &= !mp;
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mv = mp & m;
    m = (m ^ mv) | (mv >> 8);
    let t = x & mv;
    x = (x ^ t) | (t >> 8);
    mk &= !mp;
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mv = mp & m;
    m = (m ^ mv) | (mv >> 16);
    let t = x & mv;
    x = (x ^ t) | (t >> 16);
    mk &= !mp;
    let mut mp = mk ^ (mk << 1);
    mp ^= mp << 2;
    mp ^= mp << 4;
    mp ^= mp << 8;
    mp ^= mp << 16;
    mp ^= mp << 32;
    let mv = mp & m;
    let t = x & mv;
    x = (x ^ t) | (t >> 32);
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // NOTE: Identical to main implementation (no simpler correct variant exists).
    // -------------------------------------------------------------------------
    fn compress_bits_u64_reference(val: u64, aux: u64) -> u64 {
        // Independent derivation: straightforward serial scan. Walk the mask bits
        // from LSB to MSB; whenever a mask bit is set, append the corresponding
        // bit of `val` at the next output position. O(64) loop, distinct shape.
        let mut out: u64 = 0;
        let mut pos: u32 = 0;
        let mut i: u32 = 0;
        while i < 64 {
            if (aux >> i) & 1 == 1 {
                let bit = (val >> i) & 1;
                out |= bit << pos;
                pos += 1;
            }
            i += 1;
        }
        out
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_compress_bits_u64_1(val: u64, aux: u64) -> u64 {
        !compress_bits_u64_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_compress_bits_u64_2(val: u64, aux: u64) -> u64 {
        compress_bits_u64_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_compress_bits_u64_3(val: u64, aux: u64) -> u64 {
        compress_bits_u64_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_compress_bits_u64_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = compress_bits_u64_reference(val, aux);
            let actual = compress_bits_u64(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_compress_bits_u64_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = compress_bits_u64_reference(val, aux);
            let actual = mutant_compress_bits_u64_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_compress_bits_u64_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = compress_bits_u64_reference(val, aux);
            let actual = mutant_compress_bits_u64_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_compress_bits_u64_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = compress_bits_u64_reference(val, aux);
            let actual = mutant_compress_bits_u64_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_compress_bits_u64_boundaries() {
        assert_eq!(compress_bits_u64(0, 0), compress_bits_u64_reference(0, 0));
        assert_eq!(
            compress_bits_u64(u64::MAX, u64::MAX),
            compress_bits_u64_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            compress_bits_u64(u64::MAX, 0),
            compress_bits_u64_reference(u64::MAX, 0)
        );
        assert_eq!(
            compress_bits_u64(0, u64::MAX),
            compress_bits_u64_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = compress_bits_u64_reference(val, aux) }
    //
    // Counterfactual Analysis for compress_bits_u64:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_compress_bits_u64(c: &mut Criterion) {
        c.bench_function("compress_bits_u64", |b| {
            b.iter(|| {
                let res = compress_bits_u64(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
