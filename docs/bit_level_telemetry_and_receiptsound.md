# Bit-Level Telemetry and the ReceiptSound Law in BCINR

In the BCINR deterministic substrate, the collection of system metrics through the MAPE-K (Monitor-Analyze-Plan-Execute) autonomic loop is strictly governed by **Rule 11 (The ReceiptSound Law)** and **Rule 3 (Absolute Runtime Laws)**. Understanding how these rules intersect reveals how BCINR achieves self-management without timing side-channels or variable-latency execution.

## 1. Interaction with Rule 11 (The ReceiptSound Law)

Rule 11 dictates that adaptive mutation requires an irrefutable combination of structural proofs, specifically mandating an `AcceptedOutcomeReceipt` alongside a `CertifiedLearningMode` and other cryptographic guarantees. 

While the system is constantly gathering bit-level telemetry during the **Observe** phase of the MAPE-K loop, **Rule 11 strictly separates the authorities of selection and learning**. The hot path cannot use raw telemetry to speculatively or directly mutate adaptive state (such as reinforcement learning weights). 

Instead, the raw telemetry must first be cryptographically formalized into an **`AcceptedOutcomeReceipt`** (the Yield Proof). Until this receipt is fully verified by the hot path, learning remains "frozen." The admission mask ($m_{\mathrm{admitted}}$) evaluates mathematically to `0`, causing the state transition to branchlessly preserve the existing state ($x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t$). During this frozen period, deterministic selection and telemetry accumulation continue uninterrupted, but adaptive weights cannot drift based on unwitnessed or uncertified events.

## 2. Branchless Telemetry Collection (Hot Path)

The substrate collects this telemetry in the hot path without violating the Radon Law ($CC=1$) or zero-allocation bounds:

* **Mask-Based Observation:** Telemetry is gathered precisely at the point of action (e.g., when `BumpArena` allocates memory). Instead of using `if/else` bounds checks, the system uses wrapped arithmetic and boolean conversions. For instance, `success = (next_offset <= capacity)` is mathematically converted into a full-width binary mask (`0` or `!0`).
* **Fixed-Width Accumulators:** This bit-level telemetry (like allocation failure masks or stale byte counts) is ingested unconditionally into pre-allocated, fixed-width `MetricAccumulator`s in strictly $O(1)$ constant time. No variable task objects or dynamic loops are generated.

## 3. Feeding the Slow Rail Without Interrupting the Hot Path

Because the hot path (`#![no_std]`, no allocation, branchless) cannot perform the variable-workload cryptographic hashing required to generate an `AcceptedOutcomeReceipt`, it must feed the telemetry to the **slow rail**. It achieves this entirely without locks or blocking:

* **`LockFreeMpmcRing` Conduit:** The system uses a wait-free, lock-free Multi-Producer Multi-Consumer ring queue (e.g., `LockFreeMpmcRing<EventWorkItem, 64>`) to act as a bounded-latency bridge between the hot path and the slow rail.
* **Bounded Push:** The hot path (`petri_tick`) packages the telemetry into an `EventWorkItem` and pushes it into the ring using Compare-And-Swap (CAS) instructions. This push operation is capped at a strict latency (e.g., ~10 ns) and is completely branchless. If the queue is full or highly contended, it mathematically discards the item rather than blocking, preserving the hot path's fixed execution bounds.
* **Slow Rail Accumulation (`ReceiptWorker`):** A background fiber on the slow rail—which is permitted to branch, allocate memory, and perform variable-length workloads—drains the ring queue. The `ReceiptWorker` accumulates the event traces, validates admissibility, and performs heavy cryptographic hashing (like BLAKE3). It cryptographically chains these traces (`prev_chain_hash ‖ run_id ‖ op_trace ‖ topology_tag`) into the formalized `AcceptedOutcomeReceipt`.

## 4. Closing the Autonomic Loop

Once the slow rail has generated the `AcceptedOutcomeReceipt` from the telemetry, it is injected back into the hot path as a fixed, deterministic input. In the **Accept** phase of the MAPE-K loop, the `PolicyGuard` verifies the receipt alongside the `CertifiedLearningMode` and `AcceptedCertificate`. If all conjuncts hold, the admission mask mathematically evaluates to `!0`, instantly unfreezing learning and authorizing the branchless application of the adaptive mutation.
