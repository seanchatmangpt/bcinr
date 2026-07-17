//! POWL v2 Dispatcher — 8-lane CAS slots + sparse enabled index.
//!
//! # Structures
//!
//! - [`BpadDispatcher`]: 8-lane, 64-byte-aligned CAS slot table for
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
//! `fanout_pair`: single CAS attempt, all-or-nothing.
//! `on_complete`: O(OPS) scan — amortised over BPM-scale programs.
//!
//! # `no_std` compatibility
//!
//! All primitives use only `core::sync::atomic`. No heap allocation.

use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

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
// Bit-Parallel Atomic Dispatcher (BPAD)
// ---------------------------------------------------------------------------

/// Sentinel: slot is free (no op enqueued).
pub const SLOT_FREE: u32 = u32::MAX;

/// One worker slot. 64-byte aligned so independent slots occupy
/// distinct cache lines and CAS traffic on one slot does not
/// perturb its neighbours.
#[repr(C, align(64))]
pub struct BpadSlot {
    /// The index of the enqueued operation. SLOT_FREE when unoccupied.
    pub op_index: AtomicU32,
    _pad: [u8; 60],
}

// Compile-time layout asserts.
const _BPAD_SLOT_SIZE: () = assert!(core::mem::size_of::<BpadSlot>() == 64);
const _BPAD_SLOT_ALIGN: () = assert!(core::mem::align_of::<BpadSlot>() == 64);

impl BpadSlot {
    /// Construct a free, unoccupied slot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            op_index: AtomicU32::new(SLOT_FREE),
            _pad: [0u8; 60],
        }
    }
}

impl Default for BpadSlot {
    fn default() -> Self {
        Self::new()
    }
}

/// Compatibility alias for Slot.
pub type Slot = BpadSlot;

/// Branchless selector to prevent conditional jumps (satisfying CC=1).
#[inline(always)]
fn select(cond: bool, true_val: usize, false_val: usize) -> usize {
    let mask = 0usize.wrapping_sub(cond as usize);
    (true_val & mask) | (false_val & !mask)
}

/// Submission result for single-slot dispatching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubmissionResult {
    /// The index of the allocated slot (0..7).
    pub slot_id: u8,
    /// True if submission succeeded.
    pub is_ok: bool,
    /// Refusal code (0: Success, 1: RingFull, 2: ContentionFailure).
    pub refusal_code: u8,
}

/// Submission result for dual-slot fan-out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubmissionPairResult {
    /// The index of the first allocated slot.
    pub first: u8,
    /// The index of the second allocated slot.
    pub second: u8,
    /// True if both slots were atomically claimed.
    pub is_ok: bool,
    /// Refusal code (0: Success, 2: ContentionFailure, 3: InsufficientSlots).
    pub refusal_code: u8,
}

/// Claim result for worker consumption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimResult {
    /// The index of the enqueued operation.
    pub op_index: u32,
    /// True if the slot was successfully claimed and had a valid operation.
    pub is_ok: bool,
}

/// 8-lane CAS dispatcher using unified bit-parallel state.
pub struct BpadDispatcher {
    /// Bitmask representing slot occupancy (bits 0..7).
    pub occupancy: AtomicU8,
    /// 8 active worker slots + 1 garbage slot at index 8.
    pub slots: [BpadSlot; 9],
}

/// Compatibility alias for InlineDispatcher.
pub type InlineDispatcher = BpadDispatcher;

impl BpadDispatcher {
    /// Create a fresh dispatcher with all slots free.
    #[must_use]
    pub const fn new() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const Z: BpadSlot = BpadSlot::new();
        Self {
            occupancy: AtomicU8::new(0),
            slots: [Z; 9],
        }
    }

    /// Number of currently occupied slots.
    #[must_use]
    #[inline(always)]
    pub fn occupied(&self) -> usize {
        self.occupancy.load(Ordering::Acquire).count_ones() as usize
    }

    /// Try to submit `op_idx` to any free slot.
    #[inline(always)]
    pub fn try_submit(&self, op_idx: u32) -> SubmissionResult {
        let old = self.occupancy.load(Ordering::Acquire);
        let free_mask = !old;
        let slot_idx = free_mask.trailing_zeros() as u8;
        
        let is_full = slot_idx >= 8;
        let target_bit = 1u8 << (slot_idx & 7);
        let proposed = old | target_bit;
        
        // If full, proposed == old, causing the CAS to have no net effect.
        let success = self.occupancy
            .compare_exchange(old, proposed, Ordering::SeqCst, Ordering::Acquire)
            .is_ok();
            
        let is_ok = success && !is_full;
        let dest_idx = select(is_ok, slot_idx as usize, 8);
        
        // Write to target slot if successful, or to the garbage slot if failed.
        self.slots[dest_idx].op_index.store(op_idx, Ordering::Release);
        
        let refusal_code = select(is_full, 1, select(!success, 2, 0)) as u8;
        
        SubmissionResult {
            slot_id: slot_idx,
            is_ok,
            refusal_code,
        }
    }

    /// Try to claim the op stored in `slot_idx`.
    #[inline(always)]
    pub fn try_claim(&self, slot_idx: u8) -> ClaimResult {
        let occ = self.occupancy.load(Ordering::Acquire);
        let is_claimed = ((occ >> (slot_idx & 7)) & 1) == 1;
        let idx = self.slots[(slot_idx & 7) as usize].op_index.load(Ordering::Acquire);
        
        let has_op = idx != SLOT_FREE;
        let is_ok = is_claimed && has_op;
        
        ClaimResult {
            op_index: idx,
            is_ok,
        }
    }

    /// Release `slot_idx` back to the free pool.
    #[inline(always)]
    pub fn release(&self, slot_idx: u8) {
        let s_idx = (slot_idx & 7) as usize;
        
        // 1. Reset slot index first (Release ordering ensures this happens-before occupancy clear)
        self.slots[s_idx].op_index.store(SLOT_FREE, Ordering::Release);
        
        // 2. Clear occupancy bit (Release ordering synchronizes with subsequent Acquire loads)
        self.occupancy.fetch_and(!(1u8 << s_idx), Ordering::Release);
    }

    /// Atomically acquire two distinct free slots for a Par/Concur fan-out.
    #[inline(always)]
    pub fn fanout_pair(&self, left: u32, right: u32) -> SubmissionPairResult {
        let old = self.occupancy.load(Ordering::Acquire);
        let free_mask = !old;
        
        // Count free slots using branchless popcnt
        let free_count = free_mask.count_ones();
        let has_two_slots = free_count >= 2;
        
        // Extract two lowest set bits branchlessly
        let first = free_mask.trailing_zeros() as u8;
        let temp = free_mask & (free_mask.wrapping_sub(1));
        let second = temp.trailing_zeros() as u8;
        
        let target_bits = (1u8 << (first & 7)) | (1u8 << (second & 7));
        // If insufficient slots, zero out target bits to make CAS a no-op
        let acquire_mask = target_bits & (0u8.wrapping_sub(has_two_slots as u8));
        let proposed = old | acquire_mask;
        
        let success = self.occupancy
            .compare_exchange(old, proposed, Ordering::SeqCst, Ordering::Acquire)
            .is_ok();
            
        let is_ok = success && has_two_slots;
        
        let dest_first = select(is_ok, first as usize, 8);
        let dest_second = select(is_ok, second as usize, 8);
        
        // Commit operation indices to their respective slots
        self.slots[dest_first].op_index.store(left, Ordering::Release);
        self.slots[dest_second].op_index.store(right, Ordering::Release);
        
        let refusal_code = select(!has_two_slots, 3, select(!success, 2, 0)) as u8;
        
        SubmissionPairResult {
            first: first & 7,
            second: second & 7,
            is_ok,
            refusal_code,
        }
    }
}

impl Default for BpadDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SparseEnabledIndex — BPM-scale predecessor-count tracker
// ---------------------------------------------------------------------------

/// Sparse predecessor-count index for programs with up to `OPS`
/// operations.
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
    const _OPS_BOUND: () = assert!(
        OPS <= 512,
        "SparseEnabledIndex: OPS must be <= 512 (8 * 64 bits)"
    );

    /// Construct an index where every op starts with `initial_counts`.
    #[must_use]
    pub fn new(initial_counts: [u32; OPS]) -> Self {
        #[allow(clippy::let_unit_value)]
        let _ = Self::_OPS_BOUND;
        Self {
            dependents: [[0u64; 8]; OPS],
            pending_count: initial_counts.map(AtomicU32::new),
        }
    }

    /// Record that op `dependent` has op `predecessor` as one of its
    /// predecessors.
    pub fn register_edge(&mut self, predecessor: usize, dependent: usize) {
        assert!(predecessor < OPS, "predecessor index out of range");
        assert!(dependent < OPS, "dependent index out of range");
        let word = dependent / 64;
        let bit = dependent % 64;
        self.dependents[predecessor][word] |= 1u64 << bit;
    }

    /// Call when op `k` completes.
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
                    let prev = self.pending_count[j].fetch_sub(1, Ordering::AcqRel);
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

    #[test]
    fn slot_size_and_align() {
        assert_eq!(core::mem::size_of::<Slot>(), 64);
        assert_eq!(core::mem::align_of::<Slot>(), 64);
    }

    #[test]
    fn fresh_dispatcher_all_slots_free() {
        let d = BpadDispatcher::new();
        assert_eq!(d.occupied(), 0);
        for i in 0u8..8 {
            assert!(!d.try_claim(i).is_ok);
        }
    }

    #[test]
    fn try_submit_claim_release_roundtrip() {
        let d = BpadDispatcher::new();
        let res = d.try_submit(42);
        assert!(res.is_ok, "fresh dispatcher must accept submission");
        let slot = res.slot_id;
        
        let claim = d.try_claim(slot);
        assert!(claim.is_ok, "submitted op must be claimable");
        assert_eq!(claim.op_index, 42);
        
        d.release(slot);
        assert_eq!(d.occupied(), 0);
        assert!(!d.try_claim(slot).is_ok);
    }

    #[test]
    fn try_submit_fills_all_eight_slots() {
        let d = BpadDispatcher::new();
        let mut slots = [0u8; 8];
        for i in 0u32..8 {
            let res = d.try_submit(i);
            assert!(res.is_ok, "should accept 8 submissions");
            slots[i as usize] = res.slot_id;
        }
        assert_eq!(d.occupied(), 8);
        // 9th must fail (backpressure).
        let res9 = d.try_submit(99);
        assert!(!res9.is_ok);
        assert_eq!(res9.refusal_code, 1); // RingFull
    }

    #[test]
    fn fanout_pair_claims_two_distinct_slots() {
        let d = BpadDispatcher::new();
        let res = d.fanout_pair(10, 20);
        assert!(res.is_ok, "empty dispatcher must allow fanout_pair");
        assert_ne!(
            res.first, res.second,
            "fanout_pair must return distinct slot ids"
        );
        assert_eq!(d.occupied(), 2);

        // Verify correct op indices are visible.
        let op_left = d.try_claim(res.first);
        let op_right = d.try_claim(res.second);
        assert!(op_left.is_ok);
        assert_eq!(op_left.op_index, 10);
        assert!(op_right.is_ok);
        assert_eq!(op_right.op_index, 20);
    }

    #[test]
    fn fanout_pair_rolls_back_on_full_ring() {
        let d = BpadDispatcher::new();
        // Fill 7 slots manually.
        for i in 0u32..7 {
            let res = d.try_submit(i);
            assert!(res.is_ok, "should fit");
        }
        assert_eq!(d.occupied(), 7);
        // One slot remains — fanout_pair needs two — must fail and leave occupancy at 7.
        let res = d.fanout_pair(99, 100);
        assert!(!res.is_ok, "fanout_pair must fail when only 1 slot remains");
        assert_eq!(res.refusal_code, 3); // InsufficientSlots
        assert_eq!(d.occupied(), 7, "occupancy must remain unchanged");
    }

    #[test]
    fn fanout_pair_fails_on_completely_full_ring() {
        let d = BpadDispatcher::new();
        for i in 0u32..8 {
            let res = d.try_submit(i);
            assert!(res.is_ok);
        }
        let res = d.fanout_pair(50, 51);
        assert!(!res.is_ok);
        assert_eq!(res.refusal_code, 3); // InsufficientSlots
    }

    // -----------------------------------------------------------------------
    // Oracle & Differential testing
    // -----------------------------------------------------------------------

    pub struct OracleDispatcher {
        slots: std::sync::Mutex<std::vec::Vec<Option<u32>>>,
    }

    impl OracleDispatcher {
        pub fn new() -> Self {
            Self {
                slots: std::sync::Mutex::new(std::vec![None; 8]),
            }
        }

        pub fn submit(&self, op_idx: u32) -> Result<usize, String> {
            let mut guard = self.slots.lock().unwrap();
            for i in 0..8 {
                if guard[i].is_none() {
                    guard[i] = Some(op_idx);
                    return Ok(i);
                }
            }
            Err("RingFull".into())
        }

        pub fn fanout(&self, left: u32, right: u32) -> Result<(usize, usize), String> {
            let mut guard = self.slots.lock().unwrap();
            let free_indices: std::vec::Vec<usize> = guard.iter().enumerate()
                .filter(|(_, slot)| slot.is_none())
                .map(|(idx, _)| idx)
                .collect();
                
            if free_indices.len() < 2 {
                return Err("InsufficientSlots".into());
            }
            
            let l_idx = free_indices[0];
            let r_idx = free_indices[1];
            guard[l_idx] = Some(left);
            guard[r_idx] = Some(right);
            Ok((l_idx, r_idx))
        }

        pub fn release(&self, slot: usize) {
            let mut guard = self.slots.lock().unwrap();
            if slot < 8 {
                guard[slot] = None;
            }
        }
        
        pub fn get_slot(&self, slot: usize) -> Option<u32> {
            let guard = self.slots.lock().unwrap();
            if slot < 8 {
                guard[slot]
            } else {
                None
            }
        }
    }

    #[test]
    fn differential_testing_oracle() {
        let d = BpadDispatcher::new();
        let oracle = OracleDispatcher::new();
        
        let mut seed: u32 = 0x12345678;
        let mut next_random = move || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed
        };
        
        for _ in 0..10_000 {
            let op_type = next_random() % 4;
            match op_type {
                0 => {
                    let val = next_random();
                    let res_bpad = d.try_submit(val);
                    let res_oracle = oracle.submit(val);
                    assert_eq!(res_bpad.is_ok, res_oracle.is_ok());
                    if res_bpad.is_ok {
                        let idx = res_bpad.slot_id as usize;
                        assert_eq!(idx, res_oracle.unwrap());
                    }
                }
                1 => {
                    let val1 = next_random();
                    let val2 = next_random();
                    let res_bpad = d.fanout_pair(val1, val2);
                    let res_oracle = oracle.fanout(val1, val2);
                    assert_eq!(res_bpad.is_ok, res_oracle.is_ok());
                    if res_bpad.is_ok {
                        let (idx1, idx2) = res_oracle.unwrap();
                        assert_eq!(res_bpad.first as usize, idx1);
                        assert_eq!(res_bpad.second as usize, idx2);
                    }
                }
                2 => {
                    let slot = (next_random() % 8) as u8;
                    d.release(slot);
                    oracle.release(slot as usize);
                }
                _ => {
                    let slot = (next_random() % 8) as u8;
                    let claim_bpad = d.try_claim(slot);
                    let val_oracle = oracle.get_slot(slot as usize);
                    assert_eq!(claim_bpad.is_ok, val_oracle.is_some());
                    if claim_bpad.is_ok {
                        assert_eq!(claim_bpad.op_index, val_oracle.unwrap());
                    }
                }
            }
            
            let mut expected_occupied = 0;
            for i in 0..8 {
                let val_oracle = oracle.get_slot(i);
                let claim_bpad = d.try_claim(i as u8);
                assert_eq!(claim_bpad.is_ok, val_oracle.is_some());
                if claim_bpad.is_ok {
                    assert_eq!(claim_bpad.op_index, val_oracle.unwrap());
                    expected_occupied += 1;
                }
            }
            assert_eq!(d.occupied(), expected_occupied);
        }
    }

    // -----------------------------------------------------------------------
    // SparseEnabledIndex tests
    // -----------------------------------------------------------------------

    #[test]
    fn on_complete_returns_correct_newly_enabled_set() {
        let mut idx = SparseEnabledIndex::<4>::new([0, 0, 2, 1]);
        idx.register_edge(0, 2);
        idx.register_edge(1, 2);
        idx.register_edge(2, 3);

        let enabled = idx.on_complete(0);
        assert_eq!(enabled, 0b0000);

        let enabled = idx.on_complete(1);
        assert_eq!(enabled, 0b0100);

        let enabled = idx.on_complete(2);
        assert_eq!(enabled, 0b1000);
    }

    #[test]
    fn on_complete_no_dependents_returns_zero() {
        let idx = SparseEnabledIndex::<4>::new([0, 0, 0, 0]);
        assert_eq!(idx.on_complete(0), 0);
    }

    #[test]
    fn on_complete_single_dep_immediate_enable() {
        let mut idx = SparseEnabledIndex::<2>::new([0, 1]);
        idx.register_edge(0, 1);
        let enabled = idx.on_complete(0);
        assert_eq!(enabled & 0b10, 0b10);
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
