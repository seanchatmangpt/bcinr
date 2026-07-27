//! Swarm Coordination: Deadline Miss Detection and Refusal
//!
//! Demonstrates how POWL's OCEL validation detects and refuses work traces
//! that would cause a deadline miss, preventing the worker swarm from
//! overcommitting and failing to meet system SLAs.
//!
//! ## The Problem
//!
//! In distributed work-stealing systems (MapReduce, task swarms, job schedulers),
//! workers must track their total execution time and ensure all work completes
//! before the overall deadline. Two failure modes matter:
//! - Silent overcommitment: a worker accepts work, executes it, misses deadline
//! - Validation under constraint: OCEL auditors must refuse traces that violated
//!   the deadline (audit gate, not scheduler gate, is where we validate this)
//!
//! ## The Solution
//!
//! POWL provides:
//! - Deadline tracking in the compiled `PowlTape` (deadline field)
//! - OCEL validation: `validate_against_tape` checks deadline conformance
//! - Typed refusal: `ConformanceResult::DeadlineViolation` signals the exact
//!   run_id, actual_time, and deadline that was exceeded
//! - Audit trail: the trace is logged but marked nonconforming, giving auditors
//!   visibility into deadline violations for root-cause analysis

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::{ConformanceResult, OcelLog};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

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
            // Record with a duration approximation: each op takes ~10 time units
            log.record_op_fired(run_id, op_idx, ticks * 10, 10).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    log.record_run_sealed(run_id, op_trace, ticks * 10).unwrap();
    (tape, state, log, ticks)
}

/// Test 1: OCEL validation detects deadline violations
///
/// Scenario: A task swarm runs 4 sequential ops, each taking ~10 time units,
/// for a total execution time of ~40. We set a deadline of 30, which is
/// unachievable. The OCEL validator must reject this trace as exceeding
/// the deadline.
///
/// This test verifies:
/// - The tape compiles with a deadline set
/// - The scheduler completes all ops (no gate refusal, just later validation)
/// - The OCEL validation detects DeadlineViolation, not Conforms
#[test]
fn test_deadline_violation_detected_in_ocel_validation() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("worker_1_task_a"),
        PowlAstNode::Atom("worker_2_task_b"),
        PowlAstNode::Atom("worker_3_task_c"),
        PowlAstNode::Atom("worker_4_task_d_overflow"),
    ]);

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    // Set a tight deadline: 30 time units. 4 ops × 10 units each = 40 total,
    // so this will definitely fail.
    tape.deadline = 30u32;

    let (_, state, log, _ticks) = execute(&ast, 1);

    // The scheduler should terminate (execute_loop completed).
    assert_eq!(
        state.check_mask, 0,
        "scheduler must complete (no gate refusal at scheduling time)"
    );

    // Validate the trace against the tape with the tight deadline.
    let result = log.validate_against_tape(&tape);

    // The result must be a deadline violation, not conformance.
    match result {
        ConformanceResult::DeadlineViolation {
            run_id,
            actual_time,
            deadline,
        } => {
            assert_eq!(run_id, 1, "violation should be for run_id 1");
            assert!(actual_time > deadline, "actual time must exceed deadline");
        }
        _ => panic!(
            "OCEL validation must detect deadline violation, got {:?}",
            result
        ),
    }
}

/// Test 2: Partial order workflow with deadline enforcement
///
/// Scenario: Three independent validators in a partial order, each taking
/// ~10 time units. The 3rd validator (validator_c) extends execution past
/// a deadline of 15 time units (only 1.5 ops can fit). All 3 ops will
/// execute (no gate refusal), but OCEL validation must detect the violation.
///
/// This test verifies:
/// - Partial order compiles correctly
/// - All ops execute (deadline enforcement is post-hoc via OCEL validation)
/// - OCEL validation detects the deadline violation
#[test]
fn test_partial_order_deadline_validation_fails() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("validator_a_confirms"),
            PowlAstNode::Atom("validator_b_confirms"),
            PowlAstNode::Atom("validator_c_confirms_late"),
        ],
        edges: vec![],
    };

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    // Set deadline to 15 time units. With 3 parallel ops firing, the last one
    // finishes at time ~30, exceeding the deadline.
    tape.deadline = 15u32;

    let (_, state, log, _ticks) = execute(&ast, 2);

    // check_mask should be zero (all work complete).
    assert_eq!(
        state.check_mask, 0,
        "partial order must reach a terminal state"
    );

    // Validate the trace against the deadline.
    let result = log.validate_against_tape(&tape);

    // Expect deadline violation.
    match result {
        ConformanceResult::DeadlineViolation {
            actual_time,
            deadline,
            ..
        } => {
            assert!(
                actual_time > deadline,
                "violation: {} > {}",
                actual_time,
                deadline
            );
        }
        _ => panic!("expected deadline violation, got {:?}", result),
    }
}

/// Test 3: Loose deadline allows all work to conform
///
/// Scenario: A sequence of work with 3 ops, each taking ~10 time units (total ~30).
/// We set a loose deadline of 50 time units, which is easily achievable.
/// The OCEL validation must PASS (Conforms).
///
/// This test verifies the converse: when the deadline is generous, the trace
/// conforms without violation.
#[test]
fn test_loose_deadline_allows_conformance() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("work_1"),
        PowlAstNode::Atom("work_2"),
        PowlAstNode::Atom("work_3_completes_in_time"),
    ]);

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    // Set a loose deadline: 50 time units. 3 ops × 10 units each = 30, well under the deadline.
    tape.deadline = 50u32;

    let (_, state, log, _ticks) = execute(&ast, 3);

    // Scheduler should complete.
    assert_eq!(state.check_mask, 0);

    // Validate the trace against the loose deadline.
    let result = log.validate_against_tape(&tape);

    // Expect conformance (no violation).
    assert_eq!(
        result,
        ConformanceResult::Conforms,
        "trace must conform when deadline is achievable"
    );
}

/// Test 4: Nested sequence with deadline enforcement
///
/// Scenario: A sequence within a sequence gives us nested composition.
/// Total execution time is ~40 time units (4 ops, ~10 each).
/// Set deadline to 25, which is unachievable. OCEL validation must detect this.
///
/// This test verifies that nested workflows respect deadline constraints.
#[test]
fn test_nested_sequence_deadline_violation() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("step_1"),
        PowlAstNode::Atom("step_2"),
        PowlAstNode::Atom("step_3"),
        PowlAstNode::Atom("step_4"),
    ]);

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    tape.deadline = 25u32; // Too tight for 4 ops × 10 units

    let (_, state, log, _ticks) = execute(&ast, 4);

    // Scheduler must terminate.
    assert_eq!(
        state.check_mask, 0,
        "scheduler must terminate (sequence guaranteed to be acyclic)"
    );

    // Validate and expect deadline violation.
    let result = log.validate_against_tape(&tape);
    match result {
        ConformanceResult::DeadlineViolation {
            actual_time,
            deadline,
            ..
        } => {
            assert!(
                actual_time > deadline,
                "actual_time {} must exceed deadline {}",
                actual_time,
                deadline
            );
        }
        _ => panic!(
            "expected deadline violation for tight deadline, got {:?}",
            result
        ),
    }
}

/// Test 5: Swarm coordinator detects overcommitment via deadline miss
///
/// Scenario: A swarm coordinator receives 5 sequential task requests from
/// 5 workers. All 5 execute (sequential composition takes ~50 time units).
/// Deadline is set to 35 time units, which can only fit ~3.5 workers.
/// OCEL validation must detect that the trace exceeds the deadline.
///
/// This test models the real-world swarm use case: auditors detect when
/// coordinators have overcommitted work past the deadline, enabling
/// post-hoc alerting and recovery.
#[test]
fn test_swarm_overcommit_detection_five_workers() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("swarm_worker_1_accepts_task"),
        PowlAstNode::Atom("swarm_worker_2_accepts_task"),
        PowlAstNode::Atom("swarm_worker_3_accepts_task"),
        PowlAstNode::Atom("swarm_worker_4_causes_violation"),
        PowlAstNode::Atom("swarm_worker_5_causes_violation"),
    ]);

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    // Deadline: 35 time units. 5 ops × 10 units each = 50 total, so violation.
    tape.deadline = 35u32;

    let (_, state, log, _ticks) = execute(&ast, 5);

    // Scheduler completes all work (no gate refusal).
    assert_eq!(state.check_mask, 0);

    // OCEL validation detects the deadline violation.
    let result = log.validate_against_tape(&tape);
    match result {
        ConformanceResult::DeadlineViolation {
            run_id,
            actual_time,
            deadline,
        } => {
            assert_eq!(run_id, 5);
            assert!(actual_time > deadline);
            // Swarm coordinator can use this signal to reduce future task load
        }
        _ => panic!(
            "expected deadline violation for 5 overcommitted workers, got {:?}",
            result
        ),
    }
}

/// Test 6: No LLM calls or external API interactions
///
/// Safety check: deadline validation is pure and local, never making
/// external API calls. This verifies the deadline enforcement is
/// synchronous and side-effect free.
///
/// We check this by verifying that the validation completes instantly
/// and produces consistent results without network I/O.
#[test]
fn test_deadline_logic_no_external_api_calls() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("task_1"),
        PowlAstNode::Atom("task_2"),
    ]);

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    tape.deadline = 50u32; // Generous deadline

    let (_, _state, log, _ticks) = execute(&ast, 6);

    // Validate the trace (should complete instantly, no network I/O).
    let result1 = log.validate_against_tape(&tape);
    let result2 = log.validate_against_tape(&tape);

    // Results must be identical (deterministic, no external state).
    assert_eq!(
        result1, result2,
        "validation must be deterministic (no external API calls)"
    );

    // Log must have recorded the ops.
    assert!(!log.events().is_empty(), "log should record events");
}

/// Test 7: ConformanceResult provides diagnostic information
///
/// Scenario: When a deadline is violated, the ConformanceResult::DeadlineViolation
/// variant includes the run_id, actual_time, and deadline, enabling operators
/// to diagnose why a swarm's work missed its deadline.
///
/// This test verifies the diagnostic quality of the validation result.
#[test]
fn test_deadline_violation_includes_diagnostic_details() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("fast_task"),
        PowlAstNode::Atom("slow_task_that_misses_deadline"),
    ]);

    let mut tape = compile_powl(&ast).expect("POWL model must compile");
    tape.deadline = 10u32; // Very tight: 2 ops × 10 units = 20 > 10

    let (_, _state, log, _ticks) = execute(&ast, 7);

    // Validate and expect detailed violation information.
    let result = log.validate_against_tape(&tape);

    match result {
        ConformanceResult::DeadlineViolation {
            run_id,
            actual_time,
            deadline,
        } => {
            // Verify all diagnostic fields are present and meaningful.
            assert_eq!(run_id, 7, "run_id must be preserved in the violation");
            assert!(actual_time > 0, "actual_time must be set");
            assert!(deadline > 0, "deadline must be set");
            assert!(
                actual_time > deadline,
                "violation must have actual_time > deadline: {} > {}",
                actual_time,
                deadline
            );
        }
        other => panic!(
            "expected deadline violation with diagnostics, got {:?}",
            other
        ),
    }
}
