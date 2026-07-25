# Atomic Reclamation Mechanics in `LockFreeSlab`

Within the deterministic boundaries of the `bcinr` project, the `LockFreeSlab` provides safe, concurrent lock-free memory management that strictly adheres to the `#![no_std]` zero-heap allocation requirement and the Radon Law ($CC=1$).

The user request references specific atomic operations like `AtomicU64`, `fetch_or`, and `fetch_and`. While those bitwise atomic primitives are used for reclaiming lanes in the `Dispatcher` (via an occupancy bitmask), `LockFreeSlab` relies on a different, scalable integer-based atomic strategy to reclaim individual object slots.

## 1. The Zero-Allocation `#![no_std]` Boundary
`LockFreeSlab` handles memory objects with heterogeneous, independent lifecycles. To bypass the need for dynamic heap allocations (`Box`, `Vec`, or a `Mutex` protected `std::alloc`), it bounds the capacity to a compile-time fixed size `N`:

```rust
use core::sync::atomic::{AtomicU32, Ordering};

pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```
Reclamation (freeing memory slots) directly returns an object index into the stack-allocated or statically-allocated `next_indices` array, preserving the zero-allocation boundary.

## 2. Integer-Based Freelist vs Bitmask `fetch_and`
For small, fixed collections (≤ 64 slots), concurrent reclamation can be achieved efficiently by clearing an occupancy mask (e.g., `occupancy.fetch_and(!(1u64 << s_idx), Ordering::Release)`). 

However, `LockFreeSlab` is designed to support arbitrary sizes `N`. Therefore, instead of using `AtomicU64` and `fetch_and`, it employs an integer-based generational index via `AtomicU32`:
- **Generational Protection:** The atomic value acts as a "ticket" or epoch sequence to avoid the ABA problem common in lock-free linked lists.
- **Compare-And-Swap:** The atomic state relies on `compare_exchange_weak` to safely update the pointer, ensuring that if multiple threads try to interact simultaneously, they do not corrupt the reclaimed memory pool.

## 3. Branchless Mask-Selected State Transitions ($CC=1$)
Standard lock-free reclamation typically relies on unbounded CAS `while` loops to retry on failure. To satisfy the mandate against data-dependent loops (the Radon Law), `LockFreeSlab` eliminates loops entirely:

1. **Single-Pass Bound:** The `LockFreeSlab` encapsulates the atomic exchange inside a fixed `(0..1).for_each(|_| ...)` block, guaranteeing a strictly bounded execution time (≤ 200 ns).
2. **Boolean Mask Expansion:** Rather than branching (`if` statements), logical conditions (e.g., whether a slot can be safely reclaimed or allocated) are converted into full-width integer bitmasks (like `0xFFFFFFFF` or `0x00000000`) using `wrapping_sub`.
3. **Branchless Arithmetic Updates:** The new state is resolved purely via bitwise arithmetic (`(next) & mask | (head) & !mask`). If the CAS transition fails due to contention, the mutation logically zeroes out, yielding a deterministic, branchless refusal.

## Summary
Safe concurrent reclamation in `LockFreeSlab` replaces allocation-heavy `Mutex` locks and variable-latency retry loops with an $O(1)$ mask-based atomic sequence. By relying on bounded `AtomicU32` CAS updates rather than `fetch_or`/`fetch_and` bitmasks, it effectively reclaims individual slots instantly while natively satisfying both `#![no_std]` and pure branchless determinism.
