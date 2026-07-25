# ReceiptWorker Architecture: Off-Hot-Path Deterministic Hashing

The `ReceiptWorker` acts as the critical bridge between the ultra-strict, constant-time execution of the BCINR authoritative hot path and the computationally expensive cryptographic hashing required for causal receipts. By decoupling these concerns, BCINR strictly adheres to the Radon Law ($CC=1$) and the Zero-Allocation Boundary.

## 1. Hot Path Decoupling via `LockFreeMpmcRing`

To ensure the Petri net execution engine (`petri_tick`) is never blocked by cryptographic overhead, BLAKE3 hashing is explicitly forbidden on the hot path. Instead, telemetry is deferred via a lock-free Multi-Producer Multi-Consumer (MPMC) ring buffer:

*   **Zero-Allocation & $CC=1$ Push**: During a tick, `petri_tick` pushes `EventWorkItem` structures (containing `run_id`, `op_idx`, `op_trace_so_far`, and `tick_fired_mask`) to a `LockFreeMpmcRing<EventWorkItem, 64>`.
*   **Bounded Latency**: This push executes in ~$10$ ns using a constant-time CAS loop with strict retries, never allocating on the heap or initiating a branch backedge.

## 2. Background Dequeuing and Execution Integrity

A separate fiber or background worker thread calls `ReceiptWorker::drain()` to safely dequeue events and advance state.

*   **Safe Extraction**: `ReceiptWorker` pops events using `pop_t1`, establishing exclusive epoch ownership of slots in the lock-free ring.
*   **Execution Integrity (Admissibility)**: Digest equality does not equal semantic equivalence. Before sealing a receipt, the worker verifies that the operations that fired together (`tick_fired_mask`) are admissible according to the `ConcurrencyGuardTable`.
*   **Refusal Path**: If an event contains an inadmissible tick (e.g., mutually exclusive operations firing simultaneously), the entire run trace is marked as `had_inadmissible_tick`. It is completely refused and discarded once full coverage is reached, rather than generating a mathematically sound but semantically fraudulent hash chain.

## 3. `OcelCausalFrame` and Off-Path BLAKE3 Hashing

Once a workflow run is validated (its `op_trace` reaches the `full_mask`), the cryptographic seal is generated completely off the hot path. 

While the fundamental `ReceiptWorker` log entries are compact 57-byte layouts, detailed causal history is captured using `OcelCausalFrame`s (Object-Centric Event Logs):

*   **Cache-Aligned Frame Layout**: `OcelCausalFrame` is a 128-byte cache-aligned (`#[repr(C, align(64))]`) structure. It tracks `instruction_id`, `fired_mask`, `denial` polarity, packed `obj_refs`, and wall-clock timestamps.
*   **Bump-Allocated Arena (`OcelEmitArena`)**: To maintain zero-allocation bounds off the hot path, causal frames are sequentially bump-allocated in a pre-allocated 4096-frame array (`Box<[OcelCausalFrame; 4096]>`).
*   **Deterministic Serialization**: Before hashing, the 128-byte frame is deterministically compacted into a **99-byte** buffer. All implicit compiler paddings are stripped to guarantee bit-for-bit reproducibility.
*   **Chained BLAKE3 Hashing**: The BLAKE3 hasher updates its state using the predecessor's hash (`prev_chain_hash`) followed by the deterministic 99-byte frame buffer. The worker maintains the rolling chain hash, eventually returning an immutable, verifiable cryptographic receipt of the run.

### Flow Summary

```text
petri_tick (Authoritative Hot Path)
  | 
  |--(~10ns push_t1)--> LockFreeMpmcRing<EventWorkItem, 64>
                        (Zero-Allocation Boundary)

ReceiptWorker Background Thread (Slow Rail)
  |
  |-- 1. drain() from LockFreeMpmcRing
  |-- 2. ConcurrencyGuardTable admissibility check
  |-- 3. Accumulate op_trace
  |-- 4. OcelEmitArena bump-allocates OcelCausalFrame
  |-- 5. BLAKE3 hash of compacted 99-byte frame & prev_hash
  v
ReceiptLog / OcelCausalReceipt (Sealed)
```
