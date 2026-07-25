# Maintaining the Zero-Heap-Allocation Boundary in BCINR

In the `bcinr` codebase, the **Zero-Allocation Boundary** strictly requires all hot-path execution to be `#![no_std]` with zero heap allocations (i.e., no dynamic memory management via `Vec`, `Box`, or the OS). 
This principle is paired with the **Radon Law ($CC=1$)**, mandating that all logic must be executed branchlessly using bitwise polynomials and masked state selection, avoiding control-flow constructs like `if`, `match`, or unbounded `loop`s.

The codebase achieves this by delegating runtime memory management entirely to fixed-capacity, deterministic data structures: `BumpArena` and `LockFreeSlab`.

## 1. `BumpArena`: Contiguous, Epoch-Bound Memory
`BumpArena` acts as a deterministic, $O(1)$ memory allocator for contiguous chunks. It avoids dynamic heap allocation by issuing bounded abstract offsets mapped against a pre-allocated chunk.

### Zero-Allocation and Branchless Mechanics
The core state, `BumpArenaState`, is composed exclusively of integer bounds:
```rust
pub struct BumpArenaState {
    pub offset: u32,
    pub capacity: u32,
}
```
When allocating memory (`try_alloc(size)`), it completely avoids panics or early returns on out-of-bounds requests. Instead, it utilizes strict bitwise polynomial logic to conditionally advance the offset:

```rust
pub fn try_alloc(&mut self, size: u32) -> (u32, u32) {
    let current_offset = self.offset;
    let next_offset = current_offset.wrapping_add(size);
    
    // Branchlessly check capacity
    let success = (next_offset <= self.capacity) as u32;
    let mask = 0u32.wrapping_sub(success); // 0xFFFFFFFF if true, 0x00000000 if false

    // State is mutated using masked selection
    self.offset = (next_offset & mask) | (current_offset & !mask);
    (current_offset & mask, mask)
}
```
By returning the offset and a success mask, `BumpArena` maintains zero heap fragmentation and branchlessness. Any out-of-bounds attempt safely defaults back to the previous offset and reports a 0-mask failure (which is translated up the stack as a bounded `StabilityRefusal::RuntimeEnvelopeViolated`).

## 2. `LockFreeSlab`: Fixed-Size, Independent Lifecycles
For independent element lifecycles, BCINR relies on `LockFreeSlab<const N: usize>`. It manages memory slots using an $O(1)$ concurrent atomic freelist backed by fixed-size arrays.

### Zero-Allocation and Branchless Mechanics
The slab bounds its maximum capacity strictly at compile-time using the `const N` generic parameter, maintaining data inline without heap allocations:
```rust
pub struct LockFreeSlab<const N: usize> {
    pub freelist: AtomicU32,
    pub next_indices: [u32; N],
}
```
Traditional lock-free slabs rely on Compare-and-Swap (CAS) `while` loops to retry upon contention, which fundamentally violates the "no data-dependent loop termination" rule. 

To maintain the boundary, `LockFreeSlab` restricts the CAS sequence to a strict single-pass execution (using bounded masks):
```rust
pub fn alloc_t1(&self) -> (u32, u32) {
    let head = self.freelist.load(Ordering::Relaxed);
    // ...
    // Mask logic computes `can_alloc`
    // Next state is built conditionally using masks
    let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
    
    let cas_res = self.freelist.compare_exchange_weak(
        head, next, Ordering::Relaxed, Ordering::Relaxed
    );
    // ...
}
```
If the single-pass CAS fails, it returns a 0-mask rather than unbounded retrying, fulfilling the bounded worst-case execution time (≤ 200 ns) and ensuring strict $CC=1$ cyclomatic complexity.

## Summary
By combining **compile-time fixed capacities** (`capacity: u32`, `const N: usize`) with **masked bitwise arithmetic**, `BumpArena` and `LockFreeSlab` provide safe, O(1) memory mechanisms. They allow the `bcinr` substrate to fully bypass the OS heap and remain completely branchless during bounds checks and concurrency resolutions, perfectly maintaining the zero-allocation boundary.
