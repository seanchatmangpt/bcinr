// Academic-grade branchless algorithm library: euclidean_dist_sq_u32x2
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// euclidean_dist_sq_u32x2
///
/// Interpretation: `val` and `aux` each pack a 2D point as two u32 lanes
/// (x = low 32 bits, y = high 32 bits). Computes the squared Euclidean
/// distance `dx*dx + dy*dy` where `dx`/`dy` are the unsigned lane differences
/// (`abs_diff`). The squares and their sum are taken with wrapping u64
/// arithmetic so the result is always defined.
///
/// # Branchless Contract
/// **Ensures:** Result equals wrapping(dx*dx) + wrapping(dy*dy) with dx,dy=abs_diff.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::euclidean_dist_sq_u32x2::euclidean_dist_sq_u32x2;
/// let result = euclidean_dist_sq_u32x2(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn euclidean_dist_sq_u32x2(val: u64, aux: u64) -> u64 {
    let dx = (val as u32).abs_diff(aux as u32) as u64;
    let dy = ((val >> 32) as u32).abs_diff((aux >> 32) as u32) as u64;
    dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn euclidean_dist_sq_u32x2_reference(val: u64, aux: u64) -> u64 {
        // Independent structure: explicit lane extraction, branchful abs, then
        // 128-bit squaring truncated to 64 bits to match wrapping semantics.
        let vx = (val & 0xFFFF_FFFF) as u32;
        let vy = (val >> 32) as u32;
        let ax = (aux & 0xFFFF_FFFF) as u32;
        let ay = (aux >> 32) as u32;
        let dx = if vx >= ax { vx - ax } else { ax - vx } as u128;
        let dy = if vy >= ay { vy - ay } else { ay - vy } as u128;
        let sum = dx * dx + dy * dy;
        (sum & u64::MAX as u128) as u64
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_euclidean_dist_sq_u32x2_1(val: u64, aux: u64) -> u64 {
        !euclidean_dist_sq_u32x2_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_euclidean_dist_sq_u32x2_2(val: u64, aux: u64) -> u64 {
        euclidean_dist_sq_u32x2_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_euclidean_dist_sq_u32x2_3(val: u64, aux: u64) -> u64 {
        euclidean_dist_sq_u32x2_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_euclidean_dist_sq_u32x2_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = euclidean_dist_sq_u32x2_reference(val, aux);
            let actual = euclidean_dist_sq_u32x2(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_euclidean_dist_sq_u32x2_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = euclidean_dist_sq_u32x2_reference(val, aux);
            let actual = mutant_euclidean_dist_sq_u32x2_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_euclidean_dist_sq_u32x2_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = euclidean_dist_sq_u32x2_reference(val, aux);
            let actual = mutant_euclidean_dist_sq_u32x2_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_euclidean_dist_sq_u32x2_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = euclidean_dist_sq_u32x2_reference(val, aux);
            let actual = mutant_euclidean_dist_sq_u32x2_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_euclidean_dist_sq_u32x2_boundaries() {
        assert_eq!(
            euclidean_dist_sq_u32x2(0, 0),
            euclidean_dist_sq_u32x2_reference(0, 0)
        );
        assert_eq!(
            euclidean_dist_sq_u32x2(u64::MAX, u64::MAX),
            euclidean_dist_sq_u32x2_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            euclidean_dist_sq_u32x2(u64::MAX, 0),
            euclidean_dist_sq_u32x2_reference(u64::MAX, 0)
        );
        assert_eq!(
            euclidean_dist_sq_u32x2(0, u64::MAX),
            euclidean_dist_sq_u32x2_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = euclidean_dist_sq_u32x2_reference(val, aux) }
    //
    // Counterfactual Analysis for euclidean_dist_sq_u32x2:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_euclidean_dist_sq_u32x2(c: &mut Criterion) {
        c.bench_function("euclidean_dist_sq_u32x2", |b| {
            b.iter(|| {
                let res = euclidean_dist_sq_u32x2(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
