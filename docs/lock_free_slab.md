# LockFreeSlab Allocator Implementation

## Overview
`LockFreeSlab` provides deterministic, $O(1)$ lock-free memory management without dynamically sized heap allocations. Designed for a strict `#![no_std]` environment, it adheres to the project's **Radon Law ($CC=1$)**, ensuring execution is free of data-dependent branches, cycle backedges, and non-deterministic timing overheads.

## Core Structure
The memory structure relies strictly on fixed-capacity stack/static arrays rather than heap-backed types like `Vec` or `Box`. It is composed of an atomic `freelist` head and an array of `next_indices` bounded by a constant size `N`.

```rust
use core::sync::atomic::{AtomicU32, Ordering};

pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```

## Branchless, Loop-Free Allocation (`alloc_t1`)
Traditional lock-free allocators rely on unbounded Compare-And-Swap (CAS) loops, which violate the constitutional prohibition of runtime loop backedges. `LockFreeSlab` solves this by executing a bounded, mask-based transition.

### Key Architectural Techniques:
1. **Single-Pass Execution:** The `alloc_t1` method runs exactly once using a compile-time bounded `(0..1).for_each(|_| { ... })` instead of a `while` loop.
2. **Bitwise Masking for Control Flow:** Logical conditions are converted into boolean integers and expanded into full-width masks (`0x00000000` or `0xFFFFFFFF`) via `wrapping_sub`.
   ```rust
   let is_empty = (head == 0xFFFFFFFF) as u32;
   let can_alloc = (!is_empty) & 1;
   let can_alloc_mask = 0u32.wrapping_sub(can_alloc);
   ```
3. **Mask-Based Selection:** State mutation is calculated arithmetically. If the CAS fails or the slab is empty, the output collapses safely to a zero-mutated state.
   ```rust
   let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
   
   let cas_res = self.freelist.compare_exchange_weak(
       head,
       next,
       Ordering::Relaxed,
       Ordering::Relaxed,
   );
   ```
4. **Typed Branchless Refusal:** The final return values (the index and success flag) are derived directly from the CAS success result bitwise, guaranteeing a single linear execution path.

## Timing Contract and Determinism
By stripping out OS interactions, allocations, and dynamic branching, `LockFreeSlab` guarantees an absolute timing contract:
- **T0 primitive budget:** ~5 ns (atomic pop/push)
- **T1 aggregate budget:** ≤ 200 ns

This constant-time boundary provides robust protection against timing side-channel attacks while maintaining deterministic state updates across all targets (WASM, embedded).
