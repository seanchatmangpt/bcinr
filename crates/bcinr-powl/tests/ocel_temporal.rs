//! Temporal event log tests for OCEL with real wall/logical time tracking.
//!
//! These tests verify that OCEL events correctly record and serialize
//! temporal information including start_time and duration.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::{ConformanceResult, Duration, EventKind, OcelLog};
use bcinr_powl::scheduler::LogicalTime;

/// Test 1: record_op_fired with start_time and duration, verify OcelEvent carries them.
///
/// This test confirms that when an operation is recorded with explicit start_time
/// and duration parameters, those values are correctly stored in the OcelEvent
/// and can be retrieved unchanged.
#[test]
fn test_record_op_fired_with_temporal_fields() {
    let mut log = OcelLog::new();
    let run_id = 42u64;
    let op_idx = 5u32;
    let start_time: LogicalTime = 10u32;
    let duration: Duration = 7u32;

    // Record an operation fire with temporal fields
    let result = log.record_op_fired(run_id, op_idx, start_time, duration);
    assert!(result.is_ok(), "record_op_fired should succeed");

    // Verify the event was recorded
    let events = log.events();
    assert_eq!(events.len(), 1, "Log should contain exactly 1 event");

    let event = &events[0];
    assert_eq!(event.event_id, 0, "First event should have event_id 0");
    assert_eq!(event.activity, "op_fired", "Activity should be 'op_fired'");
    assert_eq!(event.start_time, start_time, "start_time should match");
    assert_eq!(event.duration, duration, "duration should match");
    assert_eq!(event.run_id, run_id, "run_id should match");
    assert_eq!(event.op_idx, op_idx, "op_idx should match");

    // Verify event_kind carries the temporal data
    match event.event_kind {
        EventKind::OpFired {
            start_time: kind_start,
            duration: kind_duration,
        } => {
            assert_eq!(kind_start, start_time, "EventKind start_time should match");
            assert_eq!(kind_duration, duration, "EventKind duration should match");
        }
        _ => panic!("event_kind should be OpFired"),
    }
}

/// Test 2: Sealed receipt includes all resource and temporal events in order.
///
/// This test verifies that a complete trace including op fires and run seal
/// events produces a consistent and replayable BLAKE3 receipt.
#[test]
fn test_sealed_receipt_includes_temporal_events() {
    let mut log = OcelLog::new();
    let run_id = 99u64;

    // Record multiple operations with varying temporal data
    let result1 = log.record_op_fired(run_id, 0, 5, 3);
    assert!(result1.is_ok());

    let result2 = log.record_op_fired(run_id, 1, 8, 4);
    assert!(result2.is_ok());

    let result3 = log.record_op_fired(run_id, 2, 12, 2);
    assert!(result3.is_ok());

    // Seal the run at a specific time
    let result_seal = log.record_run_sealed(run_id, 0b111, 14);
    assert!(result_seal.is_ok());

    // Verify event order and content
    let events = log.events();
    assert_eq!(events.len(), 4, "Should have 4 events total");

    // Op 0: start=5, duration=3
    assert_eq!(events[0].start_time, 5, "Event 0 start_time");
    assert_eq!(events[0].duration, 3, "Event 0 duration");
    assert_eq!(events[0].activity, "op_fired");

    // Op 1: start=8, duration=4
    assert_eq!(events[1].start_time, 8, "Event 1 start_time");
    assert_eq!(events[1].duration, 4, "Event 1 duration");
    assert_eq!(events[1].activity, "op_fired");

    // Op 2: start=12, duration=2
    assert_eq!(events[2].start_time, 12, "Event 2 start_time");
    assert_eq!(events[2].duration, 2, "Event 2 duration");
    assert_eq!(events[2].activity, "op_fired");

    // Seal: run_time=14
    assert_eq!(events[3].activity, "run_sealed");
    assert_eq!(events[3].start_time, 14, "Seal start_time (run_time)");

    // Seal receipt should be deterministic
    let receipt1 = log.seal_receipt();
    let receipt2 = log.seal_receipt();
    assert_eq!(
        receipt1.digest(),
        receipt2.digest(),
        "Same log should produce identical digests"
    );

    // Verify receipt contains all events
    assert_eq!(
        receipt1.event_count(),
        4,
        "Receipt should commit to all 4 events"
    );
}

/// Test 3: Temporal conformance validation with POWL tape.
///
/// This test combines the temporal event tracking with conformance checking
/// to ensure that operations fired in proper order and sealed correctly.
#[test]
fn test_temporal_conformance_with_tape() {
    // Compile a simple two-op sequence
    let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")]);
    let tape = compile_powl(&ast).expect("Tape should compile");

    let mut log = OcelLog::new();
    let run_id = 1u64;

    // Fire operations with increasing logical times
    let result_a = log.record_op_fired(run_id, 0, 5, 2);
    assert!(result_a.is_ok());

    let result_b = log.record_op_fired(run_id, 1, 7, 3);
    assert!(result_b.is_ok());

    let result_seal = log.record_run_sealed(run_id, 0b11, 10);
    assert!(result_seal.is_ok());

    // Validate conformance
    let conformance = log.validate_against_tape(&tape);
    assert_eq!(
        conformance,
        ConformanceResult::Conforms,
        "Properly sequenced operations should conform"
    );
}

/// Test 4: Multiple runs with interleaved temporal events.
///
/// This test verifies that the log correctly handles multiple concurrent runs
/// each with their own temporal tracking.
#[test]
fn test_multiple_runs_temporal_interleave() {
    let mut log = OcelLog::new();

    // Run 1: operations at times 0-5
    let _ = log.record_op_fired(1u64, 0, 0, 2);
    let _ = log.record_op_fired(1u64, 1, 2, 3);

    // Run 2: operations at times 1-4
    let _ = log.record_op_fired(2u64, 0, 1, 1);
    let _ = log.record_op_fired(2u64, 1, 2, 2);

    // Seal both runs
    let _ = log.record_run_sealed(1u64, 0b11, 5);
    let _ = log.record_run_sealed(2u64, 0b11, 4);

    let events = log.events();
    assert_eq!(events.len(), 6, "Should have 6 total events");

    // Verify temporal ordering is preserved within each run
    // Run 1 events
    assert_eq!(events[0].run_id, 1);
    assert_eq!(events[0].start_time, 0);
    assert_eq!(events[1].run_id, 1);
    assert_eq!(events[1].start_time, 2);

    // Run 2 events
    assert_eq!(events[2].run_id, 2);
    assert_eq!(events[2].start_time, 1);
    assert_eq!(events[3].run_id, 2);
    assert_eq!(events[3].start_time, 2);

    // Seals
    assert_eq!(events[4].start_time, 5, "Run 1 seal time");
    assert_eq!(events[5].start_time, 4, "Run 2 seal time");
}

/// Test 5: Event kind variants are correctly discriminated in receipts.
///
/// This test ensures that the BLAKE3 receipt includes proper event kind
/// discriminants so auditors can reconstruct event types from the digest.
#[test]
fn test_event_kind_discrimination_in_receipt() {
    let mut log = OcelLog::new();

    // Mix of different event activities
    let _ = log.record_op_fired(1u64, 0, 10, 5);
    let _ = log.record_run_sealed(1u64, 0b1, 15);

    let receipt = log.seal_receipt();

    // Verify receipt is deterministic with kind discriminant
    let events = receipt.log().events();
    assert_eq!(events.len(), 2);

    // First event is OpFired
    match events[0].event_kind {
        EventKind::OpFired { .. } => {
            // Expected
        }
        _ => panic!("Event 0 should be OpFired"),
    }

    // Second event is RunSealed
    match events[1].event_kind {
        EventKind::RunSealed { .. } => {
            // Expected
        }
        _ => panic!("Event 1 should be RunSealed"),
    }

    // Digests should match when sealing the same log
    let receipt2 = log.seal_receipt();
    assert_eq!(receipt.digest(), receipt2.digest());
}

/// Test 6: Duration edge cases (zero and maximum).
///
/// This test verifies that edge case duration values are handled correctly.
#[test]
fn test_duration_edge_cases() {
    let mut log = OcelLog::new();

    // Zero duration
    let _ = log.record_op_fired(1u64, 0, 5, 0);

    // Maximum duration
    let max_duration = u32::MAX;
    let _ = log.record_op_fired(1u64, 1, 10, max_duration);

    let events = log.events();
    assert_eq!(events[0].duration, 0, "Zero duration should be preserved");
    assert_eq!(
        events[1].duration, max_duration,
        "Max duration should be preserved"
    );
}
