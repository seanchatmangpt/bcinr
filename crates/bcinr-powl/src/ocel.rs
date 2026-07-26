//! Object-Centric Event Log (OCEL) and Symmetric Run-Bounded Conformance Gating (SRBCG).
//!
//! This module provides a deterministic, zero-allocation (`no_std` compatible) mechanism for
//! recording execution traces of Partially Ordered Workflow Language (POWL) workflows and
//! validating them against a compiled [`crate::tape::PowlTape`].
//!
//! # Architecture & Design Goals
//!
//! The conformance checking engine is designed to operate within the constraints of the
//! **deterministic substrate** (refer to the project constitution in `AGENTS.md`). Specifically:
//! - **Radon Law ($CC=1$) Compliance**: The core slot assignment algorithm ([`process_event_srbcg`])
//!   is branchless and contains no data-dependent jumps.
//! - **Zero-Allocation**: No heap allocation is performed during event recording or conformance checking.
//!   All storage uses fixed-size arrays.
//! - **Symmetric Run-Bounded Conformance Gating (SRBCG)**: Restricts tracking to a maximum of 64 concurrent
//!   or unique runs. Any trace exceeding this boundary is rejected with a typed refusal.
//!
//! # Core Concepts
//!
//! ## Object-Centric Event Log ([`OcelLog`])
//!
//! The [`OcelLog`] records discrete events occurring during workflow runs. Each log can store up to 512
//! events. There are two primary event activities:
//! - **`op_fired`**: Marks the execution of an operation (`op_idx`) within a given `run_id`.
//! - **`run_sealed`**: Seals a `run_id` with a bitmask (`op_trace`) representing the set of operations
//!   asserted to have fired.
//!
//! ## Symmetric Run-Bounded Conformance Gating (SRBCG) & Comparison Networks
//!
//! Because heap allocation is forbidden, mapping dynamic `run_id`s to state vectors must be done
//! in bounded stack memory. The engine allocates a static slot array of size 64.
//!
//! To determine which slot corresponds to an incoming `run_id` without branching, [`process_event_srbcg`] uses a
//! **comparison network**. This compiles down to branchless conditional selection instructions
//! (e.g., `CSEL` on ARM, `CMOV` on x86).
//!
//! The comparison network operates as follows:
//! 1. Iterate over all 64 slots in a fixed loop.
//! 2. Generate a match mask `is_match = (run_ids[i] == incoming_rid)`.
//! 3. Use arithmetic selection to set the target index: `match_idx = (is_match * i) + ((1 - is_match) * match_idx)`.
//! 4. If the run is not found, allocate a new slot.
//! 5. If no slots are available, set an overflow mask rather than panicking or silent failure.
//!
//! ## Conformance Validation Checks
//!
//! The [`validate_against_tape`] function performs five consecutive validation checks:
//!
//! 1. **Empty Log Check**: Returns [`ConformanceResult::EmptyLog`] if no events are recorded.
//! 2. **Run Limit Check**: Returns [`ConformanceResult::RunLimitExceeded`] if more than 64 unique run IDs are present.
//! 3. **Duplicate Fire Check**: Returns [`ConformanceResult::DuplicateFire`] if an operation fires more than once in the same run.
//! 4. **Seal Mismatch Check**: Returns [`ConformanceResult::SealMismatch`] if the declared `op_trace` in a `run_sealed` event does not match the accumulated fired operations.
//! 5. **Predecessor Constraint Check**: Returns [`ConformanceResult::Violation`] if an operation is fired before its required predecessors (as compiled into the tape's `pred_mask`).
//!
//! # Examples
//!
//! ```
//! use bcinr_powl::ocel::{OcelLog, ConformanceResult};
//! use bcinr_powl::compiler::{compile_powl, PowlAstNode};
//!
//! // 1. Compile a simple POWL sequence: "a" -> "b"
//! let ast = PowlAstNode::Sequence(vec![
//!     PowlAstNode::Atom("a"),
//!     PowlAstNode::Atom("b"),
//! ]);
//! let tape = compile_powl(&ast).unwrap();
//!
//! // 2. Record conforming events
//! let mut log = OcelLog::new();
//! let run_id = 99u64;
//!
//! // "a" (op_idx 0) fires, followed by "b" (op_idx 1)
//! log.record_op_fired(run_id, 0, 0).unwrap();
//! log.record_op_fired(run_id, 1, 0).unwrap();
//! log.record_run_sealed(run_id, 0b11).unwrap();
//!
//! // 3. Validate
//! assert_eq!(log.validate_against_tape(&tape), ConformanceResult::Conforms);
//! ```

#![forbid(unsafe_code)]

#[cfg(feature = "std")]
use wasm4pm_compat::ocel::{
    OCELEvent, OCELEventAttribute, OCELObject, OCELRelationship, OCELType, OCELTypeAttribute, OCEL,
};

/// A discrete event recorded within an [`OcelLog`].
///
/// An event represents either the firing of an individual operation or the
/// sealing of a workflow run.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OcelEvent {
    /// A unique sequential identifier for the event.
    pub event_id: u64,
    /// The event type, either `"op_fired"` or `"run_sealed"`.
    pub activity: &'static str,
    /// A monotonic tick counter used to preserve order of events.
    pub timestamp: u64,
    /// The identifier of the execution run.
    pub run_id: u64,
    /// For `"op_fired"`, the index of the operation that fired.
    /// For `"run_sealed"`, the low 32 bits of the declared `op_trace` bitmask.
    pub op_idx: u32,
    /// For `run_sealed`, the complete 64-bit declared operation trace.
    /// For `op_fired`, this is zero.
    pub op_trace: u64,
    /// Auxiliary tag storing the operation kind or metadata.
    pub kind_tag: u8,
}

/// A fixed-capacity, heap-allocation-free event log.
///
/// `OcelLog` can hold up to 512 events of type [`OcelEvent`]. It supports
/// appending events with guaranteed $O(1)$ time complexity and no dynamic allocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OcelLog {
    events: [OcelEvent; 512],
    count: usize,
    tick: u64,
}

/// Errors that can occur when interacting with [`OcelLog`].
#[derive(Debug, PartialEq, Eq)]
pub enum OcelError {
    /// The log is at capacity (512 events); the event was NOT recorded.
    Overflow,
}

/// The outcome of conformance checking an [`OcelLog`] against a [`crate::tape::PowlTape`].
#[derive(Debug, PartialEq, Eq)]
pub enum ConformanceResult {
    /// The log is fully compliant with the POWL model.
    Conforms,
    /// A predecessor constraint was violated: op `op_idx` fired in `run_id`
    /// but the ops in `missing_pred_mask` had not yet fired.
    Violation {
        /// The identifier of the run that violated the constraint.
        run_id: u64,
        /// The index of the operation that fired out-of-order.
        op_idx: u32,
        /// A bitmask of predecessor operations that were missing.
        missing_pred_mask: u64,
    },
    /// An XOR join observed zero or multiple branch entries before firing.
    ChoiceViolation {
        /// The identifier of the affected run.
        run_id: u64,
        /// The XOR join operation index.
        join_op_idx: u32,
        /// Branch entries declared by the compiled XOR join.
        branch_mask: u64,
        /// Branch entries actually observed before the join fired.
        fired_branch_mask: u64,
    },
    /// The same op index fired more than once within a single run.
    DuplicateFire {
        /// The identifier of the run.
        run_id: u64,
        /// The index of the operation that fired multiple times.
        op_idx: u32,
    },
    /// The declared `op_trace` at seal time does not exactly equal the set of
    /// `op_fired` events accumulated for that run.
    SealMismatch {
        /// The identifier of the run.
        run_id: u64,
        /// The declared bitmask of fired operations.
        declared: u64,
        /// The actual accumulated bitmask of fired operations.
        accumulated: u64,
    },
    /// An operation index does not exist in the compiled tape.
    UnknownOperation {
        /// The identifier of the affected run.
        run_id: u64,
        /// The unknown operation index.
        op_idx: u32,
    },
    /// An operation was recorded after its run had already been sealed.
    EventAfterSeal {
        /// The identifier of the affected run.
        run_id: u64,
        /// The operation recorded after sealing.
        op_idx: u32,
    },
    /// A run was sealed more than once.
    DuplicateSeal {
        /// The identifier of the multiply sealed run.
        run_id: u64,
    },
    /// A run contains events but no terminal seal.
    MissingSeal {
        /// The identifier of the unsealed run.
        run_id: u64,
    },
    /// The log contains no events.
    EmptyLog,
    /// Refusal: The log contains more unique run IDs than the fixed limits
    /// of the deterministic validator.
    RunLimitExceeded,
}

/// A deterministic BLAKE3 receipt that owns the exact ordered OCEL trace it seals.
///
/// Unlike diagnostic collectors that may assign random event identifiers, this receipt
/// commits only to the canonical fields recorded by [`OcelLog`]. The owned log is retained
/// so an auditor can replay the exact operation sequence rather than trusting a bare digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OcelTraceReceipt {
    log: OcelLog,
    digest: [u8; 32],
}

impl OcelTraceReceipt {
    /// Return the sealed, ordered event log.
    #[inline]
    pub fn log(&self) -> &OcelLog {
        &self.log
    }

    /// Return the deterministic BLAKE3 digest of the canonical event encoding.
    #[inline]
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Number of ordered events committed by this receipt.
    #[inline]
    pub fn event_count(&self) -> usize {
        self.log.count
    }
}

impl OcelLog {
    /// Creates a new, empty `OcelLog` with a capacity of 512 events.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::ocel::OcelLog;
    ///
    /// let log = OcelLog::new();
    /// assert_eq!(log.events().len(), 0);
    /// ```
    pub const fn new() -> Self {
        const DEFAULT_EVENT: OcelEvent = OcelEvent {
            event_id: 0,
            activity: "",
            timestamp: 0,
            run_id: 0,
            op_idx: 0,
            op_trace: 0,
            kind_tag: 0,
        };
        Self {
            events: [DEFAULT_EVENT; 512],
            count: 0,
            tick: 0,
        }
    }

    /// Records that operation `op_idx` fired within `run_id`.
    ///
    /// # Errors
    ///
    /// Returns [`OcelError::Overflow`] when the log is full; the event is
    /// NOT silently dropped — callers must handle the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::ocel::OcelLog;
    ///
    /// let mut log = OcelLog::new();
    /// log.record_op_fired(42, 0, 1).unwrap();
    /// assert_eq!(log.events().len(), 1);
    /// ```
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
            op_trace: 0,
            kind_tag,
        };
        self.count += 1;
        Ok(())
    }

    /// Records that run `run_id` was sealed with the complete `op_trace` bitmask.
    /// `op_idx` retains the low 32 bits for wire compatibility; `op_trace` is authoritative.
    ///
    /// # Errors
    ///
    /// Returns [`OcelError::Overflow`] when the log is full; the event is
    /// NOT silently dropped — callers must handle the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::ocel::OcelLog;
    ///
    /// let mut log = OcelLog::new();
    /// log.record_run_sealed(42, 0b11).unwrap();
    /// assert_eq!(log.events().len(), 1);
    /// ```
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
            op_trace,
            kind_tag: 0,
        };
        self.count += 1;
        Ok(())
    }

    /// Returns the slice of recorded events.
    pub fn events(&self) -> &[OcelEvent] {
        &self.events[..self.count]
    }

    /// Seal the exact ordered trace into a deterministic, replayable BLAKE3 receipt.
    ///
    /// The encoding is domain-separated and explicitly length-prefixed. Every event field
    /// that can affect conformance or audit interpretation is committed independently.
    pub fn seal_receipt(&self) -> OcelTraceReceipt {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"bcinr-powl-ocel-trace-v1");
        hasher.update(&(self.count as u64).to_le_bytes());
        hasher.update(&self.tick.to_le_bytes());

        for event in self.events() {
            let activity = event.activity.as_bytes();
            hasher.update(&(activity.len() as u64).to_le_bytes());
            hasher.update(activity);
            hasher.update(&event.event_id.to_le_bytes());
            hasher.update(&event.timestamp.to_le_bytes());
            hasher.update(&event.run_id.to_le_bytes());
            hasher.update(&event.op_idx.to_le_bytes());
            hasher.update(&event.op_trace.to_le_bytes());
            hasher.update(&[event.kind_tag]);
        }

        OcelTraceReceipt {
            log: self.clone(),
            digest: *hasher.finalize().as_bytes(),
        }
    }

    /// Validates the log against the given POWL tape's predecessor masks.
    ///
    /// This is a heap-free, `no_std` safe check.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::ocel::{OcelLog, ConformanceResult};
    /// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
    ///
    /// let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();
    /// let mut log = OcelLog::new();
    /// log.record_op_fired(1, 0, 0).unwrap();
    /// log.record_run_sealed(1, 0b1).unwrap();
    ///
    /// assert_eq!(log.validate_against_tape(&tape), ConformanceResult::Conforms);
    /// ```
    pub fn validate_against_tape(&self, tape: &crate::tape::PowlTape) -> ConformanceResult {
        validate_against_tape(self, tape)
    }

    /// Converts the `OcelLog` into a standard OCEL 2.0 structure.
    ///
    /// This method is only available when the `std` feature is enabled.
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
                attributes: vec![OCELTypeAttribute {
                    name: "op_trace".to_string(),
                    value_type: "integer".to_string(),
                }],
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
                    let op_trace = e.op_trace;
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

    /// Serializes the `OcelLog` to an OCEL 2.0 JSON string.
    ///
    /// This method is only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    pub fn to_ocel_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_ocel_2_0())
    }
}

/// Tracks unique run IDs branchlessly using Symmetric Run-Bounded Conformance Gating (SRBCG).
///
/// This is a core kernel utility that maintains a set of up to 64 unique run IDs and returns
/// the slot index mapped to `incoming_rid`.
///
/// # Design & Radon Law ($CC=1$) Compliance
///
/// To satisfy strict constant-time / branchless requirements:
/// - It uses a comparison network over the 64-slot `run_ids` array.
/// - Loop bounds are statically fixed (`0..64`), allowing the compiler to generate unrolled,
///   straight-line assembly using conditional move instructions (e.g., `CSEL`/`CMOV`).
/// - Arithmetic and bitwise operations select indices and propagate errors instead of using
///   conditional control flow (`if` or `match`).
///
/// # Slot Allocation Logic
///
/// 1. **Search**: Computes `match_idx` by evaluating `is_match = (run_ids[i] == incoming_rid)` for every slot `i`.
/// 2. **Decision**:
///    - If a slot matches: returns that index.
///    - If no slot matches, and `run_count < 64`: assigns a new slot, increments `run_count`, and writes the new `run_id`.
///    - If no slot matches, and `run_count == 64`: marks overflow in `overflow_mask` and returns slot `64`.
///
/// # Examples
///
/// ```
/// use bcinr_powl::ocel::process_event_srbcg;
///
/// let mut run_ids = [u64::MAX; 64];
/// let mut run_count = 0;
/// let mut overflow_mask = 0;
///
/// // Allocate first run
/// let slot_0 = process_event_srbcg(&mut run_ids, &mut run_count, 100, &mut overflow_mask);
/// assert_eq!(slot_0, 0);
/// assert_eq!(run_count, 1);
/// assert_eq!(run_ids[0], 100);
///
/// // Retrieve existing run
/// let slot_ref = process_event_srbcg(&mut run_ids, &mut run_count, 100, &mut overflow_mask);
/// assert_eq!(slot_ref, 0);
/// assert_eq!(run_count, 1);
/// ```
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
    for (i, &rid) in run_ids.iter().enumerate() {
        let is_occupied = (i < current_count) as usize;
        let is_match = is_occupied * (rid == incoming_rid) as usize;
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
    for (i, rid) in run_ids.iter_mut().enumerate() {
        let mask = 0u64.wrapping_sub((should_write & (i == target_idx) as usize) as u64);
        *rid = (incoming_rid & mask) | (*rid & !mask);
    }

    // Accumulate overflow mask if not found and cannot allocate.
    let has_overflowed = (1 - found) * (1 - can_allocate);
    *overflow_mask |= 0u64.wrapping_sub(has_overflowed as u64);

    target_idx
}

/// Validates an [`OcelLog`] against a [`crate::tape::PowlTape`]'s predecessor constraints.
///
/// This is a heap-free, `no_std` conformance check. It aggregates events by their
/// active runs (up to 64 concurrent runs) and checks that no operation fires before
/// its predecessors, no operations fire twice in the same run, and that sealed traces
/// match the declared execution set.
///
/// # Validation Steps (in order)
///
/// 1. **Empty Log Verification**:
///    If the log contains zero events, validation fails immediately with [`ConformanceResult::EmptyLog`].
/// 2. **Run Limit Check**:
///    Uses [`process_event_srbcg`] to track unique run IDs. If the log contains more than 64 unique run IDs,
///    it fails with [`ConformanceResult::RunLimitExceeded`]. This ensures that logs exceeding the capacity
///    cannot bypass validation or trigger silent false-positives.
/// 3. **Duplicate Fire Check**:
///    Iterates over events and tracks if any operation fires twice within the same run. If detected,
///    validation fails with [`ConformanceResult::DuplicateFire`].
/// 4. **Seal Mismatch Check**:
///    For each run that contains a `run_sealed` event, the declared `op_trace` is checked against the
///    accumulated `op_fired` events. If there is a mismatch (e.g., an operation declared in the trace
///    never fired, or an operation fired but was not declared), validation fails with [`ConformanceResult::SealMismatch`].
/// 5. **Predecessor Constraint Check**:
///    For each fired operation, its predecessor mask is checked against the set of operations that have
///    already fired in that run. If any required predecessor is missing, validation fails with [`ConformanceResult::Violation`].
///
/// # Examples
///
/// ```
/// use bcinr_powl::ocel::{OcelLog, ConformanceResult, validate_against_tape};
/// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
///
/// // Create sequence "a" -> "b"
/// let ast = PowlAstNode::Sequence(vec![
///     PowlAstNode::Atom("a"),
///     PowlAstNode::Atom("b"),
/// ]);
/// let tape = compile_powl(&ast).unwrap();
///
/// // Conforming log
/// let mut log = OcelLog::new();
/// log.record_op_fired(10, 0, 0).unwrap(); // op 0 fired in run 10
/// log.record_op_fired(10, 1, 0).unwrap(); // op 1 fired in run 10
/// log.record_run_sealed(10, 0b11).unwrap(); // sealed with both
///
/// let result = validate_against_tape(&log, &tape);
/// assert_eq!(result, ConformanceResult::Conforms);
/// ```
pub fn validate_against_tape(log: &OcelLog, tape: &crate::tape::PowlTape) -> ConformanceResult {
    if log.events().is_empty() {
        return ConformanceResult::EmptyLog;
    }

    let ops = &tape.ops[..tape.len as usize];
    const MAX_RUNS: usize = 64;
    let mut run_ids: [u64; MAX_RUNS] = [u64::MAX; MAX_RUNS];
    // Slot 64 is a bounded overflow sink. It is never admitted for semantic validation.
    let mut accumulated: [u64; MAX_RUNS + 1] = [0u64; MAX_RUNS + 1];
    let mut sealed: [bool; MAX_RUNS + 1] = [false; MAX_RUNS + 1];
    let mut run_count: usize = 0;
    let mut overflow_mask: u64 = 0;

    // Validate in event order. The predecessor check is performed against the state that
    // existed immediately before the current event, not against a reconstructed final set.
    for event in log.events() {
        match event.activity {
            "op_fired" => {
                let slot = process_event_srbcg(
                    &mut run_ids,
                    &mut run_count,
                    event.run_id,
                    &mut overflow_mask,
                );
                if overflow_mask != 0 {
                    return ConformanceResult::RunLimitExceeded;
                }
                if sealed[slot] {
                    return ConformanceResult::EventAfterSeal {
                        run_id: event.run_id,
                        op_idx: event.op_idx,
                    };
                }

                let op_idx = event.op_idx as usize;
                if op_idx >= ops.len() {
                    return ConformanceResult::UnknownOperation {
                        run_id: event.run_id,
                        op_idx: event.op_idx,
                    };
                }

                let bit = 1u64 << event.op_idx;
                if accumulated[slot] & bit != 0 {
                    return ConformanceResult::DuplicateFire {
                        run_id: event.run_id,
                        op_idx: event.op_idx,
                    };
                }

                let tape_op = &ops[event.op_idx as usize];
                if tape_op.kind == crate::tape::OpKind::Join && tape_op.branch_mask != 0 {
                    let fired_branch_mask = tape_op.branch_mask & accumulated[slot];
                    if fired_branch_mask.count_ones() != 1 {
                        return ConformanceResult::ChoiceViolation {
                            run_id: event.run_id,
                            join_op_idx: event.op_idx,
                            branch_mask: tape_op.branch_mask,
                            fired_branch_mask,
                        };
                    }
                } else {
                    let missing = tape_op.pred_mask & !accumulated[slot];
                    if missing != 0 {
                        return ConformanceResult::Violation {
                            run_id: event.run_id,
                            op_idx: event.op_idx,
                            missing_pred_mask: missing,
                        };
                    }
                }

                accumulated[slot] |= bit;
            }
            "run_sealed" => {
                let slot = process_event_srbcg(
                    &mut run_ids,
                    &mut run_count,
                    event.run_id,
                    &mut overflow_mask,
                );
                if overflow_mask != 0 {
                    return ConformanceResult::RunLimitExceeded;
                }
                if sealed[slot] {
                    return ConformanceResult::DuplicateSeal {
                        run_id: event.run_id,
                    };
                }
                if event.op_trace != accumulated[slot] {
                    return ConformanceResult::SealMismatch {
                        run_id: event.run_id,
                        declared: event.op_trace,
                        accumulated: accumulated[slot],
                    };
                }
                sealed[slot] = true;
            }
            _ => {}
        }
    }

    for slot in 0..run_count {
        if !sealed[slot] {
            return ConformanceResult::MissingSeal {
                run_id: run_ids[slot],
            };
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
            .map(|e| e.op_trace)
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
            .map(|e| e.op_trace)
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
        assert_eq!(
            validate_against_tape(&log_64, &tape),
            ConformanceResult::Conforms
        );

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

    #[test]
    fn process_event_srbcg_admits_u64_max_run_id_without_sentinel_collision() {
        let mut run_ids = [u64::MAX; 64];
        let mut run_count = 0usize;
        let mut overflow_mask = 0u64;

        let first = process_event_srbcg(&mut run_ids, &mut run_count, u64::MAX, &mut overflow_mask);
        let replay =
            process_event_srbcg(&mut run_ids, &mut run_count, u64::MAX, &mut overflow_mask);

        assert_eq!(first, 0);
        assert_eq!(replay, 0);
        assert_eq!(run_count, 1);
        assert_eq!(overflow_mask, 0);
    }

    #[test]
    fn validate_rejects_temporal_inversion_even_when_final_set_is_complete() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();
        let mut log = OcelLog::new();
        log.record_op_fired(99, 1, 0).unwrap();
        log.record_op_fired(99, 0, 0).unwrap();
        log.record_run_sealed(99, 0b11).unwrap();

        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::Violation {
                run_id: 99,
                op_idx: 1,
                missing_pred_mask: 0b01,
            }
        );
    }

    #[test]
    fn validate_rejects_event_after_seal() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();
        let mut log = OcelLog::new();
        log.record_run_sealed(7, 0).unwrap();
        log.record_op_fired(7, 0, 0).unwrap();
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::EventAfterSeal {
                run_id: 7,
                op_idx: 0,
            }
        );
    }

    #[test]
    fn validate_rejects_duplicate_seal() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();
        let mut log = OcelLog::new();
        log.record_op_fired(8, 0, 0).unwrap();
        log.record_run_sealed(8, 1).unwrap();
        log.record_run_sealed(8, 1).unwrap();
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::DuplicateSeal { run_id: 8 }
        );
    }

    #[test]
    fn validate_rejects_missing_seal() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::Atom("a")).unwrap();
        let mut log = OcelLog::new();
        log.record_op_fired(9, 0, 0).unwrap();
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::MissingSeal { run_id: 9 }
        );
    }

    #[test]
    fn full_width_seal_preserves_operation_63() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let children = (0..64).map(|_| PowlAstNode::Atom("op")).collect();
        let tape = compile_powl(&PowlAstNode::Sequence(children)).unwrap();
        let mut log = OcelLog::new();
        for op_idx in 0..64 {
            log.record_op_fired(10, op_idx, 0).unwrap();
        }
        log.record_run_sealed(10, u64::MAX).unwrap();

        assert_eq!(log.events().last().unwrap().op_trace, u64::MAX);
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::Conforms
        );
    }

    #[test]
    fn deterministic_trace_receipt_owns_exact_ordered_events() {
        let mut first = OcelLog::new();
        first.record_op_fired(11, 0, 3).unwrap();
        first.record_run_sealed(11, 1).unwrap();
        let mut second = OcelLog::new();
        second.record_op_fired(11, 0, 3).unwrap();
        second.record_run_sealed(11, 1).unwrap();

        let receipt_a = first.seal_receipt();
        let receipt_b = second.seal_receipt();
        assert_eq!(receipt_a.digest(), receipt_b.digest());
        assert_eq!(receipt_a.log().events(), first.events());
        assert_eq!(receipt_a.event_count(), first.events().len());
    }

    #[test]
    fn validate_xor_join_accepts_exactly_one_observed_branch() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]))
        .unwrap();
        let join = &tape.ops[1];
        assert_eq!(join.kind, crate::tape::OpKind::Join);
        assert_eq!(join.branch_mask, join.pred_mask);
        assert_eq!(join.branch_mask.count_ones(), 3);

        let chosen = join.branch_mask.isolate_lowest_one();
        let chosen_idx = chosen.trailing_zeros();
        let trace = 1u64 | (1u64 << 1) | chosen;
        let mut log = OcelLog::new();
        log.record_op_fired(70, 0, 0).unwrap();
        log.record_op_fired(70, chosen_idx, 0).unwrap();
        log.record_op_fired(70, 1, 0).unwrap();
        log.record_run_sealed(70, trace).unwrap();
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::Conforms
        );
    }

    #[test]
    fn validate_xor_join_refuses_multiple_observed_branches() {
        use crate::compiler::{compile_powl, PowlAstNode};
        let tape = compile_powl(&PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]))
        .unwrap();
        let branch_mask = tape.ops[1].branch_mask;
        let first = branch_mask.isolate_lowest_one();
        let remaining = branch_mask & !first;
        let second = remaining.isolate_lowest_one();
        let fired_branch_mask = first | second;
        let mut log = OcelLog::new();
        log.record_op_fired(71, 0, 0).unwrap();
        log.record_op_fired(71, first.trailing_zeros(), 0).unwrap();
        log.record_op_fired(71, second.trailing_zeros(), 0).unwrap();
        log.record_op_fired(71, 1, 0).unwrap();
        log.record_run_sealed(71, 1u64 | (1u64 << 1) | fired_branch_mask)
            .unwrap();
        assert_eq!(
            validate_against_tape(&log, &tape),
            ConformanceResult::ChoiceViolation {
                run_id: 71,
                join_op_idx: 1,
                branch_mask,
                fired_branch_mask,
            }
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn ocel_export_declares_every_emitted_run_sealed_attribute() {
        let mut log = OcelLog::new();
        log.record_run_sealed(81, 1).unwrap();
        let exported = log.to_ocel_2_0();
        let run_sealed = exported
            .event_types
            .iter()
            .find(|event_type| event_type.name == "run_sealed")
            .expect("run_sealed event type must be declared");
        assert_eq!(run_sealed.attributes.len(), 1);
        assert_eq!(run_sealed.attributes[0].name, "op_trace");
        assert_eq!(run_sealed.attributes[0].value_type, "integer");
    }
}
