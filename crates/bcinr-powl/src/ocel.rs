//! OCEL 2.0 conformance layer for POWL v2 event logs.
//!
//! Implements:
//! - [`OcelLog`] — fixed-capacity event log (no heap, `no_std` compatible with `std` feature)
//! - [`validate_against_tape`] — hostile conformance check against a POWL tape
//! - [`to_ocel_2_0`] — serialize to OCEL 2.0 JSON string
//!
//! # Law (van der Aalst constitution)
//! "If the event log cannot prove a lawful process happened, then it did not happen."

#[cfg(feature = "std")]
extern crate std;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of events stored in a single [`OcelLog`].
///
/// Overflow is *caught*: [`OcelLog::append`] returns `Err(OcelError::Overflow)`
/// when the log is full instead of silently dropping events.
pub const MAX_EVENTS: usize = 4096;

/// Maximum number of distinct run-ids that can appear in a single log.
pub const MAX_RUNS: usize = 256;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors emitted by the OCEL conformance layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcelError {
    /// An event was appended when the log was already at [`MAX_EVENTS`].
    Overflow,
    /// An op fired when its predecessor bits were not all satisfied.
    ///
    /// `(run_id, op_index, pred_mask, done_mask)` — the bits in `pred_mask`
    /// that are NOT in `done_mask` are the unsatisfied predecessors.
    PredecessorViolation { run_id: u32, op_index: u8, pred_mask: u64, done_mask: u64 },
    /// The same op fired more than once within the same run.
    DuplicateFire { run_id: u32, op_index: u8 },
    /// The run was sealed with an `op_trace` that does not match the events
    /// recorded in the log for that run.
    SealMismatch { run_id: u32 },
    /// The log is empty; nothing to validate.
    EmptyLog,
    /// A run-id appeared more than [`MAX_RUNS`] distinct times.
    TooManyRuns,
}

// ---------------------------------------------------------------------------
// Event record
// ---------------------------------------------------------------------------

/// A single OCEL 2.0 event recorded in the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OcelEvent {
    /// Monotonically increasing timestamp (nanoseconds since epoch or run-start).
    pub timestamp_ns: u64,
    /// Which run (object instance) this event belongs to.
    pub run_id: u32,
    /// Which op index on the tape fired.
    pub op_index: u8,
    /// Event type tag: 0 = op-fire, 1 = run-sealed.
    pub event_tag: u8,
    /// When `event_tag == 1`: the declared op_trace bitmask for this run.
    /// When `event_tag == 0`: 0 (unused).
    pub seal_trace: u64,
}

impl OcelEvent {
    /// Construct a normal op-fire event.
    pub const fn fire(timestamp_ns: u64, run_id: u32, op_index: u8) -> Self {
        Self { timestamp_ns, run_id, op_index, event_tag: 0, seal_trace: 0 }
    }

    /// Construct a run-sealed event.
    pub const fn seal(timestamp_ns: u64, run_id: u32, op_trace: u64) -> Self {
        Self { timestamp_ns, run_id, op_index: 0, event_tag: 1, seal_trace: op_trace }
    }
}

// ---------------------------------------------------------------------------
// OcelLog
// ---------------------------------------------------------------------------

/// Fixed-capacity OCEL 2.0 event log.
///
/// No heap allocation; safe to use in `no_std` environments when the `std`
/// feature is disabled.  Overflow is always caught — see [`OcelError::Overflow`].
pub struct OcelLog {
    events: [OcelEvent; MAX_EVENTS],
    len: usize,
}

impl OcelLog {
    /// Construct an empty log.
    pub const fn new() -> Self {
        Self {
            events: [OcelEvent { timestamp_ns: 0, run_id: 0, op_index: 0, event_tag: 0, seal_trace: 0 }; MAX_EVENTS],
            len: 0,
        }
    }

    /// Number of events currently in the log.
    pub fn len(&self) -> usize { self.len }

    /// `true` when the log contains no events.
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// Append an event.
    ///
    /// Returns `Err(OcelError::Overflow)` when [`MAX_EVENTS`] is reached.
    /// The event is **not** silently dropped.
    pub fn append(&mut self, event: OcelEvent) -> Result<(), OcelError> {
        if self.len >= MAX_EVENTS {
            return Err(OcelError::Overflow);
        }
        self.events[self.len] = event;
        self.len += 1;
        Ok(())
    }

    /// Read-only slice of all recorded events in insertion order.
    pub fn events(&self) -> &[OcelEvent] {
        &self.events[..self.len]
    }
}

impl Default for OcelLog {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Tape descriptor (subset of PowlTape needed for conformance checking)
// ---------------------------------------------------------------------------

/// A minimal view of a compiled POWL tape used by [`validate_against_tape`].
///
/// Only `pred_mask` per op index is required; the full tape types from
/// [`crate::tape`] can be adapted via [`TapeView::from_pred_masks`].
pub struct TapeView<'a> {
    /// `pred_masks[i]` is the bitmask of predecessors that must fire before op `i`.
    pub pred_masks: &'a [u64],
}

impl<'a> TapeView<'a> {
    pub fn new(pred_masks: &'a [u64]) -> Self {
        Self { pred_masks }
    }
}

// ---------------------------------------------------------------------------
// validate_against_tape
// ---------------------------------------------------------------------------

/// Per-run state during validation.
struct RunState {
    run_id: u32,
    done_mask: u64,
    declared_trace: Option<u64>,
    sealed: bool,
}

/// Hostile OCEL 2.0 conformance check against a POWL tape.
///
/// # Checks performed (ALL predecessor constraints, ALL run_ids)
///
/// 1. **Empty log** — immediately returns `Err(OcelError::EmptyLog)`.
/// 2. **Predecessor violation** — for every op-fire event, verifies that
///    `pred_mask[op_index] & !done_mask == 0` (all predecessor bits satisfied).
/// 3. **Duplicate fire** — rejects any op that fires twice in the same run.
/// 4. **Seal mismatch** — when a run-sealed event declares `op_trace`, the
///    accumulated `done_mask` for that run must exactly equal `op_trace`.
/// 5. **Too many runs** — rejects logs with more than [`MAX_RUNS`] distinct runs.
pub fn validate_against_tape(log: &OcelLog, tape: &TapeView<'_>) -> Result<(), OcelError> {
    if log.is_empty() {
        return Err(OcelError::EmptyLog);
    }

    // Per-run state: linear scan (no heap).
    let mut runs: [RunState; MAX_RUNS] = core::array::from_fn(|_| RunState {
        run_id: u32::MAX,
        done_mask: 0,
        declared_trace: None,
        sealed: false,
    });
    let mut run_count: usize = 0;

    let find_or_insert = |runs: &mut [RunState; MAX_RUNS], run_count: &mut usize, rid: u32| -> Result<usize, OcelError> {
        for i in 0..*run_count {
            if runs[i].run_id == rid {
                return Ok(i);
            }
        }
        if *run_count >= MAX_RUNS {
            return Err(OcelError::TooManyRuns);
        }
        let i = *run_count;
        runs[i].run_id = rid;
        *run_count += 1;
        Ok(i)
    };

    for event in log.events() {
        let slot = find_or_insert(&mut runs, &mut run_count, event.run_id)?;

        if event.event_tag == 1 {
            // Run-sealed event: verify declared trace matches accumulated done_mask.
            let accumulated = runs[slot].done_mask;
            if accumulated != event.seal_trace {
                return Err(OcelError::SealMismatch { run_id: event.run_id });
            }
            runs[slot].declared_trace = Some(event.seal_trace);
            runs[slot].sealed = true;
        } else {
            // Op-fire event.
            let op = event.op_index as usize;
            let done = runs[slot].done_mask;

            // Duplicate fire check.
            if done & (1u64 << op) != 0 {
                return Err(OcelError::DuplicateFire {
                    run_id: event.run_id,
                    op_index: event.op_index,
                });
            }

            // Predecessor constraint check.
            if op < tape.pred_masks.len() {
                let pred = tape.pred_masks[op];
                let unsatisfied = pred & !done;
                if unsatisfied != 0 {
                    return Err(OcelError::PredecessorViolation {
                        run_id: event.run_id,
                        op_index: event.op_index,
                        pred_mask: pred,
                        done_mask: done,
                    });
                }
            }

            runs[slot].done_mask |= 1u64 << op;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// to_ocel_2_0 / to_ocel_json
// ---------------------------------------------------------------------------

/// Serialize the log to an OCEL 2.0-compliant JSON string.
///
/// OCEL 2.0 schema requires top-level keys:
/// - `objectTypes`
/// - `eventTypes`
/// - `objects`
/// - `events`
///
/// Every event must have a `timestamp` field.
/// Every event must declare its `omap` (object references) pointing to a
/// declared object.
///
/// Requires the `std` feature (returns `None` when not available at
/// compile time — callers should gate on `#[cfg(feature = "std")]`).
#[cfg(feature = "std")]
pub fn to_ocel_2_0(log: &OcelLog) -> std::string::String {
    use std::collections::BTreeSet;
    use std::format;

    // Collect distinct run_ids and op_indices.
    let mut run_ids: BTreeSet<u32> = BTreeSet::new();
    let mut op_types: BTreeSet<u8> = BTreeSet::new();

    for ev in log.events() {
        run_ids.insert(ev.run_id);
        if ev.event_tag == 0 {
            op_types.insert(ev.op_index);
        }
    }

    // objectTypes
    let object_types = r#"[{"name":"PowlRun","attributes":[]}]"#;

    // eventTypes
    let mut et_parts: std::vec::Vec<std::string::String> = std::vec::Vec::new();
    for op in &op_types {
        et_parts.push(format!(r#"{{"name":"op_{op}","attributes":[]}}"#));
    }
    // also include seal type
    et_parts.push(r#"{"name":"run_sealed","attributes":[]}"#.to_string());
    let event_types = format!("[{}]", et_parts.join(","));

    // objects
    let mut obj_parts: std::vec::Vec<std::string::String> = std::vec::Vec::new();
    for rid in &run_ids {
        obj_parts.push(format!(
            r#"{{"id":"run_{rid}","type":"PowlRun","attributes":[]}}"#
        ));
    }
    let objects = format!("[{}]", obj_parts.join(","));

    // events
    let mut ev_parts: std::vec::Vec<std::string::String> = std::vec::Vec::new();
    for (i, ev) in log.events().iter().enumerate() {
        let ev_type = if ev.event_tag == 1 {
            "run_sealed".to_string()
        } else {
            format!("op_{}", ev.op_index)
        };
        let timestamp = ev.timestamp_ns;
        let run_id = ev.run_id;
        ev_parts.push(format!(
            r#"{{"id":"ev_{i}","type":"{ev_type}","timestamp":{timestamp},"omap":["run_{run_id}"],"attributes":[]}}"#
        ));
    }
    let events = format!("[{}]", ev_parts.join(","));

    format!(
        r#"{{"objectTypes":{object_types},"eventTypes":{event_types},"objects":{objects},"events":{events}}}"#
    )
}

/// Alias for [`to_ocel_2_0`].
#[cfg(feature = "std")]
pub fn to_ocel_json(log: &OcelLog) -> std::string::String {
    to_ocel_2_0(log)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // validate_against_tape — gap 1a: ops fire out of order
    // -----------------------------------------------------------------------

    #[test]
    fn reject_op_fired_out_of_order() {
        // Tape: op 1 requires op 0 to be done first.
        let pred_masks = [0u64, 1u64 << 0]; // op0 has no pred, op1 needs op0
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        // Fire op 1 BEFORE op 0 — predecessor violation.
        log.append(OcelEvent::fire(1, 42, 1)).unwrap();

        assert_eq!(
            validate_against_tape(&log, &tape),
            Err(OcelError::PredecessorViolation {
                run_id: 42,
                op_index: 1,
                pred_mask: 1,
                done_mask: 0,
            })
        );
    }

    #[test]
    fn accept_op_fired_in_order() {
        let pred_masks = [0u64, 1u64 << 0];
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 42, 0)).unwrap();
        log.append(OcelEvent::fire(2, 42, 1)).unwrap();

        assert_eq!(validate_against_tape(&log, &tape), Ok(()));
    }

    // -----------------------------------------------------------------------
    // gap 1b: op fires twice
    // -----------------------------------------------------------------------

    #[test]
    fn reject_duplicate_fire() {
        let pred_masks = [0u64];
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 7, 0)).unwrap();
        log.append(OcelEvent::fire(2, 7, 0)).unwrap(); // op 0 fires again

        assert_eq!(
            validate_against_tape(&log, &tape),
            Err(OcelError::DuplicateFire { run_id: 7, op_index: 0 })
        );
    }

    // -----------------------------------------------------------------------
    // gap 1c: run_sealed with wrong op_trace
    // -----------------------------------------------------------------------

    #[test]
    fn reject_seal_with_wrong_op_trace() {
        let pred_masks = [0u64, 1u64];
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 99, 0)).unwrap();
        // Done mask is 0b01 but seal claims 0b11 (op 1 never fired).
        log.append(OcelEvent::seal(2, 99, 0b11)).unwrap();

        assert_eq!(
            validate_against_tape(&log, &tape),
            Err(OcelError::SealMismatch { run_id: 99 })
        );
    }

    #[test]
    fn accept_seal_with_correct_op_trace() {
        let pred_masks = [0u64, 1u64];
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 99, 0)).unwrap();
        log.append(OcelEvent::fire(2, 99, 1)).unwrap();
        log.append(OcelEvent::seal(3, 99, 0b11)).unwrap();

        assert_eq!(validate_against_tape(&log, &tape), Ok(()));
    }

    // -----------------------------------------------------------------------
    // gap 1d: empty log
    // -----------------------------------------------------------------------

    #[test]
    fn reject_empty_log() {
        let pred_masks = [0u64];
        let tape = TapeView::new(&pred_masks);
        let log = OcelLog::new();

        assert_eq!(validate_against_tape(&log, &tape), Err(OcelError::EmptyLog));
    }

    // -----------------------------------------------------------------------
    // multiple run_ids — each checked independently
    // -----------------------------------------------------------------------

    #[test]
    fn multi_run_independent_predecessor_check() {
        // op 1 requires op 0 done.
        let pred_masks = [0u64, 1u64 << 0];
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        // run 1: correct order
        log.append(OcelEvent::fire(1, 1, 0)).unwrap();
        log.append(OcelEvent::fire(2, 1, 1)).unwrap();
        // run 2: wrong order
        log.append(OcelEvent::fire(3, 2, 1)).unwrap(); // violation

        assert!(matches!(
            validate_against_tape(&log, &tape),
            Err(OcelError::PredecessorViolation { run_id: 2, op_index: 1, .. })
        ));
    }

    #[test]
    fn multi_run_both_valid() {
        let pred_masks = [0u64, 1u64];
        let tape = TapeView::new(&pred_masks);

        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 1, 0)).unwrap();
        log.append(OcelEvent::fire(2, 1, 1)).unwrap();
        log.append(OcelEvent::fire(3, 2, 0)).unwrap();
        log.append(OcelEvent::fire(4, 2, 1)).unwrap();

        assert_eq!(validate_against_tape(&log, &tape), Ok(()));
    }

    // -----------------------------------------------------------------------
    // gap 3: overflow is caught, not silently dropped
    // -----------------------------------------------------------------------

    #[test]
    fn overflow_is_caught_not_silently_dropped() {
        let mut log = OcelLog::new();
        for i in 0..MAX_EVENTS as u64 {
            // Fill up with events on different ops (within u8 range wrapping is fine)
            let op = (i % 64) as u8;
            // We need a fresh run per event to avoid duplicate-fire issues in validate
            // but here we just test the capacity guard.
            assert!(log.append(OcelEvent::fire(i, i as u32, op)).is_ok());
        }
        // The (MAX_EVENTS + 1)-th append must fail.
        assert_eq!(
            log.append(OcelEvent::fire(u64::MAX, 0, 0)),
            Err(OcelError::Overflow)
        );
        // The log length must not exceed MAX_EVENTS.
        assert_eq!(log.len(), MAX_EVENTS);
    }

    // -----------------------------------------------------------------------
    // gap 2: to_ocel_2_0 — OCEL 2.0 schema keys and timestamp
    // -----------------------------------------------------------------------

    #[cfg(feature = "std")]
    #[test]
    fn ocel_json_has_all_required_top_level_keys() {
        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1_000_000, 1, 0)).unwrap();

        let json = to_ocel_2_0(&log);

        // Strict key assertions — not OR-chained.
        assert!(json.contains("\"objectTypes\""), "missing objectTypes");
        assert!(json.contains("\"eventTypes\""), "missing eventTypes");
        assert!(json.contains("\"objects\""), "missing objects");
        assert!(json.contains("\"events\""), "missing events");
    }

    #[cfg(feature = "std")]
    #[test]
    fn ocel_json_every_event_has_timestamp() {
        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(42_000, 1, 0)).unwrap();
        log.append(OcelEvent::fire(43_000, 1, 1)).unwrap();
        log.append(OcelEvent::seal(44_000, 1, 0b11)).unwrap();

        let json = to_ocel_2_0(&log);

        // Every event object must carry a timestamp field.
        // We check that "timestamp" appears at least as many times as events.
        let count = json.matches("\"timestamp\"").count();
        assert!(
            count >= 3,
            "expected at least 3 timestamp fields, found {count} in: {json}"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn ocel_json_objects_declared_for_all_run_ids() {
        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 10, 0)).unwrap();
        log.append(OcelEvent::fire(2, 20, 0)).unwrap();

        let json = to_ocel_2_0(&log);

        assert!(json.contains("\"run_10\""), "run_10 object not declared");
        assert!(json.contains("\"run_20\""), "run_20 object not declared");
    }

    #[cfg(feature = "std")]
    #[test]
    fn ocel_json_events_reference_declared_objects() {
        let mut log = OcelLog::new();
        log.append(OcelEvent::fire(1, 5, 0)).unwrap();

        let json = to_ocel_2_0(&log);

        // omap must reference a declared object.
        assert!(json.contains("\"omap\":[\"run_5\"]"), "omap reference missing");
    }

    #[cfg(feature = "std")]
    #[test]
    fn ocel_json_empty_log_produces_valid_schema() {
        let log = OcelLog::new();
        let json = to_ocel_2_0(&log);

        assert!(json.contains("\"objectTypes\""));
        assert!(json.contains("\"eventTypes\""));
        assert!(json.contains("\"objects\""));
        assert!(json.contains("\"events\""));
    }
}
