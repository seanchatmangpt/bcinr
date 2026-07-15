//! receipt_worker — Off-hot-path BLAKE3 receipt hashing via event ring drain.
//!
//! # Architecture
//!
//! ```text
//! petri_tick  → push EventWorkItem → LockFreeMpmcRing<EventWorkItem, 64>
//!                                           ↓ (drain, off hot path)
//!                                    ReceiptWorker::drain()
//!                                           ↓
//!                                    BLAKE3(prev_chain_hash ‖ run_id ‖ op_trace ‖ topology_tag)
//!                                           ↓
//!                                    ReceiptLog::append() → 57-byte entry
//! ```
//!
//! BLAKE3 is **never called inside `petri_tick`**. Only a ~10 ns `push_t1` is
//! on the hot path. The worker drains in a separate budget window (fiber or
//! worker thread) and appends sealed receipt entries to the log.
//!
//! # Execution integrity: admissibility is checked before sealing
//!
//! Each `EventWorkItem`'s `tick_fired_mask` is the *complete* set of ops that
//! fired together in one tick (see that field's own doc comment) — the same
//! shape of information [`crate::scheduler::scheduler_tick_guarded`] and
//! `bcinr_powl_receipt::execution::seal_execution_receipt` (a plain code
//! span, not a doc-link: this crate does not depend on `bcinr-powl-receipt`
//! -- correctly, since that crate depends on this one, not the reverse -- so
//! an intra-doc link here could never resolve regardless of how it were
//! spelled) check against a [`ConcurrencyGuardTable`] before trusting a
//! `FireSet`. [`ReceiptWorker::drain`]
//! does the same check here: before a tick's fired ops are folded into a
//! run's accumulated trace, `guards.admits(&tick's EventSet)` must hold. A
//! run that ever saw an inadmissible tick is never sealed into
//! [`ReceiptLog`] — see [`DrainResult::refused`] — closing the gap where this
//! module could seal a hash-chained, internally-consistent-looking receipt
//! for a run whose own scheduler tick fired a jointly-inadmissible set
//! (`digest equality != semantic equivalence`: a well-formed BLAKE3 chain
//! says nothing about whether the ops it attests to were ever jointly
//! executable). Callers on the unguarded [`crate::scheduler_wired::petri_tick`]
//! path (which never consults a guard table at all) should pass
//! [`ConcurrencyGuardTable::empty`] to preserve exactly the old
//! always-admits behavior — this check is additive, not a new requirement
//! on existing callers.
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

use crate::ocel::OcelLog;
use crate::scheduler::mask_to_event_set;
use crate::scheduler_wired::EventWorkItem;
use crate::tape::v2::ConcurrencyGuardTable;
use bcinr_logic::patterns::deterministic_mpmc::LockFreeMpmcRing;

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
        if idx < self.count {
            Some(&self.buf[idx])
        } else {
            None
        }
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
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pending — accumulator per run_id
// ---------------------------------------------------------------------------

struct Pending {
    run_id: u64,
    op_trace: u64,
    topo_tag: u8,
    active: bool,
    had_overflow: bool,
    /// Set when any tick contributing to this run's `op_trace` had a
    /// `tick_fired_mask` the guard table passed to `drain` did not admit.
    /// A run with this set is refused (never sealed), not sealed-with-a-flag
    /// — see the module doc comment's "Execution integrity" section.
    had_inadmissible_tick: bool,
}

impl Pending {
    const fn empty() -> Self {
        Self {
            run_id: 0,
            op_trace: 0,
            topo_tag: 0,
            active: false,
            had_overflow: false,
            had_inadmissible_tick: false,
        }
    }
}

// ---------------------------------------------------------------------------
// DrainResult
// ---------------------------------------------------------------------------

/// Outcome of one [`ReceiptWorker::drain`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DrainResult {
    /// Number of receipts sealed and appended to [`ReceiptLog`] this call.
    pub sealed: u32,
    /// Number of runs that reached `full_mask` but were refused (never
    /// sealed) this call because some tick contributing to their trace had
    /// an inadmissible `tick_fired_mask`. `FireSet != ReadySet`, and neither
    /// is automatically a receipt: reaching full coverage of `full_mask`
    /// is necessary but not sufficient for a run to be sealed.
    pub refused: u32,
}

// ---------------------------------------------------------------------------
// ReceiptWorker
// ---------------------------------------------------------------------------

/// Drains an `EventWorkItem` ring, accumulates per-run op traces, and
/// finalises receipts with real BLAKE3 when the full mask is seen.
pub struct ReceiptWorker {
    pending: [Pending; MAX_PENDING],
    /// Running chain head: previous receipt's chain_hash, fed into next BLAKE3.
    /// Initialized to all-zeros; updated after each sealed receipt.
    prev_chain_hash: [u8; 32],
    pub log: ReceiptLog,
    /// Object-Centric Event Log for POWL process conformance verification.
    pub ocel: OcelLog,
    /// Cumulative count of ring overflow events (pushes that returned 0).
    pub overflow_count: u64,
    /// Cumulative count of runs refused (never sealed) because some tick
    /// contributing to their trace had an inadmissible `tick_fired_mask`.
    pub refused_count: u64,
}

impl ReceiptWorker {
    pub const fn new() -> Self {
        const EMPTY: Pending = Pending::empty();
        Self {
            pending: [EMPTY; MAX_PENDING],
            prev_chain_hash: [0u8; 32],
            log: ReceiptLog::new(),
            ocel: OcelLog::new(),
            overflow_count: 0,
            refused_count: 0,
        }
    }

    /// Returns the cumulative overflow count.
    pub fn overflow(&self) -> u64 {
        self.overflow_count
    }

    /// Returns the cumulative count of runs refused for an inadmissible tick.
    pub fn refused(&self) -> u64 {
        self.refused_count
    }

    /// Drain up to `budget` items from `ring`, checking each item's
    /// `tick_fired_mask` against `guards` before letting it contribute to a
    /// run's accumulated trace (see the module doc comment's "Execution
    /// integrity" section). Callers on the unguarded `petri_tick` path (no
    /// concurrency gating at all) should pass `ConcurrencyGuardTable::empty()`
    /// — it admits everything, so this reduces to the pre-existing
    /// always-accumulate behavior exactly.
    ///
    /// # Complexity
    /// O(`budget` * `guards.nonfaces.len()`) — one
    /// [`ConcurrencyGuardTable::admits`] call per drained item, each itself
    /// O(`guards.nonfaces.len()`); data-dependent on both how many items are
    /// actually queued (bounded by `budget` and the ring's contents) and how
    /// many nonfaces the caller's guard table carries.
    pub fn drain(
        &mut self,
        ring: &LockFreeMpmcRing<EventWorkItem, RING_CAPACITY>,
        full_mask: u64,
        budget: usize,
        new_overflows: u64,
        guards: &ConcurrencyGuardTable,
    ) -> DrainResult {
        self.overflow_count = self.overflow_count.saturating_add(new_overflows);

        debug_assert!(full_mask != 0, "full_mask must be nonzero");

        let mut sealed = 0u32;
        let mut refused = 0u32;

        for _ in 0..budget {
            let (maybe_item, _) = ring.pop_t1();
            let item = match maybe_item {
                Some(i) => i,
                None => break,
            };

            // Find or allocate a pending slot for this run_id.
            let slot = match self.find_or_alloc(item.run_id, item.kind_tag) {
                Ok(s) => s,
                Err(()) => continue, // all slots full; skip rather than corrupt
            };

            // Execution-integrity gate: a tick's fired ops may only
            // contribute to a run's trace if the joint set that fired
            // together is admissible under `guards`. This does not drop the
            // event (OCEL still records it, and op_trace still accumulates
            // it below, so completion detection still works) — it instead
            // permanently marks the run so it is refused, not sealed, once
            // it would otherwise complete. See `InadmissibleFiredSet` in
            // `bcinr_powl_receipt::execution` for the sibling pipeline this
            // mirrors.
            if !guards.admits(&mask_to_event_set(item.tick_fired_mask)) {
                self.pending[slot].had_inadmissible_tick = true;
            }

            // Accumulate via `op_trace_so_far` (grows across a tick's own
            // events, see that field's doc comment), not `tick_fired_mask`
            // (constant across a tick's events) — using the constant value
            // here would make the *first* event of a run-completing tick
            // look like the run's completion by itself, firing the
            // seal/refuse check once per fired op instead of once per tick.
            self.pending[slot].op_trace |= item.op_trace_so_far;
            let _ = self
                .ocel
                .record_op_fired(item.run_id, item.op_idx, item.kind_tag);

            // If all ops have fired, finalise (or refuse) the receipt.
            if self.pending[slot].op_trace & full_mask == full_mask {
                if self.pending[slot].had_inadmissible_tick {
                    // Refuse: never seal a receipt for a run that included an
                    // inadmissible tick, no matter how the rest of the trace
                    // completed. Release the slot so the run_id can be
                    // reused rather than leaking it forever.
                    self.pending[slot].active = false;
                    self.refused_count = self.refused_count.saturating_add(1);
                    refused += 1;
                    continue;
                }

                let (run_id, op_trace, topo_tag) = (
                    self.pending[slot].run_id,
                    self.pending[slot].op_trace,
                    self.pending[slot].topo_tag,
                );
                let overflow_bit: u8 = if self.pending[slot].had_overflow {
                    0x80
                } else {
                    0x00
                };
                let entry = self.build_entry(run_id, op_trace, topo_tag | overflow_bit);
                // Update running chain head before appending.
                let mut chain_hash = [0u8; 32];
                chain_hash.copy_from_slice(&entry[17..49]);
                self.prev_chain_hash = chain_hash;
                self.log.append(entry);
                self.pending[slot].active = false;
                sealed += 1;
            }
        }

        DrainResult { sealed, refused }
    }

    /// Returns `Ok(slot)` if an existing or free slot is found, `Err(())` if all
    /// 16 slots are occupied by different active runs.
    fn find_or_alloc(&mut self, run_id: u64, kind_tag: u8) -> Result<usize, ()> {
        // Search for existing active slot.
        for (i, p) in self.pending.iter().enumerate() {
            if p.active && p.run_id == run_id {
                return Ok(i);
            }
        }
        // Find a free slot.
        for (i, p) in self.pending.iter_mut().enumerate() {
            if !p.active {
                p.run_id = run_id;
                p.op_trace = 0;
                p.topo_tag = kind_tag;
                p.active = true;
                p.had_overflow = false;
                return Ok(i);
            }
        }
        // All slots occupied — caller must skip this item.
        Err(())
    }

    /// Build a 57-byte receipt entry using BLAKE3, chained from `prev_chain_hash`.
    ///
    /// Hash inputs (in order):
    ///   prev_chain_hash (32 bytes) ‖ run_id (8 LE) ‖ op_trace (8 LE) ‖ topo_tag (1)
    fn build_entry(&self, run_id: u64, op_trace: u64, topo_tag: u8) -> [u8; ENTRY_BYTES] {
        let replay_ptr = self.log.next_offset();

        let mut h = blake3::Hasher::new();
        // Chain link: previous receipt's hash is the first input.
        h.update(&self.prev_chain_hash);
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

    fn make_ring() -> LockFreeMpmcRing<EventWorkItem, RING_CAPACITY> {
        LockFreeMpmcRing::new_checked().unwrap()
    }

    /// Admits everything — reduces `drain`'s admissibility check to the
    /// pre-existing always-accumulate behavior, for tests unrelated to
    /// execution-integrity gating.
    fn no_guards() -> ConcurrencyGuardTable {
        ConcurrencyGuardTable::empty()
    }

    #[test]
    fn content_hash_nonzero_for_two_op_tape() {
        use crate::compiler::{compile_powl, PowlAstNode};
        use crate::typestate::HasPowlTape;

        let tape = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();

        let hash = tape.content_hash();
        assert_ne!(
            hash, [0u8; 32],
            "content_hash must be non-zero for a real tape"
        );
    }

    #[test]
    fn content_hash_differs_for_different_pred_masks() {
        use crate::compiler::{compile_powl, PowlAstNode};
        use crate::typestate::HasPowlTape;

        let seq = compile_powl(&PowlAstNode::Sequence(vec![
            PowlAstNode::Atom("a"),
            PowlAstNode::Atom("b"),
        ]))
        .unwrap();
        let par = compile_powl(&PowlAstNode::PartialOrder {
            children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
            edges: vec![],
        })
        .unwrap();

        assert_ne!(
            seq.content_hash(),
            par.content_hash(),
            "sequential and parallel tapes must have different content hashes"
        );
    }

    #[test]
    fn worker_seals_receipt_on_full_mask() {
        let ring = make_ring();
        let run_id = 42u64;
        let full_mask = 0b11u64; // two ops

        // Two separate single-op ticks: tick 1 fires {0} alone, tick 2
        // fires {1} alone. tick_fired_mask == op_trace_so_far in each
        // event since a lone op's tick is trivially "complete" at push time.
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id,
            op_trace_so_far: 0b01,
            tick_fired_mask: 0b01,
            kind_tag: 0,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 1,
            run_id,
            op_trace_so_far: 0b10,
            tick_fired_mask: 0b10,
            kind_tag: 0,
        });

        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, full_mask, 10, 0, &no_guards());

        assert_eq!(sealed.sealed, 1, "one run must be sealed");
        assert_eq!(worker.log.len(), 1, "one receipt entry in log");
    }

    #[test]
    fn worker_chains_two_receipts() {
        let ring = make_ring();
        let full_mask = 0b1u64; // single-op runs

        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 1,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 0,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 2,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 0,
        });

        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, full_mask, 10, 0, &no_guards());

        assert_eq!(sealed.sealed, 2);
        assert_eq!(worker.log.len(), 2);

        let e0 = worker.log.entry(0).unwrap();
        let e1 = worker.log.entry(1).unwrap();

        // Chain hashes must differ.
        assert_ne!(
            &e0[17..49],
            &e1[17..49],
            "chain hashes must differ across runs"
        );

        // entry[1]'s chain_hash must include entry[0]'s chain_hash as input.
        // Recompute: BLAKE3(e0_chain_hash ‖ run_id=2 ‖ op_trace=1 ‖ topo_tag=0)
        let e0_chain_hash: [u8; 32] = e0[17..49].try_into().unwrap();
        let run_id_2: u64 = 2;
        let op_trace_1: u64 = 1;
        let mut h = blake3::Hasher::new();
        h.update(&e0_chain_hash);
        h.update(&run_id_2.to_le_bytes());
        h.update(&op_trace_1.to_le_bytes());
        h.update(&[0u8]); // topo_tag
        let expected_e1_hash = *h.finalize().as_bytes();
        assert_eq!(&e1[17..49], &expected_e1_hash,
            "entry[1] chain_hash must be BLAKE3(entry[0].chain_hash ‖ run_id ‖ op_trace ‖ topo_tag)");
    }

    #[test]
    fn replay_ptr_is_byte_offset() {
        let ring = make_ring();
        let full_mask = 0b1u64;

        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 10,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 0,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 20,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 0,
        });

        let mut worker = ReceiptWorker::new();
        worker.drain(&ring, full_mask, 10, 0, &no_guards());

        // First entry: replay_ptr at offset 0.
        let e0 = worker.log.entry(0).unwrap();
        let ptr0 = u64::from_le_bytes(e0[49..57].try_into().unwrap());
        assert_eq!(ptr0, 0, "first entry replay_ptr must be 0");

        // Second entry: replay_ptr at offset ENTRY_BYTES.
        let e1 = worker.log.entry(1).unwrap();
        let ptr1 = u64::from_le_bytes(e1[49..57].try_into().unwrap());
        assert_eq!(
            ptr1, ENTRY_BYTES as u64,
            "second entry replay_ptr must be 57"
        );
    }

    // ---------------------------------------------------------------------------
    // Proptests
    // ---------------------------------------------------------------------------

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_entry_layout_run_id_at_offset_0(run_id: u64, op_trace: u64) {
            // Build an entry directly via build_entry and verify run_id is at
            // bytes [0..8] in LE and op_trace is at bytes [8..16] in LE.
            let worker = ReceiptWorker::new();
            let entry = worker.build_entry(run_id, op_trace, 0u8);

            let stored_run_id = u64::from_le_bytes(entry[0..8].try_into().unwrap());
            prop_assert_eq!(stored_run_id, run_id,
                "run_id at bytes [0..8]: stored {:#018x}, expected {:#018x}",
                stored_run_id, run_id);

            let stored_op_trace = u64::from_le_bytes(entry[8..16].try_into().unwrap());
            prop_assert_eq!(stored_op_trace, op_trace,
                "op_trace at bytes [8..16]: stored {:#018x}, expected {:#018x}",
                stored_op_trace, op_trace);
        }
    }

    #[test]
    fn chain_is_linked() {
        let ring = make_ring();
        let full_mask = 0b1u64;

        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 100,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 0,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 200,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 1,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 300,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 2,
        });

        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, full_mask, 10, 0, &no_guards());
        assert_eq!(sealed.sealed, 3);

        let e0 = worker.log.entry(0).unwrap();
        let e1 = worker.log.entry(1).unwrap();
        let e2 = worker.log.entry(2).unwrap();

        let hash0: [u8; 32] = e0[17..49].try_into().unwrap();
        let hash1: [u8; 32] = e1[17..49].try_into().unwrap();
        let hash2: [u8; 32] = e2[17..49].try_into().unwrap();

        // Verify entry[0]: BLAKE3([0u8;32] ‖ run_id=100 ‖ op_trace=1 ‖ topo_tag=0)
        {
            let mut h = blake3::Hasher::new();
            h.update(&[0u8; 32]);
            h.update(&100u64.to_le_bytes());
            h.update(&1u64.to_le_bytes());
            h.update(&[0u8]);
            assert_eq!(
                hash0,
                *h.finalize().as_bytes(),
                "entry[0] chain_hash mismatch"
            );
        }

        // Verify entry[1] depends on entry[0]'s hash.
        {
            let mut h = blake3::Hasher::new();
            h.update(&hash0);
            h.update(&200u64.to_le_bytes());
            h.update(&1u64.to_le_bytes());
            h.update(&[1u8]);
            assert_eq!(
                hash1,
                *h.finalize().as_bytes(),
                "entry[1] must chain from entry[0]"
            );
        }

        // Verify entry[2] depends on entry[1]'s hash.
        {
            let mut h = blake3::Hasher::new();
            h.update(&hash1);
            h.update(&300u64.to_le_bytes());
            h.update(&1u64.to_le_bytes());
            h.update(&[2u8]);
            assert_eq!(
                hash2,
                *h.finalize().as_bytes(),
                "entry[2] must chain from entry[1]"
            );
        }
    }

    #[test]
    fn ring_overflow_at_65_items_drops_without_corruption() {
        let ring = make_ring();
        for i in 0u64..65 {
            ring.push_t1(EventWorkItem {
                op_idx: 0,
                run_id: i,
                op_trace_so_far: 0b1,
                tick_fired_mask: 0b1,
                kind_tag: 0,
            });
        }
        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, 0b1, 200, 0, &no_guards());
        assert!(
            sealed.sealed <= 64,
            "must not seal more than ring capacity: sealed={}",
            sealed.sealed
        );
        assert_eq!(
            worker.log.len() as u32,
            sealed.sealed,
            "log len must equal sealed count"
        );
    }

    #[test]
    fn op_trace_accumulation_is_monotone_under_reordered_ring_drain() {
        let ring = make_ring();
        let run_id = 99u64;
        let full_mask = 0b111u64;
        // Push in reverse order. All three come from one concurrent 3-op
        // tick, so tick_fired_mask (constant) is the tick's complete set
        // {0,1,2} = 0b111 on every event, while op_trace_so_far keeps its
        // original progressive (order-of-push) values to exercise the
        // "monotone under reordering" property this test is named for.
        ring.push_t1(EventWorkItem {
            op_idx: 2,
            run_id,
            op_trace_so_far: 0b100,
            tick_fired_mask: 0b111,
            kind_tag: 0,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 1,
            run_id,
            op_trace_so_far: 0b110,
            tick_fired_mask: 0b111,
            kind_tag: 0,
        });
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id,
            op_trace_so_far: 0b111,
            tick_fired_mask: 0b111,
            kind_tag: 0,
        });
        let mut worker = ReceiptWorker::new();
        let sealed = worker.drain(&ring, full_mask, 10, 0, &no_guards());
        assert_eq!(sealed.sealed, 1, "must seal after all three ops arrive");

        // Verify partial trace does not seal early. This run's tick only
        // ever fires {1,2} — op 0 never appears, so tick_fired_mask is
        // 0b110 (not 0b111) and full_mask (0b111) is never reached.
        let ring2 = make_ring();
        ring2.push_t1(EventWorkItem {
            op_idx: 2,
            run_id: 200,
            op_trace_so_far: 0b100,
            tick_fired_mask: 0b110,
            kind_tag: 0,
        });
        ring2.push_t1(EventWorkItem {
            op_idx: 1,
            run_id: 200,
            op_trace_so_far: 0b110,
            tick_fired_mask: 0b110,
            kind_tag: 0,
        });
        let mut worker2 = ReceiptWorker::new();
        let sealed2 = worker2.drain(&ring2, full_mask, 10, 0, &no_guards());
        assert_eq!(sealed2.sealed, 0, "incomplete trace must not seal");
    }

    #[test]
    fn overflow_count_zero_when_ring_never_full() {
        let ring = make_ring();
        let full_mask = 0b1u64;
        ring.push_t1(EventWorkItem {
            op_idx: 0,
            run_id: 1,
            op_trace_so_far: 0b1,
            tick_fired_mask: 0b1,
            kind_tag: 0,
        });
        let mut worker = ReceiptWorker::new();
        worker.drain(&ring, full_mask, 10, 0, &no_guards());
        assert_eq!(worker.overflow(), 0);
    }

    #[test]
    fn overflow_count_reflects_passed_delta() {
        let ring = make_ring();
        let full_mask = 0b1u64;
        let mut worker = ReceiptWorker::new();
        worker.drain(&ring, full_mask, 10, 3, &no_guards());
        assert_eq!(worker.overflow(), 3);
    }

    // -------------------------------------------------------------------------
    // Execution integrity: admissibility gate (the fix this module's doc
    // comment's "Execution integrity" section describes)
    // -------------------------------------------------------------------------

    mod execution_integrity {
        use super::*;
        use crate::tape::v2::CompiledNonFace;
        use bcinr_mfw_ir::Digest;

        /// A guard table that forbids ops 0 and 1 from firing together.
        fn guards_forbid_0_and_1() -> ConcurrencyGuardTable {
            ConcurrencyGuardTable {
                nonfaces: vec![CompiledNonFace {
                    members: bcinr_mfw_ir::EventSet::empty().with(0).with(1),
                    witness_digest: Digest::hash(b"0-1-conflict"),
                }],
            }
        }

        /// The gap this closes: previously `drain` had no `guards` parameter
        /// at all, so a tick that fired an inadmissible pair sealed a
        /// perfectly well-formed, hash-chained receipt exactly like an
        /// admissible one. This proves that no longer happens: a run whose
        /// only contributing tick fired ops 0 and 1 *together* (both events
        /// carry the same `tick_fired_mask = 0b11`, matching the real
        /// `petri_tick` invariant that every event from one tick shares an
        /// identical `tick_fired_mask`) against a guard table that forbids
        /// exactly that pair is refused, not sealed.
        #[test]
        fn drain_refuses_a_run_whose_tick_fired_an_inadmissible_pair() {
            let ring = make_ring();
            let run_id = 7u64;
            let full_mask = 0b11u64;

            // Both events come from the same tick: both carry the tick's
            // complete FireSet {0,1} in tick_fired_mask (constant across the
            // tick's events), while op_trace_so_far grows progressively in
            // firing order (op 0 first, then op 1) exactly as a real
            // `petri_tick` call would produce.
            ring.push_t1(EventWorkItem {
                op_idx: 0,
                run_id,
                op_trace_so_far: 0b01,
                tick_fired_mask: 0b11,
                kind_tag: 0,
            });
            ring.push_t1(EventWorkItem {
                op_idx: 1,
                run_id,
                op_trace_so_far: 0b11,
                tick_fired_mask: 0b11,
                kind_tag: 0,
            });

            let mut worker = ReceiptWorker::new();
            let result = worker.drain(&ring, full_mask, 10, 0, &guards_forbid_0_and_1());

            assert_eq!(
                result.sealed, 0,
                "an inadmissible-tick run must not be sealed"
            );
            assert_eq!(result.refused, 1, "exactly one run must be refused");
            assert_eq!(
                worker.refused(),
                1,
                "cumulative refused_count must reflect it"
            );
            assert_eq!(
                worker.log.len(),
                0,
                "no receipt entry may be appended for a refused run"
            );
        }

        /// The same guard table (forbids {0,1} jointly) does not refuse a
        /// run whose tick fired only a set the table actually admits — the
        /// gate must not be a blanket "any nonempty guard table refuses
        /// everything" no-op in the other direction either.
        #[test]
        fn drain_seals_normally_when_tick_fired_set_is_admissible() {
            let ring = make_ring();
            let run_id = 8u64;
            let full_mask = 0b1u64; // single-op run: {0} only, never {0,1}

            ring.push_t1(EventWorkItem {
                op_idx: 0,
                run_id,
                op_trace_so_far: 0b1,
                tick_fired_mask: 0b1,
                kind_tag: 0,
            });

            let mut worker = ReceiptWorker::new();
            let result = worker.drain(&ring, full_mask, 10, 0, &guards_forbid_0_and_1());

            assert_eq!(result.sealed, 1, "an admissible-tick run must still seal");
            assert_eq!(result.refused, 0);
            assert_eq!(worker.log.len(), 1);
        }

        /// A run spanning two ticks where only the *second* tick is
        /// inadmissible must still be refused overall — `had_inadmissible_tick`
        /// must be sticky across the whole run, not reset by a later
        /// admissible-looking check, and refusal must still be detected even
        /// though the first tick alone looked fine.
        #[test]
        fn drain_refuses_when_only_a_later_tick_is_inadmissible() {
            let ring = make_ring();
            let run_id = 9u64;
            let full_mask = 0b11u64;

            // Tick 1: only op 0 fires alone — admissible under any guard
            // table that only forbids {0,1} jointly.
            ring.push_t1(EventWorkItem {
                op_idx: 0,
                run_id,
                op_trace_so_far: 0b01,
                tick_fired_mask: 0b01,
                kind_tag: 0,
            });
            // Tick 2: op 1 fires *together with* op 0 somehow being
            // re-reported as part of the same tick's FireSet (e.g. a
            // scheduler bug re-admitting an already-done op) — this is the
            // inadmissible tick.
            ring.push_t1(EventWorkItem {
                op_idx: 1,
                run_id,
                op_trace_so_far: 0b11,
                tick_fired_mask: 0b11,
                kind_tag: 0,
            });

            let mut worker = ReceiptWorker::new();
            let result = worker.drain(&ring, full_mask, 10, 0, &guards_forbid_0_and_1());

            assert_eq!(
                result.sealed, 0,
                "must refuse once any tick was inadmissible"
            );
            assert_eq!(result.refused, 1);
            assert_eq!(worker.log.len(), 0);
        }

        /// An empty guard table (the default a caller on the still-unguarded
        /// `petri_tick` path should pass) must never refuse — this is the
        /// additive/non-regressing property the module doc comment claims.
        #[test]
        fn empty_guard_table_never_refuses() {
            let ring = make_ring();
            let run_id = 10u64;
            let full_mask = 0b11u64;

            ring.push_t1(EventWorkItem {
                op_idx: 0,
                run_id,
                op_trace_so_far: 0b01,
                tick_fired_mask: 0b11,
                kind_tag: 0,
            });
            ring.push_t1(EventWorkItem {
                op_idx: 1,
                run_id,
                op_trace_so_far: 0b11,
                tick_fired_mask: 0b11,
                kind_tag: 0,
            });

            let mut worker = ReceiptWorker::new();
            let result = worker.drain(&ring, full_mask, 10, 0, &no_guards());

            assert_eq!(result.sealed, 1);
            assert_eq!(result.refused, 0);
        }

        /// Full pipeline, no hand-constructed `EventWorkItem`s: a real
        /// `petri_tick` run over a genuinely concurrent 2-op tape, feeding a
        /// real ring, drained by a real `ReceiptWorker`. This is what
        /// exercises `petri_tick`'s `debug_assert_eq!(fired_ops_accumulator,
        /// tick_fired_mask, ...)` invariant under actual concurrent firing,
        /// not just the hand-picked mask values the unit tests above use.
        ///
        /// All ticks run to completion *before* a single `drain` call, not
        /// interleaved tick/drain/tick/drain — draining a
        /// `LockFreeMpmcRing` down to fully empty and then pushing again is
        /// a separate, pre-existing bug in `bcinr_logic`'s
        /// `LockFreeMpmcRing` (reproduced standalone against the raw ring
        /// API, unrelated to this fix: `pop_t1` after a push-pop-to-empty-
        /// then-push-again sequence silently returns `(None, 0)` instead of
        /// the pushed value). That bug is out of scope for this fix — it is
        /// not one of the confirmed gaps this session closes — so this test
        /// is written to avoid tripping it rather than mask it.
        #[test]
        fn end_to_end_petri_tick_feeds_receipt_worker_for_a_real_concurrent_tape() {
            use crate::compiler::{compile_powl, PowlAstNode};
            use crate::scheduler_wired::{petri_tick, PowlPetriState};

            let tape = compile_powl(&PowlAstNode::PartialOrder {
                children: vec![PowlAstNode::Atom("a"), PowlAstNode::Atom("b")],
                edges: vec![],
            })
            .unwrap();
            let ops = tape.ops[..tape.len as usize].to_vec();
            let full_mask = (1u64 << tape.len) - 1;

            let event_ring: LockFreeMpmcRing<EventWorkItem, RING_CAPACITY> =
                LockFreeMpmcRing::new_checked().unwrap();
            let mut state = PowlPetriState::new(tape.entry_mask);
            let run_id = 555u64;

            let mut total_overflow = 0u64;
            for _ in 0..10 {
                if state.check.words[0] == 0 {
                    break;
                }
                let result = petri_tick(&ops, &mut state, None, Some(&event_ring), run_id);
                total_overflow += u64::from(result.event_overflow_count);
            }
            assert_eq!(
                state.check.words[0], 0,
                "the whole tape must have run to completion"
            );

            let mut worker = ReceiptWorker::new();
            let drained = worker.drain(&event_ring, full_mask, 64, total_overflow, &no_guards());

            assert_eq!(
                drained.sealed, 1,
                "the run must seal exactly once, driven entirely by real petri_tick output"
            );
            assert_eq!(drained.refused, 0);
            assert_eq!(worker.log.len(), 1);
        }
    }
}
