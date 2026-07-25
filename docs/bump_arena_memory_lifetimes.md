# `BumpArena` Memory Lifetimes in BCINR

## Overview
In the BCINR deterministic substrate, memory management strictly adheres to the **Radon Law ($CC=1$)** and a **Zero-Allocation Boundary (`#![no_std]`)**. This means the authoritative hot path performs zero dynamic heap allocations (no `Box`, `Vec`, or global allocator) and operates without a Garbage Collector (GC). Instead, it uses `BumpArena` to manage memory lifetimes within statically bounded buffers sequentially.

## Sequential Allocation within a Statically Bounded Buffer
`BumpArena` provides a deterministic $O(1)$ memory allocator using a fixed-capacity buffer. The allocator bumps a cursor forward for every allocation, making memory assignment sequential.

### 1. Single-Threaded `BumpArenaState`
In `crates/bcinr-logic/src/abstractions/bump_arena.rs`, memory is allocated by adding the requested size to the current offset using wrapping arithmetic. Instead of using conditional logic (`if`/`else`) to check if there is enough space—which would violate the $CC=1$ rule—the allocator uses branchless bitwise masking:
```rust
let next_offset = current_offset.wrapping_add(size);
let success = (next_offset <= self.capacity) as u32;
let mask = 0u32.wrapping_sub(success); // 0xFFFFFFFF if success, 0x00000000 if fail
```
If the allocation fits, the mask allows the cursor to advance. If it exceeds capacity, the mask zeroes out the changes, predictably rejecting the allocation and returning `(0, 0)`.

### 2. Atomic Concurrent-Safe Bump Arena (ACSBA)
For concurrent environments, ACSBA achieves thread-safe bump allocation without the loop backedges of Compare-And-Swap (CAS) or the branching of mutex locks. It uses a single **loop-free atomic `fetch_add`**:
1. **Optimistic Claim**: Each thread claims a unique range by executing `self.offset.fetch_add(size, Ordering::SeqCst)`.
2. **Branchless Validation**: The thread branchlessly evaluates if the claimed range is within the arena's bounds and has not triggered an integer overflow.
3. **Write-Once Integrity**: If validation fails, a `0` mask is produced and the allocation is rejected. The pointer is not reverted upon failure, preserving the zero-backedge property.

## Memory Safety Guarantees (No Global Allocator / No GC)
The architecture guarantees memory safety through rigorous Hoare-logic contracts, rather than runtime tracking or garbage collection. 

1. **Holistic Lifetimes (No Deallocation/GC)**: `BumpArena` memory is **write-once and bump-only** until reset. Individual allocations are never freed or reclaimed individually during execution. Instead, the entire arena's lifetime is tied to an epoch or computational phase. Space reclamation is handled holistically by resetting the arena cursor to `0` or swapping the arena block via a slow-rail mechanism.
2. **No Overlapping Ranges**: The atomic `fetch_add` guarantees a strict mathematical conservation of space. The contract proves that any two successful concurrent allocations will never overlap, enforcing mutual exclusion intrinsically. 
3. **Guaranteed Bounds Execution**: Allocations that exceed the static capacity bound or wrap around integer limits are provably flattened to a `0` mask by the compiler's bitwise polynomial execution, averting buffer overruns entirely.
4. **Machine-Code Proofs**: Memory bounds checks are statically verified at the machine code level to consist only of sequential constant-time assembly (e.g., `setbe`, `setae`, `and`, `neg` on x86_64), enforcing memory limits securely without panics or timing side-channels.
