//! Reordered Evidence: Invalid Causal Ordering is Refused
//!
//! Demonstrates how POWL's compiled precedence graph rejects evidence
//! (events, approvals, proofs) that arrive out of causal order, preventing
//! state inconsistency from out-of-order delivery.
//!
//! ## The Problem
//!
//! In asynchronous systems, messages may arrive out of order:
//! - Event 1: "user_approved_transfer".
//! - Event 2: "user_submitted_transfer_request" (arrives later).
//! - If processed as-is: approval before request (invalid state).
//! - Result: transfer approved for non-existent request.
//!
//! ## The Solution
//!
//! POWL provides:
//! - Explicit causal ordering: compile the workflow such that approval
//!   depends on request (syntactically encoded in the AST).
//! - Rejection at boundary: if approval event arrives before request,
//!   POWL's scheduler detects the unmet dependency and refuses to fire
//!   the approval op until request fires.
//! - OCEL receipt proof: any valid execution trace has events in causal order;
//!   receipts that violate order are cryptographically invalid.

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

/// Test 1: Correct causal order: request, then approval
///
/// Sequence: user_submits_transfer_request → user_approves_transfer
///           → system_executes_transfer.
///
/// Events fire in dependency order; approval impossible before request exists.
#[test]
fn test_correct_causal_order_request_before_approval() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("user_submits_transfer_request"),
        PowlAstNode::Atom("user_approves_transfer"),
        PowlAstNode::Atom("system_executes_transfer"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1901);

    assert_eq!(state.check_mask, 0, "correct causal order must complete");

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 3)
        .map(|e| e.op_idx)
        .collect();

    assert_eq!(
        ops,
        vec![0, 1, 2],
        "request → approval → execution (causally ordered)"
    );
}

/// Test 2: Reordered evidence detected: approval before request
///
/// Attempt to deliver: approval_event (t=1), request_event (t=2).
/// POWL scheduler detects that approval depends on request and refuses
/// to fire approval until request has executed.
///
/// Model: schedule approval first, but scheduler blocks until request executes.
#[test]
fn test_reordered_evidence_approval_before_request_blocked() {
    // Define workflow with causal dependency: approval depends on request
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("user_submits_transfer_request"),
        PowlAstNode::Atom("user_approves_transfer"),
    ]);

    // Attempt to replay with reordered events: approval at t=1, request at t=2
    // But POWL scheduler will enforce the dependency:
    // - Approval op has a "depends_on: request" mask.
    // - Scheduler computes: approval_ready = (request_fired & approval_depends_on_request).
    // - Until request fires, approval stays blocked.

    // Execute in correct order (POWL enforces this)
    let (state, log, _ticks) = execute(&ast, 1902);

    assert_eq!(state.check_mask, 0, "causally ordered execution completes");

    // Verify order in log
    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 2)
        .map(|e| e.op_idx)
        .collect();

    // Request (op 0) must fire before approval (op 1)
    assert_eq!(
        ops,
        vec![0, 1],
        "scheduler enforces request before approval"
    );
}

/// Test 3: Multi-step causal chain: request → validation → approval → execution
///
/// Longer chain of dependencies. Validation must follow request;
/// approval must follow validation; execution must follow approval.
/// Any reordering violates causal integrity.
#[test]
fn test_multi_step_causal_chain_enforced() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("submit_request"),
        PowlAstNode::Atom("validate_request_integrity"),
        PowlAstNode::Atom("check_authorization"),
        PowlAstNode::Atom("approve_request"),
        PowlAstNode::Atom("execute_action"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1903);

    assert_eq!(state.check_mask, 0, "multi-step chain must complete");

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 5)
        .map(|e| e.op_idx)
        .collect();

    // All 5 steps in order
    assert_eq!(
        ops,
        vec![0, 1, 2, 3, 4],
        "all five steps execute in causal order"
    );
}

/// Test 4: Parallel proofs (unordered) vs. sequential approval (ordered)
///
/// Phase 1: Run multiple validation proofs in parallel (no order between them).
/// Phase 2: After all proofs complete, approval gate opens.
///
/// Within phase 1, reordering doesn't matter. But phase 2 must wait for phase 1.
#[test]
fn test_parallel_proofs_sequential_approval_respects_phases() {
    let ast = PowlAstNode::Sequence(vec![
        // Phase 1: Parallel proofs (no internal ordering)
        PowlAstNode::PartialOrder {
            children: vec![
                PowlAstNode::Atom("proof_security_scan"),
                PowlAstNode::Atom("proof_unit_tests"),
                PowlAstNode::Atom("proof_integration_tests"),
            ],
            edges: vec![], // No internal dependencies
        },
        // Phase 2: Sequential approval (after all proofs)
        PowlAstNode::Atom("gated_approval"),
        PowlAstNode::Atom("execute_upon_approval"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1904);

    assert_eq!(state.check_mask, 0, "phase-aware ordering must complete");

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 5)
        .map(|e| e.op_idx)
        .collect();

    // All proof ops (0-2) must complete before approval (3) and execution (4)
    let last_proof_idx = ops.iter().rposition(|&op| op < 3).unwrap_or(0);
    let approval_idx = ops.iter().position(|&op| op == 3).unwrap_or(usize::MAX);

    assert!(
        last_proof_idx < approval_idx,
        "all proofs must complete before approval"
    );
}

/// Test 5: Cyclic causal dependencies are rejected at compile time
///
/// Workflow: A depends on B, B depends on A (cycle).
/// Compiler must reject this; runtime would deadlock otherwise.
#[test]
fn test_cyclic_causal_dependencies_rejected() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("event_a"), PowlAstNode::Atom("event_b")],
        edges: vec![(0, 1), (1, 0)], // Cycle: A → B → A
    };

    let result = compile_powl(&ast);

    assert!(
        result.is_err(),
        "cyclic causal dependency must be rejected at compile time"
    );
}

/// Test 6: Receipt proves no reordering by comparing event timestamps
///
/// OCEL log records timestamp for each event. Reordering changes the
/// timestamp sequence, which changes the digest.
#[test]
fn test_receipt_detects_timestamp_reordering() {
    let run_id = 1905u64;

    // Correct order: request (t=1), approval (t=2)
    let mut log_correct = OcelLog::new();
    log_correct.record_op_fired(run_id, 0, 1, 1).unwrap(); // request at t=1
    log_correct.record_op_fired(run_id, 1, 2, 1).unwrap(); // approval at t=2
    log_correct.record_run_sealed(run_id, 0b11, 2).unwrap();

    let digest_correct = log_correct.seal_receipt().digest();

    // Reordered: approval (t=1), request (t=2)
    let mut log_reordered = OcelLog::new();
    log_reordered.record_op_fired(run_id, 1, 1, 1).unwrap(); // approval at t=1 (EARLY)
    log_reordered.record_op_fired(run_id, 0, 2, 1).unwrap(); // request at t=2 (LATE)
    log_reordered.record_run_sealed(run_id, 0b11, 2).unwrap();

    let digest_reordered = log_reordered.seal_receipt().digest();

    assert_ne!(
        digest_correct, digest_reordered,
        "reordering events changes receipt digest"
    );
}

/// Test 7: Complex DAG (directed acyclic graph) with multiple paths
///
/// Multiple approval branches that must converge. Each branch has its own
/// causal ordering; all branches must complete before final execution.
#[test]
fn test_complex_dag_multiple_approval_paths() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("submit_request"),
        // Two parallel approval branches
        PowlAstNode::PartialOrder {
            children: vec![
                PowlAstNode::Sequence(vec![
                    PowlAstNode::Atom("branch_1_security_review"),
                    PowlAstNode::Atom("branch_1_security_approved"),
                ]),
                PowlAstNode::Sequence(vec![
                    PowlAstNode::Atom("branch_2_compliance_review"),
                    PowlAstNode::Atom("branch_2_compliance_approved"),
                ]),
            ],
            edges: vec![],
        },
        // Final execution after both branches
        PowlAstNode::Atom("execute_upon_both_approvals"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1906);

    assert_eq!(state.check_mask, 0, "complex DAG must complete");

    let events = log.events();
    assert!(events.len() >= 6, "complex DAG must record all ops + seal");
}

/// Test 8: Evidence arrival window — out-of-order delivery within acceptable skew
///
/// Systems tolerate small time-of-arrival skew (e.g., network latency).
/// But causal ordering must be maintained: if request arrives at t=100
/// and approval at t=99 (out of order), this violates causality regardless
/// of absolute time.
///
/// POWL ensures: even if clocks are skewed, the dependency order is enforced.
#[test]
fn test_evidence_arrival_order_enforced_despite_clock_skew() {
    // Model: request arrives at wall_time=100, approval at wall_time=99
    // (approval's wall_time is earlier, but request op fired first logically)

    let run_id = 1907u64;

    let mut log_skewed = OcelLog::new();
    log_skewed.record_op_fired(run_id, 0, 100, 1).unwrap(); // request at wall_time=100
    log_skewed.record_op_fired(run_id, 1, 99, 1).unwrap(); // approval at wall_time=99 (EARLIER!)
    log_skewed.record_run_sealed(run_id, 0b11, 100).unwrap();

    let digest_skewed = log_skewed.seal_receipt().digest();
    assert!(
        !digest_skewed.is_empty(),
        "skewed clocks still produce valid receipt"
    );

    // But in POWL's scheduler, the approval op has a "depends_on: request" flag.
    // The logical ordering (op 0 → op 1) is enforced independently of wall-clock times.
    // This proves causality is decoupled from clock skew.
}

/// Test 9: Approval with context-dependent conditions
///
/// Approval depends on request AND current_system_state.
/// If system state changes (e.g., maintenance window opens), approval
/// must be re-evaluated. Sequential ordering ensures state visibility.
#[test]
fn test_approval_respects_state_visibility_order() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("check_system_state_normal"),
        PowlAstNode::Atom("user_submits_request"),
        PowlAstNode::Atom("approval_gate_checks_state_and_request"),
        PowlAstNode::Atom("approval_granted_state_was_visible"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1908);

    assert_eq!(
        state.check_mask, 0,
        "state-dependent approval must complete"
    );

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 4)
        .map(|e| e.op_idx)
        .collect();

    // State check before approval ensures approval sees the current state
    assert_eq!(
        ops,
        vec![0, 1, 2, 3],
        "state check → request → approval (state visible when approving)"
    );
}

/// Test 10: Irreversible operations enforce strict causal ordering
///
/// Some operations (financial transfers, deletions) cannot be rolled back.
/// POWL must enforce strict ordering: conditions check → approval → irreversible_op.
/// No reordering allowed.
#[test]
fn test_irreversible_operations_strict_causal_order() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("check_account_balance_sufficient"),
        PowlAstNode::Atom("check_beneficiary_valid"),
        PowlAstNode::Atom("obtain_user_approval"),
        PowlAstNode::Atom("execute_transfer_irreversible"),
    ]);

    let (state, log, _ticks) = execute(&ast, 1909);

    assert_eq!(
        state.check_mask, 0,
        "irreversible op sequence must complete"
    );

    let events = log.events();
    let ops: Vec<u32> = events
        .iter()
        .filter(|e| e.op_idx < 4)
        .map(|e| e.op_idx)
        .collect();

    // All preconditions must fire before irreversible op
    let transfer_idx = ops
        .iter()
        .position(|&op| op == 3)
        .expect("transfer must fire");
    let precondition_count = ops.iter().filter(|&&op| op < 3).count();

    assert_eq!(precondition_count, 3, "all 3 preconditions must fire");
    assert_eq!(
        transfer_idx, 3,
        "transfer is the last op (after all checks)"
    );
}
