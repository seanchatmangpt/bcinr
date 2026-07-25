```markdown
# `BumpArena` Memory Reclamation in BCINR

*Note: While the prompt requested searching in `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/`, the `BumpArena` and its `reset` method were located in `crates/bcinr-logic/src/mem.rs`.*

## Location and Implementation

The `BumpArena` is defined in `crates/bcinr-logic/src/mem.rs` and its `reset` method is implemented as follows:

```rust
pub fn reset(&mut self) {
    self.offset = 0;
}
```

## How Memory is Safely Reclaimed in $O(1)$ Time

Reclaiming memory in `BumpArena` is achieved by simply resetting the `offset` pointer back to `0`. This operation is strictly $O(1)$ because it only involves a single scalar assignment, regardless of how much memory was allocated or how many logical objects the arena holds. 

This approach enforces the **zero-allocation rule** and constant-time execution in the following ways:

1. **No Dynamic Deallocator Loops**: Unlike traditional allocators that must traverse free lists, coalesce adjacent blocks, or unmap pages, `BumpArena` does none of this. The memory backing the arena is statically sized and retained indefinitely. Resetting the offset simply makes the entire memory span available for subsequent bump allocations.
2. **Bypassing `Drop` Implementations**: The arena manages raw bytes (`u8`), meaning the Rust compiler does not emit or require `Drop` loops when the arena is cleared. When the `offset` is moved back to `0`, the previously allocated structs are logically invalidated and overwritten by future allocations without their destructors ever running. This guarantees deterministic, fixed-time memory reclamation.
3. **Zero-Allocation Boundary**: The underlying memory backing the arena (e.g., a pre-allocated `Vec<u8>`) is allocated exactly once during initialization (outside the hot path). During the hot-path execution, allocations only advance the `offset`, and reclaims only zero it. No system-level `malloc` or `free` calls occur, satisfying the stringent `#![no_std]` zero-allocation requirement for authoritative hot paths. 

By avoiding complex bookkeeping and destructors, `BumpArena` completely eliminates the variable execution costs (tail latencies) associated with traditional memory management.
```
