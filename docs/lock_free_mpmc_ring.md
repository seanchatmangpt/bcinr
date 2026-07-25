# LockFreeMpmcRing and Deferred BLAKE3 Hashing

The `bcinr` codebase implements a highly optimized, lock-free Multi-Producer Multi-Consumer (MPMC) ring buffer to decouple hot-path execution events from the heavy cryptographic hashing required for receipt generation.

## 1. `LockFreeMpmcRing` Implementation

The `LockFreeMpmcRing<T, N>` is a bounded, lock-free MPMC queue with deterministic index arithmetic designed to strictly adhere to the Radon Law ($CC=1$) and the Zero-Allocation Boundary.

### Contract and Guarantees
- **Timing Envelope**: Operations have a strict T1 aggregate budget (≤ 200 ns) with a bounded maximum of 10 retries on contention. Each primitive CAS attempt is budgeted at ~10 ns.
- **Allocation**: Zero heap allocations. Memory is bounded and fixed at compile time (capacity `N` must be a power of two).
- **Branchless Masking Core**: Implemented using mask-based state selection and branchless arithmetic. Pointer masking is explicitly avoided; instead, conditional branching on `cas_success` safely routes writes either to the exclusively owned slot or a discarded dummy buffer, avoiding any dangling pointer construction.

### Lock-Free Mechanics
The ring relies on two core index counters (`head` and `tail`) backed by `AtomicU32` and an array of `Slot<T>` elements. Each slot contains its own `sequence` atomic counter and an `UnsafeCell<T>` for the payload.
- **Push (`push_t1`)**: 
  1. Computes the slot index using `head & mask`.
  2. Verifies the slot is free by checking if `sequence == head`.
  3. Attempts a Compare-And-Swap (CAS) on `head`.
  4. If CAS succeeds, it establishes exclusive epoch ownership of the slot, safely writes the payload, and updates the `sequence` to `head + 1` (with `Release` semantics) to signal consumers.
- **Pop (`pop_t1`)**:
  1. Computes the slot index using `tail & mask`.
  2. Verifies data presence by checking if `sequence == tail + 1`.
  3. Attempts CAS on `tail`.
  4. If successful, reads the data and updates the `sequence` to `tail + mask + 1` (with `Release` semantics) to signal producers that the slot is vacant.

## 2. ReceiptWorker & Deferred Hashing

The `ReceiptWorker` consumes events from a 64-capacity `LockFreeMpmcRing<EventWorkItem, 64>` to process BLAKE3 hashes entirely off the hot path. 

### Architectural Flow
```text
petri_tick  → push EventWorkItem → LockFreeMpmcRing<EventWorkItem, 64>
                                          ↓ (drain, off hot path)
                                   ReceiptWorker::drain()
                                          ↓
                                   BLAKE3(prev_chain_hash ‖ run_id ‖ op_trace ‖ topology_tag)
                                          ↓
                                   ReceiptLog::append() → 57-byte entry
```
Because BLAKE3 is never called inside `petri_tick`, the hot path only incurs the ~10 ns `push_t1` overhead. A separate fiber or thread subsequently invokes `ReceiptWorker::drain()`.

### Accumulation and Execution Integrity
As items are drained from the ring, `ReceiptWorker` tracks up to 16 concurrent execution runs (`MAX_PENDING`). 
- **Admissibility Check**: Before a tick's fired operations are folded into a run's accumulated trace, the worker verifies `guards.admits()` against the `ConcurrencyGuardTable`. If a tick's execution mask is inadmissible (e.g. operations fired together that shouldn't have), the run is flagged as `had_inadmissible_tick`.
- **Trace Accumulation**: The `op_trace` bitmask accumulates as `EventWorkItem`s are processed.
- **Refusal vs. Sealing**: Once a run completes (i.e., `op_trace & full_mask == full_mask`), the worker checks the admissibility flag. If the run contained an inadmissible tick, it is entirely refused and discarded (digest equality does not equal semantic equivalence). If valid, it proceeds to cryptographic sealing.

### Cryptographic Chaining
Receipts are cryptographically chained to prevent tampering and reordering. The BLAKE3 hash function ingests the following components in order:
1. `prev_chain_hash` (32 bytes - chaining from the last sealed receipt)
2. `run_id` (8 bytes LE)
3. `op_trace` (8 bytes LE)
4. `topo_tag` (1 byte, with MSB serving as an overflow bit)

The resulting 57-byte sealed receipt entry is appended to a fixed-capacity `ReceiptLog`.

### Receipt Entry Layout (57 bytes)
| Bytes | Field        | Type |
|-------|--------------|------|
| 0..8  | `run_id`     | u64  |
| 8..16 | `op_trace`   | u64  |
| 16    | `topo_tag`   | u8   |
| 17..49| `chain_hash` | [u8; 32] |
| 49..57| `replay_ptr` | u64  |
