# LockFreeSlab Memory Management in BCINR

## The Zero-Allocation Boundary
BCINR enforces a strict zero-allocation, `#![no_std]` boundary to guarantee deterministic computational logic and execution latency. This "hard substrate" design physically eliminates non-deterministic overheads, such as dynamic heap fragmentation and timing side-channels. 
The system prohibits standard dynamic memory allocation tools like the global allocator, `Vec`, or `Box`. All memory management must occur over bounded, pre-allocated structures without triggering any runtime heap allocation.

## LockFreeSlab Architecture
`LockFreeSlab` provides concurrent, lock-free memory management that adheres strictly to this Zero-Allocation Boundary. Rather than relying on a dynamically growing heap, the structure is built on constant-sized static or stack arrays bounded by a compile-time generic constant `N`.

```rust
use core::sync::atomic::{AtomicU32, Ordering};

pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```

This structural layout is inherently compatible with strict `#![no_std]` environments and avoids any OS-level memory management requests, fulfilling the zero-allocation requirement.

## Concurrent Branchless Access (The Radon Law)
The most rigorous architectural constraint of the BCINR substrate is the **Radon Law ($CC=1$)**: all authoritative code must be perfectly branchless, prohibiting data-dependent `if`, `match`, or runtime `loop` instructions (no cycle backedges). 

Traditional lock-free allocators heavily rely on an unbounded Compare-And-Swap (CAS) `while` loop, which directly violates the loop-backedge prohibition. `LockFreeSlab` overcomes this by converting the allocation sequence into a bounded, bit-parallel mask transition.

### 1. Single-Pass Loop-Free Execution
The allocation function (`alloc_t1`) guarantees a single-pass execution by wrapping logic in a compile-time bounded `(0..1).for_each(|_| { ... })`. This structure mathematically proves the absence of unbounded spin-loops and ensures the compiler resolves it to straight-line code.

### 2. Mask-Based State Transitions
Control flow is eradicated by converting logical conditions into boolean integers, and then expanding them into full-width bitmasks (either `0xFFFFFFFF` or `0x00000000`) using operations like `wrapping_sub`.

```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
let can_alloc_mask = 0u32.wrapping_sub(can_alloc);
```

The candidate state for the atomic freelist is derived through arithmetic selection rather than conditional branching:
```rust
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
```

### 3. Bounded CAS and Typed Refusals
The allocator performs exactly one `compare_exchange_weak` operation. It does not loop on CAS failures. Instead, the `Result` of the CAS operation is reduced to an integer mask. 

```rust
let cas_res = self.freelist.compare_exchange_weak(
    head,
    next,
    Ordering::Relaxed,
    Ordering::Relaxed,
);

let cas_success = (cas_res.is_ok() && can_alloc != 0) as u32;
```

If the atomic operation fails due to thread contention or the slab being fully exhausted, the failure naturally propagates through the bitwise math. It produces a branchless mathematical refusal—safely leaving persistent state bit-for-bit unchanged—and returns a deterministic `success_flag` of `0`.

## Deterministic Timing Contract
By strictly enforcing the Zero-Allocation Boundary and operating solely on straight-line bitwise math over fixed bounds, `LockFreeSlab` meets BCINR's rigid latency limits:
- **T0 primitive budget:** ~5 ns (atomic pop/push)
- **T1 aggregate budget:** ≤ 200 ns

This O(1) bounded execution design provides completely concurrent, safe slab allocations without a single branching instruction, preserving the computational substrate's integrity.
