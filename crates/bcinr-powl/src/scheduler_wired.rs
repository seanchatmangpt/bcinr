//! Wired scheduler — bcinr-logic primitives driving the POWL hot path.
//!
//! # Branchless invariant
//!
//! The hot path in `petri_tick` (post-fire loop) is fully branchless for per-op
//! kind dispatch: `XorDispatch` and `LoopRedo` effects are applied via
//! `apply_xor_dispatch` / `apply_loop_redo` predicated helpers — no `if`/`match`
//! inside the per-op body generates a conditional branch instruction.
//!
//! `build_transition_arrays` likewise computes `effective_pred` for `Join` ops
//! with `kind_mask` arithmetic, not a `match` arm.
//!
//! The outer `while fm != 0` bit-scan loop in `petri_tick` is the standard CTZ
//! idiom; its iteration count equals `firing_mask.count_ones()`, not a predicate.
//!
//! # What is NOT branchless
//!
//! - The early-exit `if candidates == 0 { return 0; }` guard (one branch per tick,
//!   unavoidable without changing the public API).
//! - The `if let Some(r)` / `if let Some(er)` ring-push guards (cold path, optional
//!   feature; not on the hot firing path).
//! - The `match PriorityPetriEngine::new_checked(...)` error guard (one branch per
//!   tick, cold path).
//! - `dispatch_successors`: inner `if let Some(r)` is structural, not per-kind.
//!
//! # Primitives from `bcinr-logic`
//!
//! - **`PriorityPetriEngine`** — branchless priority-ordered transition firing
//!   for ≤64 ops. Replaces the `trailing_zeros` bit-scan loop in `scheduler_tick`.
//!   Already `#[inline(always)]`, already proven.
//!
//! - **`TimeWheel<N>`** — O(1) SLA deadline detection. `schedule(delay, op_bit)`
//!   at compile time; `tick()` fires the due-mask in ~2ns regardless of how many
//!   ops have active SLAs. Eliminates the O(ops) deadline scan that would otherwise
//!   dominate the SLA watchdog path.
//!
//! - **`LockFreeMpmcRing<WorkItem, N>`** — CAS-based dispatch ring for parallel SPO.
//!   When multiple branches are enabled simultaneously, they are pushed to the ring
//!   and consumed by rayon/thread workers without mutex overhead. ~10ns per push/pop.
//!
//! - **`WcetFiber<TICKS>`** — context-switch for long-running activities. When an
//!   Activity op would block (I/O, DB), the fiber suspends with `context_switch` and
//!   the scheduler tick continues firing other enabled ops. Overlap I/O with compute.
//!
//! - **`prefix_xor_u64x8` / `union_u64_slices`** — bulk check_mask propagation for
//!   PowlTapeLarge (>64 ops). One call to `union_u64_slices` folds 8 successor words.
//!
//! # Concurrency-aware scheduling (additive)
//!
//! [`petri_tick`] itself is untouched by this section — every existing caller
//! keeps firing every op in `PriorityPetriEngine`'s computed firing mask every
//! tick, exactly as before, with zero code-path change. [`petri_tick_guarded`]
//! is a new, separate entry point mirroring [`crate::scheduler::scheduler_tick_guarded`]:
//! it inserts a [`crate::scheduler::ConcurrencySelector`]-gated selection step
//! between "compute the ready set" and "actually fire," so a fired set
//! reported by this hot path is checked against a
//! [`crate::tape::v2::ConcurrencyGuardTable`] (an
//! [`bcinr_mfw_ir::ExecutableConcurrencyComplex`] compiled to per-tick guard
//! form) before it is ever reported as fired, closing the gap that made this
//! module's hot path the one wired scheduler entry point with no
//! `ConcurrencySelector`/`ConcurrencyGuardTable`/`EventSet` anywhere in it.
//! With [`crate::tape::v2::ConcurrencyGuardTable::empty`] and
//! [`crate::scheduler::StableMaximalSelector`], selection always chooses the
//! entire ready set, so `petri_tick_guarded` provably degenerates to calling
//! `petri_tick` — see `tests::concurrency_gated::empty_guards_matches_plain_tick_*`
//! below, checked tick-by-tick across two tape shapes, plus a nonempty-guard
//! test proving the gate has a real deferring effect, not just plumbing.

use bcinr_logic::{
    bitset::union_u64_slices,
    models::petri::KBitSet,
    patterns::{
        deterministic_mpmc::LockFreeMpmcRing, swar_petri::PriorityPetriEngine,
        time_wheel::TimeWheel, wcet_fiber::WcetFiber,
    },
    scan::prefix_xor_u64x8,
};

use crate::scheduler::{pred_satisfied, ConcurrencySelector};
use crate::tape::v2::ConcurrencyGuardTable;
use crate::tape::{OpKind, Powl64Op};
use bcinr_mfw_ir::EventSet;

// ---------------------------------------------------------------------------
// Branchless helpers (duplicated from scheduler.rs — private, proven)
// ---------------------------------------------------------------------------

/// Branchless OpKind equality mask.
///
/// Returns `u64::MAX` when `kind == target`, `0` otherwise.
///
/// Proof: `diff = (kind as u8) ^ (target as u8)`. Zero iff equal.
/// `(diff | diff.wrapping_neg()) >> 7` sets bit 7 iff diff != 0. Then
/// `nz.wrapping_sub(1)` maps 0 → u64::MAX, 1 → 0. ∎
#[inline(always)]
fn kind_mask(kind: OpKind, target: OpKind) -> u64 {
    let diff = (kind as u8) ^ (target as u8);
    let nz = (((diff | diff.wrapping_neg()) >> 7) & 1) as u64;
    nz.wrapping_sub(1)
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
    let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
    let active = is_xor & fire_nz;

    let chosen = op.branch_mask & op.branch_mask.wrapping_neg(); // lowest set bit
    let suppressed = op.branch_mask & !chosen;

    *choice_taken |= chosen & active;
    suppressed & active
}

/// Branchless LoopRedo handler.
///
/// When `op.kind == LoopRedo` AND `fire_mask != 0`, resets body entries in
/// done (so they can fire again), adds them to check, and increments the
/// per-slot loop counter by exactly 0 or 1 — no branch.
///
/// Returns `(done_clear_mask, check_delta)`.
#[inline(always)]
fn apply_loop_redo(op: &Powl64Op, fire_mask: u64, loop_iter: &mut u8) -> (u64, u64) {
    let is_redo = kind_mask(op.kind, OpKind::LoopRedo);
    let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
    let active = is_redo & fire_nz;

    *loop_iter = loop_iter.saturating_add((active & 1) as u8);

    let body = op.succ_mask & active;
    (body, body)
}

// ---------------------------------------------------------------------------
// PowlPetriState — hot state using KBitSet<1> (64-op tapes)
// ---------------------------------------------------------------------------

/// Scheduler hot state backed by `KBitSet<1>` (one u64 word = 64 ops).
///
/// Replaces `PowlRunState` for the ≤64-op fast path. Layout is 5×u64 = 40 bytes,
/// fitting in less than one cache line.
///
/// `Clone` is required by [`petri_tick_guarded`]'s preview step (it clones
/// `state`, runs an unguarded [`petri_tick`] on the clone to discover the
/// tick's ready set, then discards the clone) — cloning does not consume or
/// otherwise mutate the original.
#[derive(Clone)]
#[repr(C, align(64))]
pub struct PowlPetriState {
    /// Tokens currently placed (ops that have fired and completed).
    pub done: KBitSet<1>,
    /// Candidates for this tick (op bits whose predecessors are all in `done`).
    pub check: KBitSet<1>,
    /// XOR branch choice: bit = chosen branch entry.
    pub choice_taken: u64,
    /// SLA deadline wheel — fires op-index bits when their deadlines expire.
    pub sla_wheel: TimeWheel<256>,
    /// Per-op loop iteration counters.
    pub loop_iters: [u8; 64],
    /// Bitmask of ops that breached their SLA this tick.
    pub sla_breached: u64,
}

impl PowlPetriState {
    pub fn new(entry_mask: u64) -> Self {
        Self {
            done: KBitSet { words: [0u64] },
            check: KBitSet {
                words: [entry_mask],
            },
            choice_taken: 0,
            sla_wheel: TimeWheel::new(),
            loop_iters: [0u8; 64],
            sla_breached: 0,
        }
    }

    /// Schedule an SLA deadline for `op_bit` to fire in `delay` ticks.
    #[inline(always)]
    pub fn schedule_sla(&mut self, delay: usize, op_bit: u32) {
        self.sla_wheel.schedule(delay, op_bit);
    }
}

// ---------------------------------------------------------------------------
// WorkItem — dispatched to LockFreeMpmcRing for parallel SPO branches
// ---------------------------------------------------------------------------

/// An enabled op dispatched to the parallel ring.
#[derive(Clone, Copy, Default)]
pub struct WorkItem {
    /// Op index (0..63).
    pub op_idx: u32,
    /// The tape op's succ_mask (for check_mask update on completion).
    pub succ_mask: u64,
}

// ---------------------------------------------------------------------------
// EventWorkItem — dispatched to event ring for off-hot-path BLAKE3 hashing
// ---------------------------------------------------------------------------

/// One fired-op event, pushed to the event ring after each fire.
/// The ring drains asynchronously via `ReceiptWorker`; BLAKE3 never called
/// inside `petri_tick`.
///
/// Carries two distinct masks, deliberately not collapsed into one field:
/// `op_trace_so_far` drives [`crate::receipt_worker::ReceiptWorker`]'s
/// run-completion detection (which must see a strictly growing value across
/// a multi-op tick's several events, or the *first* event of a tick that
/// completes the run would falsely look like the whole run's last event —
/// see `tick_fired_mask_never_collapses_progressive_completion_detection`
/// in this module's tests for the regression this guards against);
/// `tick_fired_mask` drives admissibility checking (which needs the whole
/// tick's joint FireSet, constant across every event of that tick, not a
/// partial view of it).
#[derive(Clone, Copy, Default)]
pub struct EventWorkItem {
    /// Op index (0..63) that fired.
    pub op_idx: u32,
    /// Run-id of the workflow instance.
    pub run_id: u64,
    /// This **tick's** fired-ops-so-far bitmask: starts partial and grows
    /// (within one tick's several events, in firing order) until it equals
    /// `tick_fired_mask` on the last event of that tick. Despite the name,
    /// this is scoped to one tick, not the whole run — `petri_tick`'s
    /// `fired_ops_accumulator` is freshly zeroed every call. Consumers that
    /// want "has this run finished" should OR every drained event's value
    /// together across ticks (as `ReceiptWorker::drain` does), not compare
    /// a single event's value to a run-wide total.
    pub op_trace_so_far: u64,
    /// The **complete** set of op bits that fired during the same tick as
    /// `op_idx` — i.e. this tick's `PetriTickResult::fired_ops`. Every
    /// `EventWorkItem` pushed for the same tick carries the identical value,
    /// unlike `op_trace_so_far`.
    ///
    /// This is what [`crate::receipt_worker::ReceiptWorker::drain`] checks
    /// against a `ConcurrencyGuardTable` before letting the event
    /// contribute to a sealed receipt — admissibility is a property of a
    /// *tick's joint FireSet*, not of any single op in isolation, so this
    /// field has to carry the whole set, not a partial view of it.
    pub tick_fired_mask: u64,
    /// `OpKind` discriminant for receipt classification.
    pub kind_tag: u8,
}

// ---------------------------------------------------------------------------
// PowlPetriEngine — wired tick
// ---------------------------------------------------------------------------

/// Max parallel branches dispatched to the ring per tick.
const RING_CAPACITY: usize = 64;

/// Build per-transition input/output KBitSets and op-index table for `PriorityPetriEngine`.
///
/// Returns `(inputs, outputs, op_indices)` where each live slot `t` corresponds
/// to one enabled candidate op.
#[inline(always)]
fn build_transition_arrays(
    tape: &[Powl64Op],
    candidates: u64,
    choice_taken: u64,
) -> ([KBitSet<1>; 64], [KBitSet<1>; 64], [u32; 64]) {
    let mut inputs = [KBitSet::<1> { words: [!0u64] }; 64];
    let mut outputs = [KBitSet::<1> { words: [0] }; 64];
    let mut op_indices = [u32::MAX; 64];
    let mut t = 0usize;
    let mut bits = candidates;

    while bits != 0 && t < 64 {
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;

        let op = &tape[i];
        let op_bit = 1u64 << i;

        // Effective pred for XOR-join: unchosen branches are virtually done.
        // Branchless: kind_mask selects between join_effective and pred_mask.
        // For Join: pred_mask & choice_taken (unchosen slots excluded).
        // For all other kinds: pred_mask unchanged.
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        // POWL done-set is monotone: tokens never leave.
        // output = effective_pred | op_bit so net effect is: state gains op_bit.
        inputs[t] = KBitSet {
            words: [effective_pred],
        };
        outputs[t] = KBitSet {
            words: [effective_pred | op_bit],
        };
        op_indices[t] = i as u32;
        t += 1;
    }

    (inputs, outputs, op_indices)
}

/// Push enabled successors of a fired op to the parallel dispatch ring.
#[inline(always)]
fn dispatch_successors(
    tape: &[Powl64Op],
    op_idx: usize,
    new_done: u64,
    ring: Option<&LockFreeMpmcRing<WorkItem, RING_CAPACITY>>,
) {
    if let Some(r) = ring {
        let mut succ_bits = tape[op_idx].succ_mask & !new_done;
        while succ_bits != 0 {
            let s = succ_bits.trailing_zeros() as usize;
            succ_bits &= succ_bits - 1;
            r.push_t1(WorkItem {
                op_idx: s as u32,
                succ_mask: tape[s].succ_mask,
            });
        }
    }
}

/// Result returned by `petri_tick`.
pub struct PetriTickResult {
    /// Bitmask of ops fired this tick.
    pub fired_ops: u64,
    /// Number of event ring overflow events encountered this tick.
    pub event_overflow_count: u32,
}

/// Drives one scheduler tick using `PriorityPetriEngine` branchless core.
///
/// # What changes vs `scheduler_tick`
///
/// | Old | New |
/// |-----|-----|
/// | Manual `trailing_zeros` bit-scan | `PriorityPetriEngine::step()` — proven, `inline(always)` |
/// | No SLA detection | `TimeWheel::tick()` fires due mask, ORed into `sla_breached` |
/// | No parallel dispatch | `LockFreeMpmcRing::push_t1` for concurrent branches |
///
/// Returns a `PetriTickResult` with the bitmask of ops fired and overflow count.
#[inline(always)]
pub fn petri_tick(
    tape: &[Powl64Op],
    state: &mut PowlPetriState,
    ring: Option<&LockFreeMpmcRing<WorkItem, RING_CAPACITY>>,
    event_ring: Option<&LockFreeMpmcRing<EventWorkItem, RING_CAPACITY>>,
    run_id: u64,
) -> PetriTickResult {
    let mut overflow_count: u32 = 0;

    // --- SLA wheel: drain any expired deadlines ---
    let sla_due = state.sla_wheel.tick();
    state.sla_breached |= sla_due;

    let done = state.done.words[0];
    let candidates = state.check.words[0] & !done;

    if candidates == 0 {
        return PetriTickResult {
            fired_ops: 0,
            event_overflow_count: 0,
        };
    }

    // Build per-transition arrays for PriorityPetriEngine (stack-allocated, no heap).
    let (inputs, outputs, op_indices) =
        build_transition_arrays(tape, candidates, state.choice_taken);

    // Construct engine over the live slice. The engine fires transitions in
    // priority order (index 0 first) and accumulates the firing_mask.
    // SAFETY-note: new_checked validates TRANSITIONS <= 64.
    let initial = KBitSet { words: [done] };
    let mut engine = match PriorityPetriEngine::<1, 64>::new_checked(initial, inputs, outputs) {
        Ok(e) => e,
        Err(_) => {
            return PetriTickResult {
                fired_ops: 0,
                event_overflow_count: 0,
            }
        }
    };

    let firing_mask_64 = engine.step();

    // After step(), engine.state.current IS the new marking (monotone accumulation
    // guaranteed by output = effective_pred | op_bit — pred tokens re-placed).
    // Use it as new_done instead of accumulating from firing_mask.
    let mut new_done = engine.state.current.words[0];
    let mut new_check = 0u64;
    let mut fired_ops_accumulator = 0u64;

    // The complete set of ops firing this tick is already known here (before
    // the per-op loop below applies XorDispatch/LoopRedo on top of it):
    // engine.step() only ever adds op bits for transitions it actually fired,
    // so this delta is exactly what fired_ops_accumulator will equal once the
    // loop finishes. Computed upfront so every EventWorkItem pushed below can
    // carry the tick's whole FireSet, not just the one op it names.
    let tick_fired_mask = new_done & !done;

    let mut fm = firing_mask_64;
    while fm != 0 {
        let t_idx = fm.trailing_zeros() as usize;
        fm &= fm - 1;

        // Skip sentinel slots (unused transitions with input=!0 that never fire,
        // but guard defensively).
        if op_indices[t_idx] == u32::MAX {
            continue;
        }
        let i = op_indices[t_idx] as usize;
        let op = &tape[i];
        let op_bit = 1u64 << i;

        fired_ops_accumulator |= op_bit;
        new_done |= op_bit;
        new_check |= op.succ_mask;

        // Off hot-path: push fire event to event_ring for ReceiptWorker to drain.
        // BLAKE3 is never computed here — only a cheap push_t1 (~10 ns).
        if let Some(er) = event_ring {
            let pushed = er.push_t1(EventWorkItem {
                op_idx: i as u32,
                run_id,
                op_trace_so_far: fired_ops_accumulator,
                tick_fired_mask,
                kind_tag: op.kind as u8,
            });
            if pushed == 0 {
                overflow_count += 1;
            }
        }

        // XorDispatch: branchless — pick lowest-index branch, suppress others.
        new_done |= apply_xor_dispatch(op, op_bit, &mut state.choice_taken);

        // LoopRedo: branchless — re-enable body entries.
        let (redo_clear, redo_check) = apply_loop_redo(op, op_bit, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;

        // Parallel dispatch: push enabled successors to the ring.
        dispatch_successors(tape, i, new_done, ring);
    }

    debug_assert_eq!(
        fired_ops_accumulator, tick_fired_mask,
        "tick_fired_mask (computed upfront from engine.state.current) must equal \
         fired_ops_accumulator (accumulated by the per-op loop) — every EventWorkItem \
         pushed above assumed these stay equal"
    );

    state.done.words[0] = new_done;
    state.check.words[0] = new_check & !new_done;

    PetriTickResult {
        fired_ops: fired_ops_accumulator,
        event_overflow_count: overflow_count,
    }
}

// ---------------------------------------------------------------------------
// petri_tick_guarded — concurrency-complex-checked hot path (additive)
// ---------------------------------------------------------------------------

/// Convert a `u64` tape-slot bitmask (as used throughout `petri_tick`) into
/// an `EventSet` (as used by `ConcurrencySelector`/`ConcurrencyGuardTable`).
///
/// Duplicated from `crate::scheduler` (private there) — same convention as
/// this module's other duplicated branchless helpers (see module docs).
fn mask_to_event_set(mask: u64) -> EventSet {
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

/// Drives one scheduler tick, gating which of the ready ops actually fire
/// through a [`ConcurrencySelector`] and [`ConcurrencyGuardTable`] — the
/// `petri_tick` counterpart to
/// [`crate::scheduler::scheduler_tick_guarded`].
///
/// # Why this exists
///
/// [`petri_tick`] fires every op `PriorityPetriEngine::step` computes as
/// enabled, every tick, unconditionally — it never consults an
/// `ExecutableConcurrencyComplex`. A `FireSet` this hot path reports is not
/// thereby known to be a member of the concurrency complex; it is only known
/// to satisfy each fired op's own `pred_mask`. `petri_tick_guarded` closes
/// that gap for this crate's wired hot path exactly the way
/// `scheduler_tick_guarded` already closes it for the plain hot path.
///
/// # Protocol
///
/// 1. **Preview** (no mutation of `state`): clone `state` and run
///    [`petri_tick`] on the clone with `ring`/`event_ring` both `None` (so
///    the preview cannot double-dispatch to the real ring or double-push a
///    receipt event) to discover exactly which ops would fire absent any
///    gating — this tick's ready set. The clone's `sla_wheel` absorbs the
///    one-tick advance during preview and is then discarded; the real
///    `state.sla_wheel` is untouched by this step.
/// 2. **Select**: `selector.select_checked(ready, guards)` chooses the
///    admissible subset that may actually fire.
/// 3. **Fast path**: if selection changed nothing (`selected == ready` —
///    always true for an empty guard table with `StableMaximalSelector`),
///    delegate to the real, unguarded [`petri_tick`] on `state` directly, so
///    the observable state transition (including the real `ring`/
///    `event_ring`/`sla_wheel` side effects) is provably identical to the
///    unguarded path.
/// 4. **Divergent path**: otherwise, advance `state.sla_wheel` for real
///    exactly once, then recompute per-candidate enablement directly with
///    `pred_satisfied` (a private helper in this module, hence a plain code
///    span rather than a doc-link — the same formula `SwarMarking::try_fire`
///    uses internally, see that function's doc comment), gated on
///    `selected_bit`, so a candidate excluded from `selected` neither
///    contributes its `op_bit` to this tick's `done` set nor unblocks a
///    later-priority candidate in the same tick that depended on it. Ready-
///    but-unselected ops are carried forward into next tick's `check` set
///    (deferred, never dropped).
///
/// # Complexity
///
/// O(`candidates.count_ones()`) for the divergent path's own loop, plus
/// whatever `selector.select_checked` costs (see
/// `StableMaximalSelector::select`'s own `# Complexity` note) and one full
/// (discarded) `petri_tick` call for the preview step — data-dependent on
/// how many ops are ready this tick, not O(1), unlike the individual
/// branchless primitives it is built from.
pub fn petri_tick_guarded<S: ConcurrencySelector>(
    tape: &[Powl64Op],
    state: &mut PowlPetriState,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
    ring: Option<&LockFreeMpmcRing<WorkItem, RING_CAPACITY>>,
    event_ring: Option<&LockFreeMpmcRing<EventWorkItem, RING_CAPACITY>>,
    run_id: u64,
) -> PetriTickResult {
    // Step 1: dry-run preview on a clone — does not touch the real state,
    // the real ring, or the real event ring.
    let mut preview = state.clone();
    let would_fire = petri_tick(tape, &mut preview, None, None, run_id);
    let ready_mask = would_fire.fired_ops;
    let ready = mask_to_event_set(ready_mask);

    // Step 2: select which of the ready ops may actually fire.
    let selected = selector.select_checked(&ready, guards);
    let selected_mask = event_set_to_mask(&selected);

    // Step 3: fast path — nothing was gated away (also covers ready_mask==0,
    // since a subset of the empty set is always the empty set).
    if selected_mask == ready_mask {
        return petri_tick(tape, state, ring, event_ring, run_id);
    }

    // Step 4: divergent path. Advance the real SLA wheel exactly once (the
    // preview above only advanced the discarded clone's wheel).
    let mut overflow_count: u32 = 0;
    let sla_due = state.sla_wheel.tick();
    state.sla_breached |= sla_due;

    let done = state.done.words[0];
    let mut new_done = done;
    let mut new_check = 0u64;
    let mut fired_ops_accumulator = 0u64;

    let mut bits = state.check.words[0] & !done;
    while bits != 0 {
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;

        let op = &tape[i];
        let op_bit = 1u64 << i;

        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & state.choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        // Same enablement formula PriorityPetriEngine's SwarMarking::try_fire
        // uses internally (`current.satisfies(input)`), recomputed here
        // against `new_done` as it accumulates across this loop so a
        // deferred (unselected) predecessor correctly fails to unblock a
        // later same-tick successor — matching engine's sequential,
        // priority-ordered firing semantics exactly.
        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = sat & 1;
        let selected_bit = (selected_mask >> i) & 1;
        let fire_mask = u64::wrapping_sub(0, sat_bit & selected_bit) & op_bit;

        fired_ops_accumulator |= fire_mask;
        new_done |= fire_mask;

        let fired_this = fire_mask >> i;
        new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);

        new_done |= apply_xor_dispatch(op, fire_mask, &mut state.choice_taken);

        let (redo_clear, redo_check) = apply_loop_redo(op, fire_mask, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;

        // Cold path (per module docs): only an op that actually fired this
        // tick releases successors to the parallel ring — a deferred
        // (ready-but-unselected) candidate must not have its successors
        // dispatched. Receipt-event pushes are deferred to a post-loop pass
        // below (see comment there for why).
        if fired_this == 1 {
            dispatch_successors(tape, i, new_done, ring);
        }
    }

    // Carry forward ready-but-unselected ops so they are reconsidered next
    // tick instead of being lost.
    new_check |= ready_mask & !selected_mask;

    state.done.words[0] = new_done;
    state.check.words[0] = new_check & !new_done;

    // Push receipt events only after the loop above finishes, so
    // `tick_fired_mask` can carry this tick's *complete* FireSet
    // (`fired_ops_accumulator`'s final value) on every event — unlike the
    // engine-based `petri_tick` path, this divergent (gated) path cannot
    // know the tick's final FireSet until every candidate has been
    // considered, since a deferred candidate can prevent a later, dependent
    // candidate from firing in the same tick.
    //
    // `op_trace_so_far` is reconstructed here to match what an inline push
    // would have produced: ascending bit order is this loop's own firing
    // order (same `trailing_zeros` bit-scan idiom used above), so
    // re-scanning `fired_ops_accumulator` in that same order and OR-ing
    // progressively reproduces the exact partial-then-complete sequence a
    // caller would see from `petri_tick`'s inline push — this must stay
    // partial-then-growing, not constant, or the *first* event of a tick
    // that completes a run would look like the run's completion by itself,
    // firing `ReceiptWorker`'s seal/refuse check once per fired op instead
    // of once per tick.
    if let Some(er) = event_ring {
        let mut remaining = fired_ops_accumulator;
        let mut op_trace_so_far = 0u64;
        while remaining != 0 {
            let i = remaining.trailing_zeros() as usize;
            remaining &= remaining - 1;
            op_trace_so_far |= 1u64 << i;
            let pushed = er.push_t1(EventWorkItem {
                op_idx: i as u32,
                run_id,
                op_trace_so_far,
                tick_fired_mask: fired_ops_accumulator,
                kind_tag: tape[i].kind as u8,
            });
            if pushed == 0 {
                overflow_count += 1;
            }
        }
    }

    PetriTickResult {
        fired_ops: fired_ops_accumulator,
        event_overflow_count: overflow_count,
    }
}

// ---------------------------------------------------------------------------
// Large-tape bulk check_mask propagation — scan.prefix_xor + bitset.union
// ---------------------------------------------------------------------------

/// For PowlTapeLarge (>64 ops): given `fired_ops_bits` (up to 8 words),
/// fold all their succ_mask words into `check_mask` using `union_u64_slices`.
///
/// Replaces: `for bit in fired { check |= tape[bit].succ_mask_words[..]; }`
/// With: `union_u64_slices(check, &succ_fold)` — one SIMD call.
#[inline(always)]
pub fn propagate_check_mask_large(
    fired_words: [u64; 8],   // bitmask of fired ops (512-op space)
    succ_table: &[[u64; 8]], // succ_mask_words[op_idx]
    check_mask: &mut [u64; 8],
    done_mask: &[u64; 8],
) {
    // XOR-prefix to extract individual word contributions. Each word in
    // fired_words is independent, so prefix_xor gives us cumulative coverage.
    let covered = prefix_xor_u64x8(fired_words);

    // For each fired op, union its succ_mask into a fold buffer.
    let mut succ_fold = [0u64; 8];
    let bits_remaining = covered;
    for (word_idx, &word) in bits_remaining.iter().enumerate() {
        let mut w = word;
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            w &= w - 1;
            let op_idx = word_idx * 64 + bit;
            if op_idx < succ_table.len() {
                let mut tmp = succ_fold;
                union_u64_slices(&mut tmp, &succ_table[op_idx]);
                succ_fold = tmp;
            }
        }
    }

    // Merge into check_mask, excluding already-done ops.
    let mut check_arr = *check_mask;
    union_u64_slices(&mut check_arr, &succ_fold);
    // Mask out done ops.
    for i in 0..8 {
        check_arr[i] &= !done_mask[i];
    }
    *check_mask = check_arr;
}

// ---------------------------------------------------------------------------
// WcetFiber integration — suspend long activities off the hot tick
// ---------------------------------------------------------------------------

/// Fiber pool for Activities that need more than one tick to complete.
/// Each slot holds a suspended fiber and the op-index it serves.
pub struct FiberPool<const SLOTS: usize, const TICKS: usize> {
    pub fibers: [WcetFiber<TICKS>; SLOTS],
    pub op_indices: [u32; SLOTS],
    pub active_mask: u64, // bit i = slot i is running
}

impl<const SLOTS: usize, const TICKS: usize> FiberPool<SLOTS, TICKS> {
    pub const fn new() -> Self {
        // WcetFiber::new() is const
        Self {
            fibers: [const { WcetFiber::new() }; SLOTS],
            op_indices: [u32::MAX; SLOTS],
            active_mask: 0,
        }
    }

    /// Claim a free slot for `op_idx`. Returns slot index or `None` if full.
    #[inline(always)]
    pub fn claim(&mut self, op_idx: u32) -> Option<usize> {
        let free_slots = !self.active_mask & ((1u64 << SLOTS) - 1);
        if free_slots == 0 {
            return None;
        }
        let slot = free_slots.trailing_zeros() as usize;
        self.op_indices[slot] = op_idx;
        self.active_mask |= 1u64 << slot;
        // Reset fiber state for reuse.
        self.fibers[slot] = WcetFiber::new();
        Some(slot)
    }

    /// Release a completed slot, returning the op_idx it served.
    #[inline(always)]
    pub fn release(&mut self, slot: usize) -> u32 {
        let op_idx = self.op_indices[slot];
        self.op_indices[slot] = u32::MAX;
        self.active_mask &= !(1u64 << slot);
        op_idx
    }

    /// Advance all active fibers by one budget tick. Returns bitmask of
    /// slots that completed (all TICKS consumed → activity done).
    #[inline(always)]
    pub fn advance_all(&mut self, events: &[u32; TICKS]) -> u64 {
        let mut completed = 0u64;
        let mut active = self.active_mask;
        while active != 0 {
            let slot = active.trailing_zeros() as usize;
            active &= active - 1;
            let result = self.fibers[slot].execute_budget_fixed(events);
            // Convention: if all TICKS bits set → activity complete this epoch.
            let all_done = result == (1u64 << TICKS).wrapping_sub(1);
            let done_mask = 0u64.wrapping_sub(all_done as u64);
            completed |= done_mask & (1u64 << slot);
        }
        completed
    }
}

impl<const SLOTS: usize, const TICKS: usize> Default for FiberPool<SLOTS, TICKS> {
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
    use crate::{
        compiler::{compile_powl, PowlAstNode},
        tape::PowlTape,
    };

    fn tape_ops(tape: &PowlTape) -> Vec<Powl64Op> {
        tape.ops[..tape.len as usize].to_vec()
    }

    #[test]
    fn petri_tick_linear_chain_3() {
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
            PowlAstNode::Atom("c"),
        ]))
        .unwrap();
        let ops = tape_ops(&tape);
        let mut state = PowlPetriState::new(tape.entry_mask);
        let mut total_fired = 0u64;
        for _ in 0..10 {
            if state.check.words[0] == 0 {
                break;
            }
            total_fired += petri_tick(&ops, &mut state, None, None, 0)
                .fired_ops
                .count_ones() as u64;
        }
        assert_eq!(total_fired, 3, "all 3 ops must fire");
        assert_eq!(state.done.words[0], 0b111, "all done");
    }

    #[test]
    fn petri_tick_parallel_spo() {
        let tape = compile_powl(&PowlAstNode::PartialOrder {
            children: vec![
                PowlAstNode::Atom("p0"),
                PowlAstNode::Atom("p1"),
                PowlAstNode::Atom("p2"),
                PowlAstNode::Atom("p3"),
            ],
            edges: vec![],
        })
        .unwrap();
        let ops = tape_ops(&tape);
        let mut state = PowlPetriState::new(tape.entry_mask);
        let mut ticks = 0u32;
        let mut total = 0u64;
        while state.check.words[0] != 0 && ticks < 10 {
            total += petri_tick(&ops, &mut state, None, None, 0)
                .fired_ops
                .count_ones() as u64;
            ticks += 1;
        }
        // 4 parallel + 1 join = 5 ops total
        assert_eq!(total, 5, "all 5 ops fired");
    }

    #[test]
    fn time_wheel_sla_breach_detected() {
        let tape = compile_powl(&PowlAstNode::Atom("slow_op")).unwrap();
        let ops = tape_ops(&tape);
        let mut state = PowlPetriState::new(tape.entry_mask);
        // Schedule op 0's SLA to expire in 3 ticks.
        state.schedule_sla(3, 0);
        // Run 4 ticks (op fires on tick 1, SLA expires on tick 3).
        for _ in 0..4 {
            petri_tick(&ops, &mut state, None, None, 0);
        }
        // sla_breached should have bit 0 set after tick 3.
        assert_ne!(
            state.sla_breached & 1,
            0,
            "op 0 SLA breach should be recorded"
        );
    }

    #[test]
    fn fiber_pool_claim_release_advance() {
        let mut pool = FiberPool::<4, 8>::new();
        let slot = pool.claim(7).expect("slot available");
        assert_eq!(pool.op_indices[slot], 7);
        let events = [1u32; 8];
        let completed = pool.advance_all(&events);
        // All 8 ticks fired → all bits set in result → slot completed.
        assert_ne!(
            completed & (1 << slot),
            0,
            "fiber completes after full budget"
        );
        let op_idx = pool.release(slot);
        assert_eq!(op_idx, 7);
        assert_eq!(pool.active_mask & (1 << slot), 0);
    }

    #[test]
    fn propagate_check_mask_large_basic() {
        // Single fired op at index 0 with succ at index 1.
        let mut succ_table = vec![[0u64; 8]; 2];
        succ_table[0][0] = 0b10; // op 0 → op 1
        let fired_words = [0b1u64, 0, 0, 0, 0, 0, 0, 0];
        let done_mask = [0b1u64, 0, 0, 0, 0, 0, 0, 0];
        let mut check_mask = [0u64; 8];
        propagate_check_mask_large(fired_words, &succ_table, &mut check_mask, &done_mask);
        assert_eq!(check_mask[0], 0b10, "op 1 should be in check_mask");
    }

    // -----------------------------------------------------------------------
    // RING_CAPACITY — must be a power of two for CAS modular arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn ring_capacity_is_power_of_two() {
        assert!(
            RING_CAPACITY.is_power_of_two(),
            "RING_CAPACITY={} must be a power of two",
            RING_CAPACITY
        );
        let mask = RING_CAPACITY - 1;
        assert_eq!(RING_CAPACITY & mask, 0, "power-of-two: N & (N-1) must be 0");
    }

    #[test]
    fn petri_tick_result_has_fired_ops_field() {
        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();
        let ops = tape_ops(&tape);
        let mut state = PowlPetriState::new(tape.entry_mask);
        let result = petri_tick(&ops, &mut state, None, None, 0);
        assert_ne!(
            result.fired_ops, 0,
            "at least one op must fire on first tick"
        );
        assert_eq!(
            result.event_overflow_count, 0,
            "no event ring present, no overflow"
        );
    }

    // -------------------------------------------------------------------------
    // petri_tick_guarded — concurrency-complex-checked hot path (additive)
    // -------------------------------------------------------------------------

    mod concurrency_gated {
        use super::*;
        use crate::scheduler::StableMaximalSelector;
        use crate::tape::v2::CompiledNonFace;
        use bcinr_mfw_ir::Digest;

        /// Runs both `petri_tick` and `petri_tick_guarded` (with
        /// `StableMaximalSelector` + an empty guard table) from identical
        /// starting states, tick by tick, and asserts every observable
        /// piece of state stays in lockstep — the required proof that
        /// gating this hot path is additive, not a silent behavior change.
        fn assert_guarded_matches_plain(tape: &PowlTape) {
            let ops = tape_ops(tape);
            let mut plain_state = PowlPetriState::new(tape.entry_mask);
            let mut guarded_state = PowlPetriState::new(tape.entry_mask);
            let mut selector = StableMaximalSelector;
            let guards = ConcurrencyGuardTable::empty();

            for tick in 0..20 {
                let plain_done = plain_state.check.words[0] == 0;
                let guarded_done = guarded_state.check.words[0] == 0;
                assert_eq!(
                    plain_done, guarded_done,
                    "tick {tick}: plain/guarded disagree on termination"
                );
                if plain_done {
                    break;
                }

                let plain_fired = petri_tick(&ops, &mut plain_state, None, None, 0);
                let guarded_fired = petri_tick_guarded(
                    &ops,
                    &mut guarded_state,
                    &mut selector,
                    &guards,
                    None,
                    None,
                    0,
                );

                assert_eq!(
                    plain_fired.fired_ops, guarded_fired.fired_ops,
                    "tick {tick}: fired sets diverged (plain={:#018x}, guarded={:#018x})",
                    plain_fired.fired_ops, guarded_fired.fired_ops
                );
                assert_eq!(
                    plain_state.done.words[0], guarded_state.done.words[0],
                    "tick {tick}: done diverged"
                );
                assert_eq!(
                    plain_state.check.words[0], guarded_state.check.words[0],
                    "tick {tick}: check diverged"
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

        /// The concurrency guard must have a real effect: a nonface
        /// covering both parallel ops forbids them from firing on the same
        /// tick, deferring one to a later tick — but both must eventually
        /// fire (the guard defers, it never permanently starves), and the
        /// deferred op must never be reported in `fired_ops` on the tick
        /// it was excluded.
        #[test]
        fn nonempty_guard_defers_a_forbidden_pair_but_both_eventually_fire() {
            let ast = PowlAstNode::PartialOrder {
                children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
                edges: vec![],
            };
            let tape = compile_powl(&ast).unwrap();
            let ops = tape_ops(&tape);

            let guards = ConcurrencyGuardTable {
                nonfaces: vec![CompiledNonFace {
                    members: EventSet::empty().with(0).with(1),
                    witness_digest: Digest::hash(b"a-b-conflict"),
                }],
            };
            let mut state = PowlPetriState::new(tape.entry_mask);
            let mut selector = StableMaximalSelector;

            let fs1 = petri_tick_guarded(&ops, &mut state, &mut selector, &guards, None, None, 0);
            assert_eq!(
                fs1.fired_ops.count_ones(),
                1,
                "guard must defer one of the two conflicting ops on tick 1, got fired={:#018x}",
                fs1.fired_ops
            );
            assert!(
                fs1.fired_ops == 0b01 || fs1.fired_ops == 0b10,
                "exactly one of a (bit0) or b (bit1) must fire, got {:#018x}",
                fs1.fired_ops
            );

            let mut total_fired = fs1.fired_ops;
            for _ in 0..10 {
                if state.check.words[0] == 0 {
                    break;
                }
                let fs =
                    petri_tick_guarded(&ops, &mut state, &mut selector, &guards, None, None, 0);
                total_fired |= fs.fired_ops;
            }
            assert_eq!(
                total_fired & 0b11,
                0b11,
                "both conflicting ops must eventually fire (deferred, not dropped)"
            );
            assert_eq!(
                state.check.words[0], 0,
                "run must terminate (join must eventually fire once both preds are done)"
            );
        }

        #[test]
        fn mask_event_set_round_trip() {
            for mask in [0u64, 1, 0b1010, 0xFFFF_FFFF_FFFF_FFFF] {
                let set = mask_to_event_set(mask);
                assert_eq!(
                    event_set_to_mask(&set),
                    mask,
                    "round trip failed for {mask:#018x}"
                );
            }
        }
    }
}
