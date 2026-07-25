# BumpArena and LockFreeSlab: Maintaining the Zero-Allocation Boundary in BCINR

In the `bcinr` architecture, the **Zero-Allocation Boundary** is a non-negotiable law governing the deterministic substrate (Rule 3 of the `AGENTS.md` constitution). The authoritative runtime operates strictly under `#![no_std]` and mandates zero heap allocation. Constructs like `Box`, `Vec`, `String`, and standard garbage collection are categorically banned from the hot path. 

This strict prohibition eliminates micro-architectural timing side-channels, non-deterministic delays (like searching for free blocks or OS-level mutex locking), and unpredictable panics (like Out-of-Memory). Crucially, relying on a traditional allocator would violate the Radon Law ($CC=1$), as standard allocators rely on internal conditional branches.

With dynamic heap allocation banned, the hot path achieves complex data manipulation and deterministic memory management through statically bounded structures, mask-based state selection, and two highly specialized memory abstractions: `BumpArena` and `LockFreeSlab`.

## Achieving Complex Data Manipulation Without the Heap

When dealing with state transitions, "cloning" memory in `bcinr` does not mean making a heap copy. Instead, the runtime uses **structural cloning**. 

Data manipulation happens via bounded, fixed-size stack values or statically allocated scratch structures. The hot path calculates the candidate state branchlessly, derives an admission mask (evaluating conditions mathematically to a full-width bit mask rather than an `if` statement), and executes a field-wise masked commit. The memory footprint remains predictably bound to the stack or pre-allocated capacities, yielding complex data evolution strictly within $O(1)$ constant time and $CC=1$ cyclomatic complexity.

## Deterministic Memory Management

When state must outlive a single function's stack frame or when managing collections, the substrate utilizes two $O(1)$ branchless memory abstractions:

### 1. `BumpArena` (Variable-Sized / Write-Once State)
The `BumpArena` provides deterministic $O(1)$ memory allocation by simply incrementing an offset pointer. It is the primary tool for accumulating sequential state, logs, metric aggregations, or variable-sized contiguous blocks. 

To maintain the $CC=1$ rule, `BumpArena` does not use `if (offset + size <= capacity)`. Instead, it handles capacity exhaustion mathematically:
```rust
let success = (next_offset <= self.capacity) as u32;
let mask = 0u32.wrapping_sub(success);
self.offset = (next_offset & mask) | (current_offset & !mask);
```
If the allocation exceeds capacity, the mask forces the offset to remain unchanged, strictly enforcing boundaries without a single conditional branch. Memory in a `BumpArena` shares a homogeneous lifecycle; it is never freed individually but rather reset holistically at the end of an epoch or transaction.

### 2. `LockFreeSlab` (Fixed-Size / High-Churn State)
The `LockFreeSlab` is designed for scenarios involving fixed-size elements bounded by a constant size `N`, where data has independent, heterogeneous lifecycles (frequent granular allocations and deallocations).

It provides an atomic freelist to recycle memory instantly. However, traditional lock-free structures use Compare-and-Swap (CAS) `while` loops, which violate the `bcinr` mandate against data-dependent loops. To remain lawful, `LockFreeSlab` utilizes single-pass, bounded atomic transitions. It computes the next available memory slot using mask-based pointer selection and single CAS attempts, safely yielding an available memory slot in strict $O(1)$ constant time without loop backedges. 

## Summary
By combining fixed-size stack values, mask-based structural commits, the branchless offset increments of `BumpArena`, and the single-pass atomic freelists of `LockFreeSlab`, `bcinr` entirely eliminates the need for `Vec` and `Box`. The result is a mathematically pure, branchless hot path where memory evolution is physically bounded, strictly predictable, and allocation-free.
