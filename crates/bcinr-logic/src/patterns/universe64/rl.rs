//! # Universe64 Contract: RLKernel + RewardKernel
//! Plane: S — reads deltas; maintains running reward state.
//! Tier: T0 per-delta reward; T1 batch accumulation.
//! Scope: stateful reward accumulator; bounded good/bad masks.
//! Geometry: reward = popcount(ΔU & good_mask) − popcount(ΔU & bad_mask).
//! Delta: reads UDelta changed_mask; emits no new deltas.
//!
//! # Timing contract
//! - **T0 per-delta:** ≤ T0_BUDGET_NS
//! - **T1 batch:** ≤ T1_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Branchless popcount arithmetic on u64 masks.
//! CC=1: Absolute branchless logic.

use super::constants::UNIVERSE_WORDS;
use super::scratch::UDelta;

// ---------------------------------------------------------------------------
// RewardKernel — per-delta signed reward
// ---------------------------------------------------------------------------

/// Per-word reward specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardSpec {
    /// Bits that are good to set (reward += popcount(ΔU & good_bits)).
    pub good_bits: u64,
    /// Bits that are bad to set (reward -= popcount(ΔU & bad_bits)).
    pub bad_bits: u64,
}

impl RewardSpec {
    pub const NEUTRAL: Self = Self { good_bits: 0, bad_bits: 0 };

    /// Compute signed reward from a changed-bits mask.
    #[inline(always)]
    pub fn reward_for_change(&self, changed: u64) -> i32 {
        let good = (changed & self.good_bits).count_ones() as i32;
        let bad = (changed & self.bad_bits).count_ones() as i32;
        good - bad
    }
}

/// Global reward table: one RewardSpec per universe word.
pub struct RewardTable {
    pub specs: [RewardSpec; UNIVERSE_WORDS],
}

impl RewardTable {
    pub fn new_neutral() -> Self {
        Self { specs: [RewardSpec::NEUTRAL; UNIVERSE_WORDS] }
    }

    pub fn set_word(&mut self, word_idx: usize, spec: RewardSpec) {
        self.specs[word_idx & (UNIVERSE_WORDS - 1)] = spec;
    }

    /// Compute reward contribution from a single delta.
    #[inline(always)]
    pub fn reward_delta(&self, delta: &UDelta) -> i32 {
        let widx = delta.word_idx as usize & (UNIVERSE_WORDS - 1);
        self.specs[widx].reward_for_change(delta.changed_mask())
    }
}

// ---------------------------------------------------------------------------
// RLState — running accumulated reward
// ---------------------------------------------------------------------------

/// Running RL state maintained per epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RLState {
    /// Accumulated reward (can be negative).
    pub reward: i64,
    /// Number of deltas observed.
    pub delta_count: u32,
    /// Epoch sequence number.
    pub epoch: u32,
}

impl RLState {
    pub const fn zero() -> Self {
        Self { reward: 0, delta_count: 0, epoch: 0 }
    }

    /// True if this epoch has a positive reward.
    #[inline(always)]
    pub const fn is_positive(&self) -> bool {
        self.reward > 0
    }

    /// Normalize reward per delta (returns 0 when no deltas).
    #[inline(always)]
    pub fn mean_reward_x1000(&self) -> i64 {
        let n = self.delta_count.max(1) as i64;
        (self.reward * 1000) / n
    }
}

// ---------------------------------------------------------------------------
// RLKernel
// ---------------------------------------------------------------------------

/// Stateless RL kernel: updates RLState from UDelta stream.
pub struct RLKernel;

impl RLKernel {
    /// T0: accumulate reward from one delta.
    #[inline(always)]
    pub fn update(state: &mut RLState, delta: &UDelta, table: &RewardTable) {
        let r = table.reward_delta(delta);
        state.reward = state.reward.wrapping_add(r as i64);
        state.delta_count = state.delta_count.wrapping_add(1);
    }

    /// T1: accumulate reward from a batch of deltas.
    pub fn update_batch(state: &mut RLState, deltas: &[UDelta], table: &RewardTable) {
        for d in deltas {
            Self::update(state, d, table);
        }
    }

    /// Advance to next epoch: resets reward and counter, increments epoch.
    #[inline(always)]
    pub fn next_epoch(state: &mut RLState) {
        state.reward = 0;
        state.delta_count = 0;
        state.epoch = state.epoch.wrapping_add(1);
    }

    /// Branchless policy: returns 1 (continue) if reward >= threshold, 0 (stop) otherwise.
    #[inline(always)]
    pub fn policy_continue(state: &RLState, threshold: i64) -> u8 {
        (state.reward >= threshold) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_delta_change(word_idx: usize, changed: u64) -> UDelta {
        UDelta::new(word_idx, super::super::scratch::UScope::Cell, 0, 0, changed, !0)
    }

    #[test]
    fn test_reward_spec_positive() {
        let spec = RewardSpec { good_bits: 0b1111, bad_bits: 0 };
        assert_eq!(spec.reward_for_change(0b0011), 2);
    }

    #[test]
    fn test_reward_spec_negative() {
        let spec = RewardSpec { good_bits: 0, bad_bits: 0b1111 };
        assert_eq!(spec.reward_for_change(0b0011), -2);
    }

    #[test]
    fn test_reward_spec_mixed() {
        let spec = RewardSpec { good_bits: 0b0001, bad_bits: 0b0010 };
        // changed = 0b0011 → good=1, bad=1 → reward=0
        assert_eq!(spec.reward_for_change(0b0011), 0);
    }

    #[test]
    fn test_rl_kernel_update() {
        let mut table = RewardTable::new_neutral();
        table.set_word(0, RewardSpec { good_bits: 0xFF, bad_bits: 0 });
        let mut state = RLState::zero();
        let d = make_delta_change(0, 0b11111111);
        RLKernel::update(&mut state, &d, &table);
        assert_eq!(state.reward, 8);
        assert_eq!(state.delta_count, 1);
    }

    #[test]
    fn test_rl_next_epoch() {
        let mut state = RLState { reward: 100, delta_count: 50, epoch: 2 };
        RLKernel::next_epoch(&mut state);
        assert_eq!(state.reward, 0);
        assert_eq!(state.delta_count, 0);
        assert_eq!(state.epoch, 3);
    }

    #[test]
    fn test_policy_continue() {
        let state = RLState { reward: 5, delta_count: 1, epoch: 0 };
        assert_eq!(RLKernel::policy_continue(&state, 5), 1);
        assert_eq!(RLKernel::policy_continue(&state, 6), 0);
    }
}
