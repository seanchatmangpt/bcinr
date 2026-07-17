//! Chicago TDD integration tests — van der Aalst constitution.
//!
//! "If the event log cannot prove a lawful process happened, then it did not happen."
//!
//! These tests use STRICT key assertions (not OR-chained checks) against the
//! OCEL 2.0 JSON schema produced by [`bcinr_powl::ocel::to_ocel_2_0`].

use bcinr_powl::ocel::{
    to_ocel_2_0, validate_against_tape, OcelError, OcelEvent, OcelLog, TapeView, MAX_EVENTS,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_tape_2ops() -> [u64; 2] {
    // op 0: no predecessors; op 1: requires op 0.
    [0u64, 1u64 << 0]
}

fn three_op_chain() -> [u64; 3] {
    // 0 → 1 → 2
    [0u64, 1u64 << 0, 1u64 << 1]
}

// ---------------------------------------------------------------------------
// OCEL 2.0 JSON schema — STRICT (not OR-chained)
// ---------------------------------------------------------------------------

#[test]
fn ocel_json_strict_objecttypes_key_present() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    let json = to_ocel_2_0(&log);
    assert!(json.contains("\"objectTypes\""), "OCEL 2.0 requires top-level key 'objectTypes'; got: {json}");
}

#[test]
fn ocel_json_strict_eventtypes_key_present() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    let json = to_ocel_2_0(&log);
    assert!(json.contains("\"eventTypes\""), "OCEL 2.0 requires top-level key 'eventTypes'; got: {json}");
}

#[test]
fn ocel_json_strict_objects_key_present() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    let json = to_ocel_2_0(&log);
    assert!(json.contains("\"objects\""), "OCEL 2.0 requires top-level key 'objects'; got: {json}");
}

#[test]
fn ocel_json_strict_events_key_present() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    let json = to_ocel_2_0(&log);
    assert!(json.contains("\"events\""), "OCEL 2.0 requires top-level key 'events'; got: {json}");
}

#[test]
fn ocel_json_strict_timestamp_on_every_event() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(100, 1, 0)).unwrap();
    log.append(OcelEvent::fire(200, 1, 1)).unwrap();
    log.append(OcelEvent::seal(300, 1, 0b11)).unwrap();
    let json = to_ocel_2_0(&log);
    let count = json.matches("\"timestamp\"").count();
    // Each of the 3 events must carry a timestamp.
    assert_eq!(count, 3, "expected exactly 3 timestamp fields (one per event), found {count}; json={json}");
}

#[test]
fn ocel_json_strict_omap_references_declared_object() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 42, 0)).unwrap();
    let json = to_ocel_2_0(&log);
    // The object "run_42" must appear in "objects" array.
    assert!(json.contains("\"run_42\""), "object run_42 must be declared; got: {json}");
    // The event's omap must reference it.
    assert!(json.contains("\"omap\":[\"run_42\"]"), "event omap must reference run_42; got: {json}");
}

#[test]
fn ocel_json_strict_event_type_name_correct() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 3)).unwrap();
    let json = to_ocel_2_0(&log);
    assert!(json.contains("\"op_3\""), "event type for op_index=3 must be 'op_3'; got: {json}");
}

#[test]
fn ocel_json_strict_seal_event_type_name() {
    let mut log = OcelLog::new();
    log.append(OcelEvent::seal(1, 1, 0)).unwrap();
    let json = to_ocel_2_0(&log);
    assert!(json.contains("\"run_sealed\""), "seal event type must be 'run_sealed'; got: {json}");
}

// ---------------------------------------------------------------------------
// validate_against_tape — hostile conformance
// ---------------------------------------------------------------------------

#[test]
fn chicago_empty_log_is_rejected() {
    let pred = simple_tape_2ops();
    let tape = TapeView::new(&pred);
    let log = OcelLog::new();
    assert_eq!(validate_against_tape(&log, &tape), Err(OcelError::EmptyLog));
}

#[test]
fn chicago_out_of_order_fire_is_rejected() {
    let pred = simple_tape_2ops();
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    // op 1 fires without op 0 done.
    log.append(OcelEvent::fire(1, 1, 1)).unwrap();
    assert!(matches!(
        validate_against_tape(&log, &tape),
        Err(OcelError::PredecessorViolation { run_id: 1, op_index: 1, .. })
    ));
}

#[test]
fn chicago_duplicate_fire_is_rejected() {
    let pred = simple_tape_2ops();
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    log.append(OcelEvent::fire(2, 1, 0)).unwrap(); // duplicate
    assert_eq!(
        validate_against_tape(&log, &tape),
        Err(OcelError::DuplicateFire { run_id: 1, op_index: 0 })
    );
}

#[test]
fn chicago_seal_with_wrong_trace_is_rejected() {
    let pred = simple_tape_2ops();
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    // Only op 0 fired but seal claims both fired (0b11).
    log.append(OcelEvent::seal(2, 1, 0b11)).unwrap();
    assert_eq!(
        validate_against_tape(&log, &tape),
        Err(OcelError::SealMismatch { run_id: 1 })
    );
}

#[test]
fn chicago_lawful_three_op_chain_accepted() {
    let pred = three_op_chain();
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    log.append(OcelEvent::fire(2, 1, 1)).unwrap();
    log.append(OcelEvent::fire(3, 1, 2)).unwrap();
    log.append(OcelEvent::seal(4, 1, 0b111)).unwrap();
    assert_eq!(validate_against_tape(&log, &tape), Ok(()));
}

#[test]
fn chicago_skipping_middle_op_is_rejected() {
    // op 2 requires op 1 (which requires op 0).
    let pred = three_op_chain();
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    // skip op 1, fire op 2 directly.
    log.append(OcelEvent::fire(2, 1, 2)).unwrap();
    assert!(matches!(
        validate_against_tape(&log, &tape),
        Err(OcelError::PredecessorViolation { run_id: 1, op_index: 2, .. })
    ));
}

#[test]
fn chicago_multiple_runs_independent_validation() {
    let pred = simple_tape_2ops();
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    // run 1: lawful
    log.append(OcelEvent::fire(1, 1, 0)).unwrap();
    log.append(OcelEvent::fire(2, 1, 1)).unwrap();
    // run 2: unlawful (op 1 before op 0)
    log.append(OcelEvent::fire(3, 2, 1)).unwrap();
    assert!(matches!(
        validate_against_tape(&log, &tape),
        Err(OcelError::PredecessorViolation { run_id: 2, .. })
    ));
}

#[test]
fn chicago_overflow_is_caught_not_dropped() {
    let mut log = OcelLog::new();
    for i in 0..MAX_EVENTS as u64 {
        log.append(OcelEvent::fire(i, i as u32, 0)).unwrap();
    }
    assert_eq!(
        log.append(OcelEvent::fire(u64::MAX, u32::MAX, 0)),
        Err(OcelError::Overflow)
    );
    assert_eq!(log.len(), MAX_EVENTS);
}

#[test]
fn chicago_parallel_ops_no_pred_both_accepted() {
    // Two ops, neither depends on the other.
    let pred = [0u64, 0u64];
    let tape = TapeView::new(&pred);
    let mut log = OcelLog::new();
    log.append(OcelEvent::fire(1, 1, 1)).unwrap();
    log.append(OcelEvent::fire(2, 1, 0)).unwrap();
    assert_eq!(validate_against_tape(&log, &tape), Ok(()));
}
