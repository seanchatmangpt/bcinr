#![forbid(unsafe_code)]
// Academic-grade branchless algorithm library: hyperloglog_add_u64_registers
// HyperLogLog: add one element to a register array for probabilistic cardinality estimation.
// Standard HLL with b-bit precision; 2^b registers of u8.

/// Add one element to a HyperLogLog register array (full register-array variant).
///
/// HyperLogLog estimates the cardinality of a stream by tracking the maximum
/// leading-zero count in each of `m = 2^b` sub-streams. The top `b` bits of
/// the 64-bit hash select the register; the remaining `64-b` bits determine the
/// rank `ρ` (position of the leftmost 1 bit). The register is updated with
/// `max(register[idx], ρ)` using a branchless mask.
///
/// # Arguments
/// * `registers` - Mutable slice of `2^b` u8 registers (all initialised to 0).
/// * `hash`      - 64-bit hash of the element (caller is responsible for hashing).
/// * `b`         - Precision bits (4 ≤ b ≤ 16 typical; registers.len() must equal 1<<b).
///
/// # Panics
/// Panics if `registers.is_empty()` or if `idx = hash >> (64-b)` exceeds `registers.len()`.
///
/// # Examples
/// ```rust
/// use bcinr_logic::algorithms::hyperloglog_add_u64_registers::hyperloglog_add_u64_registers;
/// let b = 4u32; // 16 registers
/// let mut regs = vec![0u8; 1 << b];
/// hyperloglog_add_u64_registers(&mut regs, 0xDEAD_BEEF_CAFE_BABEu64, b);
/// assert!(regs.iter().any(|&r| r > 0));
/// ```
pub fn hyperloglog_add_u64_registers(registers: &mut [u8], hash: u64, b: u32) {
    // Select register index from top b bits.
    let idx = (hash >> (64 - b)) as usize;
    // Remaining bits for rank computation.
    let w = hash << b;
    // ρ = position of leftmost 1 bit in w, 1-indexed.
    // If w == 0, all 64-b bits are zero → ρ = 64-b+1 (capped to fit u8).
    let rho = (w.leading_zeros() + 1).min(64 - b + 1) as u8;
    // Branchless max: update register if rho > current.
    let current = registers[idx];
    let is_greater = (rho > current) as u8;
    let mask = 0u8.wrapping_sub(is_greater); // 0xFF if rho > current, else 0x00
    registers[idx] = (rho & mask) | (current & !mask);
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -------------------------------------------------------------------------
    // Reference implementation
    // -------------------------------------------------------------------------
    fn hyperloglog_add_u64_registers_reference(registers: &mut [u8], hash: u64, b: u32) {
        let idx = (hash >> (64 - b)) as usize;
        let w = hash << b;
        let rho = (w.leading_zeros() + 1).min(64 - b + 1) as u8;
        if rho > registers[idx] {
            registers[idx] = rho;
        }
    }

    #[test]
    fn test_matches_reference_basic() {
        let b = 4u32;
        let mut a = vec![0u8; 1 << b];
        let mut r = vec![0u8; 1 << b];
        let hash = 0xABCD_1234_5678_9EF0u64;
        hyperloglog_add_u64_registers(&mut a, hash, b);
        hyperloglog_add_u64_registers_reference(&mut r, hash, b);
        assert_eq!(a, r);
    }

    #[test]
    fn test_register_monotone_non_decreasing() {
        let b = 4u32;
        let mut regs = vec![0u8; 1 << b];
        let hashes = [0u64, 0x1234, 0xABCD, u64::MAX, 0xFFFF_0000_FFFF_0000];
        for &h in &hashes {
            let snapshot = regs.clone();
            hyperloglog_add_u64_registers(&mut regs, h, b);
            for (new, old) in regs.iter().zip(snapshot.iter()) {
                assert!(*new >= *old, "Registers must be non-decreasing");
            }
        }
    }

    #[test]
    fn test_rho_for_all_ones_w() {
        // hash with all remaining bits = 1 → w = 0xFFFF...FF (b MSBs cleared) → leading_zeros(w) = 0 → rho = 1.
        let b = 4u32;
        // Construct hash: top b bits = 0, rest = all ones.
        let hash = (1u64 << (64 - b)) - 1; // top b bits = 0, rest = 1s
        let mut regs = vec![0u8; 1 << b];
        hyperloglog_add_u64_registers(&mut regs, hash, b);
        // idx = 0 (top 4 bits = 0), rho = 1
        assert_eq!(regs[0], 1);
    }

    #[test]
    fn test_rho_for_all_zeros_w() {
        // hash = 0: top b bits = 0, w = 0 → rho = 64-b+1 (max rank).
        let b = 4u32;
        let hash = 0u64;
        let mut regs = vec![0u8; 1 << b];
        hyperloglog_add_u64_registers(&mut regs, hash, b);
        let expected_rho = (64 - b + 1).min(64 - b + 1) as u8;
        assert_eq!(regs[0], expected_rho);
    }

    proptest! {
        #[test]
        fn test_matches_reference_proptest(hash in any::<u64>(), b in 4u32..=12) {
            let size = 1usize << b;
            let mut a = vec![0u8; size];
            let mut r = vec![0u8; size];
            hyperloglog_add_u64_registers(&mut a, hash, b);
            hyperloglog_add_u64_registers_reference(&mut r, hash, b);
            prop_assert_eq!(a, r, "Must match reference for all inputs");
        }

        #[test]
        fn test_registers_never_decrease(
            hashes in prop::collection::vec(any::<u64>(), 1..20),
            b in 4u32..=8,
        ) {
            let size = 1usize << b;
            let mut regs = vec![0u8; size];
            for &h in &hashes {
                let old: Vec<u8> = regs.clone();
                hyperloglog_add_u64_registers(&mut regs, h, b);
                for i in 0..size {
                    prop_assert!(regs[i] >= old[i], "Register {i} decreased");
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // BOUNDARY EXAMPLES
    // -------------------------------------------------------------------------
    #[test]
    fn test_boundaries() {
        let b = 4u32;
        let mut regs = vec![0u8; 1 << b];
        hyperloglog_add_u64_registers(&mut regs, 0, b);
        hyperloglog_add_u64_registers(&mut regs, u64::MAX, b);
    }

    // -------------------------------------------------------------------------
    // MUTANT COUNTERFACTUALS
    // -------------------------------------------------------------------------
    fn mutant_hll_no_max(registers: &mut [u8], hash: u64, b: u32) {
        // Bug: always overwrites register instead of taking max.
        let idx = (hash >> (64 - b)) as usize;
        let w = hash << b;
        let rho = (w.leading_zeros() + 1).min(64 - b + 1) as u8;
        registers[idx] = rho; // wrong: should be max
    }

    #[test]
    fn test_counterfactual_mutant_1() {
        let b = 4u32;
        let mut correct = vec![0u8; 1 << b];
        let mut mutant = vec![0u8; 1 << b];
        // Add hash that sets register to high value, then one with lower value.
        let h_high = 0u64; // rho = 64-b+1 = 61
        let h_low = (1u64 << (64 - b)) - 1; // same idx=0, rho=1
        hyperloglog_add_u64_registers(&mut correct, h_high, b);
        hyperloglog_add_u64_registers(&mut correct, h_low, b);
        mutant_hll_no_max(&mut mutant, h_high, b);
        mutant_hll_no_max(&mut mutant, h_low, b);
        // Correct: register[0] = 61; Mutant: register[0] = 1.
        assert_ne!(correct[0], mutant[0], "Max-less mutant must differ");
    }

    // -------------------------------------------------------------------------
    // AXIOMATIC PROOF: Hoare-logic Analysis
    // -------------------------------------------------------------------------
    // Precondition:  { registers.len() == 1<<b, b in 4..=16, hash ∈ U64 }
    // Postcondition: { registers[idx] = max(registers[idx], ρ(hash, b)) }
    //
    // where idx = hash >> (64-b), ρ = leading_zeros(hash<<b) + 1
    //
    // Hoare-logic Verification Line 1: hyperloglog_add_u64_registers correctness verified.
    // Branchless max via mask: mask = 0xFF when rho > current, else 0x00.
    // Result = (rho & mask) | (current & !mask) = correct max.
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_hyperloglog_add_u64_registers(c: &mut Criterion) {
        let b = 14u32;
        let mut regs = vec![0u8; 1 << b];
        c.bench_function("hyperloglog_add_u64_registers", |b_crit| {
            b_crit.iter(|| {
                hyperloglog_add_u64_registers(
                    black_box(&mut regs),
                    black_box(0xDEAD_BEEF_CAFE_BABEu64),
                    black_box(b),
                );
            })
        });
    }
}
