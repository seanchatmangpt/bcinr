# POWL Receipt Worker and Execution Topology

There is no single `executor.rs` or `worker.rs`; rather, the background processing worker is located at **`crates/bcinr-powl/src/receipt_worker.rs`**, and the core execution logic resides in **`scheduler.rs`**, **`scheduler_wired.rs`**, and **`dispatcher.rs`**.

### 1. How `PowlTape` is Consumed
A `PowlTape` represents a compiled, flat execution graph (like a Petri net) bounded to fixed capacities (e.g., 64 operations). It is consumed by the schedulers strictly without data-dependent branches (`CC=1` requirement):
- **Structs**: It contains an array of `Powl64Op` entries, each defining prerequisites (`pred_mask`) and outputs (`succ_mask`).
- **Bitwise Evaluation**: Schedulers (like `scheduler.rs` and `scheduler_wired.rs`) consume the tape using bitset arithmetic to process state. For example, `PowlRunState` packs `done_mask`, `active_mask`, and `check_mask` into tightly packed 64-bit integers.
- **Branchless Ticks**: `PriorityPetriEngine` or custom branchless helpers (e.g., `pred_satisfied`) evaluate the transition firing logic in constant time, replacing sequential `if/else` checks with parallel bitwise equations to progress the tape state.

### 2. Concurrent State Management
The executor engine strictly avoids locks, heap allocation (`Vec`/`Box`), and dynamic branching, relying entirely on atomic and lock-free structures. 
- **`LockFreeSlab` vs. `LockFreeMpmcRing`**: While `LockFreeSlab` is the standard project primitive for fixed-size, independent lifecycle objects, the POWL execution pipeline uses a **`LockFreeMpmcRing<EventWorkItem, 64>`** for worker concurrency. The hot-path scheduler executes a bounded, lock-free `push_t1` into the ring (taking ~10ns), which the `ReceiptWorker` safely drains asynchronously to compute BLAKE3 hashes off the hot path. 
- **Atomic Dispatch Slots (`dispatcher.rs`)**: For concurrent operations (like Par/Concur), state is managed branchlessly via the `BpadDispatcher` (Bit-Parallel Atomic Dispatcher). This utilizes an 8-lane, 64-byte-aligned array of CAS slots (`BpadSlot`) backed purely by `core::sync::atomic::AtomicU32`. Each lane occupies a distinct cache line to avoid false sharing and guarantees bounded single-CAS state transitions.
