# The Hot Path / Slow Rail Boundary in BCINR

The boundary between the authoritative Hot Path (`petri_tick`) and the Slow Rail (`ReceiptWorker`) is designed to strictly preserve the Radon Law ($CC=1$) and the Zero-Allocation Boundary, while offloading computationally expensive cryptographic attestation (BLAKE3 hashing) and causal event logging.

## 1. The Decoupling Mechanism: `LockFreeMpmcRing`
To transfer execution state safely out of the hot path without blocking or allocating memory, BCINR utilizes a `LockFreeMpmcRing<EventWorkItem, 64>`.

- **Zero Allocation**: The ring is bounded and fixed at compile-time with a power-of-two capacity (64). Elements are pushed into pre-allocated slots, meaning no heap allocation ever occurs on the hot path.
- **$CC=1$ Compliance**: The hot path pushes telemetry using a constant-time CAS (Compare-And-Swap) loop (`push_t1`). It strictly bounds execution to a ~10 ns budget per attempt, with an aggregate T1 budget of ≤ 200 ns (maximum 10 retries on contention).
- **Branchless Masking**: Pointer masking and control-flow branches are explicitly avoided. Instead, conditional branching on `cas_success` safely routes writes either to the exclusively owned slot or a discarded dummy buffer. This avoids dangling pointers without introducing complex control flow.

## 2. State Transfer: `EventWorkItem`
The execution state transferred across the boundary is highly compact. The `petri_tick` hot path pushes an `EventWorkItem` into the lock-free ring containing:
- `run_id`: The 64-bit identifier of the workflow run.
- `op_idx`: The current operation index.
- `op_trace_so_far`: The accumulated trace mask up to this point.
- `tick_fired_mask`: The complete mask of all operations that fired together in the current tick.

## 3. Off-Hot-Path Processing: `ReceiptWorker` (Slow Rail)
A separate background thread or fiber drains the ring buffer (`pop_t1`) to process the events. Because BLAKE3 is strictly forbidden on the hot path, all hashing occurs here.

- **Admissibility Verification**: Digest equality does not guarantee semantic equivalence. Before folding a tick's events into the accumulated execution trace, the worker checks the `tick_fired_mask` against a `ConcurrencyGuardTable`. If the tick contains mutually exclusive operations, the run is flagged as `had_inadmissible_tick` and ultimately discarded (refused) rather than mathematically sealed.
- **Zero-Allocation Logging (`OcelEmitArena`)**: To maintain zero allocations even on the Slow Rail, the worker sequentially bump-allocates cache-aligned 128-byte `OcelCausalFrame` structures using an `OcelEmitArena` (backed by a pre-allocated `Box<[OcelCausalFrame; 4096]>`).
- **Deterministic Cryptographic Hashing**: Before hashing, the 128-byte frame is deterministically compacted down to 99 bytes to strip any implicit compiler padding, ensuring bit-for-bit reproducibility. It is then hashed via BLAKE3 alongside the predecessor's hash (`prev_chain_hash`) to maintain a tamper-proof causal chain.
- **Sealing the Receipt**: Upon successful full trace accumulation, a compact 57-byte entry (containing `run_id`, `op_trace`, `topo_tag`, `chain_hash`, and a `replay_ptr`) is appended to a fixed-capacity `ReceiptLog`.
