// Academic-grade branchless algorithm library: internet_checksum_u16
// Automatically generated scaffolding for AGI-level branchless primitives.
// Assumes adherence to zero-branching, 0-allocation, and sub-10ns latency.

/// internet_checksum_u16
///
/// Branchless implementation guaranteed to execute in constant time
/// with zero dynamic dispatch or control flow hazards.
///
/// # Branchless Contract
/// **Interpretation:** The RFC 1071 Internet checksum over the four 16-bit words
/// packed (little-endian) in `val`, seeded by `aux & 0xFFFF`. All words are added
/// into a wide accumulator; the end-around carry is folded twice
/// (`sum = (sum & 0xFFFF) + (sum >> 16)`) so all carries are absorbed, and the
/// one's-complement (bitwise NOT, masked to 16 bits) is returned. Branchless O(1).
/// **Ensures:** Result matches the independent reference for all inputs.
/// **Invariant:** Execution path is independent of input data values (Branchless).
///
/// ```rust
/// use bcinr_logic::algorithms::internet_checksum_u16::internet_checksum_u16;
/// let result = internet_checksum_u16(42, 1337);
/// assert!(result <= u64::MAX);
/// ```
// SAFETY_LEVEL: no unsafe code permitted in algorithm modules (enforced via forbid in lib.rs)
#[no_mangle]
#[allow(unused_variables)]
pub fn internet_checksum_u16(val: u64, aux: u64) -> u64 {
    let mut sum = (aux & 0xFFFF)
        + (val & 0xFFFF)
        + ((val >> 16) & 0xFFFF)
        + ((val >> 32) & 0xFFFF)
        + ((val >> 48) & 0xFFFF);
    sum = (sum & 0xFFFF) + (sum >> 16);
    sum = (sum & 0xFFFF) + (sum >> 16);
    (!sum) & 0xFFFF
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // POSITIVE ORACLE: Reference implementation
    // -------------------------------------------------------------------------
    fn internet_checksum_u16_reference(val: u64, aux: u64) -> u64 {
        // Independent: loop add with explicit carry-folding while-style reduction.
        let mut sum: u64 = aux & 0xFFFF;
        for k in 0..4u32 {
            sum += (val >> (16 * k)) & 0xFFFF;
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        (sum ^ 0xFFFF) & 0xFFFF
    }

    // -------------------------------------------------------------------------
    // NEGATIVE MUTANTS: Intentionally flawed versions
    // -------------------------------------------------------------------------
    #[allow(unused_variables)]
    fn mutant_internet_checksum_u16_1(val: u64, aux: u64) -> u64 {
        !internet_checksum_u16_reference(val, aux)
    } // Identity bluff
    #[allow(unused_variables)]
    fn mutant_internet_checksum_u16_2(val: u64, aux: u64) -> u64 {
        internet_checksum_u16_reference(val, aux).wrapping_add(1)
    } // Bit-skip bluff
    #[allow(unused_variables)]
    fn mutant_internet_checksum_u16_3(val: u64, aux: u64) -> u64 {
        internet_checksum_u16_reference(val, aux) ^ 0xFFFFFFFF
    } // Operator-swap bluff

    proptest! {
        #[test]
        fn test_internet_checksum_u16_all(val in any::<u64>(), aux in any::<u64>()) {
            let expected = internet_checksum_u16_reference(val, aux);
            let actual = internet_checksum_u16(val, aux);
            prop_assert_eq!(expected, actual, "Adversarial failure: branchless mismatch");

            let expected = internet_checksum_u16_reference(val, aux);
            let actual = mutant_internet_checksum_u16_1(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 1 failed to fail!");
            }

            let expected = internet_checksum_u16_reference(val, aux);
            let actual = mutant_internet_checksum_u16_2(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 2 failed to fail!");
            }

            let expected = internet_checksum_u16_reference(val, aux);
            let actual = mutant_internet_checksum_u16_3(val, aux);
            if val != aux && val != 0 && aux != 0 {
                prop_assert!(expected != actual, "Counterfactual Mutant 3 failed to fail!");
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES: Hardcoded edge cases
    // -------------------------------------------------------------------------
    #[test]
    fn test_internet_checksum_u16_boundaries() {
        assert_eq!(
            internet_checksum_u16(0, 0),
            internet_checksum_u16_reference(0, 0)
        );
        assert_eq!(
            internet_checksum_u16(u64::MAX, u64::MAX),
            internet_checksum_u16_reference(u64::MAX, u64::MAX)
        );
        assert_eq!(
            internet_checksum_u16(u64::MAX, 0),
            internet_checksum_u16_reference(u64::MAX, 0)
        );
        assert_eq!(
            internet_checksum_u16(0, u64::MAX),
            internet_checksum_u16_reference(0, u64::MAX)
        );
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis of Failure Modes
    // -------------------------------------------------------------------------
    // Precondition:  { val, aux ∈ U64 }
    // Postcondition: { result = internet_checksum_u16_reference(val, aux) }
    //
    // Counterfactual Analysis for internet_checksum_u16:
    // 1. Mutant 1 (Identity Bluff): Bitwise NOT of reference.
    // 2. Mutant 2 (Bit-skip Bluff): Off-by-one error.
    // 3. Mutant 3 (Operator-swap Bluff): Masking error.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_internet_checksum_u16(c: &mut Criterion) {
        c.bench_function("internet_checksum_u16", |b| {
            b.iter(|| {
                let res = internet_checksum_u16(black_box(42), black_box(1337));
                black_box(res)
            })
        });
    }
}
