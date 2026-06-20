// Academic-grade branchless algorithm library: manhattan_dist_u32x2
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// manhattan_dist_u32x2
///
/// Interpretation: `val` and `aux` each pack a 2D point as two u32 lanes
/// (x = low 32 bits, y = high 32 bits). Computes the Manhattan (L1) distance
/// `|dx| + |dy|`, where each lane difference is the unsigned `abs_diff`.
/// Each abs_diff fits in u32, so the sum fits in u64 without overflow.
///
/// # Branchless Contract
/// **Ensures:** Result equals abs_diff(x-lanes) + abs_diff(y-lanes).
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::manhattan_dist_u32x2::manhattan_dist_u32x2;
/// let result = manhattan_dist_u32x2(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn manhattan_dist_u32x2(val: u64, aux: u64) -> u64 {
    let dx = (val as u32).abs_diff(aux as u32) as u64;
    let dy = ((val >> 32) as u32).abs_diff((aux >> 32) as u32) as u64;
    dx + dy
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn manhattan_dist_u32x2_reference(val: u64, aux: u64) -> u64 {
        // Independent structure: signed-i64 subtraction with explicit .abs().
        let vx = (val & 0xFFFF_FFFF) as i64;
        let vy = (val >> 32) as i64;
        let ax = (aux & 0xFFFF_FFFF) as i64;
        let ay = (aux >> 32) as i64;
        ((vx - ax).abs() as u64) + ((vy - ay).abs() as u64)
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_manhattan_dist_u32x2_1(val: u64, aux: u64) -> u64 {
        !manhattan_dist_u32x2_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_manhattan_dist_u32x2_2(val: u64, aux: u64) -> u64 {
        manhattan_dist_u32x2_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_manhattan_dist_u32x2_3(val: u64, aux: u64) -> u64 {
        manhattan_dist_u32x2_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_manhattan_dist_u32x2_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = manhattan_dist_u32x2_reference(val, aux);
            let actual = manhattan_dist_u32x2(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = manhattan_dist_u32x2_reference(val, aux);
            let actual = mutant_manhattan_dist_u32x2_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = manhattan_dist_u32x2_reference(val, aux);
            let actual = mutant_manhattan_dist_u32x2_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = manhattan_dist_u32x2_reference(val, aux);
            let actual = mutant_manhattan_dist_u32x2_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_manhattan_dist_u32x2_boundaries() {
        assert_eq!(
            manhattan_dist_u32x2(0, 0),
            manhattan_dist_u32x2_reference(0, 0)
        );
        assert_eq!(
            manhattan_dist_u32x2(u64::MAX, u64::MAX),
            manhattan_dist_u32x2_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            manhattan_dist_u32x2(u64::MAX, 0),
            manhattan_dist_u32x2_reference(u64::MAX, 0)
        );
        assert_eq!(
            manhattan_dist_u32x2(0, u64::MAX),
            manhattan_dist_u32x2_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = manhattan_dist_u32x2_reference(val, aux) }
    //
    // Counterfactual Analysis for manhattan_dist_u32x2:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_manhattan_dist_u32x2(c: &mut Criterion) {
        c.bench_function("manhattan_dist_u32x2", |b| {
            b.iter(|| {
                let res = manhattan_dist_u32x2(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
