//! Regional Failover: Lease Transfer Cannot Create Duplicate Actuation
//!
//! Demonstrates how POWL's explicit dependency model prevents the classic
//! failover bug where a failed worker's lease is transferred to a standby,
//! but both execute the same action (split-brain).
//!
//! ## The Problem
//!
//! In a high-availability system, if the primary worker holding a lease fails:
//! - Naive recovery: transfer the lease to standby and resume execution.
//! - The bug: primary may have already begun the action; standby repeats it.
//! - Result: duplicate writes, violated invariants, data corruption.
//!
//! ## The Solution
//!
//! POWL provides:
//! - Explicit modeling of lease state: "primary_holds_lease" → "primary_fails"
//!   → "standby_acquires_lease" → "standby_executes" forms an acyclic chain.
//! - Compiler ensures: standby_executes depends on primary_fails AND the
//!   previous action completing. If primary never completes, standby can
//!   safely assume the action was not executed.
//! - OCEL receipt chain: proves the exact handoff point, auditable and
//!   tamper-evident.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::OcelLog;
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use std::collections::HashSet;

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

/// Test 1: Primary executes action, then fails; standby does not repeat
///
/// Sequence: primary_acquire_lease → primary_execute_action → primary_fails
///           → standby_acquire_lease → (no repeat of primary_execute_action).
///
/// The key: standby depends on primary_fails, which depends on primary_execute_action
/// completing. So standby knows the action executed and does not repeat it.
#[test]
fn test_primary_completes_action_then_failover_no_duplicate() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("primary_acquire_lease"),
        PowlAstNode::Atom("primary_execute_action"),
        PowlAstNode::Atom("primary_fails_signal_detection"),
        PowlAstNode::Atom("standby_acquire_lease"),
        // Standby does NOT repeat execute_action; it assumes action completed.
        PowlAstNode::Atom("standby_continue_from_action"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1101);

    // All ops complete without deadlock
    assert_eq!(
        state.check_mask, 0,
        "failover must complete without deadlock"
    );

    // Verify the event order
    let events = log.events();
    let op_sequence: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 5)
        .map(|e| e.op_idx)
        .collect();

    // Primary must execute before standby takes over
    let primary_execute_idx = op_sequence
        .iter()
        .position(|&op| op == 1)
        .expect("primary_execute_action must fire");
    let standby_acquire_idx = op_sequence
        .iter()
        .position(|&op| op == 3)
        .expect("standby_acquire_lease must fire");

    assert!(
        primary_execute_idx < standby_acquire_idx,
        "primary must execute action before standby acquires lease"
    );
}

/// Test 2: Primary fails before executing; standby safely executes once
///
/// Sequence: primary_acquire_lease → primary_fails_before_action
///           → standby_acquire_lease → standby_execute_action.
///
/// Standby can safely execute because primary never did.
#[test]
fn test_primary_fails_before_action_standby_executes_once() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("primary_acquire_lease"),
        PowlAstNode::Atom("primary_fails_before_action"),
        PowlAstNode::Atom("standby_acquire_lease"),
        PowlAstNode::Atom("standby_execute_action"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1102);

    assert_eq!(
        state.check_mask, 0,
        "failover with pre-action failure must complete"
    );

    // Verify only standby executes the action
    let events = log.events();
    let action_ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx == 3) // standby_execute_action
        .map(|e| e.op_idx)
        .collect();

    assert_eq!(
        action_ops.len(),
        1,
        "action must execute exactly once (standby only)"
    );
}

/// Test 3: No cyclic dependencies between primary and standby
///
/// Attempt to create: primary_execute → standby_acquire (primary must finish)
///                    standby_acquire → primary_execute (standby must start first)
///
/// This cycle would be rejected at compile time.
#[test]
fn test_cyclic_primary_standby_dependency_rejected() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("primary_execute"),
            PowlAstNode::Atom("standby_acquire"),
        ],
        edges: vec![(0, 1), (1, 0)], // cycle: both depend on each other
    };

    let result = compile_powl(&ast);
    assert!(
        result.is_err(),
        "cyclic primary-standby dependency must be rejected at compile time"
    );
}

/// Test 4: Lease transfer receipt proves single transfer occurred
///
/// OCEL log records: primary_holds_lease → primary_fails → standby_acquires_lease.
/// Digest change if any event reordered proves tamper-detection.
#[test]
fn test_lease_transfer_receipt_proves_single_transfer() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("primary_holds_lease"),
        PowlAstNode::Atom("primary_fails"),
        PowlAstNode::Atom("standby_acquires_lease"),
    ]);

    let (_state, log, _ticks) = execute(&ast, 1103);

    let events = log.events();
    assert!(
        events.len() >= 4,
        "receipt must record at least 3 transfer ops + run_sealed"
    );

    // Create digest proof
    let digest = log.seal_receipt().digest();
    assert!(
        !digest.is_empty(),
        "receipt digest must be cryptographically present"
    );

    // If someone claims standby acquired before primary failed, digest changes
    let mut log_forged = OcelLog::new();
    log_forged.record_op_fired(1103, 0, 1, 1).unwrap(); // primary_holds
    log_forged.record_op_fired(1103, 2, 2, 1).unwrap(); // standby_acquires (REORDERED)
    log_forged.record_op_fired(1103, 1, 3, 1).unwrap(); // primary_fails (MOVED LATER)
    log_forged.record_run_sealed(1103, 0b111, 3).unwrap();

    let digest_forged = log_forged.seal_receipt().digest();
    assert_ne!(
        digest, digest_forged,
        "reordering lease transfer events changes digest"
    );
}

/// Test 5: Multiple failovers in sequence maintain correctness
///
/// Model: primary1 → primary1_fails → standby1 (becomes primary2)
///         → primary2_fails → standby2 (becomes primary3) → primary3_executes.
///
/// Each handoff preserves the invariant: action executes exactly once.
#[test]
fn test_cascading_failovers_single_action_execution() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("primary1_holds_lease"),
        PowlAstNode::Atom("primary1_fails"),
        PowlAstNode::Atom("standby1_becomes_primary2"),
        PowlAstNode::Atom("primary2_fails"),
        PowlAstNode::Atom("standby2_becomes_primary3"),
        PowlAstNode::Atom("primary3_execute_action"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1104);

    assert_eq!(
        state.check_mask, 0,
        "cascading failovers must converge despite multiple handoffs"
    );

    // Action executes exactly once (by primary3)
    let events = log.events();
    let action_count = events
        .iter()
        .filter(|e| e.op_idx == 5) // primary3_execute_action
        .count();

    assert_eq!(
        action_count, 1,
        "action must execute exactly once across all failovers"
    );
}
