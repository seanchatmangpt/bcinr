# Role of `LockFreeMpmcRing` in `bcinr`

The `LockFreeMpmcRing` in the `bcinr` codebase serves as a deterministic, wait-free, Multi-Producer Multi-Consumer (MPMC) queue. Its primary role is to act as a zero-allocation, bounded-latency conduit that safely passes data—such as high-frequency execution events and telemetry—from the deterministic hot path to background threads (like the "slow rail") without violating the strict performance and predictability mandates of the `#![no_std]` substrate.

## 1. Zero-Allocation and Bounded Execution (`#![no_std]`)

In a `#![no_std]` environment where the "Zero-Allocation Boundary" dictates 0 heap allocations, `LockFreeMpmcRing` is defined with a fixed capacity at compile time (`const N: usize` where `N` is a power of two). The queue encapsulates an array of `Slot<T>` elements constructed exactly once at initialization. 

Because `bcinr` strictly regulates latency (T1 aggregate budget of ≤ 200 ns with ~10 ns per CAS attempt), `LockFreeMpmcRing` guarantees a fixed worst-case execution time (WCET) by capping operations at a maximum of 10 retries on contention. If the ring is full or contention is too high, operations predictably fail rather than block, fully avoiding variable-latency blocking.

## 2. Lock-Free and Wait-Free Epoch Tracking

Data is passed across threads without taking mutex locks by utilizing an **Epoch-based sequence tracking mechanism**. The ring maintains two monotonically increasing atomic counters (`head` for producers, `tail` for consumers), and a per-slot atomic `sequence` counter that tracks the slot's current "epoch" (or lap).

Instead of locking, threads rely on Compare-And-Swap (CAS) instructions and strict mathematical validation of these sequence counters:

- **Pushing (Producers)**: A producer computes its target slot using `head & mask`. It verifies the slot is free by checking if `slot.sequence == head`. If this precisely matches, the slot is in the expected epoch. The producer claims exclusive ownership of the slot via a CAS on the `head` counter (`head → head + 1`), writes its data using an `UnsafeCell`, and then advances the slot's sequence to `head + 1` with `Ordering::Release` to signal consumers.
- **Popping (Consumers)**: A consumer computes its slot using `tail & mask`. It checks if the slot contains fresh data by verifying `slot.sequence == tail + 1` (meaning a producer has completed its write). After claiming ownership via a CAS on `tail`, the consumer reads the data, and then updates the slot's sequence to `tail + mask + 1` to signal to producers that the slot is vacant for the next lap.
- **Write-in-Progress State**: Because the slot's `sequence` is only updated *after* the data is written, a consumer will naturally observe a sequence mismatch (`sequence == tail` instead of `tail + 1`) while a producer is writing, gracefully preventing dirty reads without requiring a lock.

The ABA problem is avoided because `head` and `tail` are wrapping `u32` monotonic counters, meaning a full wrap requires over 4 billion operations.

## 3. Passing Data to the Slow Rail (ReceiptWorker)

A concrete use-case of this structure in `bcinr` is decoupling hot-path event execution from heavy cryptographic operations. 

For instance, `bcinr` uses a `LockFreeMpmcRing<EventWorkItem, 64>` to pass execution traces from the `petri_tick` (hot path) to the `ReceiptWorker` (slow rail). The hot path quickly pushes `EventWorkItem`s into the queue in ~10 ns and moves on. A separate background fiber (the `ReceiptWorker`) drains the ring, accumulates the event traces, validates admissibility, and performs computationally heavy BLAKE3 hashing to cryptographically chain receipts (`prev_chain_hash ‖ run_id ‖ op_trace ‖ topology_tag`). By utilizing the `LockFreeMpmcRing`, the hot path is entirely shielded from the variable latency of cryptographic hashing.

## 4. Compliance with the Radon Law ($CC=1$)

Finally, `LockFreeMpmcRing` strictly enforces the Radon Law (Cyclomatic Complexity = 1). The implementation uses mask-based state selection and branchless arithmetic instead of control flow branches. It uses `cas_success` to evaluate whether a CAS operation succeeded, securely routing pointers to either the safely owned `UnsafeCell` slot or to a locally discarded dummy buffer. This avoids the construction of dangling pointers and branch prediction penalties, establishing a pure bitwise logic gate suitable for the "hard substrate."
