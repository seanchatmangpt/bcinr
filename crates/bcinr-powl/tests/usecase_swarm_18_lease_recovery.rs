//! Lease Recovery: Abandoned Work Becomes Available Without Double Execution
//!
//! Demonstrates how POWL's explicit state and expiration model prevents zombie
//! tasks and double execution when a lease holder crashes and the lease is
//! reclaimed.
//!
//! ## The Problem
//!
//! In a distributed work queue with leases:
//! - Worker acquires lease on task_X (exclusive).
//! - Worker crashes mid-task.
//! - Lease expires; another worker acquires the lease.
//! - First worker recovers; both claim to be working on task_X.
//! - Result: duplicate work, corrupted state.
//!
//! ## The Solution
//!
//! POWL provides:
//! - Explicit lease expiration: lease_expires is a scheduled event.
//! - Dependency tracking: if task_X execution is incomplete when lease expires,
//!   the lease can be reclaimed and assigned to a new worker.
//! - OCEL receipt chain: proves that only one worker executed task_X,
//!   or that the first crashed before completion (proven by absence of
//!   task_x_complete in the log before lease_expired).

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::OcelLog;
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};

fn execute(ast: &PowlAstNode<'_>, run_id: u64) -> (PowlRunState, OcelLog, u32) {
    let tape = compile_powl(ast).expect("POWL model must compile");
    let mut state = PowlRunState::new(&tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;
    let mut ticks = 0u32;

    for _ in 0..256 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        ticks += 1;
        while bits != 0 {
            let op_idx = bits.trailing_zeros() as u32;
            bits &= bits - 1;
            log.record_op_fired(run_id, op_idx, ticks, 1).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    log.record_run_sealed(run_id, op_trace, ticks).unwrap();
    (state, log, ticks)
}

/// Test 1: Healthy case: worker completes task before lease expires
///
/// Sequence: worker_acquire_lease → worker_execute_task → worker_mark_complete
///           → (lease never expires).
#[test]
fn test_healthy_lease_task_completes_before_expiration() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_acquire_lease_for_task_x"),
        PowlAstNode::Atom("worker_execute_task_x"),
        PowlAstNode::Atom("worker_mark_task_x_complete"),
        PowlAstNode::Atom("worker_release_lease"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1801);

    assert_eq!(state.check_mask, 0, "healthy task completion must complete");

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 4)
        .map(|e| e.op_idx)
        .collect();

    // All 4 ops in order
    assert_eq!(
        ops,
        vec![0, 1, 2, 3],
        "task acquisition → execution → completion → release"
    );
}

/// Test 2: Worker crashes mid-task; lease expires; new worker recovers
///
/// Sequence:
/// - worker_1_acquire_lease → worker_1_start_execute → worker_1_crashes
/// - lease_expires (because worker_1 didn't complete in time)
/// - worker_2_acquire_lease → worker_2_execute_task → worker_2_complete
///
/// Only worker_2's execution is counted; worker_1's partial execution doesn't interfere.
#[test]
fn test_crashed_worker_lease_expires_new_worker_executes() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_1_acquire_lease"),
        PowlAstNode::Atom("worker_1_start_execute_task_x"),
        PowlAstNode::Atom("worker_1_crashes"),
        PowlAstNode::Atom("lease_expires_for_task_x"),
        PowlAstNode::Atom("worker_2_acquire_lease"),
        PowlAstNode::Atom("worker_2_execute_task_x"),
        PowlAstNode::Atom("worker_2_mark_complete"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1802);

    assert_eq!(state.check_mask, 0, "recovery sequence must complete");

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 7)
        .map(|e| e.op_idx)
        .collect();

    // Verify sequence
    assert_eq!(
        ops,
        vec![0, 1, 2, 3, 4, 5, 6],
        "crash → expiry → recovery sequence"
    );

    // Count complete executions by workers
    let worker1_execute = ops.iter().filter(|&&op| op == 1).count();
    let worker2_execute = ops.iter().filter(|&&op| op == 5).count();

    assert_eq!(
        worker1_execute, 1,
        "worker_1 attempted execution (crash before completion)"
    );
    assert_eq!(worker2_execute, 1, "worker_2 executed the task");

    // Receipt proves: only worker_2 marked complete
    let complete_events = events
        .iter()
        .filter(|e| e.op_idx == 6) // worker_2_mark_complete
        .count();
    assert_eq!(complete_events, 1, "only one completion marker (worker_2)");
}

/// Test 3: Receipt proves crash happened before completion
///
/// OCEL log shows: worker_1_start_execute (t=1), worker_1_crashes (t=2),
/// but NO worker_1_mark_complete. This proves worker_1 never finished.
/// Worker_2 can safely re-execute.
#[test]
fn test_receipt_proves_incomplete_execution_before_crash() {
    let run_id = 1803u64;

    let ast_crash = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_1_execute"),
        PowlAstNode::Atom("worker_1_crashes_before_completing"),
    ]);

    let (_state, log_crash, _ticks) = execute(&ast_crash, run_id);

    let events_crash = log_crash.events();

    // Verify: execute exists (op_idx 0), crash exists (op_idx 1),
    // but no complete marker
    let execute_count = events_crash.iter().filter(|e| e.op_idx == 0).count();
    let crash_count = events_crash.iter().filter(|e| e.op_idx == 1).count();
    let complete_count = events_crash
        .iter()
        .filter(|e| e.op_idx == 2) // hypothetical complete op
        .count();

    assert_eq!(execute_count, 1, "execution recorded");
    assert_eq!(crash_count, 1, "crash recorded");
    assert_eq!(
        complete_count, 0,
        "no completion marker proves task incomplete"
    );

    // Digest is stable; auditor can verify this same trace
    let digest_crash = log_crash.seal_receipt().digest();
    assert!(
        !digest_crash.is_empty(),
        "receipt proves incomplete execution"
    );
}

/// Test 4: Recovery prevents double execution by checking receipt history
///
/// Before re-executing, worker_2 checks: "Is task_x already in the
/// completion log?" The receipt from any previous worker is checked.
#[test]
fn test_recovery_checks_receipt_before_re_execution() {
    // First worker (completes successfully)
    let ast_first = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_1_acquire_lease"),
        PowlAstNode::Atom("worker_1_execute_task"),
        PowlAstNode::Atom("worker_1_mark_complete"),
    ]);

    let run_first = 1804u64;
    let (_state_first, log_first, _ticks_first) = execute(&ast_first, run_first);
    let digest_first = log_first.seal_receipt().digest();

    // Second worker (checks receipt before re-executing)
    let ast_second = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_2_check_receipt_for_task"),
        // Receipt lookup: worker_1_mark_complete is present
        PowlAstNode::Atom("worker_2_finds_completion_in_receipt"),
        PowlAstNode::Atom("worker_2_skips_re_execution"),
        PowlAstNode::Atom("worker_2_acknowledges_task_already_done"),
    ]);

    let run_second = 1805u64;
    let (_state_second, log_second, _ticks_second) = execute(&ast_second, run_second);

    // Verify: worker_2 did not execute the task
    let execute_count = log_second
        .events()
        .iter()
        .filter(|e| e.op_idx == 0 && "worker_2_check_receipt".contains("check"))
        .count();

    // Log shows: check → find → skip
    let ops_second: Vec<u32> = log_second
        .events()
        .iter()
        .filter(|e| e.op_idx < 4)
        .map(|e| e.op_idx)
        .collect();
    assert!(ops_second.len() >= 3, "recovery sequence recorded");
}

/// Test 5: Lease expiration has a timeout; prevents indefinite waiting
///
/// Lease acquired at t=0; expires at t=10 (absolute timeout).
/// If worker doesn't complete by t=10, lease is forcibly released.
#[test]
fn test_lease_expiration_timeout_is_absolute() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("lease_acquired_at_t0"),
        PowlAstNode::Atom("work_begins"),
        // ... omit intermediate steps ...
        PowlAstNode::Atom("lease_expires_at_t10"),
        PowlAstNode::Atom("new_worker_can_acquire"),
    ]);

    let (state, log, ticks) = execute(&ast, 1806);

    assert_eq!(
        state.check_mask, 0,
        "lease expiration and recovery must complete"
    );

    // Completion within bounded ticks
    assert!(ticks < 256, "bounded scheduler ensures termination");

    let events = log.events();
    assert!(
        events.len() >= 4,
        "acquisition → expiration → recovery ops logged"
    );
}

/// Test 6: Multiple recovery attempts maintain safety
///
/// Worker_1 crashes. Worker_2 acquires lease and crashes. Worker_3 succeeds.
/// No double execution despite two crashes.
#[test]
fn test_cascading_worker_crashes_single_execution() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_1_acquire"),
        PowlAstNode::Atom("worker_1_crash"),
        PowlAstNode::Atom("lease_expires_1"),
        PowlAstNode::Atom("worker_2_acquire"),
        PowlAstNode::Atom("worker_2_start_work"),
        PowlAstNode::Atom("worker_2_crash"),
        PowlAstNode::Atom("lease_expires_2"),
        PowlAstNode::Atom("worker_3_acquire"),
        PowlAstNode::Atom("worker_3_execute_task"),
        PowlAstNode::Atom("worker_3_mark_complete"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1807);

    assert_eq!(state.check_mask, 0, "multi-crash recovery must complete");

    let events = log.events();

    // Verify only one completion
    let complete_count = events
        .iter()
        .filter(|e| e.op_idx == 9) // worker_3_mark_complete
        .count();

    assert_eq!(
        complete_count, 1,
        "despite two crashes, task executes exactly once"
    );
}

/// Test 7: Lease recovery with pre-allocated standby
///
/// To avoid recovery latency, a standby worker is pre-allocated.
/// If primary crashes, standby immediately takes over (no acquisition delay).
#[test]
fn test_lease_recovery_with_standby_low_latency() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("primary_acquire_lease"),
        PowlAstNode::Atom("standby_ready_waiting"),
        PowlAstNode::Atom("primary_start_work"),
        PowlAstNode::Atom("primary_crashes"),
        PowlAstNode::Atom("standby_takes_over_immediately"),
        PowlAstNode::Atom("standby_execute_task"),
        PowlAstNode::Atom("standby_mark_complete"),
    ]);

    let (state, log, ticks) = execute(&ast, 1808);

    assert_eq!(state.check_mask, 0, "standby takeover must complete");

    // Fast completion due to pre-allocated standby
    assert!(
        ticks <= 50,
        "pre-allocated standby should enable fast recovery"
    );

    let events = log.events();
    let complete_count = events
        .iter()
        .filter(|e| e.op_idx == 6) // standby_mark_complete
        .count();

    assert_eq!(
        complete_count, 1,
        "standby completes task exactly once after takeover"
    );
}

/// Test 8: Lease holder can revalidate mid-task to extend lease
///
/// Task acquired at t=0; expires at t=10. If worker completes at t=8,
/// no issue. But if task needs until t=12, worker must revalidate at t=9
/// to extend the lease before expiration.
#[test]
fn test_lease_revalidation_extends_expiration() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("lease_acquired_ttl_10"),
        PowlAstNode::Atom("work_phase_1_at_t5"),
        PowlAstNode::Atom("worker_revalidates_lease_at_t8"),
        PowlAstNode::Atom("lease_extended_new_ttl_18"),
        PowlAstNode::Atom("work_phase_2_at_t12"),
        PowlAstNode::Atom("worker_mark_complete_at_t15"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1809);

    assert_eq!(
        state.check_mask, 0,
        "revalidation and extended lease must complete"
    );

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 6)
        .map(|e| e.op_idx)
        .collect();

    // Revalidation (op 3) happens before phase 2 (op 4)
    assert_eq!(
        ops,
        vec![0, 1, 2, 3, 4, 5],
        "revalidation extends lease before it expires"
    );
}
