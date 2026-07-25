I performed a search for `slab.rs` and `LockFreeSlab` across the codebase.

While I didn't find `LockFreeSlab` or `slab.rs` directly within `crates/bcinr-cmca/src/` or `crates/bcinr-powl/src/` (I only found references to `LabelSlab` in `crates/bcinr-powl/src/tape.rs`), I located the implementation of `LockFreeSlab` in `crates/bcinr-logic/src/abstractions/lock_free_slab.rs`.

Here is the documentation on how `LockFreeSlab` implements lock-free operations branchlessly without violating the zero heap allocation boundary.

# LockFreeSlab: Branchless and Zero-Allocation Mechanics

The `LockFreeSlab` implementation strictly adheres to the project's **Zero-Allocation Boundary** and **Radon Law (CC=1)**.

### Zero Heap Allocation Boundary
The `LockFreeSlab<const N: usize>` struct completely avoids dynamic heap allocation by allocating all of its state up-front inside a statically sized structure:
```rust
pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```
By utilizing const generics (`const N: usize`), the maximum capacity `N` is resolved strictly at compile time. The elements and the linked-list traversal structure (`next_indices`) are pre-allocated inline as a fixed-size array (`[u32; N]`). Because it uses `#![no_std]` paradigms and `core::sync::atomic::AtomicU32`, it natively bypasses the global allocator, resulting in 0 heap allocations.

### Branchless Lock-Free Operations (Radon Law / CC=1)
Lock-free pop operations usually require branches (e.g., `if is_empty { return Err }` or `if compare_and_swap_fails { continue }`). `LockFreeSlab::alloc_t1()` achieves lock-free allocations branchlessly by replacing control flow with bitwise masks and arithmetic logic.

#### 1. Branchless State Checks
Instead of branching when the freelist is empty (`0xFFFFFFFF`), the algorithm computes boolean values as integers, converting them to bitwise masks:
```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
// Computes 0xFFFFFFFF if true, 0x00000000 if false
let can_alloc_mask = 0u32.wrapping_sub(can_alloc); 
```

#### 2. Mask-Based Pointer Selection
Instead of branching to decide the next head pointer (`if can_alloc { head + 1 } else { head }`), the new state is resolved unconditionally using the bitwise mask:
```rust
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
```
This forces the CPU to evaluate both states simultaneously and statically select the correct path without any data-dependent jumps.

#### 3. Branchless Atomic Execution and Return
The new pointer state is committed using `compare_exchange_weak`, a lock-free atomic primitive:
```rust
let cas_res = self.freelist.compare_exchange_weak(
    head,
    next,
    Ordering::Relaxed,
    Ordering::Relaxed,
);
```
Success state and results are derived branchlessly via masking instead of conditional returns. If the CAS fails or allocation wasn't possible, `cas_success` becomes `0`, and the return index masks out to `0`.

```rust
let cas_success = (cas_res.is_ok() && can_alloc != 0) as u32;
success = cas_success;
// Computes `head` if successful, or `0` if it failed.
result = head & (0u32.wrapping_sub(cas_success)); 
```

By mapping every logical decision into polynomials over bits, `LockFreeSlab` ensures deterministic bounded execution while operating natively on raw memory blocks.
