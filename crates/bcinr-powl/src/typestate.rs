//! POWL v2 TypeState machine — phase-indexed runner with linear-token execution.
//!
//! # Phase lattice
//!
//! ```text
//! Unvalidated → Compiled → Scheduled<KIND> → Executing<KIND> → Receipted<KIND>
//! ```
//!
//! Each transition is a consuming method, so the compiler statically forbids
//! re-use of a runner in an earlier phase.
//!
//! # Nightly features required
//!
//! - `adt_const_params`   — `TopologyKind` as a const generic parameter

#![allow(incomplete_features)]

use core::marker::PhantomData;

// =============================================================================
// TopologyKind — const generic discriminant
// =============================================================================

/// Scheduling topology that governs execution priority and retry semantics.
///
/// Used as a const generic parameter so the type system tracks topology across
/// all post-`Compiled` phases.
#[derive(PartialEq, Eq, Clone, Copy, Debug, core::marker::ConstParamTy)]
#[repr(u8)]
pub enum TopologyKind {
    Priority     = 0,
    Standard     = 1,
    Background   = 2,
    LongRunning  = 3,
    Compensating = 4,
}

// =============================================================================
// Phase markers (zero-sized, non-constructible outside this module)
// =============================================================================

/// Phase marker: tape not yet validated.
pub struct Unvalidated;

/// Phase marker: tape has passed structural validation.
///
/// The inner `()` field is private, preventing external construction.
pub struct Compiled(());

/// Phase marker: runner has been assigned a scheduling topology.
pub struct Scheduled<const KIND: TopologyKind>(());

/// Phase marker: execution has started; an [`ExecutionToken`] is in flight.
pub struct Executing<const KIND: TopologyKind>(());

/// Phase marker: execution is complete and a [`Receipt`] has been issued.
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

    // Called only from complete() — post-execution, off hot path.
    fn content_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        for op in &self.ops[..self.len as usize] {
            h.update(&op.pred_mask.to_le_bytes());
            h.update(&op.succ_mask.to_le_bytes());
            h.update(&op.ctrl.to_le_bytes());
            h.update(&[op.op_kind as u8, op.choice_group, op.depth, op.fan_out]);
        }
        *h.finalize().as_bytes()
    }
}

// =============================================================================
// ValidationError
// =============================================================================

/// Errors produced during the `Unvalidated → Compiled` transition.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValidationError {
    /// The tape contains no ops.
    EmptyTape,
    /// The tape contains more than 64 ops (bitmask overflow).
    TapeTooLarge { len: usize },
    /// No op in the tape is reachable from the entry mask (disconnected graph).
    NoEntryOp,
    /// The tape contains a cycle that would deadlock the executor.
    CyclicDependency,
    /// An op references a predecessor index that is out of bounds.
    InvalidPredecessorIndex { op: u8, pred: u8 },
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyTape                            => write!(f, "tape has no ops"),
            Self::TapeTooLarge { len }                 => write!(f, "tape has {len} ops; max is 64"),
            Self::NoEntryOp                            => write!(f, "no entry op found (all ops have predecessors)"),
            Self::CyclicDependency                     => write!(f, "tape contains a cycle"),
            Self::InvalidPredecessorIndex { op, pred } =>
                write!(f, "op {op} references predecessor {pred} which is out of bounds"),
        }
    }
}

// =============================================================================
// ExecutionDefect
// =============================================================================

/// Errors produced during the `Executing → Receipted` transition.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExecutionDefect {
    /// The op bit was already consumed (double-fire).
    OpAlreadyConsumed { bit: u64 },
    /// `assert_exhausted` was called while some ops remain unfired.
    UnexhaustedOps { remaining: u64 },
    /// The [`ExecutionToken`] presented does not belong to this runner.
    TokenMismatch,
}

impl core::fmt::Display for ExecutionDefect {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OpAlreadyConsumed { bit }        => write!(f, "op bit {bit:#b} already consumed"),
            Self::UnexhaustedOps { remaining }     => write!(f, "unfired ops remain: {remaining:#b}"),
            Self::TokenMismatch                    => write!(f, "execution token does not match this runner"),
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
    /// Total number of ops (mirrors `remaining.count_ones()` at construction).
    total: u8,
}

// Explicitly opt out of Clone/Copy — this is the linear-type emulation.
// (No `#[derive(Clone)]` or `#[derive(Copy)]`.)

impl ExecutionToken {
    /// Construct a fresh token for a tape with `op_count` ops (≤ 64).
    ///
    /// Bits `0..op_count` are set in `remaining`.
    pub(crate) fn new(op_count: usize) -> Self {
        debug_assert!(op_count <= 64, "op_count must be ≤ 64");
        // Branchless bitmask: if op_count == 64 we want all 64 bits set.
        // `(1u64 << 64)` wraps to 0 on most platforms; handle via wrapping_shl.
        let remaining = if op_count == 0 {
            0u64
        } else if op_count == 64 {
            u64::MAX
        } else {
            (1u64 << op_count).wrapping_sub(1)
        };
        Self { remaining, total: op_count as u8 }
    }

    /// Mark an op as fired.
    ///
    /// `op_bit` must be a single-bit mask (exactly one bit set) corresponding
    /// to the op that just completed.
    ///
    /// Implemented branchlessly: uses masking to detect double-fires.
    #[inline]
    pub fn consume_op(&mut self, op_bit: u64) -> Result<(), ExecutionDefect> {
        // Branchless double-fire check: if the bit is NOT in remaining, error.
        let present = self.remaining & op_bit;
        // `present == 0` → bit was already cleared (double-fire)
        // `present != 0` → ok, clear it
        //
        // We use a branchless approach: compute an error sentinel and use it.
        let already_consumed = (present == 0) as u64;
        // Write through regardless; if it was already 0 this is idempotent.
        self.remaining &= !op_bit;
        if already_consumed != 0 {
            Err(ExecutionDefect::OpAlreadyConsumed { bit: op_bit })
        } else {
            Ok(())
        }
    }

    /// Assert that all ops have been fired and consume the token.
    ///
    /// Returns an error if any bits remain set in `remaining`.
    pub fn assert_exhausted(self) -> Result<(), ExecutionDefect> {
        let remaining = self.remaining;
        // Prevent the destructor bomb from firing — we're consuming intentionally.
        core::mem::forget(self);
        if remaining != 0 {
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
    /// Unique run identifier (monotonic counter or random).
    pub run_id: u64,
    /// Bitmask recording which ops were fired (op trace).
    pub op_trace: u64,
    /// Runtime topology (mirrors the const generic parameter).
    pub topology: TopologyKind,
    /// 32-byte content hash of the tape at manufacture time.
    pub chain_hash: [u8; 32],
    /// Replay pointer — byte offset into a hypothetical event log.
    pub replay_ptr: u64,
}

impl<const KIND: TopologyKind> core::fmt::Debug for Receipt<KIND> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Receipt")
            .field("run_id",     &self.run_id)
            .field("op_trace",   &format_args!("{:#b}", self.op_trace))
            .field("topology",   &self.topology)
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
    tape:   Tape,
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
    /// Construct a new runner wrapping `tape`.
    pub fn new(tape: Tape) -> Self {
        Self {
            tape,
            run_id: new_run_id(),
            _phase: PhantomData,
        }
    }

    /// Validate the tape and advance to `Compiled`.
    ///
    /// Checks performed:
    /// - tape is non-empty
    /// - tape has ≤ 64 ops
    /// - at least one entry op exists (non-zero entry mask)
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
            tape:   self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------------
// Compiled → Scheduled<KIND>
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape> PowlRunner<Compiled, Tape> {
    /// Assign a scheduling topology and advance to `Scheduled<KIND>`.
    pub fn schedule<const KIND: TopologyKind>(self) -> PowlRunner<Scheduled<KIND>, Tape> {
        PowlRunner {
            tape:   self.tape,
            run_id: self.run_id,
            _phase: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduled<KIND> → Executing<KIND>
// ---------------------------------------------------------------------------

impl<Tape: HasPowlTape, const KIND: TopologyKind> PowlRunner<Scheduled<KIND>, Tape> {
    /// Begin execution and produce a linear [`ExecutionToken`].
    ///
    /// The token must be fully consumed (all op bits cleared) before calling
    /// [`PowlRunner::complete`].
    pub fn begin_execution(self) -> (PowlRunner<Executing<KIND>, Tape>, ExecutionToken) {
        let op_count = self.tape.op_count();
        let token = ExecutionToken::new(op_count);
        let runner = PowlRunner {
            tape:   self.tape,
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
    /// Complete execution by consuming the [`ExecutionToken`] and issuing a [`Receipt`].
    ///
    /// Returns `Err` if the token still has unfired ops.
    pub fn complete(
        self,
        token: ExecutionToken,
    ) -> Result<(PowlRunner<Receipted<KIND>, Tape>, Receipt<KIND>), ExecutionDefect> {
        let op_trace  = !token.remaining() & full_mask(self.tape.op_count());
        let remaining = token.remaining();
        // Consume token without triggering the destructor bomb.
        core::mem::forget(token);

        if remaining != 0 {
            return Err(ExecutionDefect::UnexhaustedOps { remaining });
        }

        let chain_hash = self.tape.content_hash();
        let receipt = Receipt::<KIND> {
            run_id:     self.run_id,
            op_trace,
            topology:   KIND,
            chain_hash,
            replay_ptr: self.run_id, // placeholder: real impl uses event-log offset
        };
        let runner = PowlRunner {
            tape:   self.tape,
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
    /// Access the underlying tape (read-only) after manufacture.
    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    /// Return the run identifier assigned at construction.
    pub fn run_id(&self) -> u64 {
        self.run_id
    }
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Produce a bitmask with exactly `n` low bits set (n ≤ 64).
#[inline(always)]
const fn full_mask(n: usize) -> u64 {
    if n == 0 {
        0
    } else if n >= 64 {
        u64::MAX
    } else {
        (1u64 << n).wrapping_sub(1)
    }
}

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
        op0.op_kind   = OpKind::Activity;
        tape.push(op0).unwrap();

        let mut op1 = Powl64Op::silent();
        op1.pred_mask = 1 << 0;
        op1.succ_mask = 0;
        op1.op_kind   = OpKind::Activity;
        tape.push(op1).unwrap();

        tape.entry_op = 0;
        tape.exit_op  = 1;
        tape
    }

    // -------------------------------------------------------------------------
    // Happy path: full pipeline walk-through
    // -------------------------------------------------------------------------

    #[test]
    fn happy_path_standard_topology() {
        let tape   = two_op_tape();
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
        token.consume_op(1 << 0).expect("op 0 fire");
        token.consume_op(1 << 1).expect("op 1 fire");
        assert_eq!(token.remaining(), 0);

        // Complete
        let (receipted, receipt) = executing.complete(token).expect("complete");
        assert_eq!(receipt.topology, TopologyKind::Standard);
        assert_eq!(receipt.op_trace, 0b11);
        assert_eq!(receipted.run_id(), receipt.run_id);
    }

    #[test]
    fn happy_path_priority_topology() {
        let tape     = two_op_tape();
        let compiled = PowlRunner::new(tape).validate().unwrap();
        let sched    = compiled.schedule::<{ TopologyKind::Priority }>();
        let (exec, mut tok) = sched.begin_execution();
        tok.consume_op(1).unwrap();
        tok.consume_op(2).unwrap();
        let (_, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.topology, TopologyKind::Priority);
    }

    #[test]
    fn happy_path_background_topology() {
        let tape     = two_op_tape();
        let compiled = PowlRunner::new(tape).validate().unwrap();
        let sched    = compiled.schedule::<{ TopologyKind::Background }>();
        let (exec, mut tok) = sched.begin_execution();
        tok.consume_op(1).unwrap();
        tok.consume_op(2).unwrap();
        let (_, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.topology, TopologyKind::Background);
    }

    // -------------------------------------------------------------------------
    // ValidationError paths
    // -------------------------------------------------------------------------

    #[test]
    fn validate_empty_tape_fails() {
        let tape = PowlTape::new(); // no ops pushed
        let err  = PowlRunner::new(tape).validate().unwrap_err();
        assert_eq!(err, ValidationError::EmptyTape);
    }

    #[test]
    fn validate_no_entry_op_fails() {
        // Build a tape where every op has a predecessor → entry_mask == 0.
        let mut tape = PowlTape::new();
        let mut op0  = Powl64Op::silent();
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
        let tape   = two_op_tape();
        let runner = PowlRunner::new(tape).validate().unwrap()
                                          .schedule::<{ TopologyKind::Standard }>();
        let (exec, mut tok) = runner.begin_execution();
        // Only fire op 0; leave op 1 unfired.
        tok.consume_op(1 << 0).unwrap();

        let err = exec.complete(tok).unwrap_err();
        assert!(matches!(err, ExecutionDefect::UnexhaustedOps { remaining: 0b10 }));
    }

    #[test]
    fn double_consume_op_fails() {
        let mut tok = ExecutionToken::new(2);
        tok.consume_op(1 << 0).unwrap(); // first time: ok
        let err = tok.consume_op(1 << 0).unwrap_err(); // second time: error
        assert_eq!(err, ExecutionDefect::OpAlreadyConsumed { bit: 1 });
        // Clean up to avoid destructor bomb.
        tok.consume_op(1 << 1).unwrap();
        tok.assert_exhausted().unwrap();
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
        tok.consume_op(1 << 0).unwrap();
        tok.consume_op(1 << 1).unwrap();
        tok.consume_op(1 << 2).unwrap();
        tok.assert_exhausted().unwrap();
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

    #[test]
    fn full_mask_helper() {
        assert_eq!(full_mask(0),  0);
        assert_eq!(full_mask(1),  1);
        assert_eq!(full_mask(2),  0b11);
        assert_eq!(full_mask(8),  0xFF);
        assert_eq!(full_mask(64), u64::MAX);
    }

    // -------------------------------------------------------------------------
    // Receipt fields
    // -------------------------------------------------------------------------

    #[test]
    fn receipt_op_trace_is_all_bits_for_two_op_tape() {
        let tape   = two_op_tape();
        let runner = PowlRunner::new(tape).validate().unwrap()
                                          .schedule::<{ TopologyKind::LongRunning }>();
        let (exec, mut tok) = runner.begin_execution();
        tok.consume_op(0b01).unwrap();
        tok.consume_op(0b10).unwrap();
        let (_, receipt) = exec.complete(tok).unwrap();
        assert_eq!(receipt.op_trace, 0b11);
        assert_eq!(receipt.topology, TopologyKind::LongRunning);
    }

    // -------------------------------------------------------------------------
    // content_hash tests
    // -------------------------------------------------------------------------

    #[test]
    fn content_hash_is_deterministic() {
        let tape1 = two_op_tape();
        let tape2 = two_op_tape();
        use crate::typestate::HasPowlTape;
        assert_eq!(tape1.content_hash(), tape2.content_hash(),
            "same tape structure must produce same hash");
    }

    #[test]
    fn content_hash_differs_for_different_tapes() {
        use crate::typestate::HasPowlTape;
        let tape1 = two_op_tape();
        // Build a single-op tape
        let mut tape2 = crate::tape::v2::PowlTape::new();
        let op = crate::tape::v2::Powl64Op::silent();
        tape2.push(op).unwrap();
        tape2.entry_op = 0;
        tape2.exit_op = 0;
        assert_ne!(tape1.content_hash(), tape2.content_hash(),
            "different tapes must produce different hashes");
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
