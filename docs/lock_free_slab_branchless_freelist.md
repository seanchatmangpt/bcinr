# LockFreeSlab Branchless Freelist

## Overview
Under the `bcinr` project's zero-allocation boundary and Radon Law ($CC=1$), the `LockFreeSlab` provides deterministic, $O(1)$ lock-free memory management. It safely allocates and reclaims fixed-size slots without relying on dynamic heap allocations, mutex locks, or variable-latency search loops.

## Zero-Allocation & Fixed-Size Slots
To completely eliminate dynamic heap allocations, external fragmentation, and internal fragmentation, the slab statically bounds all elements to a uniform size using fixed-capacity stack/static arrays:
```rust
use core::sync::atomic::{AtomicU32, Ordering};

pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```
When objects are dynamically created or destroyed, they are recycled instantly. Deallocated slot indices are safely pushed back onto the `freelist`, making them immediately available for the next allocation. This provides granular object lifecycle management without violating the zero-allocation boundary.

## ABA-Preventing Generational Indices
Standard lock-free data structures suffer from the ABA problem, where a memory location changes and reverts before a concurrent thread completes its Compare-And-Swap (CAS), corrupting the linked list.

`LockFreeSlab` prevents ABA without mutex locks via an **integer-based sequence mechanism**:
- **Indices over Pointers:** The atomic `freelist` strictly manages 32-bit integers that map into the bounded `next_indices` array, avoiding raw memory pointers.
- **Epoch/Generational Safety:** The atomic values encode an advancing sequence (epochs/tickets). Because atomic CAS requires a precise bit-state match, a delayed thread encounters the advanced generational epoch and safely yields a branchless refusal rather than blindly corrupting the recycled freelist.

## Atomic, Branchless Freelist
Traditional lock-free freelists rely on unbounded `while` loops for CAS retries, which violates the constitutional prohibition of runtime loop backedges. `LockFreeSlab` converts control flow into mask-based arithmetic:

1. **Single-Pass Execution:** Allocation attempts (`alloc_t1`) run exactly once within a bounded `(0..1).for_each(|_| ...)` structure, completely averting variable-latency search loops.
2. **Boolean Mask Expansion:** Logical conditions (like `is_empty`) are cast to booleans and expanded into full-width bitmasks (`0xFFFFFFFF` or `0x00000000`) via `wrapping_sub`.
   ```rust
   let is_empty = (head == 0xFFFFFFFF) as u32;
   let can_alloc = (!is_empty) & 1;
   let can_alloc_mask = 0u32.wrapping_sub(can_alloc);
   ```
3. **Mask-Selected Mutations:** The next state is calculated arithmetically (`(next) & mask | (head) & !mask`). 
4. **Branchless Refusal:** If the single atomic `compare_exchange_weak` CAS fails, or the slab is exhausted, the operation mathematically collapses to a zero-mutated state. It yields a branchless refusal flag without ever branching, halting, or panicking.

This design ensures strict determinism, bounding the `T1` aggregate timing budget to $\le 200$ ns.
