//! Temporal conformance tests for OCEL validation.
//!
//! This module tests the extended OCEL conformance checking that includes
//! temporal constraints:
//! - Duration limits per operation
//! - Lease expiry deadlines for resources
//! - Overall workflow deadline

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::{validate_against_tape, ConformanceResult, OcelLog};

/// Test 1: Duration check passes when operation stays within limit.
#[test]
fn duration_check_passes_under_limit() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    // Set max_duration for op_idx 0 to 10 time units
    tape.max_durations[0] = 10u32;
    tape.max_durations[1] = 20u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // op 0 fires with duration 5 (within limit of 10)
    log.record_op_fired(run_id, 0, 0, 5).unwrap();
    // op 1 fires with duration 15 (within limit of 20)
    log.record_op_fired(run_id, 1, 5, 15).unwrap();
    // seal with both ops
    log.record_run_sealed(run_id, 0b11, 20).unwrap();

    let result = validate_against_tape(&log, &tape);
    assert_eq!(result, ConformanceResult::Conforms);
}

/// Test 2: Duration check fails when operation exceeds limit.
#[test]
fn duration_check_fails_over_limit() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    // Set max_duration for op_idx 0 to 10 time units
    tape.max_durations[0] = 10u32;
    tape.max_durations[1] = 20u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // op 0 fires with duration 15 (EXCEEDS limit of 10)
    log.record_op_fired(run_id, 0, 0, 15).unwrap();
    // op 1 fires
    log.record_op_fired(run_id, 1, 15, 10).unwrap();
    // seal with both ops
    log.record_run_sealed(run_id, 0b11, 25).unwrap();

    let result = validate_against_tape(&log, &tape);

    // Should detect DurationViolation
    assert_eq!(
        result,
        ConformanceResult::DurationViolation {
            run_id: 42,
            op_idx: 0,
            actual_duration: 15,
            max_allowed: 10,
        }
    );
}

/// Test 3: Deadline check passes when run completes within deadline.
#[test]
fn deadline_check_passes_under_limit() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    // Set overall workflow deadline to 100 time units
    tape.deadline = 100u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // op 0 fires at time 0 with duration 10
    log.record_op_fired(run_id, 0, 0, 10).unwrap();
    // op 1 fires at time 10 with duration 20
    log.record_op_fired(run_id, 1, 10, 20).unwrap();
    // seal at time 30 (well within deadline of 100)
    log.record_run_sealed(run_id, 0b11, 30).unwrap();

    let result = validate_against_tape(&log, &tape);
    assert_eq!(result, ConformanceResult::Conforms);
}

/// Test 4: Deadline check fails when run exceeds deadline.
#[test]
fn deadline_check_fails_over_limit() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    // Set overall workflow deadline to 50 time units
    tape.deadline = 50u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // op 0 fires at time 0 with duration 30
    log.record_op_fired(run_id, 0, 0, 30).unwrap();
    // op 1 fires at time 30 with duration 40
    log.record_op_fired(run_id, 1, 30, 40).unwrap();
    // seal at time 70 (EXCEEDS deadline of 50)
    log.record_run_sealed(run_id, 0b11, 70).unwrap();

    let result = validate_against_tape(&log, &tape);

    // Should detect DeadlineViolation
    assert_eq!(
        result,
        ConformanceResult::DeadlineViolation {
            run_id: 42,
            actual_time: 70,
            deadline: 50,
        }
    );
}

/// Test 5: Multiple operations with varying durations; first one violates.
#[test]
fn duration_violation_on_second_op_in_sequence() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
        PowlAstNode::Atom("c"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    // Set different limits for each op
    tape.max_durations[0] = 50u32;
    tape.max_durations[1] = 10u32; // op 1 has tight limit
    tape.max_durations[2] = 50u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // op 0 passes
    log.record_op_fired(run_id, 0, 0, 30).unwrap();
    // op 1 EXCEEDS its 10-unit limit with 25 units
    log.record_op_fired(run_id, 1, 30, 25).unwrap();
    log.record_op_fired(run_id, 2, 55, 40).unwrap();
    log.record_run_sealed(run_id, 0b111, 95).unwrap();

    let result = validate_against_tape(&log, &tape);

    assert_eq!(
        result,
        ConformanceResult::DurationViolation {
            run_id: 42,
            op_idx: 1,
            actual_duration: 25,
            max_allowed: 10,
        }
    );
}

/// Test 6: No constraints set (zero values) should not trigger violations.
#[test]
fn zero_constraints_mean_no_limits() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let tape = compile_powl(&ast).unwrap();

    // Leave all constraints at zero (default, no limits)
    // max_durations are all 0 by default
    // deadline is 0 by default

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // Very long operations
    log.record_op_fired(run_id, 0, 0, 1000).unwrap();
    log.record_op_fired(run_id, 1, 1000, 2000).unwrap();
    // Very late seal (way past any reasonable deadline)
    log.record_run_sealed(run_id, 0b11, 3000).unwrap();

    let result = validate_against_tape(&log, &tape);
    // Should pass because no constraints are set
    assert_eq!(result, ConformanceResult::Conforms);
}

/// Test 7: Deadline of 0 means no deadline constraint.
#[test]
fn deadline_zero_means_no_constraint() {
    let ast = PowlAstNode::Atom("a");
    let tape = compile_powl(&ast).unwrap();
    // deadline defaults to 0, meaning no constraint

    let mut log = OcelLog::new();
    let run_id = 42u64;

    log.record_op_fired(run_id, 0, 0, 1).unwrap();
    // Seal extremely late (well beyond any reasonable time)
    log.record_run_sealed(run_id, 0b1, u32::MAX - 1).unwrap();

    let result = validate_against_tape(&log, &tape);
    // Should pass because deadline is 0 (no constraint)
    assert_eq!(result, ConformanceResult::Conforms);
}

/// Test 8: Temporal constraints coexist with structural checks.
#[test]
fn temporal_and_structural_checks_together() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    tape.deadline = 100u32;
    tape.max_durations[0] = 20u32;
    tape.max_durations[1] = 30u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // Fire op 1 before op 0 (structural violation: predecessor check)
    log.record_op_fired(run_id, 1, 0, 10).unwrap();

    let result = validate_against_tape(&log, &tape);

    // Should detect structural violation (missing predecessor), not temporal
    match result {
        ConformanceResult::Violation {
            run_id: r,
            op_idx: o,
            missing_pred_mask: m,
        } => {
            assert_eq!(r, 42);
            assert_eq!(o, 1);
            assert_eq!(m, 0b1); // op 0 is missing
        }
        _ => panic!("Expected Violation, got {:?}", result),
    }
}

/// Test 9: Both duration and deadline violations possible (duration checked first).
#[test]
fn duration_violation_detected_before_deadline() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("a"),
        PowlAstNode::Atom("b"),
    ]);
    let mut tape = compile_powl(&ast).unwrap();

    tape.deadline = 50u32;
    tape.max_durations[0] = 10u32;
    tape.max_durations[1] = 10u32;

    let mut log = OcelLog::new();
    let run_id = 42u64;

    // op 0 violates duration constraint
    log.record_op_fired(run_id, 0, 0, 20).unwrap(); // 20 > 10
    log.record_op_fired(run_id, 1, 20, 50).unwrap();
    // This would also violate deadline, but duration check comes first
    log.record_run_sealed(run_id, 0b11, 70).unwrap();

    let result = validate_against_tape(&log, &tape);

    // Should detect duration violation (checked during op_fired processing)
    assert_eq!(
        result,
        ConformanceResult::DurationViolation {
            run_id: 42,
            op_idx: 0,
            actual_duration: 20,
            max_allowed: 10,
        }
    );
}
