//! Resource conflict checking for POWL scheduler.
//!
//! Tests the `intervals_conflict` primitive and `ResourceRegistry` to verify that
//! operations with overlapping time intervals on shared resources are correctly
//! detected and blocked.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{intervals_conflict, OpTimeInterval, PowlRunState, ResourceRegistry};

#[test]
fn two_ops_same_resource_overlapping_intervals_detected() {
    // Test 1: Two ops, same resource, overlapping intervals → conflict detected.
    //
    // op_a: [0, 5) on resource "worker"
    // op_b: [3, 8) on resource "worker"
    // These intervals overlap at [3, 5), so they conflict.

    let interval_a = OpTimeInterval::new(0, 0, 5);
    let interval_b = OpTimeInterval::new(1, 3, 8);
    let tape = compile_powl(&PowlAstNode::Atom("test")).unwrap();

    assert!(
        intervals_conflict(&tape, 0, interval_a, 1, interval_b, "worker"),
        "overlapping intervals [0, 5) and [3, 8) must conflict"
    );
}

#[test]
fn two_ops_same_resource_nonoverlapping_intervals_allowed() {
    // Test 2: Two ops, same resource, non-overlapping intervals → no conflict.
    //
    // op_a: [0, 5) on resource "worker"
    // op_b: [5, 10) on resource "worker"
    // These intervals are disjoint (op_a ends exactly when op_b starts), so they don't conflict.

    let interval_a = OpTimeInterval::new(0, 0, 5);
    let interval_b = OpTimeInterval::new(1, 5, 10);
    let tape = compile_powl(&PowlAstNode::Atom("test")).unwrap();

    assert!(
        !intervals_conflict(&tape, 0, interval_a, 1, interval_b, "worker"),
        "non-overlapping intervals [0, 5) and [5, 10) must not conflict"
    );
}

#[test]
fn resource_registry_books_and_detects_conflicts() {
    // Integration test: ResourceRegistry tracks allocations and detects conflicts.

    let mut registry = ResourceRegistry::new();

    // Book op_a on worker: [0, 5)
    let interval_a = OpTimeInterval::new(0, 0, 5);
    registry.book_interval("worker".to_string(), interval_a);

    // Check op_b on worker: [3, 8) — should conflict with op_a
    let interval_b = OpTimeInterval::new(1, 3, 8);
    assert_eq!(
        registry.check_conflict("worker", interval_b),
        Some(0),
        "op_b should conflict with op_a (already booked)"
    );

    // Check op_c on worker: [5, 10) — should not conflict (disjoint from op_a)
    let interval_c = OpTimeInterval::new(2, 5, 10);
    assert_eq!(
        registry.check_conflict("worker", interval_c),
        None,
        "op_c should not conflict with op_a (intervals are disjoint)"
    );
}

#[test]
fn resource_registry_multiple_resources_isolated() {
    // Test that conflicts on one resource don't affect another resource.

    let mut registry = ResourceRegistry::new();

    // Book op_a on "worker": [0, 5)
    let interval_a = OpTimeInterval::new(0, 0, 5);
    registry.book_interval("worker".to_string(), interval_a);

    // Book op_a on "truck": [2, 7)
    let interval_a_truck = OpTimeInterval::new(0, 2, 7);
    registry.book_interval("truck".to_string(), interval_a_truck);

    // Check op_b on "worker": [3, 8) — should conflict
    let interval_b = OpTimeInterval::new(1, 3, 8);
    assert_eq!(
        registry.check_conflict("worker", interval_b),
        Some(0),
        "op_b should conflict with op_a on worker"
    );

    // Check op_b on "truck": [6, 9) — should conflict
    let interval_b_truck = OpTimeInterval::new(1, 6, 9);
    assert_eq!(
        registry.check_conflict("truck", interval_b_truck),
        Some(0),
        "op_b should conflict with op_a on truck"
    );

    // Check op_c on "truck": [7, 12) — should not conflict (disjoint from op_a)
    let interval_c_truck = OpTimeInterval::new(2, 7, 12);
    assert_eq!(
        registry.check_conflict("truck", interval_c_truck),
        None,
        "op_c should not conflict with op_a on truck (intervals are disjoint)"
    );
}

#[test]
fn interval_overlap_boundary_cases() {
    // Edge cases: intervals touching at boundaries.

    let tape = compile_powl(&PowlAstNode::Atom("test")).unwrap();

    // [0, 5) and [5, 10) — meet but don't overlap (half-open intervals)
    let a = OpTimeInterval::new(0, 0, 5);
    let b = OpTimeInterval::new(1, 5, 10);
    assert!(
        !intervals_conflict(&tape, 0, a, 1, b, "res"),
        "touching intervals must not conflict"
    );

    // [0, 5) and [4, 5) — b is a strict subset of a
    let a = OpTimeInterval::new(0, 0, 5);
    let b = OpTimeInterval::new(1, 4, 5);
    assert!(
        intervals_conflict(&tape, 0, a, 1, b, "res"),
        "subset intervals must conflict"
    );

    // [3, 8) and [0, 3) — a.start > b.end, disjoint
    let a = OpTimeInterval::new(0, 3, 8);
    let b = OpTimeInterval::new(1, 0, 3);
    assert!(
        !intervals_conflict(&tape, 0, a, 1, b, "res"),
        "separated intervals must not conflict"
    );
}

#[test]
fn parallel_ops_via_tape_compile_with_manual_conflict_check() {
    // Compile two parallel operations and manually verify conflict detection
    // against hypothetical resource intervals.

    let ast = PowlAstNode::PartialOrder {
        children: vec![PowlAstNode::Atom("op_a"), PowlAstNode::Atom("op_b")],
        edges: vec![],
    };
    let tape = compile_powl(&ast).unwrap();
    assert!(
        tape.len >= 2,
        "parallel ops should compile to at least 2 slots"
    );

    // Hypothetical: op_a runs [0, 5) and op_b runs [3, 8) on shared resource "worker"
    // They're ready to fire in parallel (no data dependencies), but conflict on the resource.
    let interval_a = OpTimeInterval::new(0, 0, 5);
    let interval_b = OpTimeInterval::new(1, 3, 8);

    assert!(
        intervals_conflict(&tape, 0, interval_a, 1, interval_b, "worker"),
        "parallel ops with overlapping resource intervals must conflict"
    );
}

#[test]
fn scheduler_state_tracks_blocked_reasons_for_resource_conflicts() {
    // Verify that PowlRunState can store blocked reasons corresponding to resource conflicts.
    // (This is a setup test for future scheduler_tick_with_resource integration.)

    let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("op_a"), PowlAstNode::Atom("op_b")]);
    let tape = compile_powl(&ast).unwrap();
    let mut state = PowlRunState::new(&tape);

    // Manually simulate a resource conflict: mark op_b as blocked due to resource conflict.
    let reason = "resource worker conflict with op 0".to_string();
    state.blocked_mask |= 1u64 << 1; // op_b (slot 1)
    state.blocked_reasons.push((1, reason.clone()));

    assert_eq!(
        state.blocked_mask & (1u64 << 1),
        1u64 << 1,
        "blocked_mask must mark op_b"
    );
    assert!(
        state
            .blocked_reasons
            .iter()
            .any(|(idx, r)| *idx == 1 && *r == reason),
        "blocked_reasons must record the conflict"
    );
}

#[test]
fn op_time_interval_debug_output() {
    // Verify OpTimeInterval displays useful debug info.
    let interval = OpTimeInterval::new(5, 10, 20);
    let debug_str = format!("{:?}", interval);
    assert!(
        debug_str.contains("op_idx: 5"),
        "debug output should include op_idx"
    );
    assert!(
        debug_str.contains("start: 10"),
        "debug output should include start"
    );
    assert!(
        debug_str.contains("end: 20"),
        "debug output should include end"
    );
}
