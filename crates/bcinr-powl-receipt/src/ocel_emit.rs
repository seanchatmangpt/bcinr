//! ocel_emit — bump-allocation arena for [`crate::causal_receipt::OcelCausalFrame`]s.
//!
//! `OcelEmitArena` owns a heap-allocated array of 4096 frames and fills them
//! sequentially.  Each `emit` call writes one frame and returns a reference to
//! it so the caller can immediately hand it to
//! [`crate::causal_receipt::OcelCausalReceipt::chain`].

use crate::causal_receipt::{OcelCausalFrame, PackedObjRef};
use crate::denial::DenialPolarity;

/// Fixed capacity of the arena in frames.
const ARENA_CAPACITY: usize = 4096;

/// Bump-allocation arena for [`OcelCausalFrame`]s.
///
/// Frames are allocated sequentially and never freed. When the arena is full
/// `emit` panics — callers are responsible for bounding frame counts at
/// manufacture time.
pub struct OcelEmitArena {
    frames: Box<[OcelCausalFrame; ARENA_CAPACITY]>,
    head: usize,
}

impl OcelEmitArena {
    /// Allocate a new arena on the heap.
    pub fn new() -> Self {
        // SAFETY: OcelCausalFrame is repr(C) with all-zero being a valid
        // bit-pattern for every field (all integers zero, all arrays zeroed).
        let frames = unsafe {
            let layout = std::alloc::Layout::array::<OcelCausalFrame>(ARENA_CAPACITY).unwrap();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut OcelCausalFrame;
            assert!(!ptr.is_null(), "OcelEmitArena: allocation failed");
            Box::from_raw(ptr as *mut [OcelCausalFrame; ARENA_CAPACITY])
        };
        Self { frames, head: 0 }
    }

    /// Emit one frame into the arena and return a reference to it.
    ///
    /// # Parameters
    ///
    /// - `instruction_id` — monotonic step identity
    /// - `activity_idx`   — index into an [`crate::intern::ActivityTable`]
    /// - `obj_refs`       — up to 8 `(type_idx, object_id)` pairs; excess
    ///   entries are silently truncated to 8
    /// - `denial`         — denial polarity for this step
    /// - `node_kind`      — POWL node kind classifier byte
    ///
    /// The `prior_hash` field of the new frame is left as zero bytes.  Callers
    /// that need chain integrity should copy the previous frame's hash (or the
    /// receipt's current `chain_hash`) into `frame.prior_hash` before chaining.
    ///
    /// # Panics
    ///
    /// Panics when the arena is full (4096 frames emitted).
    pub fn emit(
        &mut self,
        instruction_id: u64,
        activity_idx: u16,
        obj_refs: &[(u8, u32)],
        denial: DenialPolarity,
        node_kind: u8,
    ) -> &OcelCausalFrame {
        assert!(
            self.head < ARENA_CAPACITY,
            "OcelEmitArena: arena full (capacity = {ARENA_CAPACITY})"
        );

        let slot = &mut self.frames[self.head];

        // Monotonic placeholder timestamp: frame index in nanoseconds.
        let ts_ns = self.head as u64;

        slot.instruction_id = instruction_id;
        slot.fired_mask = denial.to_fired_mask();
        slot.denial = denial;
        slot.ts_ns = ts_ns;
        slot.activity_idx = activity_idx;
        slot.node_kind = node_kind;
        // _pad is already zeroed (arena is zero-initialised).

        // Pack up to 8 object references; zero-fill the rest. `zip` alone
        // (no explicit index) already stops at `min(8, obj_refs.len())`,
        // i.e. exactly `n` pairs -- the two fixed-size collections bound
        // the iteration themselves.
        let n = obj_refs.len().min(8);
        for (dst, &(type_idx, object_id)) in slot.obj_refs.iter_mut().zip(obj_refs.iter()) {
            *dst = PackedObjRef::new(type_idx, object_id);
        }
        for dst in slot.obj_refs.iter_mut().skip(n) {
            *dst = PackedObjRef::default();
        }

        // prior_hash is left as zero bytes; callers fill from the receipt.

        let idx = self.head;
        self.head += 1;
        &self.frames[idx]
    }

    /// Number of frames currently emitted.
    #[inline]
    pub fn len(&self) -> usize {
        self.head
    }

    /// Returns `true` when no frames have been emitted.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == 0
    }

    /// Returns a slice of all emitted frames.
    #[inline]
    pub fn as_slice(&self) -> &[OcelCausalFrame] {
        &self.frames[..self.head]
    }
}

impl Default for OcelEmitArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_receipt::OcelCausalReceipt;

    #[test]
    fn emit_basic() {
        let mut arena = OcelEmitArena::new();
        let frame = arena.emit(
            42,
            0,
            &[(1u8, 100u32), (2u8, 200u32)],
            DenialPolarity::ADMITTED,
            0,
        );
        assert_eq!(frame.instruction_id, 42);
        assert_eq!(frame.activity_idx, 0);
        assert_eq!(frame.denial, DenialPolarity::ADMITTED);
        assert_eq!(frame.fired_mask, 0);
        assert_eq!(frame.obj_refs[0].type_idx(), 1);
        assert_eq!(frame.obj_refs[0].object_id(), 100);
        assert_eq!(frame.obj_refs[1].type_idx(), 2);
        assert_eq!(frame.obj_refs[1].object_id(), 200);
        // Remaining obj_refs zero.
        assert_eq!(frame.obj_refs[2].0, 0);
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn emit_denial_sets_fired_mask() {
        let mut arena = OcelEmitArena::new();
        let frame = arena.emit(1, 0, &[], DenialPolarity::PRECONDITION_FAILED, 0);
        assert!(!frame.denial.is_admitted());
        assert_eq!(
            frame.fired_mask,
            DenialPolarity::PRECONDITION_FAILED.to_fired_mask()
        );
    }

    #[test]
    fn emit_truncates_obj_refs_to_8() {
        let mut arena = OcelEmitArena::new();
        let refs: Vec<(u8, u32)> = (0..12).map(|i| (i as u8, i as u32)).collect();
        let frame = arena.emit(1, 0, &refs, DenialPolarity::ADMITTED, 0);
        // Must not panic; only first 8 are stored.
        assert_eq!(frame.obj_refs[7].object_id(), 7);
    }

    #[test]
    fn emit_integrates_with_receipt_chain() {
        let mut arena = OcelEmitArena::new();
        let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);

        for i in 0..5u64 {
            let frame = arena.emit(i, 0, &[(0, i as u32)], DenialPolarity::ADMITTED, 0);
            receipt.chain(frame);
        }
        assert_eq!(receipt.frame_count, 5);
        assert_eq!(arena.len(), 5);
    }

    #[test]
    fn is_empty_and_len() {
        let mut arena = OcelEmitArena::new();
        assert!(arena.is_empty());
        arena.emit(0, 0, &[], DenialPolarity::ADMITTED, 0);
        assert!(!arena.is_empty());
        assert_eq!(arena.len(), 1);
    }
}
