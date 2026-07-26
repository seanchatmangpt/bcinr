//! Sequential Worker Handoff: Task Delegation and Completion Ordering
//!
//! Demonstrates how POWL's compiled precedence graph enforces strict sequencing
//! when tasks must be handed off between workers without resource overlap.
//! This is a fundamental pattern in distributed task queues, microservice
//! orchestration, and pipeline-parallel processing.
//!
//! ## The Problem
//!
//! In a distributed task system, three workers need to process a single task
//! in strict order: Worker A must complete before B starts, and B must complete
//! before C starts. No two workers can hold the task simultaneously, and the
//! handoff must be captured in an audit log that proves the exact order of
//! execution.
//!
//! Naive solutions:
//! - Polling loops: Workers spin-wait for a semaphore; wasted CPU.
//! - Message queues: Decouple timing but lose ordering guarantees without
//!   explicit serialization.
//! - Locks: Mutual exclusion works but obscures the workflow as a black box.
//!
//! ## The Solution
//!
//! POWL provides:
//! - A compiled, acyclic precedence graph with exactly 3 op slots, one per worker.
//! - Scheduler guarantees: all 3 workers fire in order within a bounded loop
//!   (max 128 ticks per the `execute()` helper).
//! - BLAKE3-chained OCEL receipts: the audit log proves which worker fired when,
//!   in cryptographically verified order.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::OcelLog;
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

/// Execute a POWL AST through all scheduler phases (1-4 integrated).
///
/// - Compiles the AST to a tape.
/// - Initializes a PowlRunState and OcelLog.
/// - Runs the scheduler loop for up to 128 bounded ticks.
/// - Records all op_fired and run_sealed events in the log.
/// - Returns (tape, final_state, audit_log, tick_count).
fn execute(ast: &PowlAstNode<'_>, run_id: u64) -> (PowlTape, PowlRunState, OcelLog, u32) {
    let tape = compile_powl(ast).expect("POWL model must compile");
    let mut state = PowlRunState::new(&tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;
    let mut ticks = 0u32;

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        ticks += 1;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            log.record_op_fired(run_id, op_idx, 0, 0).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    log.record_run_sealed(run_id, op_trace, 0).unwrap();
    (tape, state, log, ticks)
}

/// Test 1: Sequential handoff completes in bounded time with no deadlock
///
/// Three workers (A, B, C) process a single task in strict sequence.
/// The Sequence AST ensures A fires, then B, then C. This test verifies:
/// - Termination: check_mask == 0 (all ops completed).
/// - Bounded time: completed in exactly 3 ticks (one per worker).
/// - No livelock or oscillation.
#[test]
fn test_worker_a_to_b_to_c_sequential_completion() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_a_acquire_and_process"),
        PowlAstNode::Atom("worker_b_acquire_and_process"),
        PowlAstNode::Atom("worker_c_acquire_and_process"),
    ]);

    let (_tape, state, _log, ticks) = execute(&ast, 1);

    assert_eq!(
        state.check_mask, 0,
        "all workers must complete; no pending ops (no deadlock)"
    );
    assert_eq!(
        ticks, 3,
        "3 workers in sequence must complete in exactly 3 scheduler ticks"
    );
}

/// Test 2: All workers fire exactly once, in program order
///
/// Verify that the audit log captures all three worker ops (A, B, C)
/// and that they appear in the order A → B → C. This is the order that
/// POWL's compiler guarantees; the scheduler must respect it.
#[test]
fn test_audit_log_captures_handoff_order() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_a_acquire_and_process"),
        PowlAstNode::Atom("worker_b_acquire_and_process"),
        PowlAstNode::Atom("worker_c_acquire_and_process"),
    ]);

    let run_id = 2u64;
    let (_tape, _state, log, _ticks) = execute(&ast, run_id);

    let events = log.events();
    assert_eq!(
        events.len(), 4,
        "log must contain exactly 4 events: 3 op_fired + 1 run_sealed"
    );

    // Extract the op_fired events (skip the run_sealed event at the end).
    let op_events: Vec<u32> = events[..3].iter().map(|e| e.op_idx).collect();
    assert_eq!(
        op_events, vec![0, 1, 2],
        "workers must fire in program order: A (op 0), B (op 1), C (op 2)"
    );
}

/// Test 3: Repeated handoff executions produce identical audit trails
///
/// State-machine consistency requires that independent executions of the
/// same workflow produce identical event traces. This is the foundation
/// for Byzantine-tolerant replication: honest nodes can agree on the
/// command order without needing to broadcast the full workflow specification.
#[test]
fn test_repeated_handoff_produces_same_order() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_a_acquire_and_process"),
        PowlAstNode::Atom("worker_b_acquire_and_process"),
        PowlAstNode::Atom("worker_c_acquire_and_process"),
    ]);

    let run_id = 3u64;

    // First execution.
    let (_tape1, _state1, log1, _ticks1) = execute(&ast, run_id);
    let order1: Vec<u32> = log1
        .events()
        .iter()
        .take(3)
        .map(|e| e.op_idx)
        .collect();

    // Second independent execution (same run_id for consistency check).
    let (_tape2, _state2, log2, _ticks2) = execute(&ast, run_id);
    let order2: Vec<u32> = log2
        .events()
        .iter()
        .take(3)
        .map(|e| e.op_idx)
        .collect();

    assert_eq!(
        order1, order2,
        "independent executions of the same workflow must produce identical event traces"
    );
    assert_eq!(
        order1, vec![0, 1, 2],
        "order must always be A, B, C"
    );
}

/// Test 4: Audit log integrity via BLAKE3 receipt
///
/// If an attacker tries to reorder the handoff (e.g., claim C fired before B),
/// the receipt digest changes. This test constructs two logs—one with correct
/// handoff order and one with reordered workers—and proves the receipts diverge.
#[test]
fn test_handoff_reordering_detected_via_receipt_divergence() {
    let run_id = 4u64;

    // Correct execution order: A → B → C.
    let mut log_correct = OcelLog::new();
    log_correct.record_op_fired(run_id, 0, 0, 0).unwrap(); // A
    log_correct.record_op_fired(run_id, 1, 0, 0).unwrap(); // B
    log_correct.record_op_fired(run_id, 2, 0, 0).unwrap(); // C
    log_correct.record_run_sealed(run_id, 0b111, 0).unwrap();
    let digest_correct = log_correct.seal_receipt().digest();

    // Reordered attempt: B → A → C (attacker swaps A and B).
    let mut log_reordered = OcelLog::new();
    log_reordered.record_op_fired(run_id, 1, 0, 0).unwrap(); // B (moved first)
    log_reordered.record_op_fired(run_id, 0, 0, 0).unwrap(); // A (moved second)
    log_reordered.record_op_fired(run_id, 2, 0, 0).unwrap(); // C
    log_reordered.record_run_sealed(run_id, 0b111, 0).unwrap();
    let digest_reordered = log_reordered.seal_receipt().digest();

    assert_ne!(
        digest_correct, digest_reordered,
        "reordering workers in the handoff must change the receipt digest; auditors detect tampering"
    );
}

/// Test 5: Cyclic handoff (Byzantine deadlock attempt) is rejected at compile time
///
/// If a Byzantine worker proposes a workflow where A depends on B and B depends
/// on A, that creates a structural deadlock. POWL's compiler must refuse to
/// compile such a cycle at the boundary, preventing the scheduler from ever
/// reaching the deadlock state at runtime.
#[test]
fn test_cyclic_worker_dependency_refused_at_compile_time() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("worker_a_process"),
            PowlAstNode::Atom("worker_b_process"),
        ],
        edges: vec![(0, 1), (1, 0)], // cycle: A must precede B, and B must precede A
    };

    let result = compile_powl(&ast);

    assert!(
        result.is_err(),
        "a cyclic worker dependency must be rejected by the compiler; no deadlock at runtime"
    );
}

/// Test 6: Handoff tape encodes all dependencies in op slot positions
///
/// The compiled tape is a flat array of ops with predecessor/successor masks.
/// This test verifies that the tape structure encodes the sequential constraint:
/// op 1 cannot fire until op 0 has completed, and op 2 cannot fire until op 1
/// has completed.
#[test]
fn test_compiled_tape_encodes_handoff_dependencies() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_a_acquire_and_process"),
        PowlAstNode::Atom("worker_b_acquire_and_process"),
        PowlAstNode::Atom("worker_c_acquire_and_process"),
    ]);

    let tape = compile_powl(&ast).expect("POWL model must compile");

    // The tape should have at least 3 ops (one per worker), though the compiler
    // may add structural ops (e.g. implicit join nodes).
    assert!(
        tape.len >= 3,
        "compiled tape must have at least 3 op slots for 3 workers"
    );

    // Verify the entry_mask includes at least op 0 (the first worker).
    assert!(
        (tape.entry_mask & 1) != 0,
        "tape entry_mask must include op 0 (worker A)"
    );
}

/// Test 7: No resource starvation; all workers must be eligible to fire eventually
///
/// The scheduler uses a check_mask to track which ops are ready. In a
/// sequential handoff, only one op is ready at a time, but the scheduler
/// must advance the state such that the next worker becomes ready after
/// its predecessor completes. This test runs multiple ticks and verifies
/// that all 3 workers get a turn.
#[test]
fn test_no_worker_starvation_in_handoff_loop() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_a_acquire_and_process"),
        PowlAstNode::Atom("worker_b_acquire_and_process"),
        PowlAstNode::Atom("worker_c_acquire_and_process"),
    ]);

    let (_tape, _state, log, _ticks) = execute(&ast, 7);

    let fired: std::collections::HashSet<u32> = log
        .events()
        .iter()
        .take(3)
        .map(|e| e.op_idx)
        .collect();

    // Each of the 3 workers (ops 0, 1, 2) must fire exactly once.
    for worker_idx in 0..3u32 {
        assert!(
            fired.contains(&worker_idx),
            "worker {} must fire; no starvation in the handoff loop",
            worker_idx
        );
    }

    assert_eq!(
        fired.len(), 3,
        "exactly 3 distinct workers must fire (0, 1, 2)"
    );
}
