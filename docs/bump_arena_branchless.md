# BumpArena Allocation and Branchless Bounds Checking

Based on my search of the `bcinr` codebase, `BumpArena` is implemented inside `crates/bcinr-logic/src/abstractions/bump_arena.rs`. Here is how it provides memory management without standard heap allocators and maintains a strict $CC=1$ cyclomatic complexity.

## 1. Zero Heap Allocation
The codebase adheres strictly to the project's `#![no_std]` and zero-allocation boundaries. Rather than calling system or standard library allocators like `malloc` or `std::alloc`, `BumpArenaState` relies entirely on a fixed bounded memory tracking state with just two `u32` integers:
- `offset`: The current memory pointer location.
- `capacity`: The total deterministic bounds of the arena.

Because `BumpArena` simply tracks usage within a pre-established memory boundary via arithmetic, it completely eliminates dynamic heap fragmentation and traditional allocation routines.

## 2. Branchless Out-of-Memory (OOM) Checking ($CC=1$)
Under the **Radon Law ($CC=1$)**, any conditional `if` statements or branch-dependent returns are absolutely forbidden. `BumpArena` performs bounds checking and conditionally updates state purely through bitwise arithmetic.

Here is the exact branchless `try_alloc` logic used in the implementation:
```rust
pub fn try_alloc(&mut self, size: u32) -> (u32, u32) {
    let current_offset = self.offset;
    let next_offset = current_offset.wrapping_add(size);
    
    // Evaluate if next_offset exceeds capacity (1 if true, 0 if false)
    let success = (next_offset <= self.capacity) as u32;
    
    // Generate a full-width mask: 
    // 0 - 1 = 0xFFFFFFFF (All 1s, Success)
    // 0 - 0 = 0x00000000 (All 0s, Failure)
    let mask = 0u32.wrapping_sub(success);

    // Constant-time state transition using bitwise selection
    self.offset = (next_offset & mask) | (current_offset & !mask);
    
    // Returns the offset and the success_mask
    (current_offset & mask, mask)
}
```

### How the OOM fallback operates:
1. **Mathematical Evaluation**: It evaluates `next_offset <= self.capacity` to produce a `0` or `1`. If the requested size goes out-of-bounds, `success` evaluates to `0`.
2. **Mask Generation**: It computes `mask = 0u32.wrapping_sub(0)`, producing `0x00000000`. (On success, `0 - 1` intentionally underflows to `0xFFFFFFFF`).
3. **Mask-based State Commit**: `self.offset` is updated using `(next_offset & mask) | (current_offset & !mask)`. If the mask is `0` (OOM), this expression evaluates to exactly `current_offset`, leaving the state bit-for-bit unchanged, which adheres to the rule of "No mutation before complete admission".
4. **Failure Propagation**: The caller receives `(0, 0x00000000)` instead of a panicked bounds check or an `Option::None`. The caller then uses this full-width failure mask in its own subsequent branchless arithmetic logic.
