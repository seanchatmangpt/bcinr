#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::inline_always
)]
//! Branchless YAWL routing semantics engine.

use bcinr::int::popcount_u64;
use bcinr_logic::simd_dispatch::{
    add_saturating_u8x16, and_u8x16, blend_u8x16, compare_eq_u8x16, splat_u8x16,
};

#[inline(always)]
fn unpack_u64_mask_to_u8x16(mask: u64, offset: u32) -> [u8; 16] {
    let shifted = mask >> offset;
    let mut out = [0u8; 16];
    out[0] = ((shifted & 1) as u8).wrapping_neg();
    out[1] = (((shifted >> 1) & 1) as u8).wrapping_neg();
    out[2] = (((shifted >> 2) & 1) as u8).wrapping_neg();
    out[3] = (((shifted >> 3) & 1) as u8).wrapping_neg();
    out[4] = (((shifted >> 4) & 1) as u8).wrapping_neg();
    out[5] = (((shifted >> 5) & 1) as u8).wrapping_neg();
    out[6] = (((shifted >> 6) & 1) as u8).wrapping_neg();
    out[7] = (((shifted >> 7) & 1) as u8).wrapping_neg();
    out[8] = (((shifted >> 8) & 1) as u8).wrapping_neg();
    out[9] = (((shifted >> 9) & 1) as u8).wrapping_neg();
    out[10] = (((shifted >> 10) & 1) as u8).wrapping_neg();
    out[11] = (((shifted >> 11) & 1) as u8).wrapping_neg();
    out[12] = (((shifted >> 12) & 1) as u8).wrapping_neg();
    out[13] = (((shifted >> 13) & 1) as u8).wrapping_neg();
    out[14] = (((shifted >> 14) & 1) as u8).wrapping_neg();
    out[15] = (((shifted >> 15) & 1) as u8).wrapping_neg();
    out
}

#[inline(always)]
const fn nz_mask_u64(x: u64) -> u64 {
    (((x | x.wrapping_neg()) as i64) >> 63) as u64
}

#[inline(always)]
const fn z_mask_u64(x: u64) -> u64 {
    !nz_mask_u64(x)
}

/// The type of join to perform.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JoinType {
    /// XOR join: fires if exactly one incoming token is present.
    XOR = 0,
    /// AND join: fires if all incoming tokens are present.
    AND = 1,
    /// OR join (Synchronizing Merge): fires when no further tokens can reach the task.
    OR = 2,
    /// Complex join (N-out-of-M, Discriminator): fires once when threshold is met.
    Complex = 3,
    /// `ThreadMerge` join: fires if any incoming token is present.
    ThreadMerge = 4,
}

/// The type of split to perform.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitType {
    /// XOR split.
    XOR = 0,
    /// AND split.
    AND = 1,
    /// OR split.
    OR = 2,
    /// Multi-instance split: spawns multiple static instances.
    MultiInstance = 3,
    /// Dynamic multi-instance split: spawns instances dynamically based on triggers.
    DynamicMultiInstance = 4,
    /// Deferred choice split.
    DeferredChoice = 9,
    /// Interleaved routing split.
    InterleavedRouting = 5,
    /// Thread split.
    ThreadSplit = 6,
    /// Implicit termination split: ends case silently if no active tokens remain.
    ImplicitTermination = 7,
    /// Explicit termination split: annihilates all tokens in the case.
    ExplicitTermination = 8,
}

/// A cache-aligned Binary YAWL Task representation.
#[derive(Clone, Copy, Debug)]
#[repr(C, align(64))]
pub struct BYawlTask {
    /// Task identifier.
    pub id: u16,
    /// The type of join for this task.
    pub join_type: JoinType,
    /// The type of split for this task.
    pub split_type: SplitType,
    /// Minimum number of multi-instances.
    pub min_instances: u8,
    /// Maximum number of multi-instances.
    pub max_instances: u8,
    /// Instance count threshold for complex join firing.
    pub threshold_instances: u8,
    /// The bit position in the join mask tracking complex join history.
    pub join_state_bit: u8,
    /// Pattern flags (e.g. transient triggers, interleaved locks).
    pub flags: u8,
    /// Bitmask of tokens consumed from the engine state.
    pub consume_mask: u64,
    /// Bitmask of tokens produced to the engine state.
    pub produce_mask: u64,
    /// Bitmask of places cancelled by this task.
    pub cancellation_mask: u64,
    /// Bitmask of conditions/milestones that must be met.
    pub condition_mask: u64,
    /// Bitmask of places reset by this task.
    pub reset_mask: u64,
    /// Upstream reachability mask for synchronizing merge.
    pub reachability_mask: u64,
    /// Interleaved routing lock mask.
    pub interleaved_lock_mask: u64,
}

/// The YAWL routing execution engine state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BYawlEngine {
    /// Currently active place tokens.
    pub state_mask: u64,
    /// Dynamic instance counts for each place (up to 64 places).
    pub active_instances: [u8; 64],
    /// Active transient event triggers.
    pub active_triggers: u64,
    /// Tracking mask for fired complex joins.
    pub fired_joins_mask: u64,
    /// Mutex lock mask for interleaved routing.
    pub active_locks: u64,
}

impl Default for BYawlEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BYawlEngine {
    /// Creates a new, blank YAWL execution engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state_mask: 0,
            active_instances: [0; 64],
            active_triggers: 0,
            fired_joins_mask: 0,
            active_locks: 0,
        }
    }

    /// Triggers an external event (transient or persistent).
    #[inline(always)]
    pub fn trigger_event(&mut self, trigger_mask: u64) {
        self.active_triggers |= trigger_mask;
    }

    /// Spawns instances dynamically during runtime.
    #[inline(always)]
    pub fn spawn_instances(&mut self, place_bit: u8, count: u8) {
        let in_bounds_mask = nz_mask_u64(u64::from(place_bit < 64));
        let count_v = splat_u8x16(count);
        let place_v = splat_u8x16(place_bit);
        let in_bounds_v = splat_u8x16((in_bounds_mask as u8).wrapping_neg());

        let idx0 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let idx1 = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let idx2 = [32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47];
        let idx3 = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];

        let eq0 = and_u8x16(compare_eq_u8x16(place_v, idx0), in_bounds_v);
        let eq1 = and_u8x16(compare_eq_u8x16(place_v, idx1), in_bounds_v);
        let eq2 = and_u8x16(compare_eq_u8x16(place_v, idx2), in_bounds_v);
        let eq3 = and_u8x16(compare_eq_u8x16(place_v, idx3), in_bounds_v);

        let process_chunk = |active_instances: &mut [u8; 64], start: usize, eq_mask: [u8; 16]| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let added = add_saturating_u8x16(cur, count_v);
            let next = blend_u8x16(eq_mask, cur, added);
            active_instances[start..start + 16].copy_from_slice(&next);
        };

        process_chunk(&mut self.active_instances, 0, eq0);
        process_chunk(&mut self.active_instances, 16, eq1);
        process_chunk(&mut self.active_instances, 32, eq2);
        process_chunk(&mut self.active_instances, 48, eq3);

        self.state_mask |= (1u64.wrapping_shl(u32::from(place_bit) & 63)) & in_bounds_mask;
    }

    /// Executes task splits, joins, resets, locks, and cancellations branchlessly using mask calculus.
    pub fn execute_task_branchless(&mut self, task: &BYawlTask) -> u64 {
        // --- 1. Evaluate Lock & Conditions ---
        let is_release_mask = nz_mask_u64(u64::from(task.flags & 4));
        let conflict_mask = nz_mask_u64(self.active_locks & task.interleaved_lock_mask);
        let allowed_by_lock_mask = (!conflict_mask) | is_release_mask;

        let cond_diff = (self.state_mask & task.condition_mask) ^ task.condition_mask;
        let allowed_by_cond_mask = z_mask_u64(cond_diff);

        // --- 2. Reset Complex Joins ---
        let has_reset_tokens_mask =
            nz_mask_u64(self.state_mask & task.reset_mask) & allowed_by_lock_mask;
        let reset_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);

        self.fired_joins_mask &= !(reset_bit & has_reset_tokens_mask);
        self.state_mask &= !(task.reset_mask & has_reset_tokens_mask);

        // --- 3. Join Predicate Evaluations ---
        let count_ones = popcount_u64(self.state_mask & task.consume_mask);

        let c = self.state_mask & task.consume_mask;
        let join_xor_mask = nz_mask_u64(c) & z_mask_u64(c & c.wrapping_sub(1));

        let join_and_mask = z_mask_u64((self.state_mask & task.consume_mask) ^ task.consume_mask);

        let val = self.state_mask & task.consume_mask;
        let aux = self.state_mask & task.reachability_mask;
        let join_or_mask = nz_mask_u64(val) & z_mask_u64(aux & !val);

        let complex_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);
        let complex_has_fired_mask = nz_mask_u64(self.fired_joins_mask & complex_bit);
        let diff = (count_ones as i16).wrapping_sub(i16::from(task.threshold_instances));
        let threshold_met_mask = !((diff >> 15) as u64);
        let join_complex_mask = !complex_has_fired_mask & threshold_met_mask;

        let join_thread_merge_mask = nz_mask_u64(self.state_mask & task.consume_mask);

        // Multiplex Join Types
        let is_xor = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::XOR as u64));
        let is_and = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::AND as u64));
        let is_or = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::OR as u64));
        let is_complex = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::Complex as u64));
        let is_thread_merge =
            z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::ThreadMerge as u64));

        let can_join_mask = (join_xor_mask & is_xor)
            | (join_and_mask & is_and)
            | (join_or_mask & is_or)
            | (join_complex_mask & is_complex)
            | (join_thread_merge_mask & is_thread_merge);

        // Transient trigger check (WCP-23)
        let is_transient_mask = nz_mask_u64(u64::from(task.flags & 1));
        let has_transient_trigger_mask =
            is_transient_mask & nz_mask_u64(self.active_triggers & task.consume_mask);

        let fire_condition_mask = can_join_mask | has_transient_trigger_mask;
        let fired_mask = allowed_by_lock_mask & allowed_by_cond_mask & fire_condition_mask;

        // Complex Join: token consumption on bypass (vacuuming)
        let complex_fired_condition = is_complex & complex_has_fired_mask;
        let consume_on_blocked_mask =
            (!fired_mask) & complex_fired_condition & allowed_by_lock_mask & allowed_by_cond_mask;
        let do_consume_mask = fired_mask | consume_on_blocked_mask;

        // --- 4. State Updates ---
        // Consume tokens
        self.state_mask &= !(task.consume_mask & do_consume_mask);

        // Active triggers
        self.active_triggers &= !(task.consume_mask & fired_mask & is_transient_mask);

        // Active locks (Acquire)
        self.active_locks |= task.interleaved_lock_mask & fired_mask;
        // Active locks (Release)
        let release_mask = nz_mask_u64(u64::from(task.flags & 4));
        self.active_locks &= !(task.interleaved_lock_mask & fired_mask & release_mask);

        // Fired complex joins
        self.fired_joins_mask |= complex_bit & fired_mask & is_complex;

        // Cancellations
        self.state_mask &= !(task.cancellation_mask & fired_mask);
        let cancel_mask = task.cancellation_mask & fired_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut self.active_instances, 0, cancel_mask);
        process_cancel(&mut self.active_instances, 16, cancel_mask);
        process_cancel(&mut self.active_instances, 32, cancel_mask);
        process_cancel(&mut self.active_instances, 48, cancel_mask);

        // Complete MI Activity
        let is_complete_mi_mask = nz_mask_u64(u64::from(task.flags & 8));
        let clear_mi_mask = fired_mask & is_complete_mi_mask;
        let mi_cancel = task.produce_mask & clear_mi_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut self.active_instances, 0, mi_cancel);
        process_cancel(&mut self.active_instances, 16, mi_cancel);
        process_cancel(&mut self.active_instances, 32, mi_cancel);
        process_cancel(&mut self.active_instances, 48, mi_cancel);

        // --- 5. Splits & Produces ---
        let split_it_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ImplicitTermination as u64),
        );
        let split_et_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ExplicitTermination as u64),
        );
        let split_mi_mask =
            z_mask_u64((task.split_type as u64).wrapping_sub(SplitType::MultiInstance as u64));

        let should_produce_mask = fired_mask & !split_it_mask & !split_et_mask;
        self.state_mask |= task.produce_mask & should_produce_mask;

        // Explicit Termination
        let et_mask = !(fired_mask & split_et_mask);
        self.state_mask &= et_mask;
        self.fired_joins_mask &= et_mask;
        self.active_locks &= et_mask;
        let et_v = splat_u8x16(et_mask as u8);
        let process_et = |active_instances: &mut [u8; 64], start: usize| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = and_u8x16(cur, et_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_et(&mut self.active_instances, 0);
        process_et(&mut self.active_instances, 16);
        process_et(&mut self.active_instances, 32);
        process_et(&mut self.active_instances, 48);

        // Multi-Instance produce limits
        let target_idx = task.produce_mask.trailing_zeros() as usize;
        let is_mi_mask = fired_mask & split_mi_mask;
        let mi_v = splat_u8x16((is_mi_mask as u8).wrapping_neg());
        let max_inst_v = splat_u8x16(task.max_instances);
        let target_idx_v = splat_u8x16(target_idx as u8);
        let target_valid_v =
            splat_u8x16((nz_mask_u64(u64::from(target_idx < 64)) as u8).wrapping_neg());

        let idx0 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let idx1 = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let idx2 = [32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47];
        let idx3 = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];

        let process_mi = |active_instances: &mut [u8; 64], start: usize, idx_v: [u8; 16]| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let eq_mask = compare_eq_u8x16(idx_v, target_idx_v);
            let target_mask = and_u8x16(and_u8x16(eq_mask, target_valid_v), mi_v);
            let next = blend_u8x16(target_mask, cur, max_inst_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_mi(&mut self.active_instances, 0, idx0);
        process_mi(&mut self.active_instances, 16, idx1);
        process_mi(&mut self.active_instances, 32, idx2);
        process_mi(&mut self.active_instances, 48, idx3);

        fired_mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execute_task_mutant_1(engine: &mut BYawlEngine, task: &BYawlTask) -> u64 {
        let mut fake_engine = engine.clone();
        let has_reset_tokens_mask = nz_mask_u64(fake_engine.state_mask & task.reset_mask);
        let reset_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);

        fake_engine.fired_joins_mask &= !(reset_bit & has_reset_tokens_mask);
        fake_engine.state_mask &= !(task.reset_mask & has_reset_tokens_mask);

        let is_release_mask = nz_mask_u64(u64::from(task.flags & 4));
        let conflict_mask = nz_mask_u64(fake_engine.active_locks & task.interleaved_lock_mask);
        let allowed_by_lock_mask = (!conflict_mask) | is_release_mask;

        let cond_diff = (fake_engine.state_mask & task.condition_mask) ^ task.condition_mask;
        let allowed_by_cond_mask = z_mask_u64(cond_diff);

        let count_ones = popcount_u64(fake_engine.state_mask & task.consume_mask);

        let c = fake_engine.state_mask & task.consume_mask;
        let join_xor_mask = nz_mask_u64(c) & z_mask_u64(c & c.wrapping_sub(1));

        let join_and_mask =
            z_mask_u64((fake_engine.state_mask & task.consume_mask) ^ task.consume_mask);

        let val = fake_engine.state_mask & task.consume_mask;
        let aux = fake_engine.state_mask & task.reachability_mask;
        let join_or_mask = nz_mask_u64(val) & z_mask_u64(aux & !val);

        let complex_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);
        let complex_has_fired_mask = nz_mask_u64(fake_engine.fired_joins_mask & complex_bit);
        let diff = (count_ones as i16).wrapping_sub(i16::from(task.threshold_instances));
        let threshold_met_mask = !((diff >> 15) as u64);
        let join_complex_mask = !complex_has_fired_mask & threshold_met_mask;

        let join_thread_merge_mask = nz_mask_u64(fake_engine.state_mask & task.consume_mask);

        let is_xor = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::XOR as u64));
        let is_and = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::AND as u64));
        let is_or = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::OR as u64));
        let is_complex = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::Complex as u64));
        let is_thread_merge =
            z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::ThreadMerge as u64));

        let can_join_mask = (join_xor_mask & is_xor)
            | (join_and_mask & is_and)
            | (join_or_mask & is_or)
            | (join_complex_mask & is_complex)
            | (join_thread_merge_mask & is_thread_merge);

        let is_transient_mask = nz_mask_u64(u64::from(task.flags & 1));
        let has_transient_trigger_mask =
            is_transient_mask & nz_mask_u64(fake_engine.active_triggers & task.consume_mask);

        let fire_condition_mask = can_join_mask | has_transient_trigger_mask;
        let fired_mask = allowed_by_lock_mask & allowed_by_cond_mask & fire_condition_mask;

        let complex_fired_condition = is_complex & complex_has_fired_mask;
        let consume_on_blocked_mask = (!fired_mask) & complex_fired_condition;
        let do_consume_mask = fired_mask | consume_on_blocked_mask;

        fake_engine.state_mask &= !(task.consume_mask & do_consume_mask);

        fake_engine.active_triggers &= !(task.consume_mask & fired_mask & is_transient_mask);

        fake_engine.active_locks |= task.interleaved_lock_mask & fired_mask;
        let release_mask = nz_mask_u64(u64::from(task.flags & 4));
        fake_engine.active_locks &= !(task.interleaved_lock_mask & fired_mask & release_mask);

        fake_engine.fired_joins_mask |= complex_bit & fired_mask & is_complex;

        // OMITTED: Cancellations (this makes it mutant 1)

        let is_complete_mi_mask = nz_mask_u64(u64::from(task.flags & 8));
        let clear_mi_mask = fired_mask & is_complete_mi_mask;
        let mi_cancel = task.produce_mask & clear_mi_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut fake_engine.active_instances, 0, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 16, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 32, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 48, mi_cancel);

        let split_it_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ImplicitTermination as u64),
        );
        let split_et_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ExplicitTermination as u64),
        );
        let split_mi_mask =
            z_mask_u64((task.split_type as u64).wrapping_sub(SplitType::MultiInstance as u64));

        let should_produce_mask = fired_mask & !split_it_mask & !split_et_mask;
        fake_engine.state_mask |= task.produce_mask & should_produce_mask;

        let et_mask = !(fired_mask & split_et_mask);
        fake_engine.state_mask &= et_mask;
        fake_engine.fired_joins_mask &= et_mask;
        fake_engine.active_locks &= et_mask;
        let et_v = splat_u8x16(et_mask as u8);
        let process_et = |active_instances: &mut [u8; 64], start: usize| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = and_u8x16(cur, et_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_et(&mut fake_engine.active_instances, 0);
        process_et(&mut fake_engine.active_instances, 16);
        process_et(&mut fake_engine.active_instances, 32);
        process_et(&mut fake_engine.active_instances, 48);

        let target_idx = task.produce_mask.trailing_zeros() as usize;
        let is_mi_mask = fired_mask & split_mi_mask;
        let mi_v = splat_u8x16((is_mi_mask as u8).wrapping_neg());
        let max_inst_v = splat_u8x16(task.max_instances);
        let target_idx_v = splat_u8x16(target_idx as u8);
        let target_valid_v =
            splat_u8x16((nz_mask_u64(u64::from(target_idx < 64)) as u8).wrapping_neg());

        let idx0 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let idx1 = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let idx2 = [32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47];
        let idx3 = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];

        let process_mi = |active_instances: &mut [u8; 64], start: usize, idx_v: [u8; 16]| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let eq_mask = compare_eq_u8x16(idx_v, target_idx_v);
            let target_mask = and_u8x16(and_u8x16(eq_mask, target_valid_v), mi_v);
            let next = blend_u8x16(target_mask, cur, max_inst_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_mi(&mut fake_engine.active_instances, 0, idx0);
        process_mi(&mut fake_engine.active_instances, 16, idx1);
        process_mi(&mut fake_engine.active_instances, 32, idx2);
        process_mi(&mut fake_engine.active_instances, 48, idx3);

        *engine = fake_engine;
        fired_mask
    }

    fn execute_task_mutant_2(engine: &mut BYawlEngine, task: &BYawlTask) -> u64 {
        let mut fake_engine = engine.clone();
        let has_reset_tokens_mask = nz_mask_u64(fake_engine.state_mask & task.reset_mask);
        let reset_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);

        fake_engine.fired_joins_mask &= !(reset_bit & has_reset_tokens_mask);
        fake_engine.state_mask &= !(task.reset_mask & has_reset_tokens_mask);

        let is_release_mask = nz_mask_u64(u64::from(task.flags & 4));
        let conflict_mask = nz_mask_u64(fake_engine.active_locks & task.interleaved_lock_mask);
        let allowed_by_lock_mask = (!conflict_mask) | is_release_mask;

        let cond_diff = (fake_engine.state_mask & task.condition_mask) ^ task.condition_mask;
        let allowed_by_cond_mask = z_mask_u64(cond_diff);

        let count_ones = popcount_u64(fake_engine.state_mask & task.consume_mask);

        let c = fake_engine.state_mask & task.consume_mask;
        let join_xor_mask = nz_mask_u64(c) & z_mask_u64(c & c.wrapping_sub(1));

        let join_and_mask =
            z_mask_u64((fake_engine.state_mask & task.consume_mask) ^ task.consume_mask);

        let val = fake_engine.state_mask & task.consume_mask;
        // MUTATED: ignores reachability (this makes it mutant 2)
        let join_or_mask = nz_mask_u64(val);

        let complex_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);
        let complex_has_fired_mask = nz_mask_u64(fake_engine.fired_joins_mask & complex_bit);
        let diff = (count_ones as i16).wrapping_sub(i16::from(task.threshold_instances));
        let threshold_met_mask = !((diff >> 15) as u64);
        let join_complex_mask = !complex_has_fired_mask & threshold_met_mask;

        let join_thread_merge_mask = nz_mask_u64(fake_engine.state_mask & task.consume_mask);

        let is_xor = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::XOR as u64));
        let is_and = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::AND as u64));
        let is_or = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::OR as u64));
        let is_complex = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::Complex as u64));
        let is_thread_merge =
            z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::ThreadMerge as u64));

        let can_join_mask = (join_xor_mask & is_xor)
            | (join_and_mask & is_and)
            | (join_or_mask & is_or)
            | (join_complex_mask & is_complex)
            | (join_thread_merge_mask & is_thread_merge);

        let is_transient_mask = nz_mask_u64(u64::from(task.flags & 1));
        let has_transient_trigger_mask =
            is_transient_mask & nz_mask_u64(fake_engine.active_triggers & task.consume_mask);

        let fire_condition_mask = can_join_mask | has_transient_trigger_mask;
        let fired_mask = allowed_by_lock_mask & allowed_by_cond_mask & fire_condition_mask;

        let complex_fired_condition = is_complex & complex_has_fired_mask;
        let consume_on_blocked_mask = (!fired_mask) & complex_fired_condition;
        let do_consume_mask = fired_mask | consume_on_blocked_mask;

        fake_engine.state_mask &= !(task.consume_mask & do_consume_mask);

        fake_engine.active_triggers &= !(task.consume_mask & fired_mask & is_transient_mask);

        fake_engine.active_locks |= task.interleaved_lock_mask & fired_mask;
        let release_mask = nz_mask_u64(u64::from(task.flags & 4));
        fake_engine.active_locks &= !(task.interleaved_lock_mask & fired_mask & release_mask);

        fake_engine.fired_joins_mask |= complex_bit & fired_mask & is_complex;

        fake_engine.state_mask &= !(task.cancellation_mask & fired_mask);
        let cancel_mask = task.cancellation_mask & fired_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut fake_engine.active_instances, 0, cancel_mask);
        process_cancel(&mut fake_engine.active_instances, 16, cancel_mask);
        process_cancel(&mut fake_engine.active_instances, 32, cancel_mask);
        process_cancel(&mut fake_engine.active_instances, 48, cancel_mask);

        let is_complete_mi_mask = nz_mask_u64(u64::from(task.flags & 8));
        let clear_mi_mask = fired_mask & is_complete_mi_mask;
        let mi_cancel = task.produce_mask & clear_mi_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut fake_engine.active_instances, 0, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 16, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 32, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 48, mi_cancel);

        let split_it_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ImplicitTermination as u64),
        );
        let split_et_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ExplicitTermination as u64),
        );
        let split_mi_mask =
            z_mask_u64((task.split_type as u64).wrapping_sub(SplitType::MultiInstance as u64));

        let should_produce_mask = fired_mask & !split_it_mask & !split_et_mask;
        fake_engine.state_mask |= task.produce_mask & should_produce_mask;

        let et_mask = !(fired_mask & split_et_mask);
        fake_engine.state_mask &= et_mask;
        fake_engine.fired_joins_mask &= et_mask;
        fake_engine.active_locks &= et_mask;
        let et_v = splat_u8x16(et_mask as u8);
        let process_et = |active_instances: &mut [u8; 64], start: usize| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = and_u8x16(cur, et_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_et(&mut fake_engine.active_instances, 0);
        process_et(&mut fake_engine.active_instances, 16);
        process_et(&mut fake_engine.active_instances, 32);
        process_et(&mut fake_engine.active_instances, 48);

        let target_idx = task.produce_mask.trailing_zeros() as usize;
        let is_mi_mask = fired_mask & split_mi_mask;
        let mi_v = splat_u8x16((is_mi_mask as u8).wrapping_neg());
        let max_inst_v = splat_u8x16(task.max_instances);
        let target_idx_v = splat_u8x16(target_idx as u8);
        let target_valid_v =
            splat_u8x16((nz_mask_u64(u64::from(target_idx < 64)) as u8).wrapping_neg());

        let idx0 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let idx1 = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let idx2 = [32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47];
        let idx3 = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];

        let process_mi = |active_instances: &mut [u8; 64], start: usize, idx_v: [u8; 16]| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let eq_mask = compare_eq_u8x16(idx_v, target_idx_v);
            let target_mask = and_u8x16(and_u8x16(eq_mask, target_valid_v), mi_v);
            let next = blend_u8x16(target_mask, cur, max_inst_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_mi(&mut fake_engine.active_instances, 0, idx0);
        process_mi(&mut fake_engine.active_instances, 16, idx1);
        process_mi(&mut fake_engine.active_instances, 32, idx2);
        process_mi(&mut fake_engine.active_instances, 48, idx3);

        *engine = fake_engine;
        fired_mask
    }

    fn execute_task_mutant_3(engine: &mut BYawlEngine, task: &BYawlTask) -> u64 {
        let mut fake_engine = engine.clone();
        let has_reset_tokens_mask = nz_mask_u64(fake_engine.state_mask & task.reset_mask);
        let reset_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);

        fake_engine.fired_joins_mask &= !(reset_bit & has_reset_tokens_mask);
        fake_engine.state_mask &= !(task.reset_mask & has_reset_tokens_mask);

        let is_release_mask = nz_mask_u64(u64::from(task.flags & 4));
        let conflict_mask = nz_mask_u64(fake_engine.active_locks & task.interleaved_lock_mask);
        let allowed_by_lock_mask = (!conflict_mask) | is_release_mask;

        let cond_diff = (fake_engine.state_mask & task.condition_mask) ^ task.condition_mask;
        let allowed_by_cond_mask = z_mask_u64(cond_diff);

        let count_ones = popcount_u64(fake_engine.state_mask & task.consume_mask);

        let c = fake_engine.state_mask & task.consume_mask;
        let join_xor_mask = nz_mask_u64(c) & z_mask_u64(c & c.wrapping_sub(1));

        let join_and_mask =
            z_mask_u64((fake_engine.state_mask & task.consume_mask) ^ task.consume_mask);

        let val = fake_engine.state_mask & task.consume_mask;
        let aux = fake_engine.state_mask & task.reachability_mask;
        let join_or_mask = nz_mask_u64(val) & z_mask_u64(aux & !val);

        let complex_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);
        let complex_has_fired_mask = nz_mask_u64(fake_engine.fired_joins_mask & complex_bit);
        let diff = (count_ones as i16).wrapping_sub(i16::from(task.threshold_instances));
        let threshold_met_mask = !((diff >> 15) as u64);
        let join_complex_mask = !complex_has_fired_mask & threshold_met_mask;

        let join_thread_merge_mask = nz_mask_u64(fake_engine.state_mask & task.consume_mask);

        let is_xor = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::XOR as u64));
        let is_and = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::AND as u64));
        let is_or = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::OR as u64));
        let is_complex = z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::Complex as u64));
        let is_thread_merge =
            z_mask_u64((task.join_type as u64).wrapping_sub(JoinType::ThreadMerge as u64));

        let can_join_mask = (join_xor_mask & is_xor)
            | (join_and_mask & is_and)
            | (join_or_mask & is_or)
            | (join_complex_mask & is_complex)
            | (join_thread_merge_mask & is_thread_merge);

        let is_transient_mask = nz_mask_u64(u64::from(task.flags & 1));
        let has_transient_trigger_mask =
            is_transient_mask & nz_mask_u64(fake_engine.active_triggers & task.consume_mask);

        let fire_condition_mask = can_join_mask | has_transient_trigger_mask;
        let fired_mask = allowed_by_lock_mask & allowed_by_cond_mask & fire_condition_mask;

        let complex_fired_condition = is_complex & complex_has_fired_mask;
        let consume_on_blocked_mask = (!fired_mask) & complex_fired_condition;
        let do_consume_mask = fired_mask | consume_on_blocked_mask;

        fake_engine.state_mask &= !(task.consume_mask & do_consume_mask);

        fake_engine.active_triggers &= !(task.consume_mask & fired_mask & is_transient_mask);

        fake_engine.active_locks |= task.interleaved_lock_mask & fired_mask;
        let release_mask = nz_mask_u64(u64::from(task.flags & 4));
        fake_engine.active_locks &= !(task.interleaved_lock_mask & fired_mask & release_mask);

        fake_engine.fired_joins_mask |= complex_bit & fired_mask & is_complex;

        fake_engine.state_mask &= !(task.cancellation_mask & fired_mask);
        let cancel_mask = task.cancellation_mask & fired_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut fake_engine.active_instances, 0, cancel_mask);
        process_cancel(&mut fake_engine.active_instances, 16, cancel_mask);
        process_cancel(&mut fake_engine.active_instances, 32, cancel_mask);
        process_cancel(&mut fake_engine.active_instances, 48, cancel_mask);

        let is_complete_mi_mask = nz_mask_u64(u64::from(task.flags & 8));
        let clear_mi_mask = fired_mask & is_complete_mi_mask;
        let mi_cancel = task.produce_mask & clear_mi_mask;
        let zero_v = splat_u8x16(0);
        let process_cancel = |active_instances: &mut [u8; 64], start: usize, mask: u64| {
            let m = unpack_u64_mask_to_u8x16(mask, start as u32);
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let next = blend_u8x16(m, cur, zero_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_cancel(&mut fake_engine.active_instances, 0, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 16, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 32, mi_cancel);
        process_cancel(&mut fake_engine.active_instances, 48, mi_cancel);

        let split_it_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ImplicitTermination as u64),
        );
        let split_et_mask = z_mask_u64(
            (task.split_type as u64).wrapping_sub(SplitType::ExplicitTermination as u64),
        );
        let split_mi_mask =
            z_mask_u64((task.split_type as u64).wrapping_sub(SplitType::MultiInstance as u64));

        let should_produce_mask = fired_mask & !split_it_mask & !split_et_mask;
        fake_engine.state_mask |= task.produce_mask & should_produce_mask;

        let et_mask = !(fired_mask & split_et_mask);
        fake_engine.state_mask &= et_mask;
        fake_engine.fired_joins_mask &= et_mask;
        fake_engine.active_locks &= et_mask;
        // OMITTED: Clearing active_instances under explicit termination (this makes it mutant 3)

        let target_idx = task.produce_mask.trailing_zeros() as usize;
        let is_mi_mask = fired_mask & split_mi_mask;
        let mi_v = splat_u8x16((is_mi_mask as u8).wrapping_neg());
        let max_inst_v = splat_u8x16(task.max_instances);
        let target_idx_v = splat_u8x16(target_idx as u8);
        let target_valid_v =
            splat_u8x16((nz_mask_u64(u64::from(target_idx < 64)) as u8).wrapping_neg());

        let idx0 = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let idx1 = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let idx2 = [32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47];
        let idx3 = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];

        let process_mi = |active_instances: &mut [u8; 64], start: usize, idx_v: [u8; 16]| {
            let cur: [u8; 16] = active_instances[start..start + 16].try_into().unwrap();
            let eq_mask = compare_eq_u8x16(idx_v, target_idx_v);
            let target_mask = and_u8x16(and_u8x16(eq_mask, target_valid_v), mi_v);
            let next = blend_u8x16(target_mask, cur, max_inst_v);
            active_instances[start..start + 16].copy_from_slice(&next);
        };
        process_mi(&mut fake_engine.active_instances, 0, idx0);
        process_mi(&mut fake_engine.active_instances, 16, idx1);
        process_mi(&mut fake_engine.active_instances, 32, idx2);
        process_mi(&mut fake_engine.active_instances, 48, idx3);

        *engine = fake_engine;
        fired_mask
    }

    #[test]
    fn test_mutants_and_adversarial_coverage() {
        // --- Test Case 1: Cancellation Behavior (Kills Mutant 1) ---
        let mut engine_ref = BYawlEngine::new();
        engine_ref.state_mask = 0b11;
        engine_ref.active_instances[0] = 5;
        engine_ref.active_instances[1] = 10;

        let task_cancel = BYawlTask {
            id: 1,
            join_type: JoinType::AND,
            split_type: SplitType::AND,
            min_instances: 1,
            max_instances: 1,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b01,
            produce_mask: 0b100,
            cancellation_mask: 0b10,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0,
        };

        let mut engine_mut1 = engine_ref.clone();
        let mut engine_real = engine_ref.clone();

        let fired_real = engine_real.execute_task_branchless(&task_cancel);
        let fired_mut1 = execute_task_mutant_1(&mut engine_mut1, &task_cancel);

        assert_eq!(fired_real, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(fired_mut1, 0xFFFF_FFFF_FFFF_FFFF);

        // Under real implementation, cancellation_mask clears bit 1 and active_instances[1]
        assert_eq!(engine_real.state_mask, 0b100);
        assert_eq!(engine_real.active_instances[1], 0);

        // Under mutant 1, cancellation_mask is ignored
        assert_ne!(engine_real, engine_mut1);

        // --- Test Case 2: OR Join Reachability (Kills Mutant 2) ---
        let mut engine_ref = BYawlEngine::new();
        // Place 1 is active, and it is upstream (reachability_mask has it)
        engine_ref.state_mask = 0b11;

        let task_or = BYawlTask {
            id: 2,
            join_type: JoinType::OR,
            split_type: SplitType::AND,
            min_instances: 1,
            max_instances: 1,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b01, // consumes from 0
            produce_mask: 0b100,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0b10, // place 1 is upstream
            interleaved_lock_mask: 0,
        };

        let mut engine_mut2 = engine_ref.clone();
        let mut engine_real = engine_ref.clone();

        let fired_real = engine_real.execute_task_branchless(&task_or);
        let fired_mut2 = execute_task_mutant_2(&mut engine_mut2, &task_or);

        // Real engine should not fire (fired_real = 0) because token upstream at place 1 could reach place 0
        assert_eq!(fired_real, 0);
        // Mutant 2 fires because it ignores upstream reachability check
        assert_eq!(fired_mut2, 0xFFFF_FFFF_FFFF_FFFF);
        assert_ne!(engine_real, engine_mut2);

        // --- Test Case 3: Explicit Termination active_instances (Kills Mutant 3) ---
        let mut engine_ref = BYawlEngine::new();
        engine_ref.active_instances[0] = 5;
        engine_ref.state_mask = 0b01;

        let task_et = BYawlTask {
            id: 3,
            join_type: JoinType::AND,
            split_type: SplitType::ExplicitTermination,
            min_instances: 1,
            max_instances: 1,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b01,
            produce_mask: 0,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0,
        };

        let mut engine_mut3 = engine_ref.clone();
        let mut engine_real = engine_ref.clone();

        let fired_real = engine_real.execute_task_branchless(&task_et);
        let fired_mut3 = execute_task_mutant_3(&mut engine_mut3, &task_et);

        assert_eq!(fired_real, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(fired_mut3, 0xFFFF_FFFF_FFFF_FFFF);

        // Real engine clears active_instances on explicit termination
        assert_eq!(engine_real.active_instances[0], 0);
        // Mutant 3 fails to clear active_instances
        assert_eq!(engine_mut3.active_instances[0], 5);
        assert_ne!(engine_real, engine_mut3);
    }

    #[test]
    fn test_yawl_comprehensive_scenarios() {
        // --- XOR split/join ---
        let mut engine = BYawlEngine::new();
        engine.state_mask = 0b01; // token in place 0
        let task_xor = BYawlTask {
            id: 10,
            join_type: JoinType::XOR,
            split_type: SplitType::XOR,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b01,
            produce_mask: 0b10,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0,
        };
        let fired = engine.execute_task_branchless(&task_xor);
        assert_eq!(fired, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(engine.state_mask, 0b10); // consumed from 0, produced to 1

        // --- OR Join with no upstream reaches ---
        let mut engine = BYawlEngine::new();
        engine.state_mask = 0b01; // token at 0
        let task_or = BYawlTask {
            id: 11,
            join_type: JoinType::OR,
            split_type: SplitType::AND,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b01,
            produce_mask: 0b100,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0b10, // reachability doesn't have token
            interleaved_lock_mask: 0,
        };
        let fired = engine.execute_task_branchless(&task_or);
        assert_eq!(fired, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(engine.state_mask, 0b100);

        // --- Complex Join (Discriminator) with Threshold ---
        let mut engine = BYawlEngine::new();
        engine.state_mask = 0b011; // tokens at 0 and 1
        let task_complex = BYawlTask {
            id: 12,
            join_type: JoinType::Complex,
            split_type: SplitType::AND,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 2, // requires 2 tokens to fire
            join_state_bit: 5,
            flags: 0,
            consume_mask: 0b011,
            produce_mask: 0b100,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0,
        };
        let fired = engine.execute_task_branchless(&task_complex);
        assert_eq!(fired, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(engine.state_mask, 0b100); // consumed all consumed tokens, produced 2
        assert_eq!(engine.fired_joins_mask & (1 << 5), (1 << 5)); // recorded as fired

        // Re-entry of complex join (vacuuming/bypass):
        // If it already fired (fired_joins_mask contains the bit), and a token arrives:
        // it shouldn't fire again but it should vacuum/consume the incoming tokens.
        engine.state_mask = 0b001; // new token arrives at 0
        let fired = engine.execute_task_branchless(&task_complex);
        assert_eq!(fired, 0); // didn't fire
        assert_eq!(engine.state_mask, 0); // but token was consumed/vacuumed!

        // Resetting complex join:
        // If a reset token arrives, it resets the fired joins tracking mask
        engine.state_mask = 0b1000; // token at 3
        let task_reset = BYawlTask {
            id: 13,
            join_type: JoinType::AND,
            split_type: SplitType::AND,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 0,
            join_state_bit: 5, // resets bit 5
            flags: 0,
            consume_mask: 0b1000,
            produce_mask: 0,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0b1000, // resets when token at 3 is present
            reachability_mask: 0,
            interleaved_lock_mask: 0,
        };
        engine.execute_task_branchless(&task_reset);
        assert_eq!(engine.fired_joins_mask & (1 << 5), 0); // reset successful!

        // --- Interleaved Routing Lock/Release ---
        let mut engine = BYawlEngine::new();
        engine.state_mask = 0b01;
        let task_lock = BYawlTask {
            id: 14,
            join_type: JoinType::AND,
            split_type: SplitType::InterleavedRouting,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0, // acquire lock
            consume_mask: 0b01,
            produce_mask: 0b10,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0b1000, // lock bit 3
        };
        // Acquire lock
        let fired = engine.execute_task_branchless(&task_lock);
        assert_eq!(fired, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(engine.active_locks, 0b1000);

        // Try to execute a task requiring the same lock - conflict!
        engine.state_mask = 0b10;
        let task_conflict = BYawlTask {
            id: 15,
            join_type: JoinType::AND,
            split_type: SplitType::AND,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b10,
            produce_mask: 0b100,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0b1000, // conflict on lock bit 3!
        };
        let fired = engine.execute_task_branchless(&task_conflict);
        assert_eq!(fired, 0); // blocked!
        assert_eq!(engine.state_mask, 0b10); // token not consumed

        // Release the lock
        let task_release = BYawlTask {
            id: 16,
            join_type: JoinType::AND,
            split_type: SplitType::AND,
            min_instances: 0,
            max_instances: 0,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 4, // release lock flag (bit 2)
            consume_mask: 0b10,
            produce_mask: 0b100,
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0b1000, // release lock bit 3
        };
        let fired = engine.execute_task_branchless(&task_release);
        assert_eq!(fired, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(engine.active_locks, 0); // released!
        assert_eq!(engine.state_mask, 0b100); // consumed and produced!
    }
}
