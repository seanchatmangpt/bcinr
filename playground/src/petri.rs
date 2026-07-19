#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::inline_always,
)]
//! Branchless Petri net token replay engine implementation.
//! Adheres strictly to bcinr's Radon Law (CC=1), zero-alloc, and no_std constraints.

use bcinr_logic::{
    int::popcount_u64,
    mask::{is_zero_mask_u32, lt_mask_u32, min_u32, select_u32, select_u64},
};

/// Results of the token replay operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ReplayResult {
    /// Count of missing tokens supplied on demand.
    pub missing: u32,
    /// Count of remaining tokens after replay completion.
    pub remaining: u32,
    /// Count of produced tokens.
    pub produced: u32,
    /// Count of consumed tokens.
    pub consumed: u32,
}

impl ReplayResult {
    /// Creates a new `ReplayResult`.
    #[must_use]
    pub const fn new(missing: u32, remaining: u32, produced: u32, consumed: u32) -> Self {
        Self { missing, remaining, produced, consumed }
    }

    /// Calculates the fitness score branchlessly.
    ///
    /// Fitness is computed as:
    /// `1.0 - (missing + remaining) / (consumed + missing + produced)`
    /// Returns 1.0 if the denominator is 0.
    #[must_use]
    pub fn fitness(&self) -> f64 {
        let denom_u32 = self.consumed.wrapping_add(self.missing).wrapping_add(self.produced);
        let is_zero = is_zero_mask_u32(denom_u32);
        let safe_denom = select_u32(is_zero, 1, denom_u32);

        let sum_num = self.missing.wrapping_add(self.remaining);
        let raw_fitness = 1.0 - (f64::from(sum_num) / f64::from(safe_denom));

        let is_zero_64 = (u64::from(is_zero)) | (u64::from(is_zero) << 32);
        let fit_bits = select_u64(is_zero_64, (1.0f64).to_bits(), raw_fitness.to_bits());
        f64::from_bits(fit_bits)
    }

    /// Returns true if the replay was perfect (no missing and no remaining tokens).
    #[must_use]
    pub fn is_perfect(&self) -> bool {
        let sum = self.missing | self.remaining;
        sum == 0
    }
}

/// Constant-time Petri transition firing step (CC = 1).
///
/// Missing tokens are supplied on demand.
#[inline(always)]
pub fn petri_fire_transition(
    marking: &mut u64,
    in_mask: u64,
    out_mask: u64,
    missing: &mut u32,
    consumed: &mut u32,
    produced: &mut u32,
) {
    let need = in_mask & !(*marking);
    *missing = (*missing).wrapping_add(popcount_u64(need) as u32);
    *marking |= need;

    *marking = (*marking & !in_mask) | out_mask;
    *consumed = (*consumed).wrapping_add(popcount_u64(in_mask) as u32);
    *produced = (*produced).wrapping_add(popcount_u64(out_mask) as u32);
}

/// Constant-time invisible transition closure.
///
/// Bounded to a fixed count (16x16 iterations) to comply with CC = 1.
#[inline(always)]
pub fn petri_fire_invisible(marking: &mut u64, inv_in_masks: &[u64], inv_out_masks: &[u64]) {
    let len_in = inv_in_masks.len();
    let len_out = inv_out_masks.len();
    let len = min_u32(len_in as u32, len_out as u32) as usize;

    let mut in_masks = [0u64; 16];
    let mut out_masks = [0u64; 16];
    let mut valid_mask = [0u32; 16];

    for i in 0..16 {
        let is_valid = lt_mask_u32(i as u32, len as u32);
        valid_mask[i] = is_valid;
        let safe_idx = select_u32(is_valid, i as u32, 0) as usize;
        in_masks[i] = *inv_in_masks.get(safe_idx).unwrap_or(&0);
        out_masks[i] = *inv_out_masks.get(safe_idx).unwrap_or(&0);
    }

    for _ in 0..16 {
        let mut already_fired = 0u64;
        for i in 0..16 {
            let is_valid = valid_mask[i];
            let in_mask = in_masks[i];
            let out_mask = out_masks[i];

            // Is it enabled? (marking & in_mask) == in_mask
            let diff = (*marking & in_mask) ^ in_mask;
            let lo_zero = is_zero_mask_u32(diff as u32);
            let hi_zero = is_zero_mask_u32((diff >> 32) as u32);
            let zero_mask_64 = u64::from(lo_zero & hi_zero) | (u64::from(lo_zero & hi_zero) << 32);

            let is_valid_64 = u64::from(is_valid) | (u64::from(is_valid) << 32);
            let is_enabled = zero_mask_64 & is_valid_64;

            // Can it fire? (is_enabled & not already_fired)
            let can_fire = is_enabled & !already_fired;
            already_fired |= can_fire;

            let fired_marking = (*marking & !in_mask) | out_mask;
            *marking = select_u64(can_fire, fired_marking, *marking);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_firing() {
        let mut marking = 1u64; // Token at place 0
        let mut missing = 0u32;
        let mut consumed = 0u32;
        let mut produced = 0u32;

        // Transition: consumes place 0 (bit 0), produces place 1 (bit 1)
        petri_fire_transition(&mut marking, 1, 2, &mut missing, &mut consumed, &mut produced);

        assert_eq!(marking, 2);
        assert_eq!(missing, 0);
        assert_eq!(consumed, 1);
        assert_eq!(produced, 1);
    }

    #[test]
    fn test_firing_with_missing_tokens() {
        let mut marking = 0u64; // No tokens
        let mut missing = 0u32;
        let mut consumed = 0u32;
        let mut produced = 0u32;

        // Transition: consumes place 0 (bit 0), produces place 1 (bit 1)
        petri_fire_transition(&mut marking, 1, 2, &mut missing, &mut consumed, &mut produced);

        assert_eq!(marking, 2);
        assert_eq!(missing, 1);
        assert_eq!(consumed, 1);
        assert_eq!(produced, 1);
    }

    #[test]
    fn test_invisible_firing_closure() {
        let mut marking = 1u64; // Token at place 0

        // Invisible transition: consumes place 0 (bit 0), produces place 1 (bit 1)
        let inv_in = [1u64];
        let inv_out = [2u64];

        petri_fire_invisible(&mut marking, &inv_in, &inv_out);
        assert_eq!(marking, 2);
    }

    #[test]
    fn test_invisible_firing_chain() {
        let mut marking = 1u64; // Token at place 0

        // Chain of invisible transitions:
        // T1: 0 -> 1 (bit 0 -> bit 1)
        // T2: 1 -> 2 (bit 1 -> bit 2)
        let inv_in = [1u64, 2u64];
        let inv_out = [2u64, 4u64];

        petri_fire_invisible(&mut marking, &inv_in, &inv_out);
        assert_eq!(marking, 4);
    }

    #[test]
    fn test_invisible_firing_empty() {
        let mut marking = 5u64;
        petri_fire_invisible(&mut marking, &[], &[]);
        assert_eq!(marking, 5);
    }

    #[test]
    fn test_invisible_firing_no_match() {
        let mut marking = 1u64;
        let inv_in = [2u64];
        let inv_out = [4u64];
        petri_fire_invisible(&mut marking, &inv_in, &inv_out);
        assert_eq!(marking, 1);
    }

    #[test]
    fn test_replay_result_fitness_perfect() {
        let res = ReplayResult::new(0, 0, 2, 2);
        assert!(res.is_perfect());
        assert!((res.fitness() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_replay_result_fitness_imperfect() {
        let res = ReplayResult::new(1, 1, 2, 2);
        assert!(!res.is_perfect());
        // denom = consumed + missing + produced = 2 + 1 + 2 = 5
        // fitness = 1.0 - (missing + remaining) / denom = 1.0 - (1 + 1) / 5 = 0.6
        assert!((res.fitness() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_replay_result_fitness_zero() {
        let res = ReplayResult::default();
        assert!(res.is_perfect());
        assert!((res.fitness() - 1.0).abs() < 1e-9);
    }
}
