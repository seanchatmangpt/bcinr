# LockFreeSlab Mechanics in the Zero-Allocation Boundary

## The Zero-Allocation Boundary
In the BCINR substrate, deterministic execution requires a strict zero-allocation boundary (`#![no_std]`). The hot-path cannot perform any dynamic heap allocations (no `Vec`, `Box`, or OS-level memory management). Furthermore, the constitutional **Radon Law ($CC=1$)** mandates that all authoritative operations be completely branchless—free from data-dependent `if` statements, `match` blocks, or unbounded `while` loops. 

Within this environment, memory must be pre-allocated and managed deterministically. `LockFreeSlab` provides $O(1)$ concurrent memory management while strictly adhering to these laws.

## `LockFreeSlab` vs. `BumpArena`
BCINR utilizes two primary primitives for memory management, each serving distinct lifecycle requirements:

1. **`BumpArena` (Contiguous, Epoch-Bound)**
   - **Mechanism:** Advances a continuous offset pointer (often using an atomic `fetch_add` for concurrent safety).
   - **Allocation:** Supports variable-sized memory blocks.
   - **Reclamation:** Memory cannot be individually freed. Instead, the entire arena is reclaimed via a holistic, slow-rail epoch reset.

2. **`LockFreeSlab` (Fixed-Size, Independent Lifecycles)**
   - **Mechanism:** Maintains an atomic freelist mapping available slots within a fixed-size internal array (`next_indices`).
   - **Allocation:** Restricted to uniform, fixed-size elements bounded by a compile-time constant `N`.
   - **Reclamation:** Allows **atomic recycling of individual slots**. When an object's lifecycle ends, its slot can be pushed back onto the freelist for immediate concurrent reuse, avoiding the capacity exhaustion that a bump allocator would suffer under high-churn workloads.

## Achieving Concurrent Reuse Without Branches or Loops
Traditional lock-free allocators rely heavily on Compare-And-Swap (CAS) `while` loops and branching logic to handle contention and empty states. `LockFreeSlab` bypasses these conventional constructs to satisfy the $CC=1$ rule through three core innovations:

### 1. No Heap Allocation
Memory is managed structurally using fixed-capacity stack or static arrays. The slab is composed of an atomic `freelist` head and a pre-sized array:
```rust
pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```
There is zero OS interaction or dynamic memory resizing at runtime.

### 2. No CAS `while` Loops
Unbounded CAS loops violate the prohibition on runtime loop backedges, as contention could lead to variable execution time. To resolve this, `alloc_t1` limits the operation to a strict **single-pass execution**:
```rust
(0..1).for_each(|_| {
    // Single-pass CAS execution
});
```
Instead of looping until successful, the allocator attempts the CAS exactly once. If it fails due to contention or an empty freelist, the operation safely collapses and yields a deterministic failure flag, guaranteeing the aggregate timing budget remains bounded (≤ 200 ns).

### 3. No `if` Statements (Mask-Based Selection)
Logical conditions (such as checking if the freelist is empty or if the CAS succeeded) are transformed into bitwise masks rather than branching constructs. 
- Boolean checks are cast to integers (`1` or `0`).
- Full-width masks (`0xFFFFFFFF` or `0x00000000`) are generated using arithmetic operations like `wrapping_sub`.
- State mutation is resolved using bitwise arithmetic:
```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
let can_alloc_mask = 0u32.wrapping_sub(can_alloc);

// Branchless selection of the next head
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
```
If the allocation cannot proceed, the mask zeros out the mutated state, safely passing the original state through to the CAS operation. The final success indicator and allocated index are also resolved bitwise directly from the CAS result, guaranteeing a single, unbroken execution path.
