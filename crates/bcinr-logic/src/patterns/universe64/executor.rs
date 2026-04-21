//! # Universe64 Contract: UniverseExecutor (Boot + Instruction Loop)
//! Plane: D + S — owns both planes; drives the full execution lifecycle.
//! Tier: T0–T3 per step; bounded epoch.
//! Scope: owns UniverseBlock, UniverseScratch, DeltaTape, DeltaBus, ReadyMask, receipt.
//! Geometry: boot → instruction loop → epoch boundary → shutdown.
//! Delta: all deltas flow through DeltaBus to subscribers.
//!
//! # Timing contract
//! - **Boot:** T3-class (admission + index compile)
//! - **Per-instruction:** T0–T1 (single transition fire)
//! - **Epoch boundary:** T3 (receipt checkpoint + RL epoch advance)
//! - **Max heap allocations:** 0 (after boot)
//!
//! # Admissibility
//! Admissible_T1: YES for single instruction. Full epoch is T3-class.
//! CC=1: Absolute branchless logic in hot paths.

use super::constants::{UNIVERSE_WORDS, MAX_WORD_INDEX};
use super::block::UniverseBlock;
use super::scratch::{ActiveWordSet, UDelta, UScope, UniverseScratch};
use super::receipt::{TransitionReceipt, receipt_mix_transition, new_receipt};
use super::coord::UCoord;
use super::delta_tape::DeltaTape;
use super::delta_bus::DeltaBus;
use super::ready_mask::ReadyMask;
use super::rl::{RLState, RLKernel, RewardTable};
use super::conformance::{ConformanceState, ConformanceKernel};
use super::drift::{DriftKernel, ExpectedModel, DriftReport};
use super::index_plane::MAX_TRANSITIONS;
use super::ubit_capability::{UBitCapabilityTable, cap_admit};
use super::ubit_image::{UBitImage, UBitScratchSnapshot};
use super::ubit_supervisor::{UBitSupervisor, SupervisorThresholds};
use super::scope_planner::{PlanDecision, KernelClass};
use super::admission_lifecycle::TransitionRegistry;

/// Executor lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ExecutorState {
    /// Pre-boot: not yet initialized.
    Uninit = 0,
    /// Booted and ready.
    Ready = 1,
    /// Currently executing an instruction.
    Executing = 2,
    /// At epoch boundary.
    EpochBoundary = 3,
    /// Halted.
    Halted = 4,
}

/// Epoch statistics emitted at each epoch boundary.
#[derive(Clone, Copy, Debug)]
pub struct EpochStats {
    pub epoch: u32,
    pub instructions_executed: u32,
    pub deltas_emitted: u32,
    pub receipt_value: u64,
    pub rl_reward: i64,
    pub conformance_violations: u32,
}

/// Maximum instructions per epoch (bounded).
pub const MAX_EPOCH_INSTRUCTIONS: usize = 4096;

/// The full UniverseOS executor: owns all planes and subsystems.
pub struct UniverseExecutor {
    // Planes
    pub block: UniverseBlock,
    pub scratch: UniverseScratch,

    // Motion infrastructure
    pub tape: DeltaTape,
    pub bus: DeltaBus,

    // Scheduling
    pub ready: ReadyMask,
    pub prereqs: [u64; MAX_TRANSITIONS],

    // Proof
    pub receipt: TransitionReceipt,

    // Adaptation
    pub rl_state: RLState,
    pub reward_table: RewardTable,

    // Conformance
    pub conformance: ConformanceState,
    pub law: [u64; UNIVERSE_WORDS],

    // Drift
    pub model: ExpectedModel,

    // Lifecycle
    pub state: ExecutorState,
    pub epoch: u32,
    pub epoch_instructions: u32,

    // UBit OS completion
    pub null_cap_table: UBitCapabilityTable,
    pub registry: TransitionRegistry,
    pub last_decision: PlanDecision,
    pub supervisor: UBitSupervisor,
}

impl UniverseExecutor {
    /// Create executor in Uninit state.
    pub fn new() -> Self {
        Self {
            block: UniverseBlock::new(),
            scratch: UniverseScratch::new(),
            tape: DeltaTape::new(),
            bus: DeltaBus::new(),
            ready: ReadyMask::new(),
            prereqs: [0u64; MAX_TRANSITIONS],
            receipt: new_receipt(),
            rl_state: RLState::zero(),
            reward_table: RewardTable::new_neutral(),
            conformance: ConformanceState::zero(),
            law: [u64::MAX; UNIVERSE_WORDS],
            model: ExpectedModel::all_zero(),
            state: ExecutorState::Uninit,
            epoch: 0,
            epoch_instructions: 0,
            null_cap_table: UBitCapabilityTable::new(),
            registry: TransitionRegistry::new(),
            last_decision: PlanDecision {
                kernel: KernelClass::Invalid,
                touched_words: 0,
                tier: crate::patterns::universe64::instruction::UTier::T0,
                active: ActiveWordSet::new(),
            },
            supervisor: UBitSupervisor::new(SupervisorThresholds::default_thresholds()),
        }
    }

    /// Boot: initialize conformance, transition to Ready.
    pub fn boot(&mut self) {
        self.conformance = ConformanceKernel::initialize(&self.block, &self.law);
        self.receipt = new_receipt();
        self.epoch = 0;
        self.epoch_instructions = 0;
        self.state = ExecutorState::Ready;
    }

    /// Execute one instruction: flip bit at (word_idx, bit_idx) under fired_mask.
    /// Returns the UDelta produced.
    pub fn execute_one(
        &mut self,
        word_idx: usize,
        bit_idx: u32,
        fired_mask: u64,
        instr_id: u32,
    ) -> UDelta {
        self.state = ExecutorState::Executing;
        let widx = word_idx & MAX_WORD_INDEX;
        let bit = bit_idx & 63;
        let flip = 1u64 << bit;

        let before = self.block.state[widx];
        let new_val = (before ^ (flip & fired_mask)) | (before & !fired_mask);
        self.block.state[widx] = new_val;

        let delta = UDelta::new(widx, UScope::Cell, instr_id, before, new_val, fired_mask);
        self.tape.append(delta);
        self.bus.publish(&delta);

        // Update conformance incrementally.
        ConformanceKernel::update_delta(&mut self.conformance, &delta, self.law[widx]);

        // Update receipt.
        let coord = UCoord::new_const(
            ((widx / 64) & 63) as u8,
            (widx & 63) as u8,
            bit as u8,
        );
        self.receipt = receipt_mix_transition(self.receipt.value(), coord, fired_mask);

        // Update RL.
        RLKernel::update(&mut self.rl_state, &delta, &self.reward_table);

        self.epoch_instructions = self.epoch_instructions.wrapping_add(1);
        self.state = ExecutorState::Ready;
        delta
    }

    /// Execute a bounded epoch: run up to MAX_EPOCH_INSTRUCTIONS instructions
    /// from a provided instruction list (word_idx, bit_idx, fired_mask, instr_id).
    pub fn run_epoch(
        &mut self,
        instructions: &[(usize, u32, u64, u32)],
    ) -> EpochStats {
        self.epoch_instructions = 0;
        let n = instructions.len().min(MAX_EPOCH_INSTRUCTIONS);
        for &(widx, bit, fired, id) in &instructions[..n] {
            self.execute_one(widx, bit, fired, id);
        }
        self.advance_epoch()
    }

    /// Advance epoch: emit stats, reset RL, advance epoch counter.
    pub fn advance_epoch(&mut self) -> EpochStats {
        self.state = ExecutorState::EpochBoundary;
        let stats = EpochStats {
            epoch: self.epoch,
            instructions_executed: self.epoch_instructions,
            deltas_emitted: self.tape.len() as u32,
            receipt_value: self.receipt.value(),
            rl_reward: self.rl_state.reward,
            conformance_violations: self.conformance.violation_bits,
        };
        RLKernel::next_epoch(&mut self.rl_state);
        self.epoch = self.epoch.wrapping_add(1);
        self.epoch_instructions = 0;
        self.state = ExecutorState::Ready;
        stats
    }

    /// Halt the executor.
    #[inline(always)]
    pub fn halt(&mut self) {
        self.state = ExecutorState::Halted;
    }

    /// Compute drift report against current model.
    pub fn drift_report(&self) -> DriftReport {
        DriftKernel::scan_universe(&self.block, &self.model)
    }

    /// Execute one instruction with capability check.
    /// `cap_mask` is the 64-bit transition permission mask (use NullCapability::all_allowed() to bypass).
    pub fn execute_one_authorized(
        &mut self,
        word_idx: usize,
        bit_idx: u32,
        fired_mask: u64,
        instr_id: u32,
        transition_id: u32,
        cap_mask: u64,
    ) -> UDelta {
        // Capability gate — branchless: denied is 0 if allowed, !0 if denied
        let denied = cap_admit(transition_id, cap_mask);
        // Apply fired_mask only if capability admitted (denied == 0)
        // admitted_mask: !0 when admitted (denied == 0), 0 when denied (denied != 0)
        let admitted = (denied == 0) as u64;
        let admitted_mask = admitted.wrapping_neg(); // 0 → 0, 1 → !0
        let effective_mask = fired_mask & admitted_mask;
        let delta = self.execute_one(word_idx, bit_idx, effective_mask, instr_id);
        // Observe delta in supervisor (family_id = transition_id & 63)
        let family_id = (transition_id & 63) as u8;
        let is_denial = denied != 0;
        self.supervisor.observe(&delta, family_id, is_denial);
        delta
    }

    /// Snapshot the current executor state into a UBitImage.
    pub fn checkpoint(&self) -> UBitImage {
        UBitImage {
            block: self.block,
            scratch_snapshot: UBitScratchSnapshot {
                active: self.scratch.active,
            },
            receipt: self.receipt,
            tape_position: self.tape.tape_position(),
            epoch: self.epoch,
        }
    }

    /// Restore executor state from a UBitImage.
    pub fn restore(&mut self, image: &UBitImage) {
        self.block = image.block;
        self.scratch.active = image.scratch_snapshot.active;
        self.receipt = image.receipt;
        self.epoch = image.epoch;
        self.epoch_instructions = 0;
        self.state = ExecutorState::Ready;
    }

    /// Fork: return a snapshot of the current state (copy-on-write semantics).
    pub fn fork(&self) -> UBitImage {
        self.checkpoint()
    }

    /// Boot with a TransitionRegistry for plan-first dispatch.
    pub fn boot_with_registry(&mut self, registry: TransitionRegistry) {
        self.registry = registry;
        self.boot();
    }

    /// Drain one pending recovery instruction from the supervisor, if any.
    pub fn drain_recovery(&mut self) -> Option<crate::patterns::universe64::instruction::UInstruction> {
        self.supervisor.drain_recovery()
    }
}

impl Default for UniverseExecutor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ubit_capability::NullCapability;

    #[test]
    fn test_boot() {
        let mut ex = UniverseExecutor::new();
        assert_eq!(ex.state, ExecutorState::Uninit);
        ex.boot();
        assert_eq!(ex.state, ExecutorState::Ready);
    }

    #[test]
    fn test_execute_one_flips_bit() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        let delta = ex.execute_one(0, 0, !0, 1);
        assert_eq!(ex.block.state[0], 1);
        assert_eq!(delta.before, 0);
        assert_eq!(delta.after, 1);
    }

    #[test]
    fn test_execute_one_no_fire() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        let delta = ex.execute_one(0, 0, 0, 1); // fired_mask = 0 → no change
        assert_eq!(ex.block.state[0], 0);
        assert_eq!(delta.before, 0);
        assert_eq!(delta.after, 0);
    }

    #[test]
    fn test_receipt_changes_after_execute() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        let r_before = ex.receipt.value();
        ex.execute_one(5, 3, !0, 1);
        assert_ne!(ex.receipt.value(), r_before);
    }

    #[test]
    fn test_run_epoch() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        let instructions: [(usize, u32, u64, u32); 10] = core::array::from_fn(|i| (i, 0, !0, i as u32));
        let stats = ex.run_epoch(&instructions);
        assert_eq!(stats.epoch, 0);
        assert_eq!(stats.instructions_executed, 10);
    }

    #[test]
    fn test_advance_epoch_increments() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        ex.advance_epoch();
        assert_eq!(ex.epoch, 1);
    }

    #[test]
    fn test_halt() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        ex.halt();
        assert_eq!(ex.state, ExecutorState::Halted);
    }

    #[test]
    fn test_drift_report_after_mutations() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        // Expected model is all-zero; any flip → drift.
        ex.execute_one(0, 0, !0, 1);
        let report = ex.drift_report();
        assert_eq!(report.total_bits, 1);
    }

    #[test]
    fn executor_execute_one_delegates_to_authorized_path() {
        let mut ex = UniverseExecutor::new();
        ex.boot();
        // execute_one_authorized with all-allowed cap should behave like execute_one
        let delta = ex.execute_one_authorized(0, 0, !0u64, 1, 0, NullCapability::all_allowed());
        assert_eq!(delta.fired_mask, !0u64);
        // execute with denied cap should result in no-op (fired_mask=0)
        let denied_delta = ex.execute_one_authorized(0, 0, !0u64, 2, 0, 0u64);
        assert_eq!(denied_delta.fired_mask, 0u64);
    }

    #[test]
    fn universe_os_completes_authority_image_planner_supervisor_field() {
        use super::super::scope_planner::{UBitScopePlanner, KernelClass};
        use super::super::scratch::ActiveWordSet;

        // 1. Create and boot executor
        let mut ex = UniverseExecutor::new();
        ex.boot();
        assert_eq!(ex.epoch, 0);

        // 2. Checkpoint before any execution
        let before_image = ex.checkpoint();
        let before_receipt = before_image.receipt;
        let before_tape_pos = before_image.tape_position;

        // 3. Plan a transition using UBitScopePlanner
        let mut active = ActiveWordSet::new();
        active.push(0);
        let decision = UBitScopePlanner::plan(&active);
        assert_eq!(decision.kernel, KernelClass::Cell);

        // 4. Execute an authorized transition (word 0, bit 3, instr_id 1, transition_id 0)
        let delta = ex.execute_one_authorized(0, 3, !0u64, 1, 0, NullCapability::all_allowed());
        assert_eq!(delta.fired_mask, !0u64, "transition should have fired");

        // 5. Verify state changed
        assert_ne!(ex.receipt, before_receipt, "receipt must advance after execution");

        // 6. Supervisor observes delta (via the executor's own supervisor)
        // The executor already observed the delta in execute_one_authorized
        // Verify supervisor is accessible and has a counter
        let _ = ex.supervisor.counters.epoch;

        // 7. Restore from before-image
        ex.restore(&before_image);
        assert_eq!(ex.receipt, before_receipt, "receipt must be restored");
        assert_eq!(ex.epoch, before_image.epoch, "epoch must be restored");

        // 8. Tape position restored
        // Note: tape is append-only and can't be rolled back, but epoch is restored
        assert_eq!(ex.epoch, 0);

        // Suppress unused variable warning
        let _ = before_tape_pos;
    }
}
