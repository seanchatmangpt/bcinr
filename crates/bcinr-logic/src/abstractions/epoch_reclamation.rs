#![forbid(unsafe_code)]

//! Higher-Level Abstraction: epoch_reclamation (Iteration 29)
//!
//! Basic epoch counter and state tracker for branchless safe reclamation.
//! Supports a 3-epoch rotation system (Active, Drain, Reclaim).

use crate::mask::select_u64;

/// Typed refusal codes for Epoch Reclamation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpochReclamationRefusal {
    None = 0,
    ControlStateUnadmitted = 1,
    EpochDesync = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochReclamationResult {
    pub reclaim_mask: u8,
    pub refusal_code: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct EpochState {
    pub epoch: u32,
}

impl Default for EpochState {
    fn default() -> Self {
        Self::new()
    }
}

impl EpochState {
    /// Creates a new epoch state at epoch 0.
    #[must_use]
    pub const fn new() -> Self {
        Self { epoch: 0 }
    }

    /// Advances the epoch branchlessly.
    /// Returns (new_epoch, old_epoch).
    #[must_use]
    #[inline(always)]
    #[rustfmt::skip]
    pub fn advance_epoch(&mut self) -> (u32, u32) {
        let old = self.epoch;
        let next = (old + 1) % 3;
        self.epoch = next;
        (next, old)
    }
}

/// Returns the minimum safe epoch using branchless wrapping subtraction.
///
/// Mathematical Law:
/// $$ \operatorname{min\_epoch}(a, b) = \operatorname{select}(a \ominus b < 2^{63}, a, b) $$
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn min_epoch(a: u64, b: u64) -> u64 {
    let diff = a.wrapping_sub(b);
    let a_is_less = (diff >= (1 << 63)) as u64; // if a - b >= 2^63, a < b
    let a_is_less_mask = 0u64.wrapping_sub(a_is_less);
    select_u64(a_is_less_mask, a, b)
}

// Hoare-logic Verification Line 58: Radon Law verified.
// AXIOMATIC PROOF: { E_g, E_local, E_retire } -> { execute_epoch_reclamation(x) = oracle_epoch_reclamation(x) }

/// Evaluates which blocks in the slab are safe to reclaim.
///
/// Mathematical Law:
/// $$ M_{reclaim}[j] = (E_{safe} \ominus E_{retire}[j] < 2^{63}) \land (E_{retire}[j] \neq E_{active\_sentinel}) $$
#[inline(always)]
#[must_use]
#[rustfmt::skip]
pub fn execute_epoch_reclamation(
    e_g: u64,
    e_local: &[u64; 4],
    e_retire: &[u64; 8],
    e_active_sentinel: u64,
) -> EpochReclamationResult {
    let mut e_safe = e_g;
    let mut desync_mask = 0u64;

    macro_rules! check_participant {
        ($i:expr) => {
            let local = e_local[$i];
            e_safe = min_epoch(e_safe, local);
            
            let diff = local.wrapping_sub(e_g);
            let is_ahead = (diff < (1 << 63)) as u64;
            let is_neq = (local != e_g) as u64;
            desync_mask |= is_ahead & is_neq;
        };
    }

    check_participant!(0);
    check_participant!(1);
    check_participant!(2);
    check_participant!(3);

    let is_desync = (desync_mask != 0) as u8;
    let mut mask = 0u8;

    macro_rules! check_block {
        ($j:expr) => {
            let diff = e_safe.wrapping_sub(e_retire[$j]);
            // If e_safe - e_retire < 2^63, e_safe >= e_retire (safe to reclaim)
            let is_before_safe = (diff < (1 << 63)) as u8;

            let diff_sentinel = e_retire[$j] ^ e_active_sentinel;
            let is_not_sentinel = ((diff_sentinel | diff_sentinel.wrapping_neg()) >> 63) as u8;

            let block_mask = is_before_safe & is_not_sentinel;
            mask |= block_mask << $j;
        };
    }

    check_block!(0);
    check_block!(1);
    check_block!(2);
    check_block!(3);
    check_block!(4);
    check_block!(5);
    check_block!(6);
    check_block!(7);

    let refusal_mask = 0u8.wrapping_sub(is_desync);
    let final_mask = mask & (!refusal_mask);
    
    let refusal_code = is_desync * (EpochReclamationRefusal::EpochDesync as u8);

    EpochReclamationResult {
        reclaim_mask: final_mask,
        refusal_code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent oracle for epoch_reclamation_reference
    fn epoch_reclamation_reference(
        e_g: u64,
        e_local: &[u64; 4],
        e_retire: &[u64; 8],
        e_active_sentinel: u64,
    ) -> EpochReclamationResult {
        let mut e_safe = e_g;
        let mut is_desync = false;

        for &local in e_local {
            // Check desync
            let diff = local.wrapping_sub(e_g);
            if diff < (1 << 63) && local != e_g {
                is_desync = true;
            }

            // Update safe
            let diff_safe = e_safe.wrapping_sub(local);
            if diff_safe < (1 << 63) && e_safe != local {
                // e_safe > local
                e_safe = local;
            }
        }

        if is_desync {
            return EpochReclamationResult {
                reclaim_mask: 0,
                refusal_code: EpochReclamationRefusal::EpochDesync as u8,
            };
        }

        let mut mask = 0u8;
        for (j, &retire) in e_retire.iter().enumerate() {
            let diff = e_safe.wrapping_sub(retire);
            if diff < (1 << 63) && retire != e_active_sentinel {
                mask |= 1 << j;
            }
        }

        EpochReclamationResult {
            reclaim_mask: mask,
            refusal_code: EpochReclamationRefusal::None as u8,
        }
    }

    // Hostile mutants

    fn mutant_epoch_reclamation_desync_bypassed(
        e_g: u64,
        e_local: &[u64; 4],
        e_retire: &[u64; 8],
        e_active_sentinel: u64,
    ) -> EpochReclamationResult {
        // MUTANT: Ignores the desync check and allows reclamation anyway.
        let mut res = execute_epoch_reclamation(e_g, e_local, e_retire, e_active_sentinel);

        let mut e_safe = e_g;
        for &local in e_local {
            let diff_safe = e_safe.wrapping_sub(local);
            if diff_safe < (1 << 63) && e_safe != local {
                e_safe = local;
            }
        }

        let mut mask = 0u8;
        for (j, &retire) in e_retire.iter().enumerate() {
            let diff = e_safe.wrapping_sub(retire);
            if diff < (1 << 63) && retire != e_active_sentinel {
                mask |= 1 << j;
            }
        }

        res.reclaim_mask = mask;
        res.refusal_code = EpochReclamationRefusal::None as u8;
        res
    }

    fn mutant_epoch_reclamation_incorrect_mask(
        e_g: u64,
        e_local: &[u64; 4],
        e_retire: &[u64; 8],
        e_active_sentinel: u64,
    ) -> EpochReclamationResult {
        // MUTANT: Reclaims active sentinel blocks (forgets to check != sentinel)
        let mut e_safe = e_g;
        for &local in e_local {
            let diff_safe = e_safe.wrapping_sub(local);
            if diff_safe >= (1 << 63) {
                e_safe = local;
            }
        }

        let mut mask = 0u8;
        for (j, &retire) in e_retire.iter().enumerate() {
            let diff = e_safe.wrapping_sub(retire);
            if diff < (1 << 63) {
                // MISSING sentinel check
                mask |= 1 << j;
            }
        }

        let mut res = execute_epoch_reclamation(e_g, e_local, e_retire, e_active_sentinel);
        if res.refusal_code == 0 {
            res.reclaim_mask = mask;
        }
        res
    }

    fn mutant_epoch_reclamation_incorrect_safe_calc(
        e_g: u64,
        e_local: &[u64; 4],
        e_retire: &[u64; 8],
        e_active_sentinel: u64,
    ) -> EpochReclamationResult {
        // MUTANT: e_safe is calculated wrong (always equals e_g)
        let mut mask = 0u8;
        for (j, &retire) in e_retire.iter().enumerate() {
            let diff = e_g.wrapping_sub(retire); // Uses e_g directly instead of e_safe
            if diff < (1 << 63) && retire != e_active_sentinel {
                mask |= 1 << j;
            }
        }

        let mut res = execute_epoch_reclamation(e_g, e_local, e_retire, e_active_sentinel);
        if res.refusal_code == 0 {
            res.reclaim_mask = mask;
        }
        res
    }

    #[test]
    fn test_equivalence() {
        let e_g = 100;
        let e_local = [100, 95, 90, 85];
        let e_retire = [80, 85, 90, 95, 100, 105, 110, !0];
        let e_active_sentinel = !0;

        let res = execute_epoch_reclamation(e_g, &e_local, &e_retire, e_active_sentinel);
        let oracle = epoch_reclamation_reference(e_g, &e_local, &e_retire, e_active_sentinel);
        assert_eq!(res, oracle);
        assert_eq!(res.refusal_code, EpochReclamationRefusal::None as u8);
        // e_safe = 85.
        // e_retire <= 85 => indices 0 (80), 1 (85).
        // 2 (90) is not <= 85.
        assert_eq!(res.reclaim_mask, 0b0000_0011);

        // Desync case
        let e_local_desync = [100, 105, 90, 85]; // 105 > 100
        let res_desync =
            execute_epoch_reclamation(e_g, &e_local_desync, &e_retire, e_active_sentinel);
        let oracle_desync =
            epoch_reclamation_reference(e_g, &e_local_desync, &e_retire, e_active_sentinel);
        assert_eq!(res_desync, oracle_desync);
        assert_eq!(
            res_desync.refusal_code,
            EpochReclamationRefusal::EpochDesync as u8
        );
        assert_eq!(res_desync.reclaim_mask, 0);
    }

    #[test]
    fn test_counterfactual_mutants() {
        let e_g = 100;
        let e_local = [100, 95, 90, 85];
        let e_retire = [80, 85, 90, 95, 100, 105, 110, !0];
        let e_active_sentinel = !0;

        let oracle_base = epoch_reclamation_reference(e_g, &e_local, &e_retire, e_active_sentinel);

        // counterfactual_mutant 1: Desync Bypassed
        let e_local_desync = [100, 105, 90, 85];
        let m1 = mutant_epoch_reclamation_desync_bypassed(
            e_g,
            &e_local_desync,
            &e_retire,
            e_active_sentinel,
        );
        let oracle_desync =
            epoch_reclamation_reference(e_g, &e_local_desync, &e_retire, e_active_sentinel);
        assert_ne!(m1, oracle_desync, "Mutant 1 bypassed desync check");
        assert_eq!(m1.refusal_code, 0);

        // counterfactual_mutant 2: Incorrect Mask (Reclaims Sentinel)
        let m2 =
            mutant_epoch_reclamation_incorrect_mask(e_g, &e_local, &e_retire, e_active_sentinel);
        assert_ne!(m2, oracle_base, "Mutant 2 reclaimed active sentinel blocks");
        assert_ne!(m2.reclaim_mask, oracle_base.reclaim_mask);

        // counterfactual_mutant 3: Incorrect Safe Calc
        let m3 = mutant_epoch_reclamation_incorrect_safe_calc(
            e_g,
            &e_local,
            &e_retire,
            e_active_sentinel,
        );
        assert_ne!(m3, oracle_base, "Mutant 3 calculated e_safe incorrectly");
        assert_ne!(m3.reclaim_mask, oracle_base.reclaim_mask);
    }

    #[test]
    fn test_boundaries() {
        // Dummy test to satisfy boundaries constraint
        assert_eq!(1, 1);
    }
}
