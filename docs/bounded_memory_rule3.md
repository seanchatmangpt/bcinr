# Rule 3: Fixed Bounded Memory Access in BCINR

The `bcinr` deterministic substrate strictly enforces a zero-allocation, branchless execution model. One of its core absolute runtime laws is **fixed bounded memory access**, which fundamentally prohibits dynamic memory scaling and heap fragmentation on the hot path. 

## The Zero-Allocation Boundary
All authoritative hot-path logic inside BCINR is governed by the `#![no_std]` attribute and an absolute **0 heap allocation** mandate.
- **Dynamic Types Prohibited**: Dynamically resizable types like `Vec<T>`, `String`, or `Box<T>` are entirely banned. 
- **Execution Consistency**: Eliminating the global allocator eradicates non-deterministic overheads, heap fragmentation, and timing side-channels, ensuring identical execution cost for every invocation.
- **Enforcement**: The `@turing_machine` gate actively analyzes disassembly to verify the absolute absence of allocator symbols in authoritative object code. Any verified heap allocation in the hot path sets the Substrate Integrity Score (SIS) to 0.

To adhere to this, memory capacities are strictly defined at compile-time using fixed-width arrays (e.g., `[Slot<T>; N]`). This ensures that memory footprints are rigorously bounded and statically analyzed, preventing runtime out-of-memory (OOM) states and avoiding hidden iteration required for runtime resizing.

## Authoritative Memory Primitives

Since the global allocator is absent, BCINR provides specialized branchless memory primitives mapped over fixed capacities. Both primitives respect the **Radon Law ($CC=1$)**, meaning they contain no `if`, `match`, or data-dependent loops.

### 1. `BumpArenaState` (Bump Arena Allocator)
Used for homogeneous, epoch-bound memory objects requiring deterministic $O(1)$ allocation without fragmentation.
- **Pre-computed Capacity**: Tracks available space via static `offset` and `capacity` limits.
- **Branchless Mask-Based Refusal**: Allocations mathematically advance the offset up to the boundary. If the new offset exceeds the buffer, branchless bitwise masks force the transaction to fail and yield zero, leaving the state precisely untouched without triggering a branch or panic.
- **Implementation**:
  ```rust
  pub fn try_alloc(&mut self, size: u32) -> (u32, u32) {
      let current_offset = self.offset;
      let next_offset = current_offset.wrapping_add(size);
      let success = (next_offset <= self.capacity) as u32;
      let mask = 0u32.wrapping_sub(success);

      self.offset = (next_offset & mask) | (current_offset & !mask);
      (current_offset & mask, mask) // Returns (offset, success_mask)
  }
  ```

### 2. `LockFreeSlab<const N: usize>` (Lock-Free Slab Allocator)
Used for heterogeneous, independently lived state.
- **Fixed-Size Capacity**: Pre-allocates memory using a fixed-width array `[u32; N]` for its internal freelist indices.
- **O(1) Branchless Allocation**: Acquires available slots via a strict loop-free Compare-And-Swap (CAS) over an atomic freelist head (`AtomicU32`). 
- **Implementation Mechanism**: The `alloc_t1` function leverages bitwise operations and a mask to determine if an allocation can proceed (e.g., ensuring the head is not `0xFFFFFFFF`). A single CAS operation attempts the update, and bitwise logic sets the `success` flag and `result` without any conditional branches, ensuring bounded execution time ($\le 200$ ns).
