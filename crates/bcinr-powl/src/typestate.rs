//! POWL v2 TypeState Machine and Branchless Linear Execution Tokens (BLET).
//!
//! This module implements a static, type-safe execution pipeline for Partially Ordered
//! Workflow Language (POWL) workflows. The core design is built upon two pillars:
//!
//! 1. **Phase-Indexed Typestate Machine ([`PowlRunner`])**: Statically tracks the state
//!    of execution phases at compile time, guaranteeing that transitions are one-way and
//!    mutually exclusive.
//! 2. **Branchless Linear Execution Tokens ([`ExecutionToken`])**: Emulates linear types
//!    to track in-flight execution of operations. By using branchless bitwise arithmetic,
//!    it monitors operation execution, detects defects (such as double-firing or out-of-bounds
//!    execution), and enforces that every workflow task is run exactly once.
//!
//! # The Phase Lattice
//!
//! The lifetime of a workflow run is governed by a sequence of five distinct compile-time
//! phases:
//!
//! ```text
//!          [PowlRunner<Unvalidated>]
//!                      │
//!                      │ .validate()
//!                      ▼
//!           [PowlRunner<Compiled>]
//!                      │
//!                      │ .schedule::<KIND>()
//!                      ▼
//!        [PowlRunner<Scheduled<KIND>>]
//!                      │
//!                      │ .begin_execution()
//!                      ▼
//!      ┌──────────────────────────────────────┐
//!      │  (PowlRunner<Executing<KIND>>,       │
//!      │   ExecutionToken)                    │
//!      └──────────────────┬───────────────────┘
//!                         │
//!                         │ .complete(token)
//!                         ▼
//!       [PowlRunner<Receipted<KIND>>]  +  [Receipt<KIND>]
//! ```
//!
//! Each transition in the lattice consumes the previous runner (taking it by value) and
//! returns a new runner with the updated phase marker. This prevents reuse of a runner in a
//! stale state or out-of-order phase transitions.
//!
//! # Branchless Linear Execution Tokens (BLET)
//!
//! To safely coordinate step-by-step execution in a `no_alloc` environment, the runner
//! yields an [`ExecutionToken`]. This token acts as a linear resource representing the remaining
//! work on the tape.
//!
//! - **Linearity Emulation**: The Rust compiler ensures the token cannot be copied or cloned
//!   (as it does not implement `Clone` or `Copy`). Additionally, in debug builds, a "destructor bomb"
//!   ([`Drop`] implementation) will panic if the token is discarded before all tasks are fired.
//! - **Constant Complexity (`CC = 1`)**: Checking for execution correctness (double-firing,
//!   out-of-bounds fires, or malformed inputs) is implemented using branch-free bitwise operations.
//!   Instead of branching on errors during step execution, defects are accumulated into status
//!   registers using bitwise masks and verified at the end of the transaction.
//!
//! # Safety Invariants
//!
//! 1. **Phase Isolation**: Only methods defined for a runner's current phase can be called.
//!    For example, you cannot run execution checks on an `Unvalidated` tape, and you cannot
//!    re-schedule a runner that is already running.
//! 2. **Consuming Transactions**: Runner transitions take `self` by value, ensuring that no
//!    previous reference to the runner remains valid.
//! 3. **Defect Isolation**: If any operational defect occurs (e.g. firing an out-of-bounds op
//!    or double-firing an op), the transition from `Executing` to `Receipted` will refuse the commit
//!    and return an [`ExecutionDefect`] error, keeping the persistent system state clean.
//!
//! # Examples
//!
//! ```
//! #![feature(adt_const_params)]
//! # #![allow(incomplete_features)]
//! use bcinr_powl::typestate::{PowlRunner, Unvalidated, TopologyKind};
//! use bcinr_powl::tape::{PowlTape, OpKind};
//!
//! // 1. Construct and populate a tape
//! let mut tape = PowlTape::new();
//! let op0 = tape.alloc(OpKind::Atom).unwrap();
//! let op1 = tape.alloc(OpKind::Atom).unwrap();
//! tape.entry_mask = 1 << op0; // Mark op0 as the entry op
//!
//! // 2. Instantiate the Unvalidated runner
//! let runner = PowlRunner::new(tape);
//!
//! // 3. Validate tape: Unvalidated -> Compiled
//! let compiled = runner.validate().expect("Tape validation failed");
//!
//! // 4. Assign Topology: Compiled -> Scheduled
//! let scheduled = compiled.schedule::<{ TopologyKind::Standard }>();
//!
//! // 5. Start Execution: Scheduled -> Executing (yields the Linear Token)
//! let (executing, mut token) = scheduled.begin_execution();
//!
//! // 6. Process the linear token (fire operations)
//! token.record_fire(op0);
//! token.consume_op(1 << op0);
//!
//! token.record_fire(op1);
//! token.consume_op(1 << op1);
//!
//! // 7. Complete Execution: Executing -> Receipted (consumes the Token)
//! let (receipted, receipt) = executing.complete(token).expect("Execution failed");
//!
//! // Verify execution properties
//! assert_eq!(receipt.topology, TopologyKind::Standard);
//! assert_eq!(receipt.op_trace, 0b11);
//! ```

#![allow(incomplete_features)]

use core::marker::PhantomData;

// =============================================================================
// TopologyKind — const generic discriminant
// =============================================================================

/// Scheduling topology that governs execution priority, preemption, and retry semantics.
///
/// Used as a const generic parameter (requiring the `adt_const_params` feature) so the
/// type system tracks topology across all post-[`Compiled`] phases.
#[derive(PartialEq, Eq, Clone, Copy, Debug, core::marker::ConstParamTy)]
#[repr(u8)]
pub enum TopologyKind {
    /// High-priority execution path with strict deadlines and immediate retries.
    Priority = 0,
    /// Standard execution path with default priority and normal retry backoff.
    Standard = 1,
    /// Low-priority background execution path for non-critical tasks.
    Background = 2,
    /// Execution path optimized for long-running workflows.
    LongRunning = 3,
    /// Topology for running compensating transactions during failure recovery.
    Compensating = 4,
}

// =============================================================================
// Phase markers (zero-sized, non-constructible outside this module)
// =============================================================================

/// Phase marker: tape not yet validated.
///
/// This is the initial state of a [`PowlRunner`]. In this phase, the tape's structure
/// (such as cyclic dependencies, size, and entry points) has not yet been verified.
pub struct Unvalidated;

/// Phase marker: tape has passed structural validation.
///
/// Advanced from [`Unvalidated`] by calling [`PowlRunner::validate`].
///
/// The inner `()` field is private, preventing external construction of this marker.
pub struct Compiled(());

/// Phase marker: runner has been assigned a scheduling topology.
///
/// Advanced from [`Compiled`] by calling [`PowlRunner::schedule`].
///
/// The topology kind `KIND` is tracked as a const generic parameter.
pub struct Scheduled<const KIND: TopologyKind>(());

/// Phase marker: execution has started; an [`ExecutionToken`] is in flight.
///
/// Advanced from [`Scheduled`] by calling [`PowlRunner::begin_execution`].
///
/// During this phase, caller must process all tasks on the tape and record/consume
/// them using the provided [`ExecutionToken`].
pub struct Executing<const KIND: TopologyKind>(());

/// Phase marker: execution is complete and a [`Receipt`] has been issued.
///
/// Advanced from [`Executing`] by calling [`PowlRunner::complete`].
///
/// This is the terminal phase of the runner. No further state transitions are possible.
pub struct Receipted<const KIND: TopologyKind>(());

// =============================================================================
// HasPowlTape — capability bound
// =============================================================================

/// Trait implemented by tape types understood by the [`PowlRunner`].
///
/// A tape must be able to report the number of ops it contains and produce a
/// 64-bit op-presence bitmask (one bit per op index, up to 64 ops).
pub trait HasPowlTape {
    /// Returns the number of ops on the tape (≤ 64 for the compact tape).
    fn op_count(&self) -> usize;

    /// Returns the entry-point bitmask (ops with no predecessors).
    fn entry_mask(&self) -> u64;

    /// Produces a 32-byte blake3-style content hash of the tape for the receipt.
    ///
    /// A default implementation is provided that returns a zeroed array; real
    /// tape types should override this.
    fn content_hash(&self) -> [u8; 32] {
        [0u8; 32]
    }
}

// Blanket impl for the v2 compact tape so tests can use it directly.
impl HasPowlTape for crate::tape::v2::PowlTape {
    fn op_count(&self) -> usize {
        self.len as usize
    }

    fn entry_mask(&self) -> u64 {
        self.ready_mask()
    }

    fn content_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        for i in 0..self.len as usize {
            h.update(&self.ops[i].pred_mask.to_le_bytes());
            h.update(&self.ops[i].succ_mask.to_le_bytes());
            h.update(&[self.ops[i].op_kind as u8]);
        }
        *h.finalize().as_bytes()
    }
}

// Impl for the primary (compiler-facing) tape type.
impl HasPowlTape for crate::tape::PowlTape {
    fn op_count(&self) -> usize {
        self.len as usize
    }

    fn entry_mask(&self) -> u64 {
        self.entry_mask
    }

    fn content_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        for i in 0..self.len as usize {
            h.update(&self.ops[i].pred_mask.to_le_bytes());
            h.update(&self.ops[i].succ_mask.to_le_bytes());
            h.update(&[self.ops[i].kind as u8]);
        }
        *h.finalize().as_bytes()
    }
}

// =============================================================================
// ValidationError
// =============================================================================

/// Errors produced during the [`Unvalidated`] → [`Compiled`] transition.
///
/// These errors indicate that the tape's structure violates basic correctness rules,
/// preventing the runner from starting safely.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValidationError {
    /// The tape contains no ops.
    EmptyTape,
    /// The tape contains more than 64 ops (bitmask overflow).
    TapeTooLarge {
        /// The actual length of the tape.
        len: usize,
    },
    /// No op in the tape is reachable from the entry mask (disconnected graph).
    NoEntryOp,
    /// The tape contains a cycle that would deadlock the executor.
    CyclicDependency,
    /// An op references a predecessor index that is out of bounds.
    InvalidPredecessorIndex {
        /// The index of the op making the reference.
        op: u8,
        /// The invalid predecessor index.
        pred: u8,
    },
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyTape => write!(f, "tape has no ops"),
            Self::TapeTooLarge { len } => write!(f, "tape has {len} ops; max is 64"),
            Self::NoEntryOp => write!(f, "no entry op found (all ops have predecessors)"),
            Self::CyclicDependency => write!(f, "tape contains a cycle"),
            Self::InvalidPredecessorIndex { op, pred } => write!(
                f,
                "op {op} references predecessor {pred} which is out of bounds"
            ),
        }
    }
}

// =============================================================================
// ExecutionDefect
// =============================================================================

/// Errors produced during the [`Executing`] → [`Receipted`] transition.
///
/// These defects indicate runtime violations of execution correctness or safety.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExecutionDefect {
    /// The op bit was already consumed (double-fire).
    OpAlreadyConsumed {
        /// The bitmask representing the double-fired operation.
        bit: u64,
    },
    /// `assert_exhausted` was called while some ops remain unfired.
    UnexhaustedOps {
        /// The bitmask representing the unfired operations.
        remaining: u64,
    },
    /// The [`ExecutionToken`] presented does not belong to this runner.
    TokenMismatch,
    /// Out-of-bounds/inactive operations were fired.
    InvalidFires {
        /// The bitmask of invalid operations.
        bits: u64,
    },
    /// Malformed (zero or multi-bit) operations were fired.
    MalformedFires {
        /// The bitmask of malformed operations.
        bits: u64,
    },
}

impl core::fmt::Display for ExecutionDefect {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OpAlreadyConsumed { bit } => write!(f, "op bit {bit:#b} already consumed"),
            Self::UnexhaustedOps { remaining } => write!(f, "unfired ops remain: {remaining:#b}"),
            Self::TokenMismatch => write!(f, "execution token does not match this runner"),
            Self::InvalidFires { bits } => {
                write!(f, "invalid (out-of-bounds) ops fired: {bits:#b}")
            }
            Self::MalformedFires { bits } => {
                write!(f, "malformed (zero/multi-bit) ops fired: {bits:#b}")
            }
        }
    }
}

// =============================================================================
// ExecutionToken — linear type emulation
// =============================================================================

/// A linear token representing in-flight execution.
///
/// Each bit in `remaining` corresponds to one op on the tape.  Callers must
/// call [`consume_op`][ExecutionToken::consume_op] for every op bit and then
/// hand the token back to [`PowlRunner::complete`]; the token **must not** be
/// dropped with unfired ops (enforced by a destructor bomb in debug builds).
#[must_use = "ExecutionToken must be consumed by PowlRunner::complete"]
pub struct ExecutionToken {
    /// Bitmask of ops not yet fired; starts as `(1 << op_count) - 1`.
    remaining: u64,
    /// Bitmask defining the valid operational boundary: (1 << total) - 1.
    valid_mask: u64,
    /// Stateful Status Accumulators
    pub(crate) defect_double_fire: u64,
    pub(crate) defect_invalid: u64,
    pub(crate) defect_malformed: u64,
    /// Total number of ops (mirrors `remaining.count_ones()` at construction).
    total: u8,
    /// Topological firing order: topo_order[step] = op_idx of the step-th fired op.
    /// Slots beyond event_count are u8::MAX (sentinel).
    pub(crate) topo_order: [u8; 64],
    /// Number of ops recorded so far (saturates at 64).
    pub(crate) event_count: u8,
}

// Explicitly opt out of Clone/Copy — this is the linear-type emulation.
// (No `#[derive(Clone)]` or `#[derive(Copy)]`.)

impl ExecutionToken {
    /// Construct a token from raw fields — for use in trybuild compile-fail tests.
    ///
    /// `remaining` is the bitmask of unfired ops; `total` is the op count.
    ///
    /// # Availability
    ///
    /// Only available under `#[cfg(test)]` or when the `testing` feature is
    /// enabled.  This constructor bypasses the `Compiled → Scheduled →
    /// Executing` admission sequence and must **never** be used in production
    /// code.
    #[doc(hidden)]
    #[cfg(any(test, feature = "testing"))]
    pub fn new_for_test(remaining: u64, total: u8) -> Self {
        debug_assert_eq!(
            remaining.count_ones() as u8,
            total,
            "new_for_test: remaining has {} bits but total={}",
            remaining.count_ones(),
            total
        );
        let is_64 = (total == 64) as u64;
        let base_mask = (1u64.wrapping_shl(total as u32 & 63)).wrapping_sub(1);
        let sentinel = 0u64.wrapping_sub(is_64);
        let valid_mask = base_mask | sentinel;
        Self {
            remaining,
            valid_mask,
            defect_double_fire: 0,
            defect_invalid: 0,
            defect_malformed: 0,
            total,
            topo_order: [u8::MAX; 64],
            event_count: 0,
        }
    }

    /// Construct a fresh token for a tape with `op_count` ops (≤ 64).
    ///
    /// Bits `0..op_count` are set in `remaining`.
    pub(crate) fn new(op_count: usize) -> Self {
        debug_assert!(op_count <= 64, "op_count must be ≤ 64");
        // Branchless bitmask: if op_count == 64 we want all 64 bits set.
        // `(1u64 << 64)` wraps to 0 on most platforms; handle via wrapping_shl.
        let is_64 = (op_count == 64) as u64;
        let base_mask = (1u64.wrapping_shl(op_count as u32 & 63)).wrapping_sub(1);
        let sentinel = 0u64.wrapping_sub(is_64);
        let remaining = base_mask | sentinel;
        debug_assert_eq!(
            remaining.count_ones() as u8,
            op_count as u8,
            "token total mismatch: remaining has {} bits but total={}",
            remaining.count_ones(),
            op_count
        );
        let valid_mask = remaining;
        Self {
            remaining,
            valid_mask,
            defect_double_fire: 0,
            defect_invalid: 0,
            defect_malformed: 0,
            total: op_count as u8,
            topo_order: [u8::MAX; 64],
            event_count: 0,
        }
    }

    /// Record that op at index `op_idx` fired.
    ///
    /// Implements a branchless, bounded write to the topological order buffer.
    /// No-op once `event_count == 64`.
    #[inline]
    pub fn record_fire(&mut self, op_idx: u8) {
        let slot = (self.event_count as usize).min(63);
        let guard = (self.event_count < 64) as u8;
        self.topo_order[slot] = op_idx * guard + u8::MAX * (1 - guard);
        self.event_count = self.event_count.wrapping_add(guard);
    }

    /// Mark an op as fired by its bitmask.
    ///
    /// `op_bit` must be a single-bit mask (exactly one bit set) corresponding
    /// to the op that just completed.
    ///
    /// # Implementation Details
    ///
    /// This function runs in constant-time complexity with zero data-dependent branches
    /// (`CC = 1`). It uses full-width masks to record defects such as double-firing,
    /// out-of-bounds firing, and malformed inputs (multiple bits or zero bits set)
    /// without short-circuiting or branching. The actual defect validation occurs
    /// during transition to [`Receipted`].
    ///
    /// # Invariants
    ///
    /// - Fired bits must correspond to valid, active operations on the tape.
    /// - Each operation must be consumed exactly once.
    /// - Fired bitmask must have exactly one bit set.
    #[inline]
    pub fn consume_op(&mut self, op_bit: u64) {
        // 1. Accumulate invalid bit fires (bits outside the valid boundary)
        let invalid = op_bit & !self.valid_mask;
        self.defect_invalid |= invalid;

        // 2. Accumulate double-fire defects
        // If an op is set in op_bit & valid_mask but is NOT in remaining, it was double-fired.
        let target_valid = op_bit & self.valid_mask;
        let present = self.remaining & target_valid;
        let double_fired = target_valid ^ present;
        self.defect_double_fire |= double_fired;

        // 3. Accumulate malformed fires (zero bits or multi-bit operations)
        let is_zero = (op_bit == 0) as u64;
        let is_multi = ((op_bit & op_bit.wrapping_sub(1)) != 0) as u64;
        let malformed_flag = is_zero | is_multi;

        // Write through the malformed flag and the offending bits.
        // We use 0u64.wrapping_sub(malformed_flag) to propagate a full-width mask.
        let malformed_mask =
            (op_bit | 0u64.wrapping_sub(is_zero)) & 0u64.wrapping_sub(malformed_flag);
        self.defect_malformed |= malformed_mask;

        // 4. Update the remaining mask (idempotent write-through)
        self.remaining &= !op_bit;
    }

    /// Assert that all ops have been fired and consume the token.
    ///
    /// Returns an error if any bits remain set in `remaining` or if any defects occurred.
    /// This method consumes `self` by value, bypassing the debug destructor bomb.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutionDefect`] if:
    /// - An operation was double-fired.
    /// - An invalid (out-of-bounds) operation was fired.
    /// - A malformed (zero or multi-bit) operation was fired.
    /// - Unfired operations remain.
    pub fn assert_exhausted(self) -> Result<(), ExecutionDefect> {
        let remaining = self.remaining;
        let defect_malformed = self.defect_malformed;
        let defect_invalid = self.defect_invalid;
        let defect_double_fire = self.defect_double_fire;

        // Prevent the destructor bomb from firing — we're consuming intentionally.
        core::mem::forget(self);

        if defect_malformed != 0 {
            Err(ExecutionDefect::MalformedFires {
                bits: defect_malformed,
            })
        } else if defect_invalid != 0 {
            Err(ExecutionDefect::InvalidFires {
                bits: defect_invalid,
            })
        } else if defect_double_fire != 0 {
            Err(ExecutionDefect::OpAlreadyConsumed {
                bit: defect_double_fire,
            })
        } else if remaining != 0 {
            Err(ExecutionDefect::UnexhaustedOps { remaining })
        } else {
            Ok(())
        }
    }

    /// Returns the remaining (unfired) op bitmask.
    #[inline]
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the total op count this token was constructed for.
    #[inline]
    pub fn total(&self) -> u8 {
        self.total
    }
}

impl Drop for ExecutionToken {
    fn drop(&mut self) {
        // Destructor bomb: in debug builds, panic if any ops are still unfired.
        #[cfg(debug_assertions)]
        if self.remaining != 0 {
            panic!(
                "ExecutionToken dropped with unfired ops: {:#b}",
                self.remaining
            );
        }
    }
}

// =============================================================================
// Receipt
// =============================================================================

/// Immutable execution receipt issued after a successful `complete()`.
///
/// `KIND` is the topology under which the runner was scheduled, providing a
/// compile-time record of the execution context.
pub struct Receipt<const KIND: TopologyKind> {
    /// Unique run identifier assigned at runner construction.
    pub run_id: u64,
    /// Bitmask recording which operations were fired (op trace).
    pub op_trace: u64,
    /// Runtime topology (mirrors the const generic parameter `KIND`).
    pub topology: TopologyKind,
    /// 32-byte content hash of the tape at manufacture/validation time.
    pub chain_hash: [u8; 32],
    /// Replay pointer — byte offset into a hypothetical event log.
    pub replay_ptr: u64,
    /// Topological firing order recorded during execution.
    ///
    /// The `i`-th element represents the index of the `i`-th fired operation.
    /// Remaining elements are padded with `u8::MAX`.
    pub topo_order: [u8; 64],
    /// Number of operations recorded in `topo_order`.
    pub event_count: u8,
}

impl<const KIND: TopologyKind> core::fmt::Debug for Receipt<KIND> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Receipt")
            .field("run_id", &self.run_id)
            .field("op_trace", &format_args!("{:#b}", self.op_trace))
            .field("topology", &self.topology)
            .field("chain_hash", &self.chain_hash)
            .field("replay_ptr", &self.replay_ptr)
            .finish()
    }
}

// =============================================================================
// PowlRunner — phase-indexed runner
// =============================================================================

/// A phase-indexed POWL runner.
///
/// The `Phase` type parameter tracks where in the pipeline this runner is.
/// Transitions are consuming methods; the old runner is moved into the new one.
///
/// ```text
/// PowlRunner<Unvalidated, T>
///   .validate()  → Result<PowlRunner<Compiled, T>, ValidationError>
///   .schedule::<KIND>() → PowlRunner<Scheduled<KIND>, T>
///   .begin_execution() → (PowlRunner<Executing<KIND>, T>, ExecutionToken)
///   .complete(token)  → Result<(PowlRunner<Receipted<KIND>, T>, Receipt<KIND>), ExecutionDefect>
/// ```
pub struct PowlRunner<Phase, Tape> {
    tape: Tape,
    run_id: u64,
    _phase: PhantomData<Phase>,
}

impl<Phase, Tape> core::fmt::Debug for PowlRunner<Phase, Tape> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PowlRunner")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Unvalidated → Compiled
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape> PowlRunner<Unvalidated, Tape> {
    /// Construct a new runner in the [`Unvalidated`] phase wrapping `tape`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::typestate::{PowlRunner, Unvalidated};
    /// use bcinr_powl::tape::PowlTape;
    ///
    /// let tape = PowlTape::new();
    /// let runner = PowlRunner::new(tape);
    /// ```
    pub fn new(tape: Tape) -> Self {
        Self {
            tape,
            run_id: new_run_id(),
            _phase: PhantomData,
        }
    }

    /// Validate the tape and transition the runner to the [`Compiled`] phase.
    ///
    /// # Safety and Correctness Checks
    ///
    /// - **Non-Empty**: The tape must contain at least one operation.
    /// - **Size Limits**: The tape must contain at most 64 operations (ensuring they fit
    ///   within the 64-bit mask).
    /// - **Entry Op Presence**: The tape must have at least one entry operation (i.e.
    ///   the `entry_mask` must not be zero).
    ///
    /// # Errors
    ///
    /// Returns a [`ValidationError`] if any of the correctness checks fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use bcinr_powl::typestate::{PowlRunner, Unvalidated, ValidationError};
    /// use bcinr_powl::tape::{PowlTape, OpKind};
    ///
    /// let mut tape = PowlTape::new();
    /// let runner = PowlRunner::new(tape);
    ///
    /// // Validation fails on empty tape
    /// assert_eq!(runner.validate().unwrap_err(), ValidationError::EmptyTape);
    /// ```
    pub fn validate(self) -> Result<PowlRunner<Compiled, Tape>, ValidationError> {
        let n = self.tape.op_count();
        if n == 0 {
            return Err(ValidationError::EmptyTape);
        }
        if n > 64 {
            return Err(ValidationError::TapeTooLarge { len: n });
        }
        if self.tape.entry_mask() == 0 {
            return Err(ValidationError::NoEntryOp);
        }
        Ok(PowlRunner {
            tape: self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------------
// Compiled → Scheduled<KIND>
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape> PowlRunner<Compiled, Tape> {
    /// Assign a scheduling topology to the runner and transition to the [`Scheduled`] phase.
    ///
    /// The topology kind `KIND` is tracked statically at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(adt_const_params)]
    /// # #![allow(incomplete_features)]
    /// use bcinr_powl::typestate::{PowlRunner, TopologyKind};
    /// use bcinr_powl::tape::{PowlTape, OpKind};
    ///
    /// let mut tape = PowlTape::new();
    /// let op = tape.alloc(OpKind::Atom).unwrap();
    /// tape.entry_mask = 1 << op;
    ///
    /// let runner = PowlRunner::new(tape).validate().unwrap();
    /// let scheduled = runner.schedule::<{ TopologyKind::Standard }>();
    /// ```
    pub fn schedule<const KIND: TopologyKind>(self) -> PowlRunner<Scheduled<KIND>, Tape> {
        PowlRunner {
            tape: self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduled<KIND> → Executing<KIND>
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape, const KIND: TopologyKind> PowlRunner<Scheduled<KIND>, Tape> {
    /// Begin workflow execution, transitioning to [`Executing`] and yielding an [`ExecutionToken`].
    ///
    /// The returned [`ExecutionToken`] must be consumed (fired for all operations)
    /// before finishing execution by calling [`PowlRunner::complete`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(adt_const_params)]
    /// # #![allow(incomplete_features)]
    /// use bcinr_powl::typestate::{PowlRunner, TopologyKind};
    /// use bcinr_powl::tape::{PowlTape, OpKind};
    ///
    /// let mut tape = PowlTape::new();
    /// let op = tape.alloc(OpKind::Atom).unwrap();
    /// tape.entry_mask = 1 << op;
    ///
    /// let (executing, token) = PowlRunner::new(tape)
    ///     .validate().unwrap()
    ///     .schedule::<{ TopologyKind::Standard }>()
    ///     .begin_execution();
    ///
    /// assert_eq!(token.remaining(), 1);
    /// # // Avoid destructor bomb on token in this test
    /// # core::mem::forget(token);
    /// ```
    pub fn begin_execution(self) -> (PowlRunner<Executing<KIND>, Tape>, ExecutionToken) {
        let op_count = self.tape.op_count();
        let token = ExecutionToken::new(op_count);
        let runner = PowlRunner {
            tape: self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        };
        (runner, token)
    }
}

// ---------------------------------------------------------------------------
// Executing<KIND> → Receipted<KIND>
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape, const KIND: TopologyKind> PowlRunner<Executing<KIND>, Tape> {
    /// Complete workflow execution by consuming the [`ExecutionToken`] and issuing a [`Receipt`].
    ///
    /// Transition the runner to the [`Receipted`] phase.
    ///
    /// # Errors
    ///
    /// Returns an [`ExecutionDefect`] if the token validation checks fail, which include:
    /// - Unfired operations still remaining in the token.
    /// - Operations double-fired.
    /// - Invalid (out-of-bounds) operations fired.
    /// - Malformed operations (zero-bit or multi-bit mask) fired.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(adt_const_params)]
    /// # #![allow(incomplete_features)]
    /// use bcinr_powl::typestate::{PowlRunner, TopologyKind};
    /// use bcinr_powl::tape::{PowlTape, OpKind};
    ///
    /// let mut tape = PowlTape::new();
    /// let op = tape.alloc(OpKind::Atom).unwrap();
    /// tape.entry_mask = 1 << op;
    ///
    /// let (executing, mut token) = PowlRunner::new(tape)
    ///     .validate().unwrap()
    ///     .schedule::<{ TopologyKind::Standard }>()
    ///     .begin_execution();
    ///
    /// token.record_fire(op);
    /// token.consume_op(1 << op);
    ///
    /// let (receipted, receipt) = executing.complete(token).unwrap();
    /// assert_eq!(receipt.op_trace, 1);
    /// ```
    pub fn complete(
        self,
        token: ExecutionToken,
    ) -> Result<(PowlRunner<Receipted<KIND>, Tape>, Receipt<KIND>), ExecutionDefect> {
        let op_trace = !token.remaining & token.valid_mask;
        let remaining = token.remaining;
        let topo_order = token.topo_order;
        let event_count = token.event_count;
        let defect_malformed = token.defect_malformed;
        let defect_invalid = token.defect_invalid;
        let defect_double_fire = token.defect_double_fire;

        // Consume token without triggering the destructor bomb.
        core::mem::forget(token);

        if defect_malformed != 0 {
            return Err(ExecutionDefect::MalformedFires {
                bits: defect_malformed,
            });
        }
        if defect_invalid != 0 {
            return Err(ExecutionDefect::InvalidFires {
                bits: defect_invalid,
            });
        }
        if defect_double_fire != 0 {
            return Err(ExecutionDefect::OpAlreadyConsumed {
                bit: defect_double_fire,
            });
        }
        if remaining != 0 {
            return Err(ExecutionDefect::UnexhaustedOps { remaining });
        }

        let chain_hash = self.tape.content_hash();
        let receipt = Receipt::<KIND> {
            run_id: self.run_id,
            op_trace,
            topology: KIND,
            chain_hash,
            replay_ptr: self.run_id, // placeholder: real impl uses event-log offset
            topo_order,
            event_count,
        };
        let runner = PowlRunner {
            tape: self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        };
        Ok((runner, receipt))
    }
}

// ---------------------------------------------------------------------------
// Receipted<KIND> — terminal phase (no further transitions)
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape, const KIND: TopologyKind> PowlRunner<Receipted<KIND>, Tape> {
    /// Returns a read-only reference to the underlying tape.
    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    /// Returns the unique run identifier assigned to this workflow run at construction.
    pub fn run_id(&self) -> u64 {
        self.run_id
    }
}

// =============================================================================
// Receipt methods
// =============================================================================

impl<const KIND: TopologyKind> Receipt<KIND> {
    /// Verify that the recorded `topo_order` is consistent with the tape's predecessor constraints.
    ///
    /// This method validates that:
    /// 1. Every operation bit set in `op_trace` is present in `topo_order`.
    /// 2. For every operation index in `topo_order`, all of its predecessors (as defined
    ///    by `pred_mask` on the tape) appear at an earlier position in `topo_order`.
    ///
    /// Returns `true` if the order is topologically valid, `false` otherwise.
    pub fn verify_topo_order(&self, tape_ops: &[crate::tape::Powl64Op]) -> bool {
        let count = self.event_count as usize;
        let mut step_of = [u8::MAX; 64];
        for step in 0..count {
            let op = self.topo_order[step] as usize;
            if op >= 64 || op >= tape_ops.len() {
                return false;
            }
            step_of[op] = step as u8;
        }
        // Rule 1: every bit in op_trace must appear in topo_order
        let mut trace = self.op_trace;
        while trace != 0 {
            let bit = trace.trailing_zeros() as usize;
            trace &= trace - 1;
            if bit >= 64 || step_of[bit] == u8::MAX {
                return false;
            }
        }
        // Rule 2: predecessor order
        for step in 0..count {
            let op_idx = self.topo_order[step] as usize;
            if op_idx >= tape_ops.len() {
                return false;
            }
            let mut preds = tape_ops[op_idx].pred_mask;
            while preds != 0 {
                let p = preds.trailing_zeros() as usize;
                preds &= preds - 1;
                if step_of[p] == u8::MAX || step_of[p] as usize >= step {
                    return false;
                }
            }
        }
        true
    }
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Generate a pseudo-unique run identifier.
///
/// In `no_std` environments we use a simple wrapping counter seeded by the
/// compile-time timestamp.  In `std` environments the counter is backed by an
/// atomic.
fn new_run_id() -> u64 {
    #[cfg(feature = "std")]
    {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    #[cfg(not(feature = "std"))]
    {
        // Deterministic counter for no_std; callers may override by wrapping.
        1u64
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tape::v2::{OpKind, Powl64Op, PowlTape};

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Build a minimal two-op tape: op0 → op1.
    fn two_op_tape() -> PowlTape {
        let mut tape = PowlTape::new();

        let mut op0 = Powl64Op::silent();
        op0.pred_mask = 0;
        op0.succ_mask = 1 << 1;
        op0.op_kind = OpKind::Activity;
        tape.push(op0).unwrap();

        let mut op1 = Powl64Op::silent();
        op1.pred_mask = 1 << 0;
        op1.succ_mask = 0;
        op1.op_kind = OpKind::Activity;
        tape.push(op1).unwrap();

        tape.entry_op = 0;
        tape.exit_op = 1;
        tape
    }

    // -------------------------------------------------------------------------
    // Happy path: full pipeline walk-through
    // -------------------------------------------------------------------------

    #[test]
    fn happy_path_standard_topology() {
        let tape = two_op_tape();
        let runner = PowlRunner::new(tape);

        // Validate
        let compiled = runner.validate().expect("tape should be valid");

        // Schedule
        let scheduled = compiled.schedule::<{ TopologyKind::Standard }>();

        // Begin execution
        let (executing, mut token) = scheduled.begin_execution();
        assert_eq!(token.total(), 2);
        assert_eq!(token.remaining(), 0b11);

        // Fire all ops
        token.consume_op(1 << 0);
        token.consume_op(1 << 1);
        assert_eq!(token.remaining(), 0);

        // Complete
        let (receipted, receipt) = executing.complete(token).expect("complete");
        assert_eq!(receipt.topology, TopologyKind::Standard);
        assert_eq!(receipt.op_trace, 0b11);
        assert_eq!(receipted.run_id(), receipt.run_id);
    }

    #[test]
    fn happy_path_priority_topology() {
        let tape = two_op_tape();
        let compiled = PowlRunner::new(tape).validate().unwrap();
        let sched = compiled.schedule::<{ TopologyKind::Priority }>();
        let (exec, mut tok) = sched.begin_execution();
        tok.consume_op(1);
        tok.consume_op(2);
        let (_, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.topology, TopologyKind::Priority);
    }

    #[test]
    fn happy_path_background_topology() {
        let tape = two_op_tape();
        let compiled = PowlRunner::new(tape).validate().unwrap();
        let sched = compiled.schedule::<{ TopologyKind::Background }>();
        let (exec, mut tok) = sched.begin_execution();
        tok.consume_op(1);
        tok.consume_op(2);
        let (_, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.topology, TopologyKind::Background);
    }

    // -------------------------------------------------------------------------
    // ValidationError paths
    // -------------------------------------------------------------------------

    #[test]
    fn validate_empty_tape_fails() {
        let tape = PowlTape::new(); // no ops pushed
        let err = PowlRunner::new(tape).validate().unwrap_err();
        assert_eq!(err, ValidationError::EmptyTape);
    }

    #[test]
    fn validate_no_entry_op_fails() {
        // Build a tape where every op has a predecessor → entry_mask == 0.
        let mut tape = PowlTape::new();
        let mut op0 = Powl64Op::silent();
        op0.pred_mask = 1 << 1; // op0 waits for op1
        op0.succ_mask = 0;
        tape.push(op0).unwrap();

        let mut op1 = Powl64Op::silent();
        op1.pred_mask = 1 << 0; // op1 waits for op0 (cycle, but we only check entry)
        op1.succ_mask = 0;
        tape.push(op1).unwrap();

        let err = PowlRunner::new(tape).validate().unwrap_err();
        assert_eq!(err, ValidationError::NoEntryOp);
    }

    // -------------------------------------------------------------------------
    // ExecutionDefect paths
    // -------------------------------------------------------------------------

    #[test]
    fn complete_with_unfired_ops_fails() {
        let tape = two_op_tape();
        let runner = PowlRunner::new(tape)
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::Standard }>();
        let (exec, mut tok) = runner.begin_execution();
        // Only fire op 0; leave op 1 unfired.
        tok.consume_op(1 << 0);

        let err = exec.complete(tok).unwrap_err();
        assert!(matches!(
            err,
            ExecutionDefect::UnexhaustedOps { remaining: 0b10 }
        ));
    }

    #[test]
    fn double_consume_op_fails() {
        let mut tok = ExecutionToken::new(2);
        tok.consume_op(1 << 0); // first time: ok
        tok.consume_op(1 << 0); // second time: double-fire

        // Assert that the double-fire defect is accumulated.
        assert_eq!(tok.defect_double_fire, 1 << 0);

        // Clean up other ops to see if assert_exhausted still fails with double-fire.
        tok.consume_op(1 << 1);

        let err = tok.assert_exhausted().unwrap_err();
        assert_eq!(err, ExecutionDefect::OpAlreadyConsumed { bit: 1 << 0 });
    }

    #[test]
    fn assert_exhausted_with_remaining_fails() {
        let tok = ExecutionToken::new(1); // bit 0 still set
                                          // forget to avoid destructor bomb in this error path
        let err = tok.assert_exhausted().unwrap_err();
        assert!(matches!(err, ExecutionDefect::UnexhaustedOps { .. }));
    }

    #[test]
    fn assert_exhausted_after_all_consumed_succeeds() {
        let mut tok = ExecutionToken::new(3);
        tok.consume_op(1 << 0);
        tok.consume_op(1 << 1);
        tok.consume_op(1 << 2);
        tok.assert_exhausted().unwrap();
    }

    #[test]
    fn invalid_consume_op_fails() {
        let mut tok = ExecutionToken::new(2); // valid_mask is 0b11
        tok.consume_op(1 << 2); // out of bounds: bit 2

        assert_eq!(tok.defect_invalid, 1 << 2);

        // Clean up valid bits so assert_exhausted runs.
        tok.consume_op(1 << 0);
        tok.consume_op(1 << 1);

        let err = tok.assert_exhausted().unwrap_err();
        assert_eq!(err, ExecutionDefect::InvalidFires { bits: 1 << 2 });
    }

    #[test]
    fn malformed_consume_op_fails() {
        let mut tok = ExecutionToken::new(2);
        tok.consume_op(0); // malformed: zero bit
        assert_eq!(tok.defect_malformed, u64::MAX);

        // Clean up
        tok.consume_op(1 << 0);
        tok.consume_op(1 << 1);

        let err = tok.assert_exhausted().unwrap_err();
        assert_eq!(err, ExecutionDefect::MalformedFires { bits: u64::MAX });
    }

    #[test]
    fn malformed_multi_bit_consume_op_fails() {
        let mut tok = ExecutionToken::new(3);
        tok.consume_op(0b11); // malformed: multi-bit fire (bit 0 and 1)
        assert_eq!(tok.defect_malformed, 0b11);

        // Clean up
        tok.consume_op(1 << 2);
        let err = tok.assert_exhausted().unwrap_err();
        assert_eq!(err, ExecutionDefect::MalformedFires { bits: 0b11 });
    }

    // -------------------------------------------------------------------------
    // ExecutionToken edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn token_zero_ops() {
        let tok = ExecutionToken::new(0);
        assert_eq!(tok.remaining(), 0);
        tok.assert_exhausted().unwrap();
    }

    #[test]
    fn token_64_ops() {
        let tok = ExecutionToken::new(64);
        assert_eq!(tok.remaining(), u64::MAX);
        assert_eq!(tok.total(), 64);
        // Clean up without firing (use forget — we just test construction).
        core::mem::forget(tok);
    }

    // -------------------------------------------------------------------------
    // Receipt fields
    // -------------------------------------------------------------------------

    #[test]
    fn receipt_op_trace_is_all_bits_for_two_op_tape() {
        let tape = two_op_tape();
        let runner = PowlRunner::new(tape)
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::LongRunning }>();
        let (exec, mut tok) = runner.begin_execution();
        tok.consume_op(0b01);
        tok.consume_op(0b10);
        let (_, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.op_trace, 0b11);
        assert_eq!(receipt.topology, TopologyKind::LongRunning);
    }

    // -------------------------------------------------------------------------
    // Gap 2: destructor bomb
    // -------------------------------------------------------------------------

    /// Verify that dropping an `ExecutionToken` with `remaining != 0` panics in
    /// debug builds.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "ExecutionToken dropped with unfired ops")]
    fn execution_token_drop_with_remaining_panics_in_debug() {
        // remaining=0b11 (two unfired ops), total=2
        let token = ExecutionToken::new_for_test(0b11, 2);
        // Explicit drop triggers the destructor bomb.
        drop(token);
    }

    // -------------------------------------------------------------------------
    // Gap 3: total vs count_ones validation via new_for_test
    // -------------------------------------------------------------------------

    /// Verify that `new_for_test` with mismatched total panics in debug builds.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "new_for_test: remaining has")]
    fn new_for_test_total_mismatch_panics_in_debug() {
        // remaining=0b11 has 2 bits set, but total=3 is wrong
        let _tok = ExecutionToken::new_for_test(0b11, 3);
        // Must not reach here — forget to avoid destructor bomb on the token
        // (panic happens before construction completes).
    }

    // -------------------------------------------------------------------------
    // Gap 4: phase marker non-constructibility
    // -------------------------------------------------------------------------

    /// Verify that phase marker types have no public fields and no external
    /// constructor path (structural check — compilation of this test module
    /// itself demonstrates the markers are used opaquely).
    #[test]
    fn phase_markers_are_zero_sized() {
        use core::mem::size_of;
        assert_eq!(size_of::<Compiled>(), 0);
        assert_eq!(size_of::<Scheduled<{ TopologyKind::Standard }>>(), 0);
        assert_eq!(size_of::<Executing<{ TopologyKind::Standard }>>(), 0);
        assert_eq!(size_of::<Receipted<{ TopologyKind::Standard }>>(), 0);
    }

    // -------------------------------------------------------------------------
    // Track 3: topo_order tests
    // -------------------------------------------------------------------------

    #[test]
    fn verify_topo_order_linear_3op() {
        use crate::compiler::{compile_powl, PowlAstNode};

        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]);
        let tape = compile_powl(&ast).unwrap();

        // Build a runner and run it through the full pipeline.
        let runner = PowlRunner::new(tape.clone())
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::Standard }>();
        let (exec, mut token) = runner.begin_execution();

        // Fire ops 0, 1, 2 in order.
        token.record_fire(0);
        token.consume_op(1 << 0);
        token.record_fire(1);
        token.consume_op(1 << 1);
        token.record_fire(2);
        token.consume_op(1 << 2);

        let (_, receipt) = exec.complete(token).unwrap();
        assert_eq!(&receipt.topo_order[..3], &[0u8, 1, 2]);
        assert_eq!(receipt.event_count, 3);
        assert!(receipt.verify_topo_order(&tape.ops[..tape.len as usize]));
    }

    #[test]
    fn verify_topo_order_tampered_fails() {
        use crate::compiler::{compile_powl, PowlAstNode};

        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]);
        let tape = compile_powl(&ast).unwrap();

        let runner = PowlRunner::new(tape.clone())
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::Standard }>();
        let (exec, mut token) = runner.begin_execution();

        token.record_fire(0);
        token.consume_op(1 << 0);
        token.record_fire(1);
        token.consume_op(1 << 1);
        token.record_fire(2);
        token.consume_op(1 << 2);

        let (_, mut receipt) = exec.complete(token).unwrap();
        // Swap step 0 and step 1 — this violates pred constraint (op 1 requires op 0 first).
        receipt.topo_order.swap(0, 1);
        assert!(!receipt.verify_topo_order(&tape.ops[..tape.len as usize]));
    }

    #[test]
    fn verify_topo_order_missing_op_fails() {
        use crate::compiler::{compile_powl, PowlAstNode};

        let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")]);
        let tape = compile_powl(&ast).unwrap();

        let runner = PowlRunner::new(tape.clone())
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::Standard }>();
        let (exec, mut token) = runner.begin_execution();

        // Record only op 1 (skip op 0), but consume both.
        token.consume_op(1 << 0);
        token.record_fire(1);
        token.consume_op(1 << 1);

        let (_, mut receipt) = exec.complete(token).unwrap();
        // op_trace has bit 0 set but topo_order doesn't include op 0 → fail.
        // Force op_trace to include bit 0:
        receipt.op_trace = 0b11;
        assert!(!receipt.verify_topo_order(&tape.ops[..tape.len as usize]));
    }

    #[test]
    fn execution_token_record_fire_saturates_at_64() {
        // Use new_for_test with u64::MAX (64 bits set).
        let mut token = ExecutionToken::new(64);
        for i in 0..65u8 {
            token.record_fire(i.min(63));
        }
        assert_eq!(token.event_count, 64);
        // Clean up.
        core::mem::forget(token);
    }

    // -------------------------------------------------------------------------
    // ExecutionToken: 256 consume_op cycles wrapping correctly
    // -------------------------------------------------------------------------

    #[test]
    fn consume_op_256_cycles_no_panic() {
        for _ in 0..4 {
            let mut tok = ExecutionToken::new(64);
            assert_eq!(tok.total(), 64);
            for bit_idx in 0..64u64 {
                tok.consume_op(1u64 << bit_idx);
            }
            tok.assert_exhausted()
                .expect("all ops consumed, must be exhausted");
        }
    }

    #[test]
    fn consume_op_wraps_at_bit_63() {
        let mut tok = ExecutionToken::new(64);
        tok.consume_op(1u64 << 63);
        for i in 0..63u64 {
            tok.consume_op(1u64 << i);
        }
        tok.assert_exhausted().unwrap();
    }

    // -------------------------------------------------------------------------
    // Typestate consuming transitions — each phase is single-use
    // -------------------------------------------------------------------------

    #[test]
    fn typestate_unvalidated_to_compiled_consuming() {
        let tape = two_op_tape();
        let runner = PowlRunner::new(tape);
        let compiled = runner.validate().expect("must compile");
        let _ = compiled;
    }

    #[test]
    fn typestate_compiled_to_scheduled_consuming() {
        let tape = two_op_tape();
        let compiled = PowlRunner::new(tape).validate().unwrap();
        let scheduled = compiled.schedule::<{ TopologyKind::Standard }>();
        let _ = scheduled;
    }

    #[test]
    fn typestate_scheduled_to_executing_consuming() {
        let tape = two_op_tape();
        let scheduled = PowlRunner::new(tape)
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::Priority }>();
        let (executing, tok) = scheduled.begin_execution();
        core::mem::forget(tok);
        let _ = executing;
    }

    #[test]
    fn typestate_executing_to_receipted_consuming() {
        let tape = two_op_tape();
        let (exec, mut tok) = PowlRunner::new(tape)
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::Background }>()
            .begin_execution();
        tok.consume_op(1 << 0);
        tok.consume_op(1 << 1);
        let (receipted, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.topology, TopologyKind::Background);
        assert_eq!(receipted.run_id(), receipt.run_id);
    }

    #[test]
    fn typestate_receipted_tape_accessible() {
        let tape = two_op_tape();
        let (exec, mut tok) = PowlRunner::new(tape)
            .validate()
            .unwrap()
            .schedule::<{ TopologyKind::LongRunning }>()
            .begin_execution();
        tok.consume_op(1 << 0);
        tok.consume_op(1 << 1);
        let (receipted, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipted.run_id(), receipt.run_id);
    }

    // -------------------------------------------------------------------------
    // Compile-fail documentation
    // -------------------------------------------------------------------------
    //
    // The following patterns are illegal and must NOT compile.  They are shown
    // as non-`compile_fail` doc-examples here to serve as living documentation;
    // a `trybuild` test suite in `tests/compile_fail/` enforces the errors.
    //
    // 1. Calling `.schedule()` on an `Unvalidated` runner:
    //    ```compile_fail
    //    use bcinr_powl::typestate::{PowlRunner, TopologyKind, Unvalidated};
    //    use bcinr_powl::tape::v2::PowlTape;
    //    let r: PowlRunner<Unvalidated, PowlTape> = PowlRunner::new(PowlTape::new());
    //    let _ = r.schedule::<{ TopologyKind::Standard }>(); // ERROR: method not found
    //    ```
    //
    // 2. Calling `.begin_execution()` on a `Compiled` runner:
    //    ```compile_fail
    //    use bcinr_powl::typestate::{PowlRunner, TopologyKind};
    //    use bcinr_powl::tape::v2::PowlTape;
    //    let mut tape = PowlTape::new();
    //    tape.push(bcinr_powl::tape::v2::Powl64Op::silent()).unwrap();
    //    let compiled = PowlRunner::new(tape).validate().unwrap();
    //    let _ = compiled.begin_execution(); // ERROR: method not found on Compiled
    //    ```
    //
    // 3. Dropping an ExecutionToken with unfired ops (debug_assertions):
    //    ```should_panic
    //    use bcinr_powl::typestate::ExecutionToken;
    //    let _tok = ExecutionToken::new(1); // drops here → destructor bomb fires
    //    ```
    //
    // 4. Cloning an ExecutionToken (trait not derived → compile error):
    //    ```compile_fail
    //    use bcinr_powl::typestate::ExecutionToken;
    //    let tok = ExecutionToken::new(1);
    //    let _tok2 = tok.clone(); // ERROR: no method `clone`
    //    ```
}
