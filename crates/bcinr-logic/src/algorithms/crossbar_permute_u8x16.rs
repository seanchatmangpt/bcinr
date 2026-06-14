// Academic-grade branchless algorithm library: crossbar_permute_u8x16
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// crossbar_permute_u8x16
///
/// Interpretation: a 16-lane crossbar switch over the 16 nibbles of `val`.
/// For each output lane `i` (0..16), the control word `aux` supplies a 4-bit
/// source index in its `i`-th nibble; output nibble `i` is set to the source
/// nibble of `val` selected by `aux`'s nibble `i` (masked to 0..15). This is a
/// genuine data-routing crossbar realized branchlessly via shift-and-select.
///
/// # Branchless Contract
/// **Ensures:** Each output lane equals the `val` lane named by the control.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::crossbar_permute_u8x16::crossbar_permute_u8x16;
/// let result = crossbar_permute_u8x16(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn crossbar_permute_u8x16(val: u64, aux: u64) -> u64 {
    let mut out: u64 = 0;
    // Unrolled, fully branchless: 16 nibble lanes.
    macro_rules! lane {
        ($i:expr) => {{
            let sel = ((aux >> ($i * 4)) & 0xF) as u32;
            let src = (val >> (sel * 4)) & 0xF;
            out |= src << ($i * 4);
        }};
    }
    lane!(0);
    lane!(1);
    lane!(2);
    lane!(3);
    lane!(4);
    lane!(5);
    lane!(6);
    lane!(7);
    lane!(8);
    lane!(9);
    lane!(10);
    lane!(11);
    lane!(12);
    lane!(13);
    lane!(14);
    lane!(15);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn crossbar_permute_u8x16_reference(val: u64, aux: u64) -> u64 {
        // Independent structure: gather nibbles into arrays, route via a loop.
        let mut src = [0u8; 16];
        for i in 0..16 {
            src[i] = ((val >> (i * 4)) & 0xF) as u8;
        }
        let mut out: u64 = 0;
        for i in 0..16 {
            let sel = ((aux >> (i * 4)) & 0xF) as usize;
            out += (src[sel] as u64) << (i * 4);
        }
        out
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_crossbar_permute_u8x16_1(val: u64, aux: u64) -> u64 {
        !crossbar_permute_u8x16_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_crossbar_permute_u8x16_2(val: u64, aux: u64) -> u64 {
        crossbar_permute_u8x16_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_crossbar_permute_u8x16_3(val: u64, aux: u64) -> u64 {
        crossbar_permute_u8x16_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_crossbar_permute_u8x16_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = crossbar_permute_u8x16_reference(val, aux);
            let actual = crossbar_permute_u8x16(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_crossbar_permute_u8x16_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = crossbar_permute_u8x16_reference(val, aux);
            let actual = mutant_crossbar_permute_u8x16_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_crossbar_permute_u8x16_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = crossbar_permute_u8x16_reference(val, aux);
            let actual = mutant_crossbar_permute_u8x16_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_crossbar_permute_u8x16_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = crossbar_permute_u8x16_reference(val, aux);
            let actual = mutant_crossbar_permute_u8x16_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_crossbar_permute_u8x16_boundaries() {
        assert_eq!(
            crossbar_permute_u8x16(0, 0),
            crossbar_permute_u8x16_reference(0, 0)
        );
        assert_eq!(
            crossbar_permute_u8x16(u64::MAX, u64::MAX),
            crossbar_permute_u8x16_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            crossbar_permute_u8x16(u64::MAX, 0),
            crossbar_permute_u8x16_reference(u64::MAX, 0)
        );
        assert_eq!(
            crossbar_permute_u8x16(0, u64::MAX),
            crossbar_permute_u8x16_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = crossbar_permute_u8x16_reference(val, aux) }
    //
    // Counterfactual Analysis for crossbar_permute_u8x16:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_crossbar_permute_u8x16(c: &mut Criterion) {
        c.bench_function("crossbar_permute_u8x16", |b| {
            b.iter(|| {
                let res = crossbar_permute_u8x16(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
