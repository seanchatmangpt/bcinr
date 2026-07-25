# Compile-Time Array Bounds in BCINR

In the `bcinr` substrate, deterministic computational logic is physically enforced through the **Radon Law ($CC=1$)** and a strict **Zero-Allocation Boundary (`#![no_std]`)**. Consequently, dynamic structures like `Vec<T>` or `Box<T>` are entirely prohibited in the hot path. 

## The Prohibition of `Vec<T>`
Relying on `Vec<T>` violates multiple core architectural laws:
1. **Heap Dependency**: Dynamically resizable structures depend on a global memory allocator, introducing non-deterministic execution costs, fragmentation, and timing side-channels.
2. **Hidden Loop Backedges**: Runtime resizing of dynamic arrays introduces implicit unbounded loops and conditional logic, violating the absolute branchlessness requirement ($CC=1$).
3. **Execution Bounding**: Object code audits actively search for allocator symbols or bounds-checking panic paths. A `Vec` inherently triggers these rules, immediately dropping the Substrate Integrity Score (SIS) to 0 and failing the `@turing_machine` gate.

## Enforcement via Compile-Time Bounds
Instead of runtime heaps, data pools must be statically dimensioned at compile-time using fixed-width arrays, often parameterized via `const N: usize` (e.g., `[Slot<T>; N]` or `[u32; N]`). By doing this, memory boundaries become an explicit contractual guarantee. Out-of-memory states are resolved deterministically through bitwise mask refusals rather than runtime panics or cyclic dynamic allocation logic.

---

## Authoritative Memory Primitives

Since global allocation is banned, BCINR implements loop-free $O(1)$ memory abstractions mapped directly over these fixed-width, compile-time arrays.

### `LockFreeSlab<const N: usize>`
Used for heterogeneous objects with independent lifecycles.
- **Fixed-Width Freelist**: Relies on a compile-time bounded array `[u32; N]` to manage available slots rather than dynamically pushing to a heap-backed queue.
- **Loop-Free CAS**: Bounding the size at compile-time allows `LockFreeSlab` to allocate memory via a strictly bounded single-pass execution. Unbounded Compare-And-Swap (CAS) `while` loops are stripped out in favor of purely arithmetic mask-selected state transitions over the `N`-sized capacity.

### `BumpArena` (and ACSBA)
Used for homogenous, epoch-bound memory allocations where space is reclaimed holistically.
- **Static Buffer Capacity**: Operates by advancing an atomic offset pointer over a pre-computed, fixed-capacity buffer. It can never dynamically scale beyond its strict compile-time bounds.
- **Mask-Based Refusal**: Instead of conditional branching to check bounds (`if offset > capacity`), it calculates branchless bitwise masks. If an allocation request exceeds the static limits, the mask forces the transaction to evaluate to zero, seamlessly refusing the allocation in mathematically constant time.

By substituting dynamic structures with `const N` compile-time arrays, `BumpArena` and `LockFreeSlab` maintain execution that is mathematically fixed, allocation-free, and irrefutably deterministic.
