//! # Universe64 Contract: ScenarioRunner (Seeded Rollout)
//! Plane: D (block state) + S (scratch, deltas, receipt).
//! Tier: T2–T3 (multi-step rollout; bounded epoch count).
//! Scope: seeded PRNG drives instruction selection; bounded step count.
//! Geometry: steps × (admit → fire → delta → receipt) cycles.
//! Delta: each step emits UDelta; collected in DeltaTape.
//!
//! # Timing contract
//! - **Per-step budget:** ≤ T2_BUDGET_NS (one instruction cycle)
//! - **Full rollout:** ≤ T3_BUDGET_NS × MAX_SCENARIO_STEPS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES for single step; full rollout is T3-class.
//! CC=1: Absolute branchless logic in inner step kernel.

use super::constants::{UNIVERSE_WORDS, MAX_WORD_INDEX};
use super::block::UniverseBlock;
use super::scratch::{UDelta, UScope};
use super::receipt::{TransitionReceipt, receipt_mix_transition, new_receipt};
use super::coord::UCoord;
use super::delta_tape::DeltaTape;
use super::rl::{RLState, RLKernel, RewardTable};

/// Maximum steps in a bounded scenario rollout.
pub const MAX_SCENARIO_STEPS: usize = 1024;

/// Scenario configuration.
#[derive(Clone, Copy, Debug)]
pub struct ScenarioConfig {
    /// PRNG seed for instruction selection.
    pub seed: u64,
    /// Number of rollout steps (clamped to MAX_SCENARIO_STEPS).
    pub max_steps: u32,
    /// Fired mask applied to every step (usually !0).
    pub fired_mask: u64,
}

impl ScenarioConfig {
    pub const fn default_config() -> Self {
        Self {
            seed: 0xDEADBEEFCAFEBABE,
            max_steps: 256,
            fired_mask: u64::MAX,
        }
    }
}

/// Scenario run summary.
#[derive(Clone, Copy, Debug)]
pub struct ScenarioSummary {
    pub steps_executed: u32,
    pub total_bits_changed: u64,
    pub final_receipt: TransitionReceipt,
    pub final_reward: i64,
}

/// Branchless SplitMix64 PRNG step.
#[inline(always)]
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

// ---------------------------------------------------------------------------
// ScenarioRunner
// ---------------------------------------------------------------------------

/// Seeded rollout engine: drives a UniverseBlock through MAX_SCENARIO_STEPS
/// instruction cycles using a PRNG for word/bit selection.
pub struct ScenarioRunner;

impl ScenarioRunner {
    /// Run a bounded rollout. Returns scenario summary.
    pub fn run(
        block: &mut UniverseBlock,
        tape: &mut DeltaTape,
        rl_state: &mut RLState,
        reward_table: &RewardTable,
        config: &ScenarioConfig,
    ) -> ScenarioSummary {
        let mut rng = config.seed;
        let mut receipt = new_receipt();
        let steps = (config.max_steps as usize).min(MAX_SCENARIO_STEPS);
        let mut total_changed = 0u64;

        for step in 0..steps {
            // PRNG-driven word and bit selection.
            let r0 = splitmix64(&mut rng);
            let r1 = splitmix64(&mut rng);
            let word_idx = (r0 as usize) & MAX_WORD_INDEX;
            let bit_idx = (r1 & 63) as u32;
            let flip_mask = 1u64 << bit_idx;

            let before = block.state[word_idx];
            // Branchless XOR flip gated by fired_mask.
            let new_val = before ^ (flip_mask & config.fired_mask);
            block.state[word_idx] = new_val;

            let delta = UDelta::new(
                word_idx, UScope::Cell, step as u32, before, new_val, config.fired_mask,
            );
            tape.append(delta);
            RLKernel::update(rl_state, &delta, reward_table);
            total_changed = total_changed.wrapping_add((before ^ new_val).count_ones() as u64);

            // Mix into receipt.
            let coord = UCoord::new_const(
                ((word_idx / 64) & 63) as u8,
                (word_idx & 63) as u8,
                bit_idx as u8,
            );
            receipt = receipt_mix_transition(receipt.value(), coord, config.fired_mask);
        }

        ScenarioSummary {
            steps_executed: steps as u32,
            total_bits_changed: total_changed,
            final_receipt: receipt,
            final_reward: rl_state.reward,
        }
    }

    /// Run from a known-good state snapshot (for reproducibility testing).
    pub fn run_from_snapshot(
        snapshot: [u64; UNIVERSE_WORDS],
        config: &ScenarioConfig,
    ) -> ScenarioSummary {
        let mut block = UniverseBlock::new();
        block.state = snapshot;
        let mut tape = DeltaTape::new();
        let mut rl_state = RLState::zero();
        let reward_table = RewardTable::new_neutral();
        Self::run(&mut block, &mut tape, &mut rl_state, &reward_table, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splitmix64_nonzero() {
        let mut s = 42u64;
        let v = splitmix64(&mut s);
        assert_ne!(v, 0);
        assert_ne!(s, 42);
    }

    #[test]
    fn test_splitmix64_deterministic() {
        let mut s1 = 12345u64;
        let mut s2 = 12345u64;
        assert_eq!(splitmix64(&mut s1), splitmix64(&mut s2));
    }

    #[test]
    fn test_scenario_run_basic() {
        let config = ScenarioConfig { seed: 0xABC, max_steps: 10, fired_mask: !0 };
        let summary = ScenarioRunner::run_from_snapshot([0u64; UNIVERSE_WORDS], &config);
        assert_eq!(summary.steps_executed, 10);
        assert!(summary.total_bits_changed > 0);
    }

    #[test]
    fn test_scenario_deterministic() {
        let config = ScenarioConfig::default_config();
        let s1 = ScenarioRunner::run_from_snapshot([0u64; UNIVERSE_WORDS], &config);
        let s2 = ScenarioRunner::run_from_snapshot([0u64; UNIVERSE_WORDS], &config);
        assert_eq!(s1.final_receipt.value(), s2.final_receipt.value());
    }

    #[test]
    fn test_scenario_step_limit() {
        let config = ScenarioConfig { seed: 1, max_steps: 5000, fired_mask: !0 };
        let summary = ScenarioRunner::run_from_snapshot([0u64; UNIVERSE_WORDS], &config);
        assert_eq!(summary.steps_executed, MAX_SCENARIO_STEPS as u32);
    }
}
