//! POWL v2 Dispatcher — 8-lane CAS slots + sparse enabled index.
//!
//! # Structures
//!
//! - [`InlineDispatcher`]: 8-lane, 64-byte-aligned CAS slot table for
//!   fan-out scheduling of Par/Concur arm pairs.
//! - [`SparseEnabledIndex`]: BPM-scale predecessor-count tracker that
//!   returns a bitmask of newly-enabled ops on completion.
//!
//! # Concur marker
//!
//! [`detect_concur_marker`] identifies the Par/Concur gate emitted by
//! the POWL v2 compiler (`ctrl == u64::MAX` and `op_kind == OpKind::Concur`).
//!
//! # Timing contract
//!
//! `try_submit` / `try_claim` / `release`: O(1), single CAS each.
//! `fanout_pair`: two CAS attempts; rolls back left if right fails (all-or-nothing).
//! `on_complete`: O(OPS) scan — amortised over BPM-scale programs.
//!
//! # `no_std` compatibility
//!
//! All primitives use only `core::sync::atomic`. No heap allocation.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// ---------------------------------------------------------------------------
// OpKind for the POWL v2 op stream
// ---------------------------------------------------------------------------

/// Operation kinds recognised by the POWL v2 runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    /// Standard activity node.
    Activity,
    /// Partial-order gate (XOR / AND split/join).
    PartialOrderGate,
    /// Par/Concur fan-out marker emitted by the compiler.
    /// `ctrl == u64::MAX` when this variant is active.
    Concur,
}

// ---------------------------------------------------------------------------
// Powl64Op — flat op representation used by the dispatcher
// ---------------------------------------------------------------------------

/// A single schedulable operation in a POWL v2 program.
#[derive(Clone, Copy, Debug)]
pub struct Powl64Op {
    /// All bits must be set in `completed` before this op fires.
    pub pred_mask: u64,
    /// Bits to OR into `completed` when this op finishes.
    pub succ_mask: u64,
    /// Control word. `u64::MAX` is the Concur marker sentinel.
    pub ctrl: u64,
    /// Variant tag.
    pub op_kind: OpKind,
}

// ---------------------------------------------------------------------------
// InlineDispatcher — 8-lane 64-byte-aligned CAS slot table
// ---------------------------------------------------------------------------

/// Sentinel: slot is free (no op enqueued).
pub const SLOT_FREE: u32 = u32::MAX;

/// One worker slot. 64-byte aligned so independent slots occupy
/// distinct cache lines and CAS traffic on one slot does not
/// perturb its neighbours.
///
/// Layout (64 bytes total):
/// ```text
/// offset 0..4   op_index  : AtomicU32   — SLOT_FREE when free
/// offset 4      claimed   : AtomicBool
/// offset 5..64  _pad      : [u8; 59]
/// ```
#[repr(C, align(64))]
pub struct Slot {
    /// Dense op index of the enqueued op. [`SLOT_FREE`] when the
    /// slot is available for a new submission.
    pub op_index: AtomicU32,
    /// `true` once a producer has CAS-claimed this slot.
    pub claimed: AtomicBool,
    _pad: [u8; 59],
}

// Compile-time layout asserts.
const _SLOT_SIZE: () = assert!(core::mem::size_of::<Slot>() == 64);
const _SLOT_ALIGN: () = assert!(core::mem::align_of::<Slot>() == 64);

impl Slot {
    /// Construct a free, unclaimed slot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            op_index: AtomicU32::new(SLOT_FREE),
            claimed: AtomicBool::new(false),
            _pad: [0u8; 59],
        }
    }

    /// `true` if no producer currently holds this slot.
    #[must_use]
    #[inline(always)]
    pub fn is_free(&self) -> bool {
        !self.claimed.load(Ordering::Acquire)
    }

    /// Return this slot to the free pool.
    ///
    /// The caller must have previously obtained this slot via
    /// [`InlineDispatcher::try_submit`] or [`InlineDispatcher::fanout_pair`].
    #[inline(always)]
    pub fn release(&self) {
        self.op_index.store(SLOT_FREE, Ordering::Relaxed);
        // Release ordering makes the op_index reset visible before
        // the claimed flag is cleared.
        self.claimed.store(false, Ordering::Release);
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::new()
    }
}

/// 8-lane CAS dispatcher.
///
/// Each of the 8 slots is an independent 64-byte-aligned unit.
/// Producers call [`try_submit`][Self::try_submit] to enqueue an op
/// index; workers call [`try_claim`][Self::try_claim] to consume it.
/// [`fanout_pair`][Self::fanout_pair] atomically acquires two distinct
/// slots (all-or-nothing) for Par/Concur fan-out.
pub struct InlineDispatcher {
    pub slots: [Slot; 8],
}

impl InlineDispatcher {
    /// Create a fresh dispatcher with all slots free.
    #[must_use]
    pub const fn new() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const Z: Slot = Slot::new();
        Self { slots: [Z; 8] }
    }

    /// Try to submit `op_idx` to any free slot.
    ///
    /// Scans slots 0..8 in order; performs a CAS on `claimed` to win
    /// exclusive ownership, then writes `op_idx` with `Release`
    /// ordering so consumers see it after the claim flag.
    ///
    /// Returns `Some(slot_id)` on success, `None` if all 8 slots are
    /// occupied (backpressure).
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn try_submit(&self, op_idx: u32) -> Option<u8> {
        for (i, slot) in self.slots.iter().enumerate() {
            if slot
                .claimed
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::Acquire)
                .is_ok()
            {
                slot.op_index.store(op_idx, Ordering::Release);
                return Some(i as u8);
            }
        }
        None
    }

    /// Try to claim the op stored in `slot`.
    ///
    /// Loads the slot's `op_index`; if the slot is claimed and carries
    /// a real op index (not [`SLOT_FREE`]), returns `Some(op_idx)`.
    /// The slot remains claimed — the caller must call
    /// [`release`][Self::release] when processing is complete.
    ///
    /// Returns `None` if the slot is free or not yet written.
    #[must_use]
    #[inline(always)]
    pub fn try_claim(&self, slot: u8) -> Option<u32> {
        let s = &self.slots[slot as usize];
        // Only expose the op if the slot is actually claimed.
        let is_claimed = s.claimed.load(Ordering::Acquire);
        let idx = s.op_index.load(Ordering::Acquire);
        if is_claimed && idx != SLOT_FREE {
            Some(idx)
        } else {
            None
        }
    }

    /// Release `slot` back to the free pool.
    ///
    /// Safe to call only after the worker has finished processing the
    /// op obtained from [`try_claim`][Self::try_claim].
    #[inline(always)]
    pub fn release(&self, slot: u8) {
        self.slots[slot as usize].release();
    }

    /// Atomically acquire two distinct free slots for a Par/Concur
    /// fan-out.
    ///
    /// If the left claim succeeds but the right fails, the left slot
    /// is released so the dispatcher state stays consistent.
    /// Callers observe "all-or-nothing" backpressure.
    ///
    /// Returns `Some([left_slot, right_slot])` on success,
    /// `None` when the ring cannot supply two free slots.
    #[must_use]
    pub fn fanout_pair(&self, left: u32, right: u32) -> Option<[u8; 2]> {
        let left_slot = self.try_submit(left)?;
        match self.try_submit(right) {
            Some(right_slot) => {
                debug_assert_ne!(left_slot, right_slot, "fanout_pair must use distinct slots");
                Some([left_slot, right_slot])
            }
            None => {
                // Roll back left so a failed fan-out leaves no debris.
                self.release(left_slot);
                None
            }
        }
    }

    /// Number of currently occupied slots.
    #[must_use]
    pub fn occupied(&self) -> usize {
        self.slots.iter().filter(|s| !s.is_free()).count()
    }
}

impl Default for InlineDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SparseEnabledIndex — BPM-scale predecessor-count tracker
// ---------------------------------------------------------------------------

/// Sparse predecessor-count index for programs with up to `OPS`
/// operations.
///
/// For each op `k`, `dependents[k]` is a 512-bit bitmask (8 × u64)
/// recording all ops that list `k` among their predecessors.
/// `pending_count[j]` is the number of unsatisfied predecessors
/// remaining for op `j`.
///
/// When op `k` completes, [`on_complete`][Self::on_complete]
/// decrements `pending_count` for every dependent and returns a
/// bitmask of ops whose count reached zero (newly enabled).
///
/// # Const parameter
///
/// `OPS` must be ≤ 512 because each `dependents` row is 8 × u64 =
/// 512 bits.
pub struct SparseEnabledIndex<const OPS: usize> {
    /// `dependents[k]` — bitmask of ops that have `k` in their
    /// predecessor set. Bit `j` is set when op `j` depends on op `k`.
    pub dependents: [[u64; 8]; OPS],
    /// Remaining predecessor count for each op. An op is enabled when
    /// its count reaches zero.
    pub pending_count: [AtomicU32; OPS],
}

impl<const OPS: usize> SparseEnabledIndex<OPS> {
    /// Compile-time bound: OPS must be ≤ 512 (8 × 64 bits).
    /// Evaluated once per monomorphisation; compatible with `generic_const_exprs`.
    const _OPS_BOUND: () =
        assert!(OPS <= 512, "SparseEnabledIndex: OPS must be <= 512 (8 * 64 bits)");

    /// Construct an index where every op starts with `initial_counts`
    /// as its pending predecessor count and no dependents are
    /// registered.
    ///
    /// Call [`register_edge`][Self::register_edge] (or set
    /// `dependents` directly) before the first [`on_complete`][Self::on_complete].
    #[must_use]
    pub fn new(initial_counts: [u32; OPS]) -> Self {
        #[allow(clippy::let_unit_value)]
        let _ = Self::_OPS_BOUND; // force evaluation at instantiation
        Self {
            dependents: [[0u64; 8]; OPS],
            pending_count: initial_counts.map(AtomicU32::new),
        }
    }

    /// Record that op `dependent` has op `predecessor` as one of its
    /// predecessors.
    ///
    /// Sets bit `dependent` in `dependents[predecessor]`.
    ///
    /// # Panics
    ///
    /// Panics if `predecessor >= OPS` or `dependent >= OPS`.
    pub fn register_edge(&mut self, predecessor: usize, dependent: usize) {
        assert!(predecessor < OPS, "predecessor index out of range");
        assert!(dependent < OPS, "dependent index out of range");
        let word = dependent / 64;
        let bit = dependent % 64;
        self.dependents[predecessor][word] |= 1u64 << bit;
    }

    /// Call when op `k` completes.
    ///
    /// Decrements `pending_count` for every op that depends on `k`
    /// (those recorded in `dependents[k]`). Returns a 512-bit
    /// bitmask packed into `u64` where bit `j` is set if op `j`
    /// just became newly enabled (count transitioned from 1 → 0).
    ///
    /// # Note on bitmask encoding
    ///
    /// The return value is a single `u64` covering the lowest 64 ops
    /// (bits 0..63). For programs with more than 64 ops use
    /// [`on_complete_wide`][Self::on_complete_wide] which returns the
    /// full 8-word array.
    ///
    /// # Panics
    ///
    /// Panics if `k >= OPS`.
    pub fn on_complete(&self, k: usize) -> u64 {
        self.on_complete_wide(k)[0]
    }

    /// Same as [`on_complete`][Self::on_complete] but returns the
    /// full 512-bit enabled bitmask as `[u64; 8]`.
    pub fn on_complete_wide(&self, k: usize) -> [u64; 8] {
        assert!(k < OPS, "op index out of range");
        let mut newly_enabled = [0u64; 8];
        let dep_row = &self.dependents[k];
        for word_idx in 0..8usize {
            let mut mask = dep_row[word_idx];
            while mask != 0 {
                let bit = mask.trailing_zeros() as usize;
                mask &= mask - 1; // clear lowest set bit (branchless pop)
                let j = word_idx * 64 + bit;
                if j < OPS {
                    // AcqRel: the decrement synchronises with the
                    // producer that incremented this count.
                    let prev = self.pending_count[j].fetch_sub(1, Ordering::AcqRel);
                    // prev == 1 means count just hit zero → newly enabled.
                    if prev == 1 {
                        newly_enabled[word_idx] |= 1u64 << bit;
                    }
                }
            }
        }
        newly_enabled
    }
}

// ---------------------------------------------------------------------------
// Concur marker detection
// ---------------------------------------------------------------------------

/// `true` iff `op` is the Par/Concur fan-out marker emitted by the
/// POWL v2 compiler.
///
/// The compiler sets `ctrl = u64::MAX` and `op_kind = OpKind::Concur`
/// to signal that the two successor ops must be dispatched in parallel.
#[must_use]
#[inline(always)]
pub fn detect_concur_marker(op: &Powl64Op) -> bool {
    op.ctrl == u64::MAX && op.op_kind == OpKind::Concur
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // InlineDispatcher tests
    // -----------------------------------------------------------------------

    #[test]
    fn slot_size_and_align() {
        assert_eq!(core::mem::size_of::<Slot>(), 64);
        assert_eq!(core::mem::align_of::<Slot>(), 64);
    }

    #[test]
    fn fresh_dispatcher_all_slots_free() {
        let d = InlineDispatcher::new();
        assert_eq!(d.occupied(), 0);
        for i in 0u8..8 {
            assert!(d.try_claim(i).is_none());
        }
    }

    #[test]
    fn try_submit_claim_release_roundtrip() {
        let d = InlineDispatcher::new();
        let slot = d.try_submit(42).expect("fresh dispatcher must accept submission");
        let op = d.try_claim(slot).expect("submitted op must be claimable");
        assert_eq!(op, 42);
        d.release(slot);
        assert_eq!(d.occupied(), 0);
        assert!(d.try_claim(slot).is_none());
    }

    #[test]
    fn try_submit_fills_all_eight_slots() {
        let d = InlineDispatcher::new();
        let mut slots = [0u8; 8];
        for i in 0u32..8 {
            slots[i as usize] = d.try_submit(i).expect("should accept 8 submissions");
        }
        assert_eq!(d.occupied(), 8);
        // 9th must fail (backpressure).
        assert!(d.try_submit(99).is_none());
    }

    #[test]
    fn fanout_pair_claims_two_distinct_slots() {
        let d = InlineDispatcher::new();
        let pair = d.fanout_pair(10, 20).expect("empty dispatcher must allow fanout_pair");
        assert_ne!(pair[0], pair[1], "fanout_pair must return distinct slot ids");
        assert_eq!(d.occupied(), 2);

        // Verify correct op indices are visible.
        let op_left = d.try_claim(pair[0]).expect("left slot must carry op 10");
        let op_right = d.try_claim(pair[1]).expect("right slot must carry op 20");
        assert_eq!(op_left, 10);
        assert_eq!(op_right, 20);
    }

    #[test]
    fn fanout_pair_rolls_back_on_full_ring() {
        let d = InlineDispatcher::new();
        // Fill 7 slots manually.
        for i in 0u32..7 {
            d.try_submit(i).expect("should fit");
        }
        assert_eq!(d.occupied(), 7);
        // One slot remains — fanout_pair needs two — must fail and leave occupancy at 7.
        assert!(
            d.fanout_pair(99, 100).is_none(),
            "fanout_pair must fail when only 1 slot remains"
        );
        assert_eq!(d.occupied(), 7, "rollback must restore occupancy");
    }

    #[test]
    fn fanout_pair_fails_on_completely_full_ring() {
        let d = InlineDispatcher::new();
        for i in 0u32..8 {
            d.try_submit(i).unwrap();
        }
        assert!(d.fanout_pair(50, 51).is_none());
    }

    // -----------------------------------------------------------------------
    // SparseEnabledIndex tests
    // -----------------------------------------------------------------------

    #[test]
    fn on_complete_returns_correct_newly_enabled_set() {
        // 4-op program:
        //   op0 →(pred of)→ op2
        //   op1 →(pred of)→ op2
        //   op2 →(pred of)→ op3
        // initial pending: op0=0, op1=0, op2=2, op3=1
        let mut idx = SparseEnabledIndex::<4>::new([0, 0, 2, 1]);
        idx.register_edge(0, 2); // op2 depends on op0
        idx.register_edge(1, 2); // op2 depends on op1
        idx.register_edge(2, 3); // op3 depends on op2

        // Complete op0: op2 count 2→1, not enabled yet.
        let enabled = idx.on_complete(0);
        assert_eq!(enabled, 0b0000, "op2 still has 1 unsatisfied predecessor");

        // Complete op1: op2 count 1→0, now enabled (bit 2).
        let enabled = idx.on_complete(1);
        assert_eq!(enabled, 0b0100, "op2 (bit 2) must be newly enabled");

        // Complete op2: op3 count 1→0, now enabled (bit 3).
        let enabled = idx.on_complete(2);
        assert_eq!(enabled, 0b1000, "op3 (bit 3) must be newly enabled");
    }

    #[test]
    fn on_complete_no_dependents_returns_zero() {
        let idx = SparseEnabledIndex::<4>::new([0, 0, 0, 0]);
        // op0 has no dependents registered.
        assert_eq!(idx.on_complete(0), 0);
    }

    #[test]
    fn on_complete_single_dep_immediate_enable() {
        let mut idx = SparseEnabledIndex::<2>::new([0, 1]);
        idx.register_edge(0, 1);
        // op0 completes → op1 count 1→0 → enabled.
        let enabled = idx.on_complete(0);
        assert_eq!(enabled & 0b10, 0b10, "op1 (bit 1) must be enabled");
    }

    // -----------------------------------------------------------------------
    // detect_concur_marker tests
    // -----------------------------------------------------------------------

    #[test]
    fn detect_concur_marker_true_only_with_concur_kind_and_full_ctrl() {
        let marker = Powl64Op {
            pred_mask: 0,
            succ_mask: 0,
            ctrl: u64::MAX,
            op_kind: OpKind::Concur,
        };
        assert!(detect_concur_marker(&marker));
    }

    #[test]
    fn detect_concur_marker_false_if_ctrl_not_max() {
        let op = Powl64Op {
            pred_mask: 0,
            succ_mask: 0,
            ctrl: 0,
            op_kind: OpKind::Concur,
        };
        assert!(!detect_concur_marker(&op));
    }

    #[test]
    fn detect_concur_marker_false_if_wrong_op_kind() {
        let op = Powl64Op {
            pred_mask: 0,
            succ_mask: 0,
            ctrl: u64::MAX,
            op_kind: OpKind::Activity,
        };
        assert!(!detect_concur_marker(&op));
    }

    #[test]
    fn detect_concur_marker_false_for_po_gate_even_with_max_ctrl() {
        let op = Powl64Op {
            pred_mask: 0,
            succ_mask: 0,
            ctrl: u64::MAX,
            op_kind: OpKind::PartialOrderGate,
        };
        assert!(!detect_concur_marker(&op));
    }
}
