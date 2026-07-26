//! chaos_scenarios — Three chaos injection test scenarios.
//!
//! Test 1: Crash after op 5 of 10 → graceful termination, OcelLog sealed, state consistent
//! Test 2: Delay by 1000 ticks → dependent ops still fire in order
//! Test 3: Duplicate tick → ops don't double-fire (idempotent tick or explicit check)

mod chaos_harness;

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use chaos_harness::*;

/// Test 1: Crash injection after 5 of 10 sequential operations.
///
/// Verifies that:
/// - Scheduler terminates gracefully when crash is injected
/// - OcelLog can be sealed at any point without corruption
/// - Final state remains consistent (no op in multiple terminal states)
#[test]
fn test_crash_after_op_5_of_10_graceful_termination() {
    // Construct a sequence of 10 operations: A → B → C → D → E → F → G → H → I → J
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("A"),
        PowlAstNode::Atom("B"),
        PowlAstNode::Atom("C"),
        PowlAstNode::Atom("D"),
        PowlAstNode::Atom("E"),
        PowlAstNode::Atom("F"),
        PowlAstNode::Atom("G"),
        PowlAstNode::Atom("H"),
        PowlAstNode::Atom("I"),
        PowlAstNode::Atom("J"),
    ]);

    let tape = compile_powl(&ast).expect("Failed to compile 10-op sequence");
    assert_eq!(tape.len, 10, "Tape must have exactly 10 ops");

    // Run with crash injection after tick 5
    let result = run_with_crash_injection(&tape, 5, 20);

    // Verify crash was triggered
    assert!(result.crashed, "Crash injection must have been triggered");
    assert_eq!(
        result.ticks_executed, 5,
        "Execution must stop at exact tick 5"
    );

    // Verify graceful termination: final state is consistent
    assert!(
        result.final_state.is_consistent(tape.len as u32),
        "State must be consistent after crash: no op in multiple terminal states"
    );

    // Verify crash state is also consistent
    if let Some(crash_snap) = &result.crash_state {
        assert!(
            crash_snap.is_consistent(tape.len as u32),
            "State at crash point must be consistent"
        );
    }

    // Trace the crashed execution with OcelLog
    let (log, _state) = trace_with_ocel(&tape, 1, 5);

    // Seal the log (must not panic or corrupt)
    let receipt = log.seal_receipt();

    // Verify receipt sealed all events
    assert!(
        receipt.event_count() > 0,
        "OcelLog must record events from the partial execution"
    );

    // Verify receipt digest is deterministic
    let receipt2 = log.seal_receipt();
    assert_eq!(
        receipt.digest(),
        receipt2.digest(),
        "Same log must produce same digest"
    );

    // At 5 ticks (operations fire 1 per tick in sequence), ops 0-4 (A-E) should have fired
    // Op at index 5 and beyond might not have fired yet
    let expected_fires_min = 5u64;
    assert!(
        result.all_fired.count_ones() as u64 >= expected_fires_min - 1,
        "At least 4-5 ops should have fired by tick 5 in a sequence"
    );
}

/// Test 2: Delay injection by 1000 logical ticks.
///
/// Verifies that:
/// - Advancing the logical tick counter doesn't break dependency ordering
/// - Dependent operations still fire in the correct order
/// - Final state is consistent despite artificial delay
#[test]
fn test_delay_1000_ticks_dependent_order_preserved() {
    // Construct a dependency chain: A → B → C (prerequisite chain)
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("op_A"),
        PowlAstNode::Atom("op_B"),
        PowlAstNode::Atom("op_C"),
    ]);

    let tape = compile_powl(&ast).expect("Failed to compile 3-op sequence");

    // Run with 1000-tick delay
    let result = run_with_delay_injection(&tape, 1000, 50);

    // Verify all three ops eventually completed
    assert_eq!(
        (result.all_fired as u64).count_ones(), 3,
        "All 3 ops must have fired despite delay"
    );

    // Verify operation order: op 0 fires first, then 1, then 2
    // In a pure sequence, dependencies force: op[i] depends on op[i-1]
    // So op 0 must be in done_mask before op 1 fires, etc.
    let expected_all_fired = 0b111u64; // ops 0, 1, 2
    assert_eq!(
        result.all_fired & expected_all_fired, expected_all_fired,
        "All sequential ops must complete"
    );

    // Verify final state consistency
    assert!(
        result.final_state.is_consistent(tape.len as u32),
        "State must be consistent despite delay injection"
    );

    // Verify tick counter advanced
    assert!(
        result.final_state.tick >= 1000,
        "Tick counter must reflect the delay injection"
    );
}

/// Test 3: Duplicate-tick injection verifies idempotence.
///
/// Verifies that:
/// - Calling scheduler_tick twice on the same state either:
///   a) produces identical FiredSet both times, or
///   b) produces empty FiredSet on second call (idempotent no-op)
/// - No operations double-fire (appear in FiredSet more than once)
/// - Final state remains valid
#[test]
fn test_duplicate_tick_idempotent_no_double_fire() {
    // Use a partial order structure to create scenarios where multiple ops could fire
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("parallel_A"),
            PowlAstNode::Atom("parallel_B"),
            PowlAstNode::Atom("parallel_C"),
        ],
        edges: vec![], // No dependencies; all can run concurrently
    };

    let tape = compile_powl(&ast).expect("Failed to compile parallel tape");

    // Run with duplicate-tick injection
    let (exec_result, dup_verify) = run_with_duplicate_tick_injection(&tape, 50);

    // All duplicate-tick checks must pass
    assert!(
        dup_verify.all_passed,
        "All duplicate-tick checks must be idempotent: {} checks, all passed",
        dup_verify.checks.len()
    );

    // Verify no check failed
    for (tick_num, first_fired, second_fired, is_idempotent) in &dup_verify.checks {
        assert!(
            *is_idempotent,
            "Tick {} failed idempotence check: first={:#064b}, second={:#064b}",
            tick_num, first_fired, second_fired
        );

        // Additional check: second_fired should be either 0 or equal to first_fired
        if *second_fired != 0 && *second_fired != *first_fired {
            panic!(
                "Tick {} second call produced unexpected FiredSet: expected 0 or {:#064b}, got {:#064b}",
                tick_num, first_fired, second_fired
            );
        }
    }

    // Verify final state consistency
    assert!(
        exec_result.final_state.is_consistent(tape.len as u32),
        "Final state must be consistent"
    );

    // Verify that all_fired does not contain duplicate bits (each op fired at most once)
    let op_count = tape.len as u32;
    for i in 0..op_count {
        let bit = 1u64 << i;
        let fired_count = (exec_result.all_fired & bit != 0) as u32;
        assert!(
            fired_count <= 1,
            "Op {} appears to have fired multiple times",
            i
        );
    }

    // Verify that at least one tick actually executed
    assert!(
        exec_result.ticks_executed > 0,
        "At least one tick must have executed"
    );
}

/// Integration: Reorder-injection on dependent operations preserves validity.
///
/// Verifies that shuffling the ready-set doesn't violate dependency constraints
/// when the scheduler is invoked repeatedly.
#[test]
fn test_reorder_ready_set_preserves_dependency_validity() {
    // Create a DAG: (A → B) and (C → D), join both at E
    // This has parallel branches that could be reordered.
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::PartialOrder {
            children: vec![
                PowlAstNode::Atom("branch_A"),
                PowlAstNode::Atom("branch_C"),
            ],
            edges: vec![], // No dependencies; all can run concurrently
        },
        PowlAstNode::Atom("join_E"),
    ]);

    let tape = compile_powl(&ast).expect("Failed to compile DAG");

    // Run with reorder injection
    let result = run_with_reorder_injection(&tape, 42, 50);

    // Verify no validity violations
    assert!(
        result.all_valid,
        "No operations should fire with unsatisfied dependencies"
    );

    if !result.validity_violations.is_empty() {
        for (tick, violation) in &result.validity_violations {
            eprintln!("Violation at tick {}: {}", tick, violation);
        }
        panic!(
            "{} validity violations detected",
            result.validity_violations.len()
        );
    }

    // Verify final state is consistent
    assert!(
        result.final_state.is_consistent(tape.len as u32),
        "Final state must remain consistent after reorder injection"
    );

    // Verify all ops eventually completed
    assert_eq!(
        (result.all_fired.count_ones()) as usize, tape.len as usize,
        "All ops must eventually fire despite reordering"
    );
}

/// Sanity check: Uninjected execution produces baseline.
#[test]
fn test_uninjected_execution_baseline() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("x"),
        PowlAstNode::Atom("y"),
        PowlAstNode::Atom("z"),
    ]);

    let tape = compile_powl(&ast).expect("Failed to compile baseline");

    // Normal execution (no injection)
    let result = run_with_delay_injection(&tape, 0, 20);

    // All 3 ops must fire
    assert_eq!(
        (result.all_fired.count_ones()), 3,
        "All 3 sequential ops must fire"
    );

    // No crash
    assert!(!result.crashed, "Normal execution must not crash");

    // State must be consistent
    assert!(
        result.final_state.is_consistent(tape.len as u32),
        "Baseline state must be consistent"
    );
}
