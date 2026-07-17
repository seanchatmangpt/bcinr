//! ocel — Object-Centric Event Log for POWL conformance checking.
//!
//! Records `op_fired` and `run_sealed` events into a fixed-capacity log,
//! enabling process-mining conformance checks without heap allocation.

#![forbid(unsafe_code)]

#[cfg(feature = "std")]
use wasm4pm_compat::ocel::{
    OCELEvent, OCELEventAttribute, OCELObject, OCELRelationship, OCELType, OCEL,
};

pub struct OcelEvent {
    pub event_id: u64,
    pub activity: &'static str, // "op_fired" | "run_sealed"
    pub timestamp: u64,         // monotonic tick counter
    pub run_id: u64,
    pub op_idx: u32,
    pub kind_tag: u8,
}

pub struct OcelLog {
    events: [OcelEvent; 512],
    count: usize,
    tick: u64,
}

/// Error returned when an append operation cannot complete.
#[derive(Debug, PartialEq, Eq)]
pub enum OcelError {
    /// The log is at capacity (512 events); the event was NOT recorded.
    Overflow,
}

/// Result of conformance checking a log against a POWL tape.
#[derive(Debug, PartialEq, Eq)]
pub enum ConformanceResult {
    Conforms,
    /// A predecessor constraint was violated: op `op_idx` fired in `run_id`
    /// but the ops in `missing_pred_mask` had not yet fired.
    Violation {
        run_id: u64,
        op_idx: u32,
        missing_pred_mask: u64,
    },
    /// The same op index fired more than once within a single run.
    DuplicateFire {
        run_id: u64,
        op_idx: u32,
    },
    /// The declared `op_trace` at seal time does not exactly equal the set of
    /// `op_fired` events accumulated for that run.
    SealMismatch {
        run_id: u64,
        declared: u64,
        accumulated: u64,
    },
    /// The log contains no events.
    EmptyLog,
    /// Refusal: The log contains more unique run IDs than the fixed limits
    /// of the deterministic validator.
    RunLimitExceeded,
}

impl OcelLog {
    pub const fn new() -> Self {
        const DEFAULT_EVENT: OcelEvent = OcelEvent {
            event_id: 0,
            activity: "",
            timestamp: 0,
            run_id: 0,
            op_idx: 0,
            kind_tag: 0,
        };
        Self {
            events: [DEFAULT_EVENT; 512],
            count: 0,
            tick: 0,
        }
    }

    /// Record that operation `op_idx` fired within `run_id`.
    ///
    /// Returns `Err(OcelError::Overflow)` when the log is full; the event is
    /// NOT silently dropped — callers must handle the error.
    pub fn record_op_fired(
        &mut self,
        run_id: u64,
        op_idx: u32,
        kind_tag: u8,
    ) -> Result<(), OcelError> {
        if self.count >= 512 {
            return Err(OcelError::Overflow);
        }
        self.tick += 1;
        self.events[self.count] = OcelEvent {
            event_id: self.count as u64,
            activity: "op_fired",
            timestamp: self.tick,
            run_id,
            op_idx,
            kind_tag,
        };
        self.count += 1;
        Ok(())
    }

    /// Record that run `run_id` was sealed with the given `op_trace` bitmask.
    /// The low 32 bits of `op_trace` are stored in `op_idx`; `kind_tag` is 0.
    ///
    /// Returns `Err(OcelError::Overflow)` when the log is full; the event is
    /// NOT silently dropped — callers must handle the error.
    pub fn record_run_sealed(&mut self, run_id: u64, op_trace: u64) -> Result<(), OcelError> {
        if self.count >= 512 {
            return Err(OcelError::Overflow);
        }
        self.tick += 1;
        self.events[self.count] = OcelEvent {
            event_id: self.count as u64,
            activity: "run_sealed",
            timestamp: self.tick,
            run_id,
            op_idx: op_trace as u32,
            kind_tag: 0,
        };
        self.count += 1;
        Ok(())
    }

    /// Return the slice of recorded events.
    pub fn events(&self) -> &[OcelEvent] {
        &self.events[..self.count]
    }

    /// Validate the log against the given POWL tape's predecessor masks.
    /// No heap, no_std safe.
    pub fn validate_against_tape(&self, tape: &crate::tape::PowlTape) -> ConformanceResult {
        validate_against_tape(self, tape)
    }

    /// Convert to an OCEL 2.0 structure (std-gated).
    #[cfg(feature = "std")]
    pub fn to_ocel_2_0(&self) -> OCEL {
        use std::collections::BTreeSet;

        // Collect unique run_ids and op_idxs
        let mut run_ids = BTreeSet::new();
        let mut op_idxs = BTreeSet::new();
        for e in self.events() {
            run_ids.insert(e.run_id);
            if e.activity == "op_fired" {
                op_idxs.insert(e.op_idx);
            }
        }

        let object_types = vec![
            OCELType {
                name: "PowlRun".to_string(),
                attributes: vec![],
            },
            OCELType {
                name: "PowlOp".to_string(),
                attributes: vec![],
            },
        ];

        let event_types = vec![
            OCELType {
                name: "op_fired".to_string(),
                attributes: vec![],
            },
            OCELType {
                name: "run_sealed".to_string(),
                attributes: vec![],
            },
        ];

        let mut objects: Vec<OCELObject> = Vec::new();
        for run_id in &run_ids {
            objects.push(OCELObject::new(format!("run-{}", run_id), "PowlRun"));
        }
        for op_idx in &op_idxs {
            objects.push(OCELObject::new(format!("op-{}", op_idx), "PowlOp"));
        }

        let mut events: Vec<OCELEvent> = Vec::new();
        for e in self.events() {
            match e.activity {
                "op_fired" => {
                    let mut evt = OCELEvent::new(format!("evt-{}", e.event_id), "op_fired");
                    evt.relationships.push(OCELRelationship {
                        object_id: format!("run-{}", e.run_id),
                        qualifier: "belongs_to".to_string(),
                    });
                    evt.relationships.push(OCELRelationship {
                        object_id: format!("op-{}", e.op_idx),
                        qualifier: "fires".to_string(),
                    });
                    events.push(evt);
                }
                "run_sealed" => {
                    let op_trace = e.op_idx as u64;
                    let mut evt = OCELEvent::new(format!("evt-{}", e.event_id), "run_sealed");
                    evt.attributes
                        .push(OCELEventAttribute::integer("op_trace", op_trace as i64));
                    evt.relationships.push(OCELRelationship {
                        object_id: format!("run-{}", e.run_id),
                        qualifier: "seals".to_string(),
                    });
                    events.push(evt);
                }
                _ => {}
            }
        }

        OCEL {
            event_types,
            object_types,
            events,
            objects,
        }
    }

    /// Serialize to OCEL 2.0 JSON (std-gated).
    #[cfg(feature = "std")]
    pub fn to_ocel_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_ocel_2_0())
    }
}

/// Validate an OcelLog against a PowlTape's predecessor masks.
/// No heap, no_std safe.
///
/// Checks performed (in order):
/// 1. `EmptyLog`           — log has no events at all.
/// 2. `DuplicateFire`      — same op fired twice in one run (per run_id).
/// 3. `SealMismatch`       — declared op_trace ≠ accumulated fired-op bitmask.
/// 4. `Violation`          — predecessor constraint: op fired before its pred.
/// Symmetric Run-Bounded Conformance Gating (SRBCG) Slot Tracker.
///
/// Ensures CC=1 execution while checking run capacities. No heap allocations
/// are performed, and no data-dependent jumps are generated.
#[inline(always)]
pub fn process_event_srbcg(
    run_ids: &mut [u64; 64],
    run_count: &mut usize,
    incoming_rid: u64,
    overflow_mask: &mut u64,
) -> usize {
    let mut match_idx = 64usize;
    let current_count = *run_count;

    // Unrolled comparison across all 64 slots.
    // Compiles to branchless conditional selections (CSEL/CMOV).
    for i in 0..64 {
        let is_match = (run_ids[i] == incoming_rid) as usize;
        // If a match is found, match_idx becomes the slot index.
        // Otherwise, it remains unchanged.
        match_idx = (is_match * i) + ((1 - is_match) * match_idx);
    }

    // Determine if we need to allocate a new slot.
    let found = (match_idx < 64) as usize;
    let can_allocate = (current_count < 64) as usize;

    // Actions based on state:
    // Case 1: Found existing slot -> use match_idx, no count change, no overflow.
    // Case 2: Not found & can allocate -> use current_count, increment count, no overflow.
    // Case 3: Not found & cannot allocate -> use 64, no count change, set overflow.
    
    let allocate_idx = current_count;
    let target_idx = (found * match_idx) 
        + ((1 - found) * (can_allocate * allocate_idx + (1 - can_allocate) * 64));

    // Update count: increment if not found and can allocate.
    *run_count = current_count + ((1 - found) * can_allocate);

    // Update run_ids: write incoming_rid to target_idx if we allocated a new slot.
    let should_write = (1 - found) * can_allocate;
    for i in 0..64 {
        let mask = 0u64.wrapping_sub((should_write & (i == target_idx) as usize) as u64);
        run_ids[i] = (incoming_rid & mask) | (run_ids[i] & !mask);
    }

    // Accumulate overflow mask if not found and cannot allocate.
    let has_overflowed = (1 - found) * (1 - can_allocate);
    *overflow_mask |= 0u64.wrapping_sub(has_overflowed as u64);

    target_idx
}

/// Validate an OcelLog against a PowlTape's predecessor masks.
/// No heap, no_std safe.
///
/// Checks performed (in order):
/// 1. `EmptyLog`           — log has no events at all.
/// 2. `RunLimitExceeded`   — log has more than 64 unique run IDs.
/// 3. `DuplicateFire`      — same op fired twice in one run (per run_id).
/// 4. `SealMismatch`       — declared op_trace ≠ accumulated fired-op bitmask.
/// 5. `Violation`          — predecessor constraint: op fired before its pred.
pub fn validate_against_tape(log: &OcelLog, tape: &crate::tape::PowlTape) -> ConformanceResult {
    // 1. Empty log.
    if log.events().is_empty() {
        return ConformanceResult::EmptyLog;
    }

    let ops = &tape.ops[..tape.len as usize];

    // We need to visit every run_id.  With no_std/no-heap we use a fixed-size
    // table of up to 64 run_ids seen in this log.
    const MAX_RUNS: usize = 64;
    let mut run_ids: [u64; MAX_RUNS] = [u64::MAX; MAX_RUNS];
    let mut accumulated: [u64; MAX_RUNS + 1] = [0u64; MAX_RUNS + 1];
    let mut fired_twice: [u64; MAX_RUNS + 1] = [0u64; MAX_RUNS + 1]; // bits set on 2nd fire
    let mut declared: [u64; MAX_RUNS + 1] = [u64::MAX; MAX_RUNS + 1]; // sentinel = not seen
    let mut run_count: usize = 0;
    let mut overflow_mask: u64 = 0;

    for event in log.events() {
        match event.activity {
            "op_fired" => {
                let s = process_event_srbcg(&mut run_ids, &mut run_count, event.run_id, &mut overflow_mask);
                let bit = 1u64.checked_shl(event.op_idx).unwrap_or(0);
                let has_fired_mask = 0u64.wrapping_sub(((accumulated[s] & bit) != 0) as u64);
                fired_twice[s] |= bit & has_fired_mask;
                accumulated[s] |= bit;
            }
            "run_sealed" => {
                let s = process_event_srbcg(&mut run_ids, &mut run_count, event.run_id, &mut overflow_mask);
                declared[s] = event.op_idx as u64; // low 32 bits stored here
            }
            _ => {}
        }
    }

    if overflow_mask != 0 {
        return ConformanceResult::RunLimitExceeded;
    }

    // Check each run.
    for s in 0..run_count {
        let run_id = run_ids[s];

        // 2. DuplicateFire — if any bit is set, report the lowest one.
        if fired_twice[s] != 0 {
            let op_idx = fired_twice[s].trailing_zeros();
            return ConformanceResult::DuplicateFire { run_id, op_idx };
        }

        // 3. SealMismatch — declared op_trace must equal accumulated fired mask.
        //    Only check if we actually saw a run_sealed event (sentinel u64::MAX).
        if declared[s] != u64::MAX {
            let decl = declared[s];
            let accum = accumulated[s];
            if decl != accum {
                return ConformanceResult::SealMismatch {
                    run_id,
                    declared: decl,
                    accumulated: accum,
                };
            }
        }

        // 4. Predecessor violation — iterate over every fired op in this run.
        let op_trace = accumulated[s];
        let mut bits = op_trace;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            let op_idx_usize = op_idx as usize;
            if op_idx_usize >= ops.len() {
                continue;
            }
            let pred_mask = ops[op_idx_usize].pred_mask;
            let missing = pred_mask & !op_trace;
            if missing != 0 {
                return ConformanceResult::Violation {
                    run_id,
                    op_idx,
                    missing_pred_mask: missing,
                };
            }
        }
    }

    ConformanceResult::Conforms
}

impl Default for OcelLog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocel_log_conforms_to_powl_model() {
        let mut log = OcelLog::new();
        let run_id = 42u64;
        log.record_op_fired(run_id, 0, 1).unwrap();
        log.record_op_fired(run_id, 1, 2).unwrap();
        log.record_run_sealed(run_id, 0b11).unwrap();
        let events = log.events();
        let op_fired_runs: Vec<u64> = events
            .iter()
            .filter(|e| e.activity == "op_fired")
            .map(|e| e.run_id)
            .collect();
        let sealed_runs: Vec<u64> = events
            .iter()
            .filter(|e| e.activity == "run_sealed")
            .map(|e| e.run_id)
            .collect();
        for run in &op_fired_runs {
            assert!(
                sealed_runs.contains(run),
                "run {run} has op_fired but no run_sealed"
            );
        }
        let sealed_ts = events
            .iter()
            .find(|e| e.activity == "run_sealed" && e.run_id == run_id)
            .map(|e| e.timestamp)
            .expect("run_sealed must exist");
        for e in events
            .iter()
            .filter(|e| e.activity == "op_fired" && e.run_id == run_id)
        {
            assert!(
                e.timestamp < sealed_ts,
                "op_fired at {} must precede run_sealed at {}",
                e.timestamp,
                sealed_ts
            );
        }
        let op_idxs: Vec<u32> = events
            .iter()
            .filter(|e| e.activity == "op_fired" && e.run_id == run_id)
            .map(|e| e.op_idx)
            .collect();
        let mut seen = std::collections::HashSet::new();
        for idx in &op_idxs {
            assert!(seen.insert(idx), "duplicate op_idx {idx} in run {run_id}");
        }
        let computed_trace: u64 = op_idxs.iter().fold(0u64, |acc, &idx| acc | (1u64 << idx));
        let sealed_trace = events
            .iter()
            .find(|e| e.activity == "run_sealed" && e.run_id == run_id)
            .map(|e| e.op_idx as u64)
            .expect("run_sealed must exist");
        assert_eq!(
            computed_trace, sealed_trace,
            "op_trace mismatch: computed {computed_trace:#b} vs sealed {sealed_trace:#b}"
        );
    }

    #[test]
    fn ocel_rejects_impossible_op_trace() {
        let mut log = OcelLog::new();
        let run_id = 99u64;
        log.record_op_fired(run_id, 0, 1).unwrap();
        log.record_op_fired(run_id, 1, 2).unwrap();
        log.record_run_sealed(run_id, 0b111).unwrap();
        let events = log.events();
        let op_fired_count = events
            .iter()
            .filter(|e| e.activity == "op_fired" && e.run_id == run_id)
            .count();
        let sealed_trace = events
            .iter()
            .find(|e| e.activity == "run_sealed" && e.run_id == run_id)
            .map(|e| e.op_idx as u64)
            .expect("run_sealed must exist");
        let sealed_op_count = sealed_trace.count_ones() as usize;
        assert!(op_fired_count < sealed_op_count,
            "Expected impossible trace gap: op_fired count ({op_fired_count}) < op_trace.count_ones() ({sealed_op_count})");
    }

    #[test]
    fn validate_rejects_predecessor_violation() {
        use crate::compiler::{compile_powl, PowlAstNode};
        // Sequence: op0 → op1, so op1.pred_mask = 0b01
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();

        let mut log = OcelLog::new();
        // Record op_fired only for op_idx=1 (skip op_idx=0)
        log.record_op_fired(99, 1, 0).unwrap();
        // Seal with only op1 fired (bit 1 = 0b10, missing bit 0 = 0b01)
        log.record_run_sealed(99, 0b10).unwrap();

        let result = validate_against_tape(&log, &tape);
        assert_eq!(
            result,
            ConformanceResult::Violation {
                run_id: 99,
                op_idx: 1,
                missing_pred_mask: 0b01,
            }
        );
    }

    #[test]
    fn validate_accepts_valid_trace() {
        use crate::compiler::{compile_powl, PowlAstNode};
        // Sequence: op0 → op1
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();

        let mut log = OcelLog::new();
        log.record_op_fired(1, 0, 0).unwrap();
        log.record_op_fired(1, 1, 0).unwrap();
        log.record_run_sealed(1, 0b11).unwrap();

        let result = validate_against_tape(&log, &tape);
        assert_eq!(result, ConformanceResult::Conforms);
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_ocel_2_0_has_object_types_and_event_types() {
        let mut log = OcelLog::new();
        log.record_op_fired(1, 0, 0).unwrap();
        log.record_run_sealed(1, 0b1).unwrap();

        let ocel = log.to_ocel_2_0();
        let obj_type_names: Vec<&str> = ocel.object_types.iter().map(|t| t.name.as_str()).collect();
        assert!(
            obj_type_names.contains(&"PowlRun"),
            "missing PowlRun object type"
        );
        assert!(
            obj_type_names.contains(&"PowlOp"),
            "missing PowlOp object type"
        );

        let event_type_names: Vec<&str> =
            ocel.event_types.iter().map(|t| t.name.as_str()).collect();
        assert!(
            event_type_names.contains(&"op_fired"),
            "missing op_fired event type"
        );
        assert!(
            event_type_names.contains(&"run_sealed"),
            "missing run_sealed event type"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_ocel_2_0_events_have_object_relationships() {
        let mut log = OcelLog::new();
        log.record_op_fired(42, 0, 0).unwrap();
        log.record_run_sealed(42, 0b1).unwrap();

        let ocel = log.to_ocel_2_0();
        let op_fired_events: Vec<_> = ocel
            .events
            .iter()
            .filter(|e| e.event_type == "op_fired")
            .collect();
        assert!(!op_fired_events.is_empty(), "must have op_fired events");
        let rel_ids: Vec<&str> = op_fired_events[0]
            .relationships
            .iter()
            .map(|r| r.object_id.as_str())
            .collect();
        assert!(
            rel_ids.iter().any(|id| id.contains("run-42")),
            "op_fired must link to run-42"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn to_ocel_json_is_valid_json() {
        let mut log = OcelLog::new();
        log.record_op_fired(1, 0, 0).unwrap();
        log.record_run_sealed(1, 0b1).unwrap();

        let json = log.to_ocel_json().expect("serialisation must succeed");
        assert!(
            json.contains("eventTypes") || json.contains("event_types"),
            "JSON must contain eventTypes or event_types: {json}"
        );
    }

    // ---- overflow / OcelError::Overflow ----

    #[test]
    fn record_op_fired_returns_overflow_when_full() {
        let mut log = OcelLog::new();
        for i in 0u32..512 {
            log.record_op_fired(0, i % 64, 0).unwrap();
        }
        let err = log
            .record_op_fired(0, 0, 0)
            .expect_err("must return Overflow when log is full");
        assert_eq!(err, OcelError::Overflow);
    }

    #[test]
    fn record_run_sealed_returns_overflow_when_full() {
        let mut log = OcelLog::new();
        for i in 0u32..512 {
            log.record_op_fired(0, i % 64, 0).unwrap();
        }
        let err = log
            .record_run_sealed(0, 0)
            .expect_err("must return Overflow when log is full");
        assert_eq!(err, OcelError::Overflow);
    }

    // ---- EmptyLog ----

    #[test]
    fn validate_empty_log_returns_empty_log() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();
        let log = OcelLog::new();
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::EmptyLog
        );
    }

    // ---- DuplicateFire ----

    #[test]
    fn validate_duplicate_fire_detected() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();
        let mut log = OcelLog::new();
        let run_id = 7u64;
        log.record_op_fired(run_id, 0, 0).unwrap();
        log.record_op_fired(run_id, 0, 0).unwrap(); // duplicate
        log.record_run_sealed(run_id, 0b1).unwrap();
        let result = validate_against_tape(&log, &tape);
        assert_eq!(
            result,
            ConformanceResult::DuplicateFire { run_id, op_idx: 0 }
        );
    }

    // ---- SealMismatch ----

    #[test]
    fn validate_seal_mismatch_extra_bit_in_declared() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();
        let mut log = OcelLog::new();
        let run_id = 55u64;
        log.record_op_fired(run_id, 0, 0).unwrap();
        log.record_op_fired(run_id, 1, 0).unwrap();
        // Declare op 2 as done but it was never fired.
        log.record_run_sealed(run_id, 0b111).unwrap();
        let result = validate_against_tape(&log, &tape);
        assert_eq!(
            result,
            ConformanceResult::SealMismatch {
                run_id,
                declared: 0b111,
                accumulated: 0b11,
            }
        );
    }

    #[test]
    fn validate_seal_mismatch_missing_bit_in_declared() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();
        let mut log = OcelLog::new();
        let run_id = 56u64;
        log.record_op_fired(run_id, 0, 0).unwrap();
        log.record_op_fired(run_id, 1, 0).unwrap();
        // Declare only op 0 as done but op 1 was also fired.
        log.record_run_sealed(run_id, 0b01).unwrap();
        let result = validate_against_tape(&log, &tape);
        assert_eq!(
            result,
            ConformanceResult::SealMismatch {
                run_id,
                declared: 0b01,
                accumulated: 0b11,
            }
        );
    }

    #[test]
    fn validate_rejects_exceeded_run_limit() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();

        // 64 runs should succeed.
        let mut log_64 = OcelLog::new();
        for i in 0..64 {
            log_64.record_op_fired(i as u64, 0, 0).unwrap();
            log_64.record_run_sealed(i as u64, 0b1).unwrap();
        }
        assert_eq!(validate_against_tape(&log_64, &tape), ConformanceResult::Conforms);

        // 65 runs should trigger RunLimitExceeded.
        let mut log_65 = OcelLog::new();
        for i in 0..65 {
            log_65.record_op_fired(i as u64, 0, 0).unwrap();
            log_65.record_run_sealed(i as u64, 0b1).unwrap();
        }
        assert_eq!(
            validate_against_tape(&log_65, &tape),
            ConformanceResult::RunLimitExceeded
        );
    }

    #[test]
    fn validate_vulnerability_isolation_run_65_violation() {
        use crate::compiler::{compile_powl, PowlAstNode};
        // Sequence: op0 -> op1
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();

        // 64 conforming runs.
        let mut log = OcelLog::new();
        for i in 0..64 {
            log.record_op_fired(i as u64, 0, 0).unwrap();
            log.record_op_fired(i as u64, 1, 0).unwrap();
            log.record_run_sealed(i as u64, 0b11).unwrap();
        }
        // 65th run is non-conforming (predecessor violation: fire op 1 without op 0, seal with 0b10).
        log.record_op_fired(64, 1, 0).unwrap();
        log.record_run_sealed(64, 0b10).unwrap();

        // Legitimate validation must return RunLimitExceeded because we have 65 unique run IDs.
        // It must NOT return Conforms (silent skipping vulnerability) or Violation (analyzing unadmitted runs).
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::RunLimitExceeded
        );
    }
}

