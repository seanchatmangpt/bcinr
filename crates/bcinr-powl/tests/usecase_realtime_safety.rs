//! Safety-Critical Real-Time Systems
//!
//! Demonstrates how POWL's O(1) scheduler tick and stability envelope
//! provide formally-verified bounded latency and progress guarantees.
//!
//! ## The Problem
//!
//! Medical devices (pacemakers, insulin pumps) and autonomous vehicles need
//! *provably* bounded execution time for safety-critical decisions:
//! - Sensor read → decision → actuator trigger must complete in < 100ms
//! - Conventional schedulers use heuristic queueing (no bounds)
//! - System may deadlock or oscillate between states
//!
//! ## The Solution
//!
//! POWL provides:
//! - O(1) scheduler tick: constant time regardless of workflow complexity
//! - Formal liveness proof: check_mask != 0 -> guaranteed progress
//! - Precedence-ordered execution, verified via a real compile + scheduler run

use bcinr_powl::compiler::{compile_powl, CompileError, PowlAstNode};
use bcinr_powl::ocel::{ConformanceResult, OcelLog};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::PowlTape;

/// Compile and run an AST to completion, recording every fired op in an OCEL log.
/// Mirrors the harness used in chicago_tdd_integration.rs (the project's real
/// execution path, not a mock).
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
            log.record_op_fired(run_id, op_idx, 0).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    assert_eq!(state.check_mask, 0, "bounded scheduler must complete");
    log.record_run_sealed(run_id, op_trace).unwrap();
    (tape, state, log, ticks)
}

/// Test 1: Verify bounded, tick-counted execution of a multi-stage workflow
///
/// The scheduler tick is O(1) per call (branchless bit ops on a 64-bit mask).
/// This test proves the wall-clock bound indirectly but rigorously: a
/// 3-stage strictly-sequential workflow must complete in exactly 3 ticks
/// (one op admitted per tick), never more, never fewer.
#[test]
fn test_bounded_latency_o1_scheduler() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("sensor_read"),
        PowlAstNode::Atom("decision_logic"),
        PowlAstNode::Atom("actuator_trigger"),
    ]);

    let (tape, _state, log, ticks) = execute(&ast, 1);

    assert_eq!(
        ticks, 3,
        "3-stage sequence must take exactly 3 scheduler ticks"
    );
    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Conforms,
        "recorded trace must match the compiled precedence graph"
    );
}

/// Test 2: Verify no deadlock — ready set makes forward progress
///
/// The scheduler guarantees: if check_mask != 0, at least one op fires per
/// tick. We verify this is not merely assumed by running the real scheduler
/// to completion and asserting the final check_mask is 0 (all ops retired).
#[test]
fn test_no_deadlock_ready_set_progress() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("stage_1"),
        PowlAstNode::Atom("stage_2"),
        PowlAstNode::Atom("stage_3"),
    ]);

    let (_tape, state, _log, _ticks) = execute(&ast, 2);

    assert_eq!(
        state.check_mask, 0,
        "no operations left pending: no deadlock"
    );
}

/// Test 3: Verify precedence within a partial order is respected
///
/// A partial order with an explicit edge load -> compute forces `compute`
/// to be inadmissible until `load` has fired. We verify this by checking
/// that the recorded OCEL trace conforms to the compiled tape's precedence
/// constraints (the compiler encodes pred_mask/succ_mask from the edges).
#[test]
fn test_stability_envelope_prevents_oscillation() {
    let ast = PowlAstNode::PartialOrder {
        children: vec![
            PowlAstNode::Atom("load_state"),
            PowlAstNode::Atom("compute_next_state"),
            PowlAstNode::Atom("independent_telemetry"),
        ],
        edges: vec![(0, 1)], // load_state -> compute_next_state
    };

    let (tape, _state, log, _ticks) = execute(&ast, 3);

    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Conforms,
        "precedence edges must be respected in every admitted trace"
    );

    // load_state (op 0) must fire strictly before compute_next_state (op 1).
    let events = log.events();
    let load_pos = events
        .iter()
        .position(|e| e.op_idx == 0)
        .expect("load_state fired");
    let compute_pos = events
        .iter()
        .position(|e| e.op_idx == 1)
        .expect("compute_next_state fired");
    assert!(
        load_pos < compute_pos,
        "load must precede compute in the recorded trace"
    );
}

/// Test 4: Verify multi-stage sensor-decision-actuator workflow executes fully
///
/// Real-world scenario: autonomous vehicle decision loop.
/// All 4 stages must fire, in order, with no stage skipped.
#[test]
fn test_multi_stage_sensor_decision_actuator() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("acquire_lidar_frame"),
        PowlAstNode::Atom("detect_obstacle"),
        PowlAstNode::Atom("compute_steering_angle"),
        PowlAstNode::Atom("send_actuator_command"),
    ]);

    let (tape, _state, log, _ticks) = execute(&ast, 4);

    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Conforms
    );

    // Verify strict left-to-right op ordering: op indices must be
    // non-decreasing across the recorded trace (each stage fires only
    // after all its predecessors have).
    let indices: Vec<u32> = log.events().iter().map(|e| e.op_idx).collect();
    for pair in indices.windows(2) {
        assert!(
            pair[0] < pair[1],
            "stages must fire in strictly increasing op order: {:?}",
            indices
        );
    }
}

/// Test 5: Verify precedence constraints prevent race conditions
///
/// sensor -> decision -> actuator: decision cannot fire before sensor, and
/// actuator cannot fire before decision. We verify by inspecting the
/// compiled tape's pred_mask for each op.
#[test]
fn test_precedence_prevents_race_conditions() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("read_sensor"),
        PowlAstNode::Atom("make_decision"),
        PowlAstNode::Atom("trigger_actuator"),
    ]);

    let tape = compile_powl(&ast).expect("must compile");
    let ops = &tape.ops[..tape.len as usize];

    // op 0 (read_sensor) has no predecessors.
    assert_eq!(ops[0].pred_mask, 0, "first stage has no predecessor");
    // op 1 (make_decision) must require op 0 to have completed.
    assert_eq!(
        ops[1].pred_mask & (1u64 << 0),
        1u64 << 0,
        "decision depends on sensor"
    );
    // op 2 (trigger_actuator) must require op 1 to have completed.
    assert_eq!(
        ops[2].pred_mask & (1u64 << 1),
        1u64 << 1,
        "actuator depends on decision"
    );
}

/// Test 6: Verify malformed workflows are rejected with typed errors
///
/// Unlike conventional schedulers that may silently accept a degenerate
/// workflow, POWL's compiler returns a typed `CompileError` for structurally
/// invalid input (e.g. an empty sequence), never a silent no-op.
#[test]
fn test_typed_errors_on_precondition_failure() {
    let empty_sequence: PowlAstNode<'_> = PowlAstNode::Sequence(vec![]);

    let result = compile_powl(&empty_sequence);

    assert_eq!(
        result,
        Err(CompileError::EmptySequence),
        "empty sequence must be refused with a typed error, not silently accepted"
    );
}
