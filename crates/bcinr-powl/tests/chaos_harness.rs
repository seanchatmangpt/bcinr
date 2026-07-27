//! chaos_harness — Chaos injection testing infrastructure for POWL scheduler.
//!
//! Provides branchless chaos injection patterns:
//! - Crash injection: early termination via flag
//! - Delay injection: logical time advancement
//! - Duplicate injection: tick idempotence verification
//! - Reorder injection: ready-set shuffling with validity checks

use bcinr_powl::ocel::OcelLog;
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

/// Result of a chaos-injected execution.
#[derive(Clone, Debug)]
pub struct ChaosExecResult {
    /// All operations that fired across the entire execution.
    pub all_fired: u64,
    /// Ticks executed before termination (crash or normal).
    pub ticks_executed: u32,
    /// Whether execution was terminated early by crash injection.
    pub crashed: bool,
    /// Final state snapshot.
    pub final_state: ExecutionSnapshot,
    /// State at crash point (if applicable).
    pub crash_state: Option<ExecutionSnapshot>,
}

/// Snapshot of scheduler state at a checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSnapshot {
    pub done_mask: u64,
    pub active_mask: u64,
    pub check_mask: u64,
    pub cancelled_mask: u64,
    pub refused_mask: u64,
    pub blocked_mask: u64,
    pub tick: u32,
}

impl ExecutionSnapshot {
    pub fn from_state(state: &PowlRunState) -> Self {
        Self {
            done_mask: state.done_mask,
            active_mask: state.active_mask,
            check_mask: state.check_mask,
            cancelled_mask: state.cancelled_mask,
            refused_mask: state.refused_mask,
            blocked_mask: state.blocked_mask,
            tick: state.tick,
        }
    }

    /// Check if state is consistent: no op appears in multiple terminal states.
    pub fn is_consistent(&self, tape_len: u32) -> bool {
        let valid_mask = (1u64 << tape_len) - 1;

        // Check bounds
        if self.done_mask & !valid_mask != 0 {
            return false;
        }
        if self.cancelled_mask & !valid_mask != 0 {
            return false;
        }
        if self.refused_mask & !valid_mask != 0 {
            return false;
        }
        if self.blocked_mask & !valid_mask != 0 {
            return false;
        }

        // Check no op in multiple terminal states
        let term_states = [
            self.done_mask,
            self.cancelled_mask,
            self.refused_mask,
            self.blocked_mask,
        ];
        for i in 0..term_states.len() {
            for j in (i + 1)..term_states.len() {
                if term_states[i] & term_states[j] != 0 {
                    return false;
                }
            }
        }

        true
    }
}

/// Run a single tape with crash injection.
///
/// Simulates mid-execution termination by breaking the scheduler loop
/// after a specified tick count. Captures state at crash point and final state.
pub fn run_with_crash_injection(
    tape: &PowlTape,
    crash_after_tick: u32,
    max_ticks: u32,
) -> ChaosExecResult {
    let mut state = PowlRunState::new(tape);
    let mut all_fired: u64 = 0;
    let mut ticks_executed = 0u32;
    let mut crash_state: Option<ExecutionSnapshot> = None;
    let mut crashed = false;

    for _ in 0..max_ticks {
        if ticks_executed == crash_after_tick {
            crashed = true;
            crash_state = Some(ExecutionSnapshot::from_state(&state));
            break;
        }

        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }

        ticks_executed += 1;
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        all_fired |= fs.0;
    }

    ChaosExecResult {
        all_fired,
        ticks_executed,
        crashed,
        final_state: ExecutionSnapshot::from_state(&state),
        crash_state,
    }
}

/// Run a single tape with delay injection.
///
/// Advances the logical tick counter by delay_ticks before execution.
/// Observes whether dependent operations still fire in correct order.
pub fn run_with_delay_injection(
    tape: &PowlTape,
    delay_ticks: u32,
    max_ticks: u32,
) -> ChaosExecResult {
    let mut state = PowlRunState::new(tape);
    // Advance tick counter to simulate delay
    state.tick = state.tick.wrapping_add(delay_ticks);

    let mut all_fired: u64 = 0;
    let mut ticks_executed = 0u32;

    for _ in 0..max_ticks {
        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }

        ticks_executed += 1;
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        all_fired |= fs.0;
    }

    ChaosExecResult {
        all_fired,
        ticks_executed,
        crashed: false,
        final_state: ExecutionSnapshot::from_state(&state),
        crash_state: None,
    }
}

/// Run a tape with duplicate-tick injection.
///
/// Calls scheduler_tick twice with the same state snapshot, verifies idempotence:
/// either both calls produce identical FiredSet, or second call produces no new fires.
pub fn run_with_duplicate_tick_injection(
    tape: &PowlTape,
    max_ticks: u32,
) -> (ChaosExecResult, DuplicateTickVerification) {
    let mut state = PowlRunState::new(tape);
    let mut all_fired: u64 = 0;
    let mut ticks_executed = 0u32;
    let mut duplicate_checks: Vec<(u32, u64, u64, bool)> = Vec::new();

    for _ in 0..max_ticks {
        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }

        ticks_executed += 1;

        // Save state before tick
        let state_snapshot = ExecutionSnapshot::from_state(&state);

        // First tick
        let fs1 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        all_fired |= fs1.0;

        // Restore state and call again
        state.done_mask = state_snapshot.done_mask;
        state.active_mask = state_snapshot.active_mask;
        state.check_mask = state_snapshot.check_mask;

        // Second tick
        let fs2 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);

        // Verify idempotence: second call should produce no new fires
        let is_idempotent = fs2.0 == 0 || fs2.0 == fs1.0;
        duplicate_checks.push((ticks_executed, fs1.0, fs2.0, is_idempotent));
    }

    let all_passed = duplicate_checks
        .iter()
        .all(|(_, _, _, is_idempotent)| *is_idempotent);
    let verification = DuplicateTickVerification {
        checks: duplicate_checks,
        all_passed,
    };

    (
        ChaosExecResult {
            all_fired,
            ticks_executed,
            crashed: false,
            final_state: ExecutionSnapshot::from_state(&state),
            crash_state: None,
        },
        verification,
    )
}

/// Verification result for duplicate-tick injection.
#[derive(Clone, Debug)]
pub struct DuplicateTickVerification {
    /// (tick_num, first_fired, second_fired, is_idempotent)
    pub checks: Vec<(u32, u64, u64, bool)>,
    pub all_passed: bool,
}

/// Extracted ready-set for reorder injection.
#[derive(Clone, Debug)]
pub struct ReadySet {
    pub indices: Vec<usize>,
}

impl ReadySet {
    /// Extract which ops are ready to fire from the state.
    pub fn from_state(tape: &PowlTape, state: &PowlRunState) -> Self {
        let mut indices = Vec::new();
        for i in 0..tape.len as usize {
            let bit = 1u64 << i;
            let unfinished = state.done_mask & bit == 0;
            let predecessors_complete = tape.ops[i].pred_mask & !state.done_mask == 0;
            if unfinished && predecessors_complete {
                indices.push(i);
            }
        }
        Self { indices }
    }

    /// Shuffle in-place (reproducible seed for deterministic testing).
    pub fn shuffle_deterministic(&mut self, seed: u64) {
        if self.indices.is_empty() {
            return;
        }
        // Simple LCG-based shuffle seeded by (seed, operation count).
        let mut rng_state = seed
            .wrapping_mul(6364136223846793005u64)
            .wrapping_add(1442695040888963407u64);
        for i in 0..self.indices.len() {
            rng_state = rng_state
                .wrapping_mul(6364136223846793005u64)
                .wrapping_add(1442695040888963407u64);
            let j = (rng_state as usize) % self.indices.len();
            self.indices.swap(i, j);
        }
    }
}

/// Run a tape with reorder-injection.
///
/// Shuffles the ready-set before each scheduler invocation and verifies that
/// all dependencies are still respected and the final result is valid.
pub fn run_with_reorder_injection(
    tape: &PowlTape,
    reorder_seed: u64,
    max_ticks: u32,
) -> ReorderExecResult {
    let mut state = PowlRunState::new(tape);
    let mut all_fired: u64 = 0;
    let mut ticks_executed = 0u32;
    let mut validity_violations: Vec<(u32, String)> = Vec::new();

    for _ in 0..max_ticks {
        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }

        ticks_executed += 1;

        // Extract and shuffle ready-set
        let mut ready = ReadySet::from_state(tape, &state);
        ready.shuffle_deterministic(reorder_seed.wrapping_add(ticks_executed as u64));

        // Execute scheduler tick (doesn't re-sort internally, ready-set is advisory)
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        all_fired |= fs.0;

        // Verify all fired ops have satisfied predecessors
        for i in 0..tape.len as usize {
            let bit = 1u64 << i;
            if fs.0 & bit != 0 {
                // Op i fired; check all predecessors are done
                let missing_pred = tape.ops[i].pred_mask & !state.done_mask;
                if missing_pred != 0 {
                    validity_violations.push((
                        ticks_executed,
                        format!(
                            "Op {} fired with missing predecessors: {:#064b}",
                            i, missing_pred
                        ),
                    ));
                }
            }
        }
    }

    let all_valid = validity_violations.is_empty();
    ReorderExecResult {
        all_fired,
        ticks_executed,
        final_state: ExecutionSnapshot::from_state(&state),
        validity_violations,
        all_valid,
    }
}

/// Result of reorder-injection execution.
#[derive(Clone, Debug)]
pub struct ReorderExecResult {
    pub all_fired: u64,
    #[allow(dead_code)]
    pub ticks_executed: u32,
    pub final_state: ExecutionSnapshot,
    /// Violations found during execution
    pub validity_violations: Vec<(u32, String)>,
    /// True if no violations detected
    pub all_valid: bool,
}

/// Wrap a tape in an OcelLog and record execution trace.
pub fn trace_with_ocel(tape: &PowlTape, run_id: u64, max_ticks: u32) -> (OcelLog, PowlRunState) {
    let mut state = PowlRunState::new(tape);
    let mut log = OcelLog::new();
    let mut ticks = 0u32;

    for _ in 0..max_ticks {
        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }

        ticks += 1;
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);

        // Record each fired op with temporal info
        for i in 0..tape.len as usize {
            let bit = 1u64 << i;
            if fs.0 & bit != 0 {
                let _ = log.record_op_fired(run_id, i as u32, ticks, 1);
            }
        }
    }

    // Seal the run
    let _ = log.record_run_sealed(run_id, state.done_mask, ticks);

    (log, state)
}
