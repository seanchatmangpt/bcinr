# Zero-Allocation Memory Management in BCINR

BCINR enforces a strict zero-allocation, branchless execution model across its core substrate to guarantee deterministic computational logic. This is achieved by adhering to the **Radon Law ($CC=1$)**, which prohibits data-dependent branches and dynamically sized heap allocations in the hot path. 

Memory management relies on `#![no_std]` execution, `BumpArena`, and `LockFreeSlab` primitives.

## The `#![no_std]` and Zero-Allocation Boundary

All authoritative hot paths and computational logic layers (like `bcinr-logic` and `bcinr-cmca`) strictly compile with `#![no_std]`. This ensures:
- **0 Heap Allocations:** Functions are barred from using the global memory allocator, completely eliminating dynamic heap allocations (`Box`, `Vec`, etc.).
- **Deterministic Latency:** Without a heap allocator or dynamic memory fragmentation, execution avoids timing side-channels and non-deterministic overheads.
- **Portability:** The core substrate runs seamlessly and deterministically across embedded environments and WebAssembly (WASM) without conditional compilation.

## BumpArena: Deterministic O(1) Allocation

`BumpArenaState` provides a branchless bump allocator for deterministic $O(1)$ memory allocation without heap fragmentation. It operates over a fixed-capacity buffer by incrementing an internal offset pointer.

### Branchless Allocation
To conform with the $CC=1$ cyclomatic complexity rule, `try_alloc` computes allocations using bitwise masking rather than conditional `if/else` branching:
```rust
let current_offset = self.offset;
let next_offset = current_offset.wrapping_add(size);
let success = (next_offset <= self.capacity) as u32;
let mask = 0u32.wrapping_sub(success); // yields 0xFFFFFFFF if successful, 0x00000000 if not

self.offset = (next_offset & mask) | (current_offset & !mask);
```
If the capacity is exceeded, the mask isolates the current offset, preventing mutation while returning a predictable, branchless rejection.

### Atomic Concurrent-Safe Bump Arena (ACSBA)
The repository innovates on concurrent multi-threaded bump allocation by avoiding traditional Compare-And-Swap (CAS) loops, as loops would introduce cycle backedges and violate the Radon Law. The **ACSBA** design uses a single loop-free `fetch_add` operation:
1. Atomically advances the offset.
2. Computes candidate bounds and overflow limits.
3. Combines validation conditions into a single branchless success mask.
4. Masks the output, either yielding an exclusive contiguous block or a branchless `0` refusal.

## LockFreeSlab: O(1) Atomic Freelist

`LockFreeSlab` manages fixed-size memory slots using an O(1) concurrent atomic freelist. Like `BumpArena`, it rigorously circumvents data-dependent branches and loops.

### Execution Mechanism
In the `alloc_t1()` method, state transitions execute using strict masking:
- Reads the current `head` from the `AtomicU32` freelist.
- Derives `is_empty` and `can_alloc` via branchless bitwise logic.
- Executes a single, non-looping `compare_exchange_weak`.
- Re-masks the final `success` and `result` states based on the CAS flag. 

This bit-parallel masking guarantees lock-free state transitions complete in constant time (budget ≤ 200 ns) under absolute sequential instruction sets, preserving the $CC=1$ constitutional mandate.
