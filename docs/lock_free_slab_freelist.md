# LockFreeSlab: Integer-Based Freelist Mechanism

## Overview
The `LockFreeSlab` is a zero-allocation memory primitive designed for the `bcinr` deterministic substrate. In strict compliance with the **Radon Law ($CC=1$)** and a `#![no_std]` environment, it completely eliminates dynamic heap allocations (such as `Vec` or `Box`). By pre-allocating a fixed capacity `N` at compile time, it ensures deterministic execution latency ($O(1)$ operations bounded to $\le 200$ ns) without OS-level allocator dependencies.

## Eliminating Fragmentation
Unlike general-purpose allocators that suffer from external and internal fragmentation due to dynamic sizing, `LockFreeSlab` bounds all elements to a uniform size using static fixed-width arrays:
- **Zero External Fragmentation:** Memory is mapped over a contiguous static array space.
- **Zero Internal Fragmentation:** Every slot in the slab represents an identically sized element, mathematically preventing padding or wasted block tails.

## Managing Heterogeneous Object Lifecycles (Recycling)
While `BumpArena` manages homogeneous, epoch-bound lifecycles requiring a holistic reset, `LockFreeSlab` safely manages **heterogeneous, disparate object states**.
When an object is dynamically created or destroyed at an unpredictable time, the slab recycles it instantly:
- The structure maintains an array `next_indices: [u32; N]` which tracks the integer linked-list of free slots.
- When an object is deallocated, its index is safely pushed back onto the `freelist`, making it immediately available for the next allocation. This high-churn recyclability provides granular, independent state management while maintaining fixed bounds.

## The Integer-Based Freelist & Epoch ABA Prevention
Standard lock-free data structures suffer from the ABA problem—where a memory location is modified and changed back to its original state before a concurrent thread completes its Compare-And-Swap (CAS), corrupting the linked list. 
The `LockFreeSlab` neutralizes this via an **integer-based sequence mechanism**:
- **Indices over Pointers:** Rather than manipulating raw memory pointers, the atomic `freelist` head strictly manages 32-bit integers that map securely into the bounded `next_indices` array.
- **Epoch Safety:** The ABA problem is bypassed by incorporating monotonic progression (epochs/tickets) within the atomic values. Instead of the `freelist` reverting to an identical bit-state when a slot is recycled, atomic state transitions logically encode an advancing sequence. Because the atomic CAS requires a precise bit-state match, a delayed thread encounters the advanced epoch and safely yields a branchless refusal, rather than blindly corrupting the recycled freelist.

## Branchless `alloc_t1` State Transitions
Traditional lock-free freelists rely on unbounded `while` loops for CAS retries. `LockFreeSlab` enforces the $CC=1$ cyclic constraint by translating control flow into **mask-based arithmetic**:
1. **Single-Pass CAS:** Allocation attempts execute exactly once bounded by a `(0..1).for_each(|_| ...)` structure, completely averting runtime loop backedges.
2. **Boolean Mask Expansion:** Logical conditions (e.g., `is_empty` or `cas_success`) are cast to booleans and expanded via `wrapping_sub` into full-width bitmasks (`0xFFFFFFFF` or `0x00000000`).
3. **Mask-Selected Mutations:** The next state is resolved arithmetically (`(next) & mask | (head) & !mask`). If the single atomic `compare_exchange_weak` CAS fails, or the slab is exhausted, the operation mathematically collapses to a zero-mutated state. The function safely returns a branchless refusal flag without ever branching, halting, or panicking.
