//! receipt_worker — Off-hot-path BLAKE3 receipt hashing via event ring drain.
//!
//! # Architecture
//!
//! ```text
//! petri_tick  → push EventWorkItem → LockFreeMpmcRing<EventWorkItem, 64>
//!                                           ↓ (drain, off hot path)
//!                                    ReceiptWorker::drain()
//!                                           ↓
//!                                    BLAKE3(run_id ‖ op_trace ‖ topology_tag)
//!                                           ↓
//!                                    ReceiptLog::append() → 57-byte entry
//! ```
//!
//! BLAKE3 is **never called inside `petri_tick`**. Only a ~10 ns `push_t1` is
//! on the hot path. The worker drains in a separate budget window (fiber or
//! worker thread) and appends sealed receipt entries to the log.
//!
//! # Receipt entry layout (57 bytes, portable LE)
//!
//! | Bytes | Field        | Type |
//! |-------|--------------|------|
//! | 0..8  | `run_id`     | u64  |
//! | 8..16 | `op_trace`   | u64  |
//! | 16    | `topo_tag`   | u8   |
//! | 17..49| `chain_hash` | [u8; 32] |
//! | 49..57| `replay_ptr` | u64  |

#![forbid(unsafe_code)]

use bcinr_logic::patterns::deterministic_mpmc::LockFreeMpmcRing;
use crate::scheduler_wired::EventWorkItem;

const RING_CAPACITY: usize = 64;

/// Byte length of one receipt entry.
pub const ENTRY_BYTES: usize = 57;

/// Maximum number of in-flight runs tracked simultaneously.
pub const MAX_PENDING: usize = 16;

/// Maximum receipt log entries (pre-allocated).
pub const MAX_LOG_ENTRIES: usize = 256;

// ---------------------------------------------------------------------------
// ReceiptLog — append-only, fixed-capacity
// ---------------------------------------------------------------------------

/// Append-only receipt log. Each entry is exactly `ENTRY_BYTES` bytes.
pub struct ReceiptLog {
    buf: [[u8; ENTRY_BYTES]; MAX_LOG_ENTRIES],
    count: usize,
}

impl ReceiptLog {
    pub const fn new() -> Self {
        Self {
            buf: [[0u8; ENTRY_BYTES]; MAX_LOG_ENTRIES],
            count: 0,
        }
    }

    /// Byte offset of the next entry (= `replay_ptr` for the current entry).
    #[inline(always)]
    pub fn next_offset(&self) -> u64 {
        (self.count * ENTRY_BYTES) as u64
    }

    /// Number of sealed entries.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns the raw bytes of entry `idx`, or `None` if out of range.
    pub fn entry(&self, idx: usize) -> Option<&[u8; ENTRY_BYTES]> {
        if idx < self.count { Some(&self.buf[idx]) } else { None }
    }

    /// Append a pre-serialised entry. Returns the byte offset of the appended entry.
    fn append(&mut self, entry: [u8; ENTRY_BYTES]) -> u64 {
        let offset = self.next_offset();
        if self.count < MAX_LOG_ENTRIES {
            self.buf[self.count] = entry;
            self.count += 1;
        }
        offset
    }
}

impl Default for ReceiptLog {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Pending — accumulator per run_id
// ---------------------------------------------------------------------------

struct Pending {
    run_id: u64,
    op_trace: u64,
    topo_tag: u8,
    active: bool,
}

impl Pending {
    const fn empty() -> Self {
        Self { run_id: 0, op_trace: 0, topo_tag: 0, active: false }
    }
}

// ---------------------------------------------------------------------------
// ReceiptWorker
// ---------------------------------------------------------------------------

/// Drains an `EventWorkItem` ring, accumulates per-run op traces, and
/// finalises receipts with real BLAKE3 when the full mask is seen.
pub struct ReceiptWorker {
    pending: [Pending; MAX_PENDING],
    pub log: ReceiptLog,
}

impl ReceiptWorker {
    pub const fn new() -> Self {
        const EMPTY: Pending = Pending::empty();
        Self {
            pending: [EMPTY; MAX_PENDING],
            log: ReceiptLog::new(),
        }
    }

    /// Drain up to `budget` items from `ring`. Returns number of receipts sealed.
    pub fn drain(
        &mut self,
        ring: &LockFreeMpmcRing<EventWorkItem, RING_CAPACITY>,
        full_mask: u64,
        budget: usize,
    ) -> u32 {
        let mut sealed = 0u32;

        for _ in 0..budget {
            let (maybe_item, _) = ring.pop_t1();
            let item = match maybe_item {
                Some(i) => i,
                None => break,
            };

            // Find or allocate a pending slot for this run_id.
            let slot = self.find_or_alloc(item.run_id, item.kind_tag);

            self.pending[slot].op_trace |= item.op_trace_so_far;

            // If all ops have fired, finalise the receipt.
            if self.pending[slot].op_trace & full_mask == full_mask {
                let (run_id, op_trace, topo_tag) = (
                    self.pending[slot].run_id,
                    self.pending[slot].op_trace,
                    self.pending[slot].topo_tag,
                );
                let entry = self.build_entry(run_id, op_trace, topo_tag);
                self.log.append(entry);
                self.pending[slot].active = false;
                sealed += 1;
            }
        }

        sealed
    }

    fn find_or_alloc(&mut self, run_id: u64, kind_tag: u8) -> usize {
        // Search for existing active slot.
        for (i, p) in self.pending.iter().enumerate() {
            if p.active && p.run_id == run_id {
                return i;
            }
        }
        // Find a free slot.
        for (i, p) in self.pending.iter_mut().enumerate() {
            if !p.active {
                p.run_id = run_id;
                p.op_trace = 0;
                p.topo_tag = kind_tag;
                p.active = true;
                return i;
            }
        }
        // Evict slot 0 (oldest) if full — shouldn't happen in normal operation.
        self.pending[0].active = false;
        self.pending[0].run_id = run_id;
        self.pending[0].op_trace = 0;
        self.pending[0].topo_tag = kind_tag;
        self.pending[0].active = true;
        0
    }

    /// Build a 57-byte receipt entry using BLAKE3.
    fn build_entry(&self, run_id: u64, op_trace: u64, topo_tag: u8) -> [u8; ENTRY_BYTES] {
        let replay_ptr = self.log.next_offset();

        // BLAKE3 hash of the canonical receipt fields.
        let mut h = blake3::Hasher::new();
        h.update(&run_id.to_le_bytes());
        h.update(&op_trace.to_le_bytes());
        h.update(&[topo_tag]);
        let chain_hash = *h.finalize().as_bytes();

        // Serialize into the 57-byte portable layout.
        let mut entry = [0u8; ENTRY_BYTES];
        entry[0..8].copy_from_slice(&run_id.to_le_bytes());
        entry[8..16].copy_from_slice(&op_trace.to_le_bytes());
        entry[16] = topo_tag;
        entry[17..49].copy_from_slice(&chain_hash);
        entry[49..57].copy_from_slice(&replay_ptr.to_le_bytes());
        entry
    }
}

impl Default for ReceiptWorker {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ring() -> LockFreeMpmcRing<EventWorkItem, RING_CAPACITY> {
        LockFreeMpmcRing::new_checked().unwrap()
    }

    #[test]
    fn content_hash_nonzero_for_two_op_tape() {
            use crate::typestate::HasPowlTape;
        use crate::compiler::{compile_powl, PowlAstNode};

        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ])).unwrap();

        let hash = tape.content_hash();
        assert_ne!(hash, [0u8; 32], "content_hash must be non-zero for a real tape");
    }

    #[test]
    fn content_hash_differs_for_different_pred_masks() {
        use crate::typestate::HasPowlTape;
        use crate::compiler::{compile_powl, PowlAstNode};

        let seq = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"), PowlAstNode::Atom("b"),
        ])).unwrap();
        let par = compile_powl(&PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        }).unwrap();

        assert_ne!(seq.content_hash(), par.content_hash(),
            "sequential and parallel tapes must have different content hashes");
    }

    #[test]
    fn worker_seals_receipt_on_full_mask() {
        let ring = make_ring();
        let run_id = 42u64;
        let full_mask = 0b11u64; // two ops

        ring.push_t1(EventWorkItem { op_idx: 0, run_id, op_trace_so_far: 0b01, kind_tag: 0 });
        ring.push_t1(EventWorkItem { op_idx: 1, run_id, op_trace_so_far: 0b11, kind_tag: 0 });

        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, full_mask, 10);

        assert_eq!(sealed, 1, "one run must be sealed");
        assert_eq!(worker.log.len(), 1, "one receipt entry in log");
    }

    #[test]
    fn worker_chains_two_receipts() {
        let ring = make_ring();
        let full_mask = 0b1u64; // single-op runs

        ring.push_t1(EventWorkItem { op_idx: 0, run_id: 1, op_trace_so_far: 0b1, kind_tag: 0 });
        ring.push_t1(EventWorkItem { op_idx: 0, run_id: 2, op_trace_so_far: 0b1, kind_tag: 0 });

        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, full_mask, 10);

        assert_eq!(sealed, 2);
        assert_eq!(worker.log.len(), 2);

        // Chain hashes must differ (different run_ids → different BLAKE3 inputs).
        let e0 = worker.log.entry(0).unwrap();
        let e1 = worker.log.entry(1).unwrap();
        assert_ne!(&e0[17..49], &e1[17..49], "chain hashes must differ across runs");
    }

    #[test]
    fn replay_ptr_is_byte_offset() {
        let ring = make_ring();
        let full_mask = 0b1u64;

        ring.push_t1(EventWorkItem { op_idx: 0, run_id: 10, op_trace_so_far: 0b1, kind_tag: 0 });
        ring.push_t1(EventWorkItem { op_idx: 0, run_id: 20, op_trace_so_far: 0b1, kind_tag: 0 });

        let mut worker = ReceiptWorker::new();
        worker.drain(&ring, full_mask, 10);

        // First entry: replay_ptr at offset 0.
        let e0 = worker.log.entry(0).unwrap();
        let ptr0 = u64::from_le_bytes(e0[49..57].try_into().unwrap());
        assert_eq!(ptr0, 0, "first entry replay_ptr must be 0");

        // Second entry: replay_ptr at offset ENTRY_BYTES.
        let e1 = worker.log.entry(1).unwrap();
        let ptr1 = u64::from_le_bytes(e1[49..57].try_into().unwrap());
        assert_eq!(ptr1, ENTRY_BYTES as u64, "second entry replay_ptr must be 57");
    }
}
