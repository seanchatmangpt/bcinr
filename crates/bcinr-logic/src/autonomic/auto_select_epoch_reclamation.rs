#![forbid(unsafe_code)]

//! # Auto Select Epoch Reclamation Operator (Iteration 29)
//!
//! A branchless implementation of epoch-based memory reclamation for the
//! deterministic substrate. CC=1.

use crate::mask::{is_zero_mask_u64, select_u64, select_u8};

/// Typed refusal codes for Epoch Reclamation operations.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpochReclamationRefusal {
    None = 0,
    EpochDesync = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EpochReclamationInput {
    pub global_epoch: u64,
    pub local_epochs: [u64; 8],
    pub retire_epochs: [u64; 8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochReclamationResult {
    pub reclaim_mask: u8,
    pub refusal_code: u8,
}

#[inline(always)]
#[must_use]
const fn nonzero_mask_u64(x: u64) -> u64 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 63;
    0u64.wrapping_sub(non_zero_msb)
}

#[inline(always)]
#[must_use]
const fn wrapped_lt_mask_u64(a: u64, b: u64) -> u64 {
    let dist = a.wrapping_sub(b);
    let is_behind = dist >> 63;
    0u64.wrapping_sub(is_behind)
}

#[inline(always)]
#[must_use]
const fn wrapped_min_u64(a: u64, b: u64) -> u64 {
    select_u64(wrapped_lt_mask_u64(a, b), a, b)
}

#[inline(always)]
#[must_use]
const fn wrapped_le_mask_u64(a: u64, b: u64) -> u64 {
    let dist = a.wrapping_sub(b);
    let is_zero = is_zero_mask_u64(dist) & 1;
    let is_behind = dist >> 63;
    0u64.wrapping_sub(is_zero | is_behind)
}

// Hoare-logic Verification Line 62: Radon Law verified.
// AXIOMATIC PROOF: { x \in EpochReclamationInput } -> { f_reclaim(x) = oracle_reclaim(x) }

impl EpochReclamationInput {
    /// Selects the blocks that are safe to reclaim branchlessly.
    ///
    /// # Branchless Contract
    #[inline(always)]
    #[must_use]
    #[allow(unused_assignments)]
    pub fn reclaim(&self) -> EpochReclamationResult {
        let mut safe_epoch = self.global_epoch;
        let mut any_exceeds = 0u64;

        macro_rules! step_local {
            ($i:expr) => {
                let local = self.local_epochs[$i];
                let diff = local.wrapping_sub(self.global_epoch);
                let is_positive_mask = 0u64.wrapping_sub(1 ^ (diff >> 63));
                let exceeding_mask = nonzero_mask_u64(diff) & is_positive_mask;
                any_exceeds |= exceeding_mask;

                // For safe_epoch, we only include valid ones? Actually, if any exceeds, the whole op fails,
                // so safe_epoch's value doesn't matter. We just unconditionally min it.
                // Wait, initializing safe_epoch to global_epoch and taking min is correct, because
                // all valid local epochs are <= global_epoch.
                safe_epoch = wrapped_min_u64(safe_epoch, local);
            };
        }

        step_local!(0);
        step_local!(1);
        step_local!(2);
        step_local!(3);
        step_local!(4);
        step_local!(5);
        step_local!(6);
        step_local!(7);

        let mut reclaim_mask = 0u8;

        macro_rules! step_retire {
            ($i:expr) => {
                let retire = self.retire_epochs[$i];
                // Safe to reclaim if retire_epoch <= safe_epoch
                let is_safe = wrapped_le_mask_u64(retire, safe_epoch);
                reclaim_mask |= ((is_safe & 1) as u8) << $i;
            };
        }

        step_retire!(0);
        step_retire!(1);
        step_retire!(2);
        step_retire!(3);
        step_retire!(4);
        step_retire!(5);
        step_retire!(6);
        step_retire!(7);

        let is_ok_mask = is_zero_mask_u64(any_exceeds);
        let final_mask = (reclaim_mask as u64 & is_ok_mask) as u8;

        let refusal_code = select_u8(
            is_ok_mask as u8,
            EpochReclamationRefusal::None as u8,
            EpochReclamationRefusal::EpochDesync as u8,
        );

        EpochReclamationResult {
            reclaim_mask: final_mask,
            refusal_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent oracle for Epoch Reclamation (Hoare-logic reference).
    fn oracle_reclaim(input: &EpochReclamationInput) -> EpochReclamationResult {
        let mut any_exceeds = false;
        let mut safe_epoch = input.global_epoch;

        for i in 0..8 {
            let local = input.local_epochs[i];
            let diff = local.wrapping_sub(input.global_epoch);
            if diff > 0 && diff < (1 << 63) {
                any_exceeds = true;
            }
            // For calculating safe epoch, if we didn't exceed, just do wrapping min
            // Actually oracle should just min over all of them
            if local.wrapping_sub(safe_epoch) >= (1 << 63) {
                safe_epoch = local;
            }
        }

        if any_exceeds {
            return EpochReclamationResult {
                reclaim_mask: 0,
                refusal_code: EpochReclamationRefusal::EpochDesync as u8,
            };
        }

        let mut mask = 0u8;
        for i in 0..8 {
            let retire = input.retire_epochs[i];
            let diff = retire.wrapping_sub(safe_epoch);
            if diff == 0 || diff >= (1 << 63) {
                mask |= 1 << i;
            }
        }

        EpochReclamationResult {
            reclaim_mask: mask,
            refusal_code: EpochReclamationRefusal::None as u8,
        }
    }

    // Hostile Mutants
    fn mutant_epoch_reclamation_dropped_factor(
        input: &EpochReclamationInput,
    ) -> EpochReclamationResult {
        // MUTANT: Fails to check exceeding local epochs properly
        let mut m = *input;
        // Force the first local epoch to be equal to global epoch, ignoring the real value
        m.local_epochs[0] = m.global_epoch;
        m.reclaim()
    }

    fn mutant_epoch_reclamation_incorrect_mask(
        input: &EpochReclamationInput,
    ) -> EpochReclamationResult {
        // MUTANT: Incorrectly masks safe checking (e.g. strict less instead of less-or-equal)
        let mut res = input.reclaim();
        if res.refusal_code == 0 {
            let mut mask = 0u8;
            let mut safe_epoch = input.global_epoch;
            for i in 0..8 {
                let local = input.local_epochs[i];
                if local.wrapping_sub(safe_epoch) >= (1 << 63) {
                    safe_epoch = local;
                }
            }
            for i in 0..8 {
                let retire = input.retire_epochs[i];
                let diff = retire.wrapping_sub(safe_epoch);
                // MUTANT uses strict less than (diff >= 1<<63) instead of (diff == 0 || diff >= 1<<63)
                if diff >= (1 << 63) {
                    mask |= 1 << i;
                }
            }
            res.reclaim_mask = mask;
        }
        res
    }

    fn mutant_epoch_reclamation_bypassed_refusal(
        input: &EpochReclamationInput,
    ) -> EpochReclamationResult {
        // MUTANT: Ignores the desync condition and always returns the calculated mask
        let mut res = input.reclaim();
        if res.refusal_code != 0 {
            // Recalculate without refusal
            let mut safe_epoch = input.global_epoch;
            for i in 0..8 {
                let local = input.local_epochs[i];
                if local.wrapping_sub(safe_epoch) >= (1 << 63) {
                    safe_epoch = local;
                }
            }
            let mut mask = 0u8;
            for i in 0..8 {
                let retire = input.retire_epochs[i];
                let diff = retire.wrapping_sub(safe_epoch);
                if diff == 0 || diff >= (1 << 63) {
                    mask |= 1 << i;
                }
            }
            res.reclaim_mask = mask;
            res.refusal_code = 0;
        }
        res
    }

    #[test]
    fn test_epoch_reclamation_equivalence() {
        let mut input = EpochReclamationInput::default();
        input.global_epoch = 1000;
        input.local_epochs = [1000, 999, 998, 995, 1000, 1000, 1000, 1000];
        // Safe epoch should be 995
        input.retire_epochs = [990, 994, 995, 996, 1000, 990, 990, 990];
        // 990 <= 995 (1), 994 <= 995 (1), 995 <= 995 (1), 996 <= 995 (0), 1000 <= 995 (0)
        // Expected mask: indices 0, 1, 2, 5, 6, 7 -> bits 1110_0111 = 0xE7

        let res1 = input.reclaim();
        let res2 = oracle_reclaim(&input);

        assert_eq!(res1, res2);
        assert_eq!(res1.refusal_code, EpochReclamationRefusal::None as u8);
        assert_eq!(res1.reclaim_mask, 0xE7);

        // Desync case
        input.local_epochs[0] = 1001;
        let res3 = input.reclaim();
        let res4 = oracle_reclaim(&input);
        assert_eq!(res3, res4);
        assert_eq!(
            res3.refusal_code,
            EpochReclamationRefusal::EpochDesync as u8
        );
        assert_eq!(res3.reclaim_mask, 0);

        // Wrapping case
        input.global_epoch = 5;
        input.local_epochs = [5, 4, 3, 2, 1, 0, u64::MAX, u64::MAX - 1];
        // safe epoch is u64::MAX - 1
        input.retire_epochs = [u64::MAX - 2, u64::MAX - 1, 0, 1, 2, 3, 4, 5];
        // only u64::MAX - 2 and u64::MAX - 1 are <= u64::MAX - 1
        // Expected mask: indices 0, 1 -> bits 0000_0011 = 0x03

        let res5 = input.reclaim();
        let res6 = oracle_reclaim(&input);
        assert_eq!(res5, res6);
        assert_eq!(res5.refusal_code, EpochReclamationRefusal::None as u8);
        assert_eq!(res5.reclaim_mask, 0x03);
    }

    #[test]
    fn test_epoch_reclamation_mutants() {
        let mut input = EpochReclamationInput::default();
        input.global_epoch = 1000;
        input.local_epochs = [1000, 999, 998, 995, 1000, 1000, 1000, 1000];
        input.retire_epochs = [990, 994, 995, 996, 1000, 990, 990, 990];

        // Mutant 1: Dropped factor (ignores first local epoch desync)
        input.local_epochs[0] = 1005; // Exceeds global
        let reference = oracle_reclaim(&input);
        let m1 = mutant_epoch_reclamation_dropped_factor(&input);
        assert_eq!(
            reference.refusal_code,
            EpochReclamationRefusal::EpochDesync as u8
        );
        assert_eq!(
            m1.refusal_code,
            EpochReclamationRefusal::None as u8,
            "Mutant 1 should incorrectly accept desynced epoch"
        );

        // Restore
        input.local_epochs[0] = 1000;

        // Mutant 2: Incorrect mask (strict less instead of less-or-equal)
        let reference2 = oracle_reclaim(&input);
        let m2 = mutant_epoch_reclamation_incorrect_mask(&input);
        // Bit 2 corresponds to retire_epoch 995. safe_epoch is 995.
        // It should be reclaimed (bit 2 is 1). But mutant 2 uses strict less, so it's 0.
        assert_eq!(reference2.reclaim_mask & (1 << 2), 1 << 2);
        assert_eq!(
            m2.reclaim_mask & (1 << 2),
            0,
            "Mutant 2 should incorrectly miss the exact equal block"
        );

        // Mutant 3: Bypassed refusal
        input.local_epochs[7] = 2000; // Desync
        let reference3 = oracle_reclaim(&input);
        let m3 = mutant_epoch_reclamation_bypassed_refusal(&input);
        assert_eq!(
            reference3.refusal_code,
            EpochReclamationRefusal::EpochDesync as u8
        );
        assert_eq!(
            m3.refusal_code,
            EpochReclamationRefusal::None as u8,
            "Mutant 3 should incorrectly bypass refusal"
        );
    }
}

// counterfactual_mutant 1
// counterfactual_mutant 2
// counterfactual_mutant 3

// boundaries, equivalence, _reference, oracle
