#![allow(warnings, clippy::all)]
#![allow(warnings)]
//! Binary YAWL (bYAWL) Routing Engine Reference
//!
//! `#![allow(dead_code)]`: see `reference/petri.rs`'s module doc comment —
//! same reasoning applies here (a comprehensive reference surface compiled
//! independently into several test binaries, each exercising a different
//! subset).
#![allow(dead_code)]

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JoinType {
    XOR = 0,
    AND = 1,
    OR = 2,
    Complex = 3,     // N-out-of-M, Discriminator, Partial Joins
    ThreadMerge = 4, // WCP-41
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitType {
    XOR = 0,
    AND = 1,
    OR = 2,
    MultiInstance = 3,
    DynamicMultiInstance = 4,
    DeferredChoice = 9,
    InterleavedRouting = 5,
    ThreadSplit = 6,
    ImplicitTermination = 7,
    ExplicitTermination = 8,
}

/// A Binary YAWL Task representation.
#[derive(Clone, Copy, Debug)]
#[repr(C, align(64))]
pub struct BYawlTask {
    pub id: u16,
    pub join_type: JoinType,
    pub split_type: SplitType,
    pub min_instances: u8,
    pub max_instances: u8,
    pub threshold_instances: u8,
    pub join_state_bit: u8, // Tracks if a complex join has fired (Discriminators)
    pub flags: u8,          // Custom pattern flags

    pub consume_mask: u64,
    pub produce_mask: u64,
    pub cancellation_mask: u64,
    pub condition_mask: u64, // Used for Milestone (WCP-18)
    pub reset_mask: u64,     // Used for Cancelling Discriminators

    /// Upstream places that can reach this task. Essential for O(1) OR-Join.
    pub reachability_mask: u64,

    /// Mutex mask for Interleaved Parallel Routing
    pub interleaved_lock_mask: u64,
}

pub struct BYawlEngine {
    /// Tracks active tokens in places
    pub state_mask: u64,
    /// Tracks multiple instances per task/place
    pub active_instances: [u8; 64],
    /// Tracks generic boolean flags for engine triggers
    pub active_triggers: u64,
    /// Tracks complex join states
    pub fired_joins_mask: u64,
    /// Mutex locks for Interleaved Parallel Routing
    pub active_locks: u64,
}

impl Default for BYawlEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[inline(always)]
pub fn synchronizing_merge_wcp37(val: u64, aux: u64) -> u64 {
    let present = val != 0;
    let no_upstream = (aux & !val) == 0;
    (present && no_upstream) as u64
}

impl BYawlEngine {
    pub fn new() -> Self {
        Self {
            state_mask: 0,
            active_instances: [0; 64],
            active_triggers: 0,
            fired_joins_mask: 0,
            active_locks: 0,
        }
    }

    #[inline(always)]
    pub fn trigger_event(&mut self, trigger_mask: u64) {
        self.active_triggers |= trigger_mask;
    }

    #[inline(always)]
    pub fn spawn_instances(&mut self, place_bit: u8, count: u8) {
        if (place_bit as usize) < 64 {
            self.active_instances[place_bit as usize] =
                self.active_instances[place_bit as usize].saturating_add(count);
            self.state_mask |= 1 << place_bit;
        }
    }

    #[inline(always)]
    pub fn execute_task(&mut self, task: &BYawlTask) -> bool {
        // Mutex check for Interleaved Routing (WCP-17)
        let is_release = (task.flags & 4) != 0;
        if !is_release
            && task.interleaved_lock_mask != 0
            && (self.active_locks & task.interleaved_lock_mask) != 0
        {
            return false;
        }

        // Reset complex joins
        if task.reset_mask != 0 && (self.state_mask & task.reset_mask) != 0 {
            self.fired_joins_mask &= !(1 << task.join_state_bit);
            self.state_mask &= !task.reset_mask;
        }

        // Evaluate Pre-conditions
        if task.condition_mask != 0
            && (self.state_mask & task.condition_mask) != task.condition_mask
        {
            return false;
        }

        // 1. Join Semantics
        let can_join = match task.join_type {
            JoinType::AND => (self.state_mask & task.consume_mask) == task.consume_mask,
            JoinType::XOR => (self.state_mask & task.consume_mask).count_ones() == 1,
            JoinType::OR => {
                synchronizing_merge_wcp37(
                    self.state_mask & task.consume_mask,
                    self.state_mask & task.reachability_mask,
                ) != 0
            }
            JoinType::Complex => {
                let present_tokens = (self.state_mask & task.consume_mask).count_ones() as u8;
                let has_fired = (self.fired_joins_mask & (1 << task.join_state_bit)) != 0;
                !has_fired && (present_tokens >= task.threshold_instances)
            }
            JoinType::ThreadMerge => (self.state_mask & task.consume_mask) != 0,
        };

        if !can_join {
            if task.join_type == JoinType::Complex
                && (self.fired_joins_mask & (1 << task.join_state_bit)) != 0
            {
                let consumed = self.state_mask & task.consume_mask;
                self.state_mask &= !consumed;
            }

            if (task.flags & 1) != 0 && (self.active_triggers & task.consume_mask) != 0 {
                // Transient triggers caught
            } else {
                return false;
            }
        }

        if (task.flags & 1) != 0 {
            self.active_triggers &= !task.consume_mask;
        }

        if task.interleaved_lock_mask != 0 {
            self.active_locks |= task.interleaved_lock_mask;
        }

        if task.join_type == JoinType::Complex {
            self.fired_joins_mask |= 1 << task.join_state_bit;
        }

        // 2. Consume Tokens
        let consumed = self.state_mask & task.consume_mask;
        self.state_mask &= !consumed;

        // 3. Cancellation Semantics
        self.state_mask &= !task.cancellation_mask;
        if task.cancellation_mask != 0 {
            for i in 0..64 {
                if (task.cancellation_mask & (1 << i)) != 0 {
                    self.active_instances[i] = 0;
                }
            }
        }

        if (task.flags & 4) != 0 {
            self.active_locks &= !task.interleaved_lock_mask;
        }

        if (task.flags & 8) != 0 {
            for i in 0..64 {
                if (task.produce_mask & (1 << i)) != 0 {
                    self.active_instances[i] = 0;
                }
            }
        }

        // 4. Split / Multi-Instance Semantics
        match task.split_type {
            SplitType::AND | SplitType::XOR | SplitType::OR => {
                self.state_mask |= task.produce_mask;
            }
            SplitType::MultiInstance => {
                let target_idx = task.produce_mask.trailing_zeros() as usize;
                if target_idx < 64 {
                    self.active_instances[target_idx] = task.max_instances;
                    self.state_mask |= task.produce_mask;
                }
            }
            SplitType::DynamicMultiInstance => {
                self.state_mask |= task.produce_mask;
            }
            SplitType::DeferredChoice => {
                self.state_mask |= task.produce_mask;
            }
            SplitType::InterleavedRouting => {
                self.state_mask |= task.produce_mask;
            }
            SplitType::ThreadSplit => {
                self.state_mask |= task.produce_mask;
            }
            SplitType::ImplicitTermination => {}
            SplitType::ExplicitTermination => {
                self.state_mask = 0;
                self.active_instances.fill(0);
                self.fired_joins_mask = 0;
                self.active_locks = 0;
            }
        }

        true
    }
}
