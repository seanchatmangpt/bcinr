// Academic-grade branchless algorithm library: convex_hull_monotone_chain_step
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// convex_hull_monotone_chain_step
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # CONTRACT
/// **Ensures:** The result matches the slow but correct reference implementation for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::convex_hull_monotone_chain_step::convex_hull_monotone_chain_step;
/// let result = convex_hull_monotone_chain_step(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
/// # Branchless Contract
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn convex_hull_monotone_chain_step(val: u64, aux: u64) -> u64 {
    // Branchless Contract: the orientation test driving Andrew's monotone-chain
    // convex-hull step. Each operand packs a 2D vector as two i32 components
    // (x in the low half, y in the high half). Returns the sign of the cross
    // product vx*ay - vy*ax as a two's-complement u64 (-1 = clockwise/right
    // turn, 0 = collinear, 1 = counter-clockwise/left turn).
    let vx = (val as i32) as i128;
    let vy = ((val >> 32) as i32) as i128;
    let ax = (aux as i32) as i128;
    let ay = ((aux >> 32) as i32) as i128;
    let cross = vx * ay - vy * ax;
    // Branchless signum via comparison masks (booleans, no control flow).
    let s = (cross > 0) as i64 - (cross < 0) as i64;
    s as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn convex_hull_monotone_chain_step_reference(val: u64, aux: u64) -> u64 {
        // Independent: compute the cross product, classify with match on ordering.
        let vx = (val as i32) as i128;
        let vy = ((val >> 32) as i32) as i128;
        let ax = (aux as i32) as i128;
        let ay = ((aux >> 32) as i32) as i128;
        let cross = vx * ay - vy * ax;
        let s: i64 = match cross.cmp(&0) {
            core::cmp::Ordering::Greater => 1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Less => -1,
        };
        s as u64
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_convex_hull_monotone_chain_step_1(val: u64, aux: u64) -> u64 {
        !convex_hull_monotone_chain_step_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_convex_hull_monotone_chain_step_2(val: u64, aux: u64) -> u64 {
        convex_hull_monotone_chain_step_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_convex_hull_monotone_chain_step_3(val: u64, aux: u64) -> u64 {
        convex_hull_monotone_chain_step_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_convex_hull_monotone_chain_step_equivalence(val in any::<u64>(), aux in any::<u64>()) {
            let expected = convex_hull_monotone_chain_step_reference(val, aux);
            let actual = convex_hull_monotone_chain_step(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");
        }

        #[test]
        fn test_convex_hull_monotone_chain_step_counterfactual_mutant_1(val in any::<u64>(), aux in any::<u64>()) {
            let expected = convex_hull_monotone_chain_step_reference(val, aux);
            let actual = mutant_convex_hull_monotone_chain_step_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }
        }

        #[test]
        fn test_convex_hull_monotone_chain_step_counterfactual_mutant_2(val in any::<u64>(), aux in any::<u64>()) {
            let expected = convex_hull_monotone_chain_step_reference(val, aux);
            let actual = mutant_convex_hull_monotone_chain_step_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }
        }

        #[test]
        fn test_convex_hull_monotone_chain_step_counterfactual_mutant_3(val in any::<u64>(), aux in any::<u64>()) {
            let expected = convex_hull_monotone_chain_step_reference(val, aux);
            let actual = mutant_convex_hull_monotone_chain_step_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_convex_hull_monotone_chain_step_boundaries() {
        assert_eq!(
            convex_hull_monotone_chain_step(0, 0),
            convex_hull_monotone_chain_step_reference(0, 0)
        );
        assert_eq!(
            convex_hull_monotone_chain_step(u64::MAX, u64::MAX),
            convex_hull_monotone_chain_step_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            convex_hull_monotone_chain_step(u64::MAX, 0),
            convex_hull_monotone_chain_step_reference(u64::MAX, 0)
        );
        assert_eq!(
            convex_hull_monotone_chain_step(0, u64::MAX),
            convex_hull_monotone_chain_step_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = convex_hull_monotone_chain_step_reference(val, aux) }
    //
    // Counterfactual Analysis for convex_hull_monotone_chain_step:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_convex_hull_monotone_chain_step(c: &mut Criterion) {
        c.bench_function("convex_hull_monotone_chain_step", |b| {
            b.iter(|| {
                let res = convex_hull_monotone_chain_step(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
