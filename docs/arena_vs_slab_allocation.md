# Architectural Guidelines: `BumpArena` vs `LockFreeSlab`

BCINR enforces a strict zero-allocation, branchless execution model across its core substrate to guarantee deterministic computational logic. All hot-path memory management must adhere to the **Radon Law ($CC=1$)**, utilizing `#![no_std]` environments with zero dynamic heap allocations. 

To achieve this, BCINR provides two primary memory primitives: `BumpArena` and `LockFreeSlab`. Choosing between them depends strictly on the lifecycle conditions and allocation patterns of the underlying data.

## 1. `BumpArena`: Contiguous, Epoch-Bound Memory

`BumpArena` provides deterministic $O(1)$ memory allocation by incrementing an internal offset pointer over a fixed-capacity buffer. In concurrent contexts, it utilizes an Atomic Concurrent-Safe Bump Arena (ACSBA) with a loop-free atomic `fetch_add` strategy to advance the offset.

### Lifecycle Conditions that Mandate `BumpArena`:
* **Homogeneous Lifecycles (Epoch-Based):** Mandated when objects share the exact same lifecycle. Memory is never freed individually; instead, the entire arena is reset holistically by a slow-rail mechanism at the end of a transaction, frame, or epoch.
* **Variable-Sized Allocations:** Mandated when allocations require dynamic sizes or contiguous byte blocks. The bump allocator simply advances the offset by the requested `size` without enforcing fixed slot boundaries.
* **Write-Once Processing:** Ideal for accumulating sequential state, logs, metric aggregations, or trees where elements are only appended and never removed individually. Space is not reclaimed on branchless refusal.

## 2. `LockFreeSlab`: Fixed-Size, Independent Lifecycles

`LockFreeSlab` manages fixed-size memory slots using an $O(1)$ concurrent atomic freelist. It executes bounded, mask-based Compare-And-Swap (CAS) state transitions without runtime loop backedges, strictly preserving the $CC=1$ cyclomatic complexity rule while providing concurrency.

### Lifecycle Conditions that Mandate `LockFreeSlab`:
* **Heterogeneous/Independent Lifecycles:** Mandated when objects are created and destroyed dynamically at unpredictable times. The atomic freelist allows individual slots to be reclaimed and reused instantly.
* **Fixed-Size Elements:** Mandated when the data structure consists of uniform, identically sized objects bounded by a constant size `N`. The slab relies on an internal array of `next_indices` to map available slots.
* **High-Churn Recyclability:** Necessary for state management where frequent granular allocations and deallocations occur. Reusing memory immediately avoids exhausting capacity, which would otherwise happen in a bump allocator prior to an epoch reset.

## Summary Matrix

| Feature | `BumpArena` (ACSBA) | `LockFreeSlab` |
|---------|---------------------|----------------|
| **Allocation Size** | Variable / Contiguous | Fixed-size slots (Constant `N`) |
| **Deallocation** | Holistic reset (epoch-based) | Individual slot recycling (freelist) |
| **Concurrency Mechanism** | Loop-free `fetch_add` mask logic | Mask-based, single-pass CAS |
| **Primary Use Case** | Epoch processing, sequential metrics | High-churn independent state pools |
