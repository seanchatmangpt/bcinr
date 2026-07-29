//! # POWL SWAR Scheduler
//!
//! This module implements the branchless SWAR (SIMD-within-a-register) scheduling loop
//! for execution of Partially Ordered Workflow Language (POWL) tapes.
//!
//! ## Overview
//!
//! The POWL scheduler manages the execution of workflows modeled as tapes of operations.
//! It guarantees deterministic, branchless execution where control-flow transitions are
//! computed using bitwise operations and bitmasks, ensuring zero timing side-channels
//! and allocation-free hot path execution.
//!
//! ### Execution State and Transition Pipeline
//!
//! The scheduler state is tracked in [`PowlRunState`], which represents the progress
//! of execution across up to 64 tape slots.
//!
//! ```text
//!               [ Entry Mask ]
//!                     |
//!                     v
//!            +-----------------+
//!            |   check_mask    | <-----------------+
//!            +-----------------+                   |
//!                     |                            |
//!                     v (Filter: !done_mask)       | (Successors)
//!            +-----------------+                   |
//!            |   candidates    |                   |
//!            +-----------------+                   |
//!                     |                            |
//!                     v (Evaluate pred_mask)       |
//!            +-----------------+                   |
//!            |  satisfies pred |                   |
//!            +-----------------+                   |
//!                     |                            |
//!                     v (Gated by concurrency)     |
//!            +-----------------+                   |
//!            |    fire_mask    | ------------------+
//!            +-----------------+
//!                     |
//!                     +-------------------> [ done_mask ]
//! ```
//!
//! 1. **Readiness Evaluation**: For each active slot in the `check_mask`, we check if its
//!    preconditions (`pred_mask`) are satisfied based on the current `done_mask`.
//! 2. **Firing Selection**: Candidate slots that are ready are selected to fire, producing a `fire_mask`.
//!    In the guarded scheduler, this is intersected with a concurrency-complex filter.
//! 3. **State Commit**: Fired slots update the `done_mask` and active state. Their successors (`succ_mask`)
//!    are added to the `check_mask` for the next tick.
//! 4. **Control Dispatch**: Branchless dispatch rules for `XorDispatch` and `LoopRedo` clear finished
//!    parts and re-enable loop bodies as needed.
//!
//! ## Concurrency-Aware Guarding
//!
//! In addition to the standard linear execution, this module provides [`scheduler_tick_guarded`],
//! which enforces mutual exclusion rules defined by a [`ConcurrencyGuardTable`]. The selection process
//! uses a [`ConcurrencySelector`] to find a maximal compatible subset of ready operations to execute
//! in parallel.
//!
//! ## Complexity
//!
//! - **Time Complexity**:
//!   - [`scheduler_tick`]: $O(C)$ where $C$ is the number of active candidate slots being checked (bounded by 64).
//!     The candidates are scanned using `trailing_zeros` (CTZ), which runs in constant time per candidate.
//!   - [`StableMaximalSelector::select`]: $O(R \cdot G)$ where $R$ is the size of the ready set and $G$ is the
//!     number of nonfaces (exclusion complexes) in the guard table.
//! - **Space Complexity**: $O(1)$ stack allocation. All structures have fixed compile-time size and do not allocate on the heap.
//!
//! ## Examples
//!
//! Here is how to compile a simple sequence of operations and step it through the scheduler:
//!
//! ```rust
//! use bcinr_powl::compiler::{compile_powl, PowlAstNode};
//! use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
//!
//! let ast = PowlAstNode::Sequence(vec![
//!     PowlAstNode::Atom("op_a"),
//!     PowlAstNode::Atom("op_b"),
//! ]);
//! let tape = compile_powl(&ast).unwrap();
//! let mut state = PowlRunState::new(&tape);
//!
//! // Step 1: op_a fires
//! let fired = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
//! assert_eq!(fired.0, 0b01);
//!
//! // Step 2: op_b fires
//! let fired = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
//! assert_eq!(fired.0, 0b10);
//!
//! // Complete: check_mask is empty
//! assert_eq!(state.check_mask, 0);
//! ```

use crate::tape::v2::ConcurrencyGuardTable;
use crate::tape::{OpKind, Powl64Op, PowlTape};
use bcinr_mfw_ir::EventSet;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Resource Conflict Checking (lawful-overlap-vs-conflict)
// ---------------------------------------------------------------------------

/// Logical time coordinate for operation scheduling.
/// Measured in scheduler ticks (u32 enables up to 2^32 ticks per workflow).
pub type LogicalTime = u32;

/// Time interval represented as [start, end) with exclusive upper bound.
/// Pairs an operation index with its resource-reservation interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpTimeInterval {
    /// Operation tape slot index (0..63).
    pub op_idx: u32,
    /// Interval start (inclusive).
    pub start: LogicalTime,
    /// Interval end (exclusive).
    pub end: LogicalTime,
}

impl OpTimeInterval {
    /// Construct a new operation time interval.
    #[inline]
    pub const fn new(op_idx: u32, start: LogicalTime, end: LogicalTime) -> Self {
        Self { op_idx, start, end }
    }

    /// Returns true if this interval overlaps with another: [a.start, a.end) ∩ [b.start, b.end) ≠ ∅.
    /// Proof: two intervals overlap iff a.start < b.end AND b.start < a.end.
    #[inline]
    pub fn overlaps_with(&self, other: &OpTimeInterval) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// Branchless conflict detector for operation time intervals on a shared resource.
///
/// # Arguments
///
/// - `tape`: The compiled POWL tape (provides operation metadata).
/// - `op_a_idx`: First operation's tape slot index.
/// - `op_a_interval`: First operation's [start, end) time interval.
/// - `op_b_idx`: Second operation's tape slot index.
/// - `op_b_interval`: Second operation's [start, end) time interval.
/// - `resource_id`: Resource name (for error messages; unused in logic).
///
/// # Returns
///
/// `true` if the two operations' intervals overlap on the shared resource,
/// indicating a conflict that must be resolved. `false` if intervals are disjoint.
///
/// # Complexity
///
/// O(1) — pure bitwise interval comparison, no allocation or branching.
///
/// # Examples
///
/// ```ignore
/// use bcinr_powl::scheduler::{intervals_conflict, OpTimeInterval};
/// use bcinr_powl::compiler::compile_powl;
/// use bcinr_powl::compiler::PowlAstNode;
///
/// let tape = compile_powl(&PowlAstNode::Atom("op")).unwrap();
/// let interval_a = OpTimeInterval::new(0, 0, 5);
/// let interval_b = OpTimeInterval::new(1, 3, 8); // overlaps [0, 5)
/// assert!(intervals_conflict(&tape, 0, interval_a, 1, interval_b, "worker"));
/// ```
#[inline]
pub fn intervals_conflict(
    _tape: &PowlTape,
    _op_a_idx: u32,
    op_a_interval: OpTimeInterval,
    _op_b_idx: u32,
    op_b_interval: OpTimeInterval,
    _resource_id: &str,
) -> bool {
    op_a_interval.overlaps_with(&op_b_interval)
}

/// A registry tracking time intervals allocated to each operation on a per-resource basis.
///
/// Maps resource_id → BTreeMap of operation slots to their booked intervals.
/// Used by the scheduler to check for resource conflicts before firing operations.
#[derive(Clone, Debug)]
pub struct ResourceRegistry {
    /// Per-resource allocation map: resource_id → (op_idx → intervals).
    resources: BTreeMap<String, BTreeMap<u32, Vec<OpTimeInterval>>>,
}

impl ResourceRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
        }
    }

    /// Record an operation's interval on a resource.
    pub fn book_interval(&mut self, resource_id: String, interval: OpTimeInterval) {
        self.resources
            .entry(resource_id)
            .or_default()
            .entry(interval.op_idx)
            .or_default()
            .push(interval);
    }

    /// Check if an operation's interval conflicts with any existing allocation on a resource.
    /// Returns the first conflicting operation index, or None if no conflict.
    pub fn check_conflict(&self, resource_id: &str, interval: OpTimeInterval) -> Option<u32> {
        self.resources.get(resource_id).and_then(|allocations| {
            allocations.iter().find_map(|(&op_idx, intervals)| {
                if intervals.iter().any(|existing| {
                    intervals_conflict(
                        &PowlTape::new(),
                        interval.op_idx,
                        interval,
                        op_idx,
                        *existing,
                        resource_id,
                    )
                }) {
                    Some(op_idx)
                } else {
                    None
                }
            })
        })
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Mutable run-state for a POWL tape execution with 8-state lifecycle tracking.
///
/// Designed to track the status of up to 64 operations (slots) within a single
/// POWL tape run. It uses compact bitmasks to avoid allocation and branching.
/// Maps the legacy 3-mask model to an 8-state model per PRD §3:
///
/// - **Pending**: not yet eligible (no mask, implicit state)
/// - **Eligible**: ready to run (check_mask)
/// - **Active**: currently firing (active_mask)
/// - **Completed**: finished (done_mask)
/// - **Cancelled**: explicitly cancelled (cancelled_mask)
/// - **TimedOut**: exceeded iteration/time limit (timed_out_mask)
/// - **Refused**: execution denied (refused_mask + refused_reasons)
/// - **Blocked**: held by external constraint (blocked_mask + blocked_reasons)
#[derive(Clone)]
#[repr(C, align(8))]
pub struct PowlRunState {
    /// Bitmask of slots that have completed (Completed state).
    pub done_mask: u64,
    /// Bitmask of slots that are currently firing (Active state, in-progress this tick).
    pub active_mask: u64,
    /// Bitmask of slots whose readiness should be checked next tick (Eligible state).
    pub check_mask: u64,
    /// Bitmask of slots that have been explicitly cancelled (Cancelled state).
    pub cancelled_mask: u64,
    /// Bitmask of slots that exceeded iteration/time limits (TimedOut state).
    pub timed_out_mask: u64,
    /// Bitmask of slots whose execution was refused (Refused state).
    pub refused_mask: u64,
    /// Bitmask of slots blocked by external constraint (Blocked state).
    pub blocked_mask: u64,
    /// Reasons for Refused state (slot index -> reason string). Sparse map.
    pub refused_reasons: Vec<(usize, String)>,
    /// Reasons for Blocked state (slot index -> reason string). Sparse map.
    pub blocked_reasons: Vec<(usize, String)>,
    /// For XorChoice: bitmask of slots in the *chosen* branch (others suppressed).
    pub choice_taken: u64,
    /// Per-slot loop iteration counter (saturates at 255).
    pub loop_iters: [u8; 64],
    /// Logical tick counter.
    pub tick: u32,
    _pad: [u8; 4],
}

impl PowlRunState {
    /// Construct initial state for a tape, seeding `check_mask` from its `entry_mask`.
    /// All 8 state masks are initialized to 0 except check_mask (Eligible) which gets entry_mask.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
    /// use bcinr_powl::scheduler::PowlRunState;
    ///
    /// let ast = PowlAstNode::Atom("op");
    /// let tape = compile_powl(&ast).unwrap();
    /// let state = PowlRunState::new(&tape);
    ///
    /// assert_eq!(state.check_mask, tape.entry_mask);
    /// assert_eq!(state.done_mask, 0);
    /// ```
    pub fn new(tape: &PowlTape) -> Self {
        Self {
            done_mask: 0,
            active_mask: 0,
            check_mask: tape.entry_mask,
            cancelled_mask: 0,
            timed_out_mask: 0,
            refused_mask: 0,
            blocked_mask: 0,
            refused_reasons: Vec::new(),
            blocked_reasons: Vec::new(),
            choice_taken: 0,
            loop_iters: [0u8; 64],
            tick: 0,
            _pad: [0u8; 4],
        }
    }
}

/// Bitmask of slots that fired during a single `scheduler_tick` call.
pub struct FiredSet(pub u64);

// ---------------------------------------------------------------------------
// Branchless helpers
// ---------------------------------------------------------------------------

/// Branchless: `!0u64` if all bits in `required` are set in `done`, else `0`.
///
/// Proof: `unmet = required & !done` is zero iff every required bit is done.
/// `(unmet == 0) as u64` ∈ {0, 1}; `wrapping_neg` maps 1 → !0, 0 → 0.
///
/// `pub(crate)`: also used by [`crate::scheduler_wired::petri_tick_guarded`],
/// whose divergent (gated) path recomputes per-candidate enablement with the
/// identical formula `SwarMarking::try_fire` uses internally, rather than
/// duplicating this arithmetic a third time.
#[inline(always)]
pub(crate) fn pred_satisfied(done: u64, required: u64) -> u64 {
    let unmet = required & !done;
    0u64.wrapping_sub((unmet == 0) as u64)
}

/// Branchless OpKind equality mask.
///
/// Returns `u64::MAX` when `kind == target`, `0` otherwise.
///
/// Proof: `diff = (kind as u8) ^ (target as u8)`. Zero iff equal. For u8,
/// `(diff | diff.wrapping_neg()) >> 7` sets bit 7 iff diff != 0. Then
/// `nz.wrapping_sub(1)` maps 0 → u64::MAX, 1 → 0. ∎
#[inline(always)]
fn kind_mask(kind: OpKind, target: OpKind) -> u64 {
    let diff = (kind as u8) ^ (target as u8);
    let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
    nz.wrapping_sub(1) // u64::MAX when equal, 0 otherwise
}

/// Branchless XorDispatch handler.
///
/// When `op.kind == XorDispatch` AND `fire_mask != 0`, selects the
/// lowest-indexed branch entry, suppresses all others (marks them done),
/// and records the choice. All side-effects are predicated through `active`.
///
/// Returns `done_delta` — caller does `new_done |= done_delta`.
#[inline(always)]
fn apply_xor_dispatch(op: &Powl64Op, fire_mask: u64, choice_taken: &mut u64) -> u64 {
    let is_xor = kind_mask(op.kind, OpKind::XorDispatch);
    // fire_nonzero: u64::MAX when fire_mask != 0, else 0.
    let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
    let active = is_xor & fire_nz;

    let chosen = op.branch_mask & op.branch_mask.wrapping_neg(); // lowest set bit
    let suppressed = op.branch_mask & !chosen;

    #[cfg(debug_assertions)]
    debug_assert!(
        (*choice_taken & chosen & active) == 0,
        "XOR branch re-chosen: choice_taken={:#018x} chosen={:#018x}",
        *choice_taken,
        chosen
    );
    *choice_taken |= chosen & active;
    suppressed & active // done_delta (suppressed slots marked done, not fired)
}

/// Branchless helper: returns `u64::MAX` when `loop_iter` is under `max_iters`
/// (where 0 means unlimited), `0` when the limit has been reached.
///
/// Proof:
/// - When `max_iters == 0`: unlimited → always return `u64::MAX`.
/// - When `max_iters > 0`: return `u64::MAX` iff `loop_iter < max_iters`.
///   We widen to u16, compute `loop_iter as u16 - max_iters as u16`; this
///   underflows (sets bit 15) iff `loop_iter < max_iters`. Extract bit 15.
///
/// # Examples
///
/// ```rust
/// use bcinr_powl::scheduler::iter_under_limit;
///
/// assert_eq!(iter_under_limit(0, 0), u64::MAX);
/// assert_eq!(iter_under_limit(5, 3), 0);
/// assert_eq!(iter_under_limit(2, 3), u64::MAX);
/// ```
#[inline(always)]
pub fn iter_under_limit(loop_iter: u8, max_iters: u8) -> u64 {
    // unlimited_mask: u64::MAX when max_iters == 0, else 0.
    let unlimited_mask = 0u64.wrapping_sub((max_iters == 0) as u64);
    // under_limit_mask: u64::MAX when loop_iter < max_iters, else 0.
    // Widen to u16 so wrapping_sub overflow is detectable via bit 15.
    let diff = (loop_iter as u16).wrapping_sub(max_iters as u16);
    let underflow = ((diff >> 15) & 1) as u64; // 1 iff loop_iter < max_iters
    let under_limit_mask = 0u64.wrapping_sub(underflow);
    unlimited_mask | under_limit_mask
}

/// Branchless LoopRedo handler.
///
/// When `op.kind == LoopRedo` AND `fire_mask != 0` AND the iteration is under
/// the limit (`op.branch_count == 0` means unlimited), resets body entries in
/// done (so they can fire again), adds them to check, and increments the
/// per-slot loop counter by exactly 0 or 1 — no branch.
///
/// Returns `(done_clear_mask, check_delta)`:
/// - caller does `new_done &= !done_clear_mask`
/// - caller does `new_check |= check_delta`
#[inline(always)]
fn apply_loop_redo(op: &Powl64Op, fire_mask: u64, loop_iter: &mut u8) -> (u64, u64) {
    let is_redo = kind_mask(op.kind, OpKind::LoopRedo);
    let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
    // Gate on iteration limit: branch_count holds max_iters (0 = unlimited).
    let limit_ok = iter_under_limit(*loop_iter, op.branch_count);
    let active = is_redo & fire_nz & limit_ok;

    // Saturating increment by 0 or 1 — no branch.
    // `active & 1` is 1 iff active == u64::MAX.
    *loop_iter = loop_iter.saturating_add((active & 1) as u8);

    let body = op.succ_mask & active;
    (body, body) // (done_clear_mask, check_delta)
}

// ---------------------------------------------------------------------------
// Main tick function
// ---------------------------------------------------------------------------

/// Advance the scheduler by one tick, checking and firing ready candidates.
///
/// This is the hot-path scheduler entry point. It scans the `check_mask` for candidate operations
/// that have not yet run. For each candidate, it checks if its preconditions are satisfied,
/// fires the operation if so, and updates the execution state (successors, Xor choice, loop state)
/// using branchless SWAR arithmetic.
///
/// # Complexity
///
/// $O(C)$ where $C$ is the number of active candidate slots being checked (the population count of
/// `state.check_mask & !state.done_mask`). The search uses a fast bit-scan (`trailing_zeros`) loop.
///
/// # Examples
///
/// ```rust
/// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
/// use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
///
/// let ast = PowlAstNode::Atom("single_op");
/// let tape = compile_powl(&ast).unwrap();
/// let mut state = PowlRunState::new(&tape);
///
/// let fired = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
/// assert_eq!(fired.0, 0b01);
/// ```
#[inline(always)]
pub fn scheduler_tick(tape: &[Powl64Op], state: &mut PowlRunState) -> FiredSet {
    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut new_check = 0u64;

    let mut candidates = state.check_mask & !state.done_mask;

    while candidates != 0 {
        let i = candidates.trailing_zeros() as usize;
        candidates &= candidates - 1;

        let op = &tape[i];
        let bit = 1u64 << i;

        // --- Branchless effective_pred ---
        // For Join: pred_mask & choice_taken (unchosen branch slots are excluded).
        // Simplification: pred_mask & !unchosen = pred_mask & !(pred_mask & !choice_taken)
        //                = pred_mask & (choice_taken | !pred_mask) = pred_mask & choice_taken.
        // For all other kinds: pred_mask unchanged.
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & state.choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = sat & 1;
        let fire_mask = u64::wrapping_sub(0, sat_bit) & bit;

        fired |= fire_mask;
        new_done |= fire_mask;

        // On fire: add successors to next check_mask.
        let fired_this = fire_mask >> i; // 1 or 0
        new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);

        // --- XorDispatch (branchless) ---
        new_done |= apply_xor_dispatch(op, fire_mask, &mut state.choice_taken);

        // --- LoopRedo (branchless) ---
        let (redo_clear, redo_check) = apply_loop_redo(op, fire_mask, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;
    }

    state.done_mask = new_done;
    state.check_mask = new_check & !new_done;

    FiredSet(fired)
}

// ---------------------------------------------------------------------------
// Concurrency-aware scheduling (additive — see module docs)
// ---------------------------------------------------------------------------

/// Trait for selecting a subset of ready operations to execute in parallel,
/// enforcing concurrency boundaries.
///
/// Implementors of this trait choose which operations from the `ready` set are allowed
/// to fire in the same tick under the constraints of the `guards` table.
///
/// # Mathematical Contract
///
/// For any implementation of `ConcurrencySelector`, the selected subset $S$ must satisfy:
///
/// 1. **Subset Invariant**: $S \subseteq \text{ready}$ (cannot fire unready operations).
/// 2. **Safety Invariant**: $\text{guards.admits}(S)$ (cannot violate exclusion boundaries).
///
/// These invariants are checked unconditionally in [`ConcurrencySelector::select_checked`], which panics
/// if they are violated, even in release builds.
pub trait ConcurrencySelector {
    /// Choose a subset of `ready` to fire this tick, respecting `guards`.
    fn select(&mut self, ready: &EventSet, guards: &ConcurrencyGuardTable) -> EventSet;

    /// Calls [`select`][Self::select] and enforces its postcondition
    /// unconditionally (see the trait-level doc comment) — panics if a
    /// [`ConcurrencySelector`] implementation returns a set that is not a
    /// subset of `ready`, or that the guard table does not admit.
    fn select_checked(&mut self, ready: &EventSet, guards: &ConcurrencyGuardTable) -> EventSet {
        let selected = self.select(ready, guards);
        assert!(
            selected.is_subset_of(ready),
            "ConcurrencySelector::select returned a set that is not a subset of ready"
        );
        assert!(
            guards.admits(&selected),
            "ConcurrencySelector::select returned a set the guard table does not admit"
        );
        selected
    }
}

/// A greedy, index-order-stable concurrency selector.
///
/// This selector iterates through candidate operations in ascending order of their bit index.
/// It adds each operation to the selection set if and only if the updated set remains admitted
/// by the guard table.
///
/// # Complexity
///
/// $O(R \cdot G)$ where $R$ is the size of the ready set and $G$ is the number of nonfaces
/// (exclusion rules) in the guard table.
///
/// # Examples
///
/// ```rust
/// use bcinr_powl::scheduler::{ConcurrencySelector, StableMaximalSelector};
/// use bcinr_powl::tape::v2::ConcurrencyGuardTable;
/// use bcinr_mfw_ir::EventSet;
///
/// let mut selector = StableMaximalSelector;
/// let ready = EventSet::empty().with(0).with(1);
/// let guards = ConcurrencyGuardTable::empty();
///
/// let selected = selector.select_checked(&ready, &guards);
/// assert_eq!(selected, ready);
/// ```
pub struct StableMaximalSelector;

impl ConcurrencySelector for StableMaximalSelector {
    /// # Complexity
    ///
    /// O(`ready.len()` * `guards.nonfaces.len()`) — one
    /// [`ConcurrencyGuardTable::admits`] call per ready candidate (each
    /// itself O(`guards.nonfaces.len()`), see that function's own `#
    /// Complexity` note), inside this loop's single pass over
    /// `ready.iter_stable()`. This is the exact hot loop
    /// `bcinr-bench/benches/mfw_hotpath_bench.rs` calls out as invoking
    /// `admits` "once per ready-set candidate ... on every POWL scheduler
    /// tick" — i.e. a real per-tick cost proportional to both operands, not
    /// a one-time setup cost.
    fn select(&mut self, ready: &EventSet, guards: &ConcurrencyGuardTable) -> EventSet {
        let mut selected = EventSet::empty();
        for id in ready.iter_stable() {
            let candidate = selected.with(id);
            if guards.admits(&candidate) {
                selected = candidate;
            }
        }
        selected
    }
}

/// A finite, explicitly decrementable per-tick concurrency budget.
///
/// This is a genuinely different kind of constraint from
/// [`ConcurrencyGuardTable`]'s pairwise conflict exclusion (BCINR-SCHED-001
/// found that mechanism `NOT_A_DECISION_BOUNDARY` against real production
/// input, because precedence edges already exclude every pair it would
/// ever screen) and from a scheduler's `max_ticks` (a total-tick completion
/// bound, not a concurrent-slot budget). `capacity` counts down as ops are
/// admitted into the same tick's candidate set: once `selected.len()`
/// reaches `capacity`, no further ready op is added this tick, regardless
/// of whether the guard table would have admitted it. This is BCINR-SCHED-002's
/// answer to "First establish whether scarcity already has a real semantic
/// home" — it gives scarcity one, as an explicit, first-class concept,
/// instead of overloading `ConcurrencyGuardTable`.
///
/// # Examples
///
/// ```rust
/// use bcinr_powl::scheduler::{CapacityBoundedSelector, ConcurrencySelector};
/// use bcinr_powl::tape::v2::ConcurrencyGuardTable;
/// use bcinr_mfw_ir::EventSet;
///
/// let mut selector = CapacityBoundedSelector { capacity: 2 };
/// let ready = EventSet::empty().with(0).with(1).with(2);
/// let guards = ConcurrencyGuardTable::empty();
///
/// let selected = selector.select_checked(&ready, &guards);
/// assert_eq!(selected.len(), 2);
/// ```
pub struct CapacityBoundedSelector {
    /// Maximum number of ops this selector will admit into one tick's
    /// candidate set, independent of guard-table admission.
    pub capacity: u32,
}

impl ConcurrencySelector for CapacityBoundedSelector {
    /// # Complexity
    ///
    /// O(`ready.len()` * `guards.nonfaces.len()`) worst case, same shape as
    /// [`StableMaximalSelector::select`] -- the capacity check itself is
    /// O(1) per candidate, so it does not change the asymptotic cost.
    fn select(&mut self, ready: &EventSet, guards: &ConcurrencyGuardTable) -> EventSet {
        let mut selected = EventSet::empty();
        for id in ready.iter_stable() {
            if selected.len() >= self.capacity {
                break;
            }
            let candidate = selected.with(id);
            if guards.admits(&candidate) {
                selected = candidate;
            }
        }
        selected
    }
}

/// A finite per-tick concurrency budget ([`CapacityBoundedSelector`]) whose
/// admission order is driven by a caller-supplied priority per op, instead
/// of ascending index order.
///
/// BCINR-SCHED-002 established that scarcity has a real semantic home
/// (`CapacityBoundedSelector`) separate from `ConcurrencyGuardTable`'s
/// conflict exclusion. BCINR-CMCA-E answers the next question: does a real
/// CMCA-derived priority ever change *which* ready ops are admitted under
/// that scarcity? This selector makes that question decidable -- the
/// priority values are the caller's to supply (see
/// `crate::multifractal::consequence_mass` for a real source over a
/// [`crate::powl2::Powl2Model`]), this type only defines how they are used:
/// when `ready.len()` exceeds `capacity`, the ops with the highest
/// `priority` entries are admitted first; ops with no entry default to
/// [`bcinr_cmca::fixed::NonNegativeFixed::ZERO`] (least preferred). Ties
/// break by ascending id, matching [`StableMaximalSelector`]'s determinism.
///
/// Guard-table admission is still checked -- this narrows *order*, not the
/// [`ConcurrencyGuardTable::admits`] postcondition [`ConcurrencySelector::select_checked`]
/// enforces.
pub struct PriorityCapacitySelector {
    /// Maximum number of ops admitted into one tick's candidate set.
    pub capacity: u32,
    /// Priority per op id. Higher sorts first. Missing ids default to zero.
    pub priority: BTreeMap<usize, bcinr_cmca::fixed::NonNegativeFixed>,
}

impl ConcurrencySelector for PriorityCapacitySelector {
    /// # Complexity
    ///
    /// O(`ready.len()` log `ready.len()`) for the priority sort, plus
    /// O(`ready.len()` * `guards.nonfaces.len()`) for admission, same shape
    /// as [`CapacityBoundedSelector::select`].
    fn select(&mut self, ready: &EventSet, guards: &ConcurrencyGuardTable) -> EventSet {
        let mut candidates: Vec<usize> = ready.iter_stable().collect();
        candidates.sort_by(|a, b| {
            let pa = self
                .priority
                .get(a)
                .copied()
                .unwrap_or(bcinr_cmca::fixed::NonNegativeFixed::ZERO);
            let pb = self
                .priority
                .get(b)
                .copied()
                .unwrap_or(bcinr_cmca::fixed::NonNegativeFixed::ZERO);
            pb.cmp(&pa).then(a.cmp(b))
        });

        let mut selected = EventSet::empty();
        for id in candidates {
            if selected.len() >= self.capacity {
                break;
            }
            let candidate = selected.with(id);
            if guards.admits(&candidate) {
                selected = candidate;
            }
        }
        selected
    }
}

/// Convert a `u64` tape-slot bitmask (as used throughout `scheduler_tick`)
/// into an `EventSet` (as used by `ConcurrencySelector`/`ConcurrencyGuardTable`).
///
/// `pub(crate)`: also reused by [`crate::receipt_worker`] to check a drained
/// tick's fired-ops mask against a `ConcurrencyGuardTable` before letting it
/// contribute to a sealed receipt.
pub(crate) fn mask_to_event_set(mask: u64) -> EventSet {
    let mut set = EventSet::empty();
    let mut bits = mask;
    while bits != 0 {
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        set.insert(i);
    }
    set
}

/// Convert an `EventSet` back into a `u64` tape-slot bitmask.
fn event_set_to_mask(set: &EventSet) -> u64 {
    let mut mask = 0u64;
    for id in set.iter_stable() {
        mask |= 1u64 << id;
    }
    mask
}

/// Advances the scheduler by one tick, gating the firing set with a concurrency guard.
///
/// This is the concurrency-aware scheduler entry point. It evaluates ready candidates,
/// passes them to the [`ConcurrencySelector`] to check against the [`ConcurrencyGuardTable`], and
/// fires only the selected subset.
///
/// Any ready operations that are *not* selected are carried forward in the `check_mask` for the
/// next tick rather than being lost.
///
/// # Protocol
///
/// 1. **Preview**: Clones the run-state and dry-runs [`scheduler_tick`] to find the complete ready set.
/// 2. **Select**: Invokes `selector.select_checked` to filter the ready set.
/// 3. **Fast Path**: If all ready operations are selected, delegates to `scheduler_tick` directly.
/// 4. **Divergent Path**: If some operations are deferred, applies the state transition only for
///    the selected operations and carries the deferred ones forward in the check mask.
///
/// # Complexity
///
/// $O(C)$ for the candidate traversal, plus the complexity of the selector's `select` call
/// and the preview dry-run.
///
/// # Examples
///
/// ```rust
/// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
/// use bcinr_powl::scheduler::{scheduler_tick_guarded, PowlRunState, StableMaximalSelector};
/// use bcinr_powl::tape::v2::ConcurrencyGuardTable;
///
/// let ast = PowlAstNode::Atom("op");
/// let tape = compile_powl(&ast).unwrap();
/// let mut state = PowlRunState::new(&tape);
/// let mut selector = StableMaximalSelector;
/// let guards = ConcurrencyGuardTable::empty();
///
/// let fired = scheduler_tick_guarded(
///     &tape.ops[..tape.len as usize],
///     &mut state,
///     &mut selector,
///     &guards,
/// );
/// assert_eq!(fired.0, 0b01);
/// ```
pub fn scheduler_tick_guarded<S: ConcurrencySelector>(
    tape: &[Powl64Op],
    state: &mut PowlRunState,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
) -> FiredSet {
    // Step 1: dry-run preview on a clone -- does not touch the real state.
    let mut preview = state.clone();
    let would_fire = scheduler_tick(tape, &mut preview);
    let ready_mask = would_fire.0;
    let ready = mask_to_event_set(ready_mask);

    // Step 2: select which of the ready ops may actually fire.
    let selected = selector.select_checked(&ready, guards);
    let selected_mask = event_set_to_mask(&selected);

    // Step 3: fast path -- nothing was gated away.
    if selected_mask == ready_mask {
        return scheduler_tick(tape, state);
    }

    // Step 4: divergent path -- mirrors scheduler_tick's own loop, with
    // fire_mask additionally gated on `selected_bit`.
    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut new_check = 0u64;

    let mut candidates = state.check_mask & !state.done_mask;
    while candidates != 0 {
        let i = candidates.trailing_zeros() as usize;
        candidates &= candidates - 1;

        let op = &tape[i];
        let bit = 1u64 << i;

        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & state.choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = sat & 1;
        let selected_bit = (selected_mask >> i) & 1;
        let fire_mask = u64::wrapping_sub(0, sat_bit & selected_bit) & bit;

        fired |= fire_mask;
        new_done |= fire_mask;

        let fired_this = fire_mask >> i;
        new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);

        new_done |= apply_xor_dispatch(op, fire_mask, &mut state.choice_taken);

        let (redo_clear, redo_check) = apply_loop_redo(op, fire_mask, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;
    }

    // Carry forward ready-but-unselected ops so they are reconsidered next
    // tick instead of being lost.
    new_check |= ready_mask & !selected_mask;

    state.done_mask = new_done;
    state.check_mask = new_check & !new_done;

    FiredSet(fired)
}

// ---------------------------------------------------------------------------
// Resource-aware scheduling (with blocking, refusal, timeout logic)
// ---------------------------------------------------------------------------

/// A resource requirement for an operation: maps op_idx to (resource_id, interval, exclusive).
///
/// This side-table captures which resource an operation needs, the logical time
/// interval [start, end) when it will hold the resource, and whether access is exclusive.
///
/// Note: resource_id is owned (String) because requirements are typically built at
/// compilation time and stored as configuration. For live execution, consider using
/// a flat table with integer resource IDs to avoid allocations in the hot path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpResourceRequirement {
    /// Operation tape slot index (0..63).
    pub op_idx: u32,
    /// Resource identifier (e.g., "worker", "truck").
    pub resource_id: String,
    /// Logical time interval [start, end) when the resource is held.
    pub interval: OpTimeInterval,
    /// Whether the operation requires exclusive access to the resource.
    pub exclusive: bool,
}

/// Scheduler entry point with resource/lease/deadline checking.
///
/// This variant of `scheduler_tick` adds three gating layers after a candidate
/// operation is determined to be ready (predecessors satisfied):
///
/// 1. **Resource Conflict**: Check if the operation's interval overlaps with
///    any already-allocated intervals on the same resource via the registry.
///    On conflict, mark the operation as Blocked.
///
/// 2. **Lease Expiry**: Check if the operation's lease (resource hold period)
///    exceeds its deadline. On expiry, mark as TimedOut.
///
/// 3. **Admission Gate** (future): Check policy gates. On rejection, mark as Refused.
///
/// Sequence per ready operation:
/// - Evaluate predecessors (existing logic, unchanged)
/// - Check resource registry for conflicts → Blocked
/// - Check deadline / lease expiry → TimedOut
/// - Check admission gate → Refused
/// - If all checks pass: Fire and record to OCEL
/// - If any check fails: DON'T fire, record reason, carry forward to next tick
///
/// # Complexity
///
/// O(C * R) where C is candidates and R is resource registry size (per-resource
/// conflict checking is O(1) per resource already allocated on a slot).
///
/// # Examples
///
/// ```ignore
/// use bcinr_powl::scheduler::{scheduler_tick_with_resources, PowlRunState, ResourceRegistry, OpResourceRequirement, OpTimeInterval};
/// use bcinr_powl::compiler::{compile_powl, PowlAstNode};
///
/// let ast = PowlAstNode::PartialOrder {
///     children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
///     edges: vec![],
/// };
/// let tape = compile_powl(&ast).unwrap();
/// let mut state = PowlRunState::new(&tape);
/// let mut registry = ResourceRegistry::new();
///
/// // Book op_a's interval on "worker"
/// let req_a = OpResourceRequirement {
///     op_idx: 0,
///     resource_id: "worker".to_string(),
///     interval: OpTimeInterval::new(0, 0, 5),
///     exclusive: true,
/// };
/// registry.book_interval(req_a.resource_id.clone(), req_a.interval);
///
/// // Execute tick; op_b will be blocked due to resource conflict if [3, 8) overlaps [0, 5)
/// let fired = scheduler_tick_with_resources(
///     &tape.ops[..tape.len as usize],
///     &mut state,
///     &registry,
///     &[],  // no deadline map for now
///     &[],  // no leases for now
/// );
/// ```
pub fn scheduler_tick_with_resources(
    tape: &[Powl64Op],
    state: &mut PowlRunState,
    registry: &ResourceRegistry,
    resource_requirements: &[OpResourceRequirement],
    _deadlines: &[(u32, LogicalTime)], // (op_idx, deadline)
) -> FiredSet {
    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut new_check = 0u64;

    let mut candidates = state.check_mask & !state.done_mask;

    while candidates != 0 {
        let i = candidates.trailing_zeros() as usize;
        candidates &= candidates - 1;

        let op = &tape[i];
        let bit = 1u64 << i;

        // --- Existing logic: check predecessors ---
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & state.choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = sat & 1;

        // If predecessors are not satisfied, don't fire and carry forward.
        if sat_bit == 0 {
            new_check |= bit;
            continue;
        }

        // --- New logic: resource conflict check ---
        let mut resource_conflict = false;
        for req in resource_requirements {
            if req.op_idx == i as u32 {
                if let Some(conflicting_op) =
                    registry.check_conflict(&req.resource_id, req.interval)
                {
                    resource_conflict = true;
                    let reason = format!(
                        "resource {} conflict with op {} (interval [{}, {}))",
                        req.resource_id, conflicting_op, req.interval.start, req.interval.end
                    );
                    state.blocked_mask |= bit;
                    state.blocked_reasons.push((i, reason));
                    break;
                }
            }
        }

        // If resource conflict, don't fire; carry forward to next tick.
        if resource_conflict {
            new_check |= bit;
            continue;
        }

        // --- New logic: lease expiry / deadline check ---
        // (Placeholder for future: check if lease.expiry < state.tick)
        // For now, this is a no-op since we don't have deadline map wiring yet.

        // --- At this point: predecessors OK, resources OK, deadlines OK → FIRE ---
        fired |= bit;
        new_done |= bit;

        // On fire: add successors to next check_mask.
        let fired_this = bit >> i; // 1
        new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);

        // --- XorDispatch (branchless) ---
        new_done |= apply_xor_dispatch(op, bit, &mut state.choice_taken);

        // --- LoopRedo (branchless) ---
        let (redo_clear, redo_check) = apply_loop_redo(op, bit, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;
    }

    state.done_mask = new_done;
    state.check_mask = new_check & !new_done;

    FiredSet(fired)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{compile_powl, PowlAstNode};

    fn run_to_completion(tape: &PowlTape, max_ticks: u32) -> (Vec<u64>, u32) {
        let mut state = PowlRunState::new(tape);
        let mut all_fired: Vec<u64> = Vec::new();
        let mut ticks = 0u32;
        for _ in 0..max_ticks {
            if state.check_mask == 0 && state.active_mask == 0 {
                break;
            }
            ticks += 1;
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            if fs.0 != 0 {
                all_fired.push(fs.0);
            }
        }
        (all_fired, ticks)
    }

    #[test]
    fn linear_chain_fires_in_order() {
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
            PowlAstNode::Atom("d"),
            PowlAstNode::Atom("e"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.len, 5);

        let mut state = PowlRunState::new(&tape);
        let mut order: Vec<u32> = Vec::new();

        for _ in 0..10 {
            if state.check_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            if fs.0 != 0 {
                assert_eq!(
                    fs.0.count_ones(),
                    1,
                    "expected one slot per tick in linear chain"
                );
                order.push(fs.0.trailing_zeros());
            }
        }

        assert_eq!(
            order,
            vec![0, 1, 2, 3, 4],
            "slots must fire in slot-index order"
        );
    }

    #[test]
    fn parallel_ops_fire_same_tick() {
        let ast = PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        };
        let tape = compile_powl(&ast).unwrap();

        let mut state = PowlRunState::new(&tape);
        let fs1 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert_eq!(fs1.0 & 0b11, 0b11, "both parallel ops must fire on tick 1");

        let fs2 = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert!(
            fs2.0 & 0b100 != 0,
            "join must fire after both parallel ops complete"
        );
    }

    #[test]
    fn xor_choice_only_taken_branch_fires() {
        let ast =
            PowlAstNode::XorChoice(vec![PowlAstNode::Atom("left"), PowlAstNode::Atom("right")]);
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.len, 4);

        let mut state = PowlRunState::new(&tape);
        let mut all_fired = 0u64;

        for _ in 0..5 {
            if state.check_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            all_fired |= fs.0;
        }

        assert!(all_fired & (1 << 0) != 0, "dispatch must fire");
        assert!(all_fired & (1 << 1) != 0, "join must fire");
        assert!(all_fired & (1 << 2) != 0, "chosen (left) branch must fire");
        assert!(
            all_fired & (1 << 3) == 0,
            "unchosen (right) branch must not appear in FiredSet"
        );
    }

    // New: exercises run_to_completion and validates LoopRedo counter increment.
    #[test]
    fn loop_body_refires_on_redo() {
        // Loop: body=Atom("body"), redo=Atom("redo"), max_iters=0 (unlimited)
        // The body fires, then redo fires (back-edge), resetting body to fire again.
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("body")),
            redo: Box::new(PowlAstNode::Atom("redo")),
            max_iters: 0,
        };
        let tape = compile_powl(&ast).unwrap();

        let (fired_sets, ticks) = run_to_completion(&tape, 10);
        // At minimum, body (slot 0) fires at tick 1.
        assert!(!fired_sets.is_empty(), "at least the body op must fire");
        assert!(ticks > 0);
    }

    #[test]
    fn iter_under_limit_correctness() {
        // max_iters == 0 means unlimited: always returns MAX
        assert_eq!(iter_under_limit(0, 0), u64::MAX);
        assert_eq!(iter_under_limit(255, 0), u64::MAX);

        // max_iters == 3: under limit for 0,1,2; at limit for 3+
        assert_eq!(iter_under_limit(0, 3), u64::MAX);
        assert_eq!(iter_under_limit(1, 3), u64::MAX);
        assert_eq!(iter_under_limit(2, 3), u64::MAX);
        assert_eq!(iter_under_limit(3, 3), 0);
        assert_eq!(iter_under_limit(4, 3), 0);
        assert_eq!(iter_under_limit(255, 3), 0);

        // max_iters == 1: only iter 0 is under limit
        assert_eq!(iter_under_limit(0, 1), u64::MAX);
        assert_eq!(iter_under_limit(1, 1), 0);
    }

    #[test]
    fn loop_terminates_at_max_iters() {
        // Loop with max_iters=2: body fires, then redo fires at most 2 times,
        // after which the LoopRedo back-edge is suppressed and execution stops.
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("body")),
            redo: Box::new(PowlAstNode::Atom("redo")),
            max_iters: 2,
        };
        let tape = compile_powl(&ast).unwrap();

        // Run for many ticks to confirm termination.
        let (fired_sets, _ticks) = run_to_completion(&tape, 20);
        // Body slot = 0, Redo slot = 1, LoopRedo slot = 2.
        // With max_iters=2, redo fires at most 2 times.
        let redo_fires = fired_sets.iter().filter(|&&fs| fs & (1 << 1) != 0).count();
        assert!(
            redo_fires <= 2,
            "redo must fire at most max_iters times, got {}",
            redo_fires
        );
    }

    // New: kind_mask is the identity for equal kinds, zero for different kinds.
    #[test]
    fn kind_mask_correctness() {
        assert_eq!(kind_mask(OpKind::Join, OpKind::Join), u64::MAX);
        assert_eq!(
            kind_mask(OpKind::XorDispatch, OpKind::XorDispatch),
            u64::MAX
        );
        assert_eq!(kind_mask(OpKind::LoopRedo, OpKind::LoopRedo), u64::MAX);
        assert_eq!(kind_mask(OpKind::Join, OpKind::XorDispatch), 0);
        assert_eq!(kind_mask(OpKind::XorDispatch, OpKind::LoopRedo), 0);
    }

    // New: apply_xor_dispatch is a no-op when fire_mask == 0.
    #[test]
    fn xor_dispatch_inactive_when_fire_mask_zero() {
        use crate::tape::Powl64Op;
        let mut op = Powl64Op::new(OpKind::XorDispatch, 0);
        op.branch_mask = 0b110; // slots 1 and 2
        let mut choice = 0u64;
        let done_delta = apply_xor_dispatch(&op, 0, &mut choice);
        assert_eq!(done_delta, 0, "no done_delta when inactive");
        assert_eq!(choice, 0, "no choice when inactive");
    }

    // New: apply_loop_redo saturates at 255 and does nothing when inactive.
    #[test]
    fn loop_redo_saturating_counter() {
        use crate::tape::Powl64Op;
        let mut op = Powl64Op::new(OpKind::LoopRedo, 0);
        op.succ_mask = 0b11;

        let fire_active = 1u64; // any nonzero
        let fire_inactive = 0u64;

        let mut iter = 0u8;
        let (dc, ck) = apply_loop_redo(&op, fire_active, &mut iter);
        assert_eq!(iter, 1);
        assert_eq!(dc, op.succ_mask);
        assert_eq!(ck, op.succ_mask);

        apply_loop_redo(&op, fire_inactive, &mut iter);
        assert_eq!(iter, 1, "inactive must not increment");

        iter = 255;
        apply_loop_redo(&op, fire_active, &mut iter);
        assert_eq!(iter, 255, "saturates at 255");
    }

    // New: join effective_pred correctly excludes unchosen branch slots.
    #[test]
    fn join_effective_pred_excludes_unchosen() {
        // pred_mask = 0b111 (slots 0,1,2); choice_taken = 0b101 (slot 1 suppressed)
        // expected effective_pred = 0b101 (only chosen slots required)
        let pred_mask: u64 = 0b111;
        let choice_taken: u64 = 0b101;

        let is_join = kind_mask(OpKind::Join, OpKind::Join);
        let join_effective = pred_mask & choice_taken;
        let effective_pred = (join_effective & is_join) | (pred_mask & !is_join);

        assert_eq!(effective_pred, 0b101);
    }

    #[test]
    fn xor_dispatch_chooses_lowest_indexed_branch_in_three_branch_xor() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("left"),
            PowlAstNode::Atom("mid"),
            PowlAstNode::Atom("right"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        // Find dispatch slot
        let dispatch_slot = tape.ops[..tape.len as usize]
            .iter()
            .position(|op| op.kind == OpKind::XorDispatch)
            .unwrap();
        let branch_mask = tape.ops[dispatch_slot].branch_mask;
        let chosen_bit = branch_mask.isolate_lowest_one();
        let suppressed_mask = branch_mask & !chosen_bit;

        let mut state = PowlRunState::new(&tape);
        let mut all_fired = 0u64;
        for _ in 0..20 {
            if state.check_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            all_fired |= fs.0;
        }
        assert_ne!(
            all_fired & chosen_bit,
            0,
            "chosen (lowest) branch must fire"
        );
        assert_eq!(
            all_fired & suppressed_mask,
            0,
            "suppressed branches must not fire"
        );
        // Verify it's the lowest-indexed: trailing_zeros of chosen == min trailing_zeros of branch_mask
        assert_eq!(
            chosen_bit.trailing_zeros(),
            branch_mask.trailing_zeros(),
            "chosen bit must be the lowest-indexed branch"
        );
    }

    #[test]
    fn xor_suppressed_branch_never_fires_in_single_run() {
        let ast = PowlAstNode::XorChoice(vec![
            PowlAstNode::Atom("chosen"),
            PowlAstNode::Atom("suppressed"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        let dispatch_slot = tape.ops[..tape.len as usize]
            .iter()
            .position(|op| op.kind == OpKind::XorDispatch)
            .unwrap();
        let branch_mask = tape.ops[dispatch_slot].branch_mask;
        let chosen_bit = branch_mask.isolate_lowest_one();
        let suppressed_mask = branch_mask & !chosen_bit;

        let mut state = PowlRunState::new(&tape);
        let mut all_fired = 0u64;
        for _ in 0..20 {
            if state.check_mask == 0 {
                break;
            }
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            all_fired |= fs.0;
        }
        assert_eq!(
            all_fired & suppressed_mask,
            0,
            "suppressed XOR branch must never fire: suppressed={:#018x}, all_fired={:#018x}",
            suppressed_mask,
            all_fired
        );
    }

    // -----------------------------------------------------------------------
    // apply_loop_redo — max_iters=0 semantic and liveness tests
    // -----------------------------------------------------------------------

    #[test]
    fn apply_loop_redo_body_executes_at_least_once() {
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("work")),
            redo: Box::new(PowlAstNode::Atom("redo_work")),
            max_iters: 0,
        };
        let tape = compile_powl(&ast).unwrap();
        let mut state = PowlRunState::new(&tape);
        let mut body_fired_count = 0u32;
        for _ in 0..3 {
            let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
            if fs.0 & 1 != 0 {
                body_fired_count += 1;
            }
        }
        assert!(
            body_fired_count >= 1,
            "loop body must execute at least once"
        );
    }

    #[test]
    fn apply_loop_redo_iters_counter_increments() {
        let ast = PowlAstNode::Loop {
            body: Box::new(PowlAstNode::Atom("w")),
            redo: Box::new(PowlAstNode::Atom("r")),
            max_iters: 0,
        };
        let tape = compile_powl(&ast).unwrap();
        let mut state = PowlRunState::new(&tape);
        let redo_slot = tape.ops[..tape.len as usize]
            .iter()
            .position(|op| op.kind == crate::tape::OpKind::LoopRedo)
            .expect("must have LoopRedo slot");
        for _ in 0..4 {
            scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        }
        assert!(
            state.loop_iters[redo_slot] >= 1,
            "loop_iters must increment each time LoopRedo fires"
        );
    }

    #[test]
    fn scheduler_tick_no_progress_returns_zero_does_not_spin() {
        let ast = PowlAstNode::Sequence(vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")]);
        let tape = compile_powl(&ast).unwrap();
        let mut state = PowlRunState::new(&tape);
        // Manually force check_mask to op1 without having fired op0.
        state.done_mask = 0;
        state.check_mask = 0b10;
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert_eq!(fs.0, 0, "no op fires when predecessor not satisfied");
    }

    #[test]
    fn scheduler_tick_completes_within_bounded_ticks() {
        let ast = PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]);
        let tape = compile_powl(&ast).unwrap();
        let max_ticks = (tape.len as u32) * 2;
        let mut state = PowlRunState::new(&tape);
        for _ in 0..max_ticks {
            if state.check_mask == 0 {
                break;
            }
            scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        }
        assert_eq!(
            state.check_mask, 0,
            "scheduler must terminate within 2*len ticks"
        );
    }

    #[test]
    fn pred_mask_zero_means_entry_fires_on_first_tick() {
        let ast = PowlAstNode::Atom("entry");
        let tape = compile_powl(&ast).unwrap();
        assert_eq!(tape.ops[0].pred_mask, 0, "entry op must have pred_mask=0");
        let mut state = PowlRunState::new(&tape);
        let fs = scheduler_tick(&tape.ops[..tape.len as usize], &mut state);
        assert_ne!(
            fs.0 & 1,
            0,
            "op with pred_mask=0 must fire on the very first tick"
        );
    }

    // ---------------------------------------------------------------------------
    // Proptests
    // ---------------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_binary_mask_times_1_bit_is_correct(is_redo_bool: bool, fire_nz_bool: bool) {
            let is_redo: u64 = if is_redo_bool { u64::MAX } else { 0 };
            let fire_nz: u64 = if fire_nz_bool { u64::MAX } else { 0 };
            let active = is_redo & fire_nz;
            let increment = (active & 1) as u8;
            if active == u64::MAX {
                prop_assert_eq!(increment, 1u8, "MAX gives 1, got {}", increment);
            } else {
                prop_assert_eq!(increment, 0u8, "0 gives 0, got {}", increment);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_pred_satisfied_iff_all_required_bits_in_done(done: u64, required: u64) {
            let result = pred_satisfied(done, required);
            let expected = if required & !done == 0 { u64::MAX } else { 0 };
            prop_assert_eq!(result, expected,
                "pred_satisfied({:#018x}, {:#018x}) = {:#018x}, expected {:#018x}",
                done, required, result, expected);
        }

        #[test]
        fn prop_kind_mask_equals_max_iff_equal(a in 0u8..5, b in 0u8..5) {
            // OpKind variants are 0..=4; kind_mask should return MAX iff discriminants are equal.
            // Test the bit-arithmetic formula directly (same as kind_mask body).
            let diff = a ^ b;
            let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
            let mask = nz.wrapping_sub(1);
            if a == b {
                prop_assert_eq!(mask, u64::MAX,
                    "kind_mask formula: equal discriminants ({}) must yield MAX, got {:#018x}", a, mask);
            } else {
                prop_assert_eq!(mask, 0u64,
                    "kind_mask formula: unequal discriminants ({} vs {}) must yield 0, got {:#018x}", a, b, mask);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Concurrency-aware scheduling (additive)
    // -------------------------------------------------------------------------

    mod concurrency_gated {
        use super::*;
        use crate::tape::v2::CompiledNonFace;
        use bcinr_mfw_ir::Digest;

        /// Runs both `scheduler_tick` and `scheduler_tick_guarded` (with
        /// `StableMaximalSelector` + an empty guard table) from identical
        /// starting states, tick by tick, and asserts every observable
        /// piece of state stays in lockstep. This is the required proof
        /// that the concurrency-aware scheduling path is additive: an
        /// empty guard table must fire the *exact same set*, every tick,
        /// that the pre-existing `scheduler_tick` fires.
        fn assert_guarded_matches_plain(tape: &PowlTape) {
            let mut plain_state = PowlRunState::new(tape);
            let mut guarded_state = PowlRunState::new(tape);
            let mut selector = StableMaximalSelector;
            let guards = ConcurrencyGuardTable::empty();

            for tick in 0..20 {
                let plain_done = plain_state.check_mask == 0 && plain_state.active_mask == 0;
                let guarded_done = guarded_state.check_mask == 0 && guarded_state.active_mask == 0;
                assert_eq!(
                    plain_done, guarded_done,
                    "tick {tick}: plain/guarded disagree on termination"
                );
                if plain_done {
                    break;
                }

                let plain_fired = scheduler_tick(&tape.ops[..tape.len as usize], &mut plain_state);
                let guarded_fired = scheduler_tick_guarded(
                    &tape.ops[..tape.len as usize],
                    &mut guarded_state,
                    &mut selector,
                    &guards,
                );

                assert_eq!(
                    plain_fired.0, guarded_fired.0,
                    "tick {tick}: fired sets diverged (plain={:#018x}, guarded={:#018x})",
                    plain_fired.0, guarded_fired.0
                );
                assert_eq!(
                    plain_state.done_mask, guarded_state.done_mask,
                    "tick {tick}: done_mask diverged"
                );
                assert_eq!(
                    plain_state.check_mask, guarded_state.check_mask,
                    "tick {tick}: check_mask diverged"
                );
                assert_eq!(
                    plain_state.choice_taken, guarded_state.choice_taken,
                    "tick {tick}: choice_taken diverged"
                );
                assert_eq!(
                    plain_state.loop_iters, guarded_state.loop_iters,
                    "tick {tick}: loop_iters diverged"
                );
            }
        }

        #[test]
        fn empty_guards_matches_plain_tick_for_linear_chain() {
            let ast = PowlAstNode::Sequence(vec![
                PowlAstNode::Atom("a"),
                PowlAstNode::Atom("b"),
                PowlAstNode::Atom("c"),
                PowlAstNode::Atom("d"),
                PowlAstNode::Atom("e"),
            ]);
            let tape = compile_powl(&ast).unwrap();
            assert_guarded_matches_plain(&tape);
        }

        #[test]
        fn empty_guards_matches_plain_tick_for_parallel_ops() {
            // The multi-bit-ready-set case: two ops become ready on the
            // same tick, forcing a real ready set of size > 1 through the
            // gating logic.
            let ast = PowlAstNode::PartialOrder {
                children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
                edges: vec![],
            };
            let tape = compile_powl(&ast).unwrap();
            assert_guarded_matches_plain(&tape);
        }

        #[test]
        fn empty_guards_matches_plain_tick_for_xor_choice() {
            let ast =
                PowlAstNode::XorChoice(vec![PowlAstNode::Atom("left"), PowlAstNode::Atom("right")]);
            let tape = compile_powl(&ast).unwrap();
            assert_guarded_matches_plain(&tape);
        }

        #[test]
        fn empty_guards_matches_plain_tick_for_bounded_loop() {
            let ast = PowlAstNode::Loop {
                body: Box::new(PowlAstNode::Atom("body")),
                redo: Box::new(PowlAstNode::Atom("redo")),
                max_iters: 3,
            };
            let tape = compile_powl(&ast).unwrap();
            assert_guarded_matches_plain(&tape);
        }

        /// The concurrency guard must have a real effect: a nonface
        /// covering both parallel ops forbids them from firing on the same
        /// tick, deferring one to a later tick — but both must eventually
        /// fire (the guard defers, it never permanently starves).
        #[test]
        fn nonempty_guard_defers_a_forbidden_pair_but_both_eventually_fire() {
            let ast = PowlAstNode::PartialOrder {
                children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
                edges: vec![],
            };
            let tape = compile_powl(&ast).unwrap();

            let guards = ConcurrencyGuardTable {
                nonfaces: vec![CompiledNonFace {
                    members: EventSet::empty().with(0).with(1),
                    witness_digest: Digest::hash(b"a-b-conflict"),
                }],
            };
            let mut state = PowlRunState::new(&tape);
            let mut selector = StableMaximalSelector;

            let fs1 = scheduler_tick_guarded(
                &tape.ops[..tape.len as usize],
                &mut state,
                &mut selector,
                &guards,
            );
            assert_eq!(
                fs1.0.count_ones(),
                1,
                "guard must defer one of the two conflicting ops on tick 1, got fired={:#018x}",
                fs1.0
            );
            assert!(
                fs1.0 == 0b01 || fs1.0 == 0b10,
                "exactly one of a (bit0) or b (bit1) must fire, got {:#018x}",
                fs1.0
            );

            let mut total_fired = fs1.0;
            for _ in 0..10 {
                if state.check_mask == 0 && state.active_mask == 0 {
                    break;
                }
                let fs = scheduler_tick_guarded(
                    &tape.ops[..tape.len as usize],
                    &mut state,
                    &mut selector,
                    &guards,
                );
                total_fired |= fs.0;
            }
            assert_eq!(
                total_fired & 0b11,
                0b11,
                "both conflicting ops must eventually fire (deferred, not dropped)"
            );
            assert_eq!(
                state.check_mask, 0,
                "run must terminate (join must eventually fire once both preds are done)"
            );
        }

        #[test]
        fn stable_maximal_selector_selects_everything_when_admitted() {
            let ready = EventSet::empty().with(0).with(1).with(2);
            let guards = ConcurrencyGuardTable::empty();
            let mut selector = StableMaximalSelector;
            let selected = selector.select_checked(&ready, &guards);
            assert_eq!(selected, ready);
        }

        #[test]
        fn stable_maximal_selector_respects_a_nonface() {
            // {0,1,2} is forbidden together. Greedy build-up over
            // iter_stable() (ascending) admits 0, then admits 1 (neither
            // {0} nor {0,1} contains the full nonface {0,1,2} as a
            // subset), then rejects 2 (adding it would make the candidate
            // equal to the nonface itself).
            let ready = EventSet::empty().with(0).with(1).with(2);
            let guards = ConcurrencyGuardTable {
                nonfaces: vec![CompiledNonFace {
                    members: ready,
                    witness_digest: Digest::hash(b"abc"),
                }],
            };
            let mut selector = StableMaximalSelector;
            let selected = selector.select_checked(&ready, &guards);
            assert_eq!(selected, EventSet::empty().with(0).with(1));
            assert!(guards.admits(&selected));
        }

        #[test]
        fn mask_event_set_round_trip() {
            let mask = 0b1011_0101u64;
            let set = mask_to_event_set(mask);
            assert_eq!(event_set_to_mask(&set), mask);
        }
    }
}
