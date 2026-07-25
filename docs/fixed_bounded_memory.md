# Fixed Bounded Memory Access in BCINR

The `bcinr` deterministic substrate enforces a strict zero-allocation, branchless execution model. One of its absolute runtime laws is **fixed bounded memory access**, which fundamentally bans dynamic memory scaling and heap fragmentation in the hot path.

## The Zero-Allocation Boundary

All authoritative hot-path logic inside BCINR is governed by the `#![no_std]` attribute and a strict **0 heap allocation** mandate.
- **Dynamic Types Banned**: Dynamically resizable types like `Vec<T>`, `String`, or `Box<T>` are entirely prohibited.
- **Execution Consistency**: Eliminating the global allocator eradicates non-deterministic overheads, heap fragmentation, and timing side-channels, ensuring identical execution cost for every invocation.
- **Audit Verification**: The `@turing_machine` gate actively analyzes disassembly to verify the absolute absence of allocator symbols in authoritative object code.

## Compile-Time Arrays and Bounded Structures

Instead of relying on dynamic heaps, memory capacities must be strictly defined at compile-time using fixed-width arrays.
- **Static Dimensions**: Data pools are defined using arrays such as `[Slot<T>; N]`, where `N` is an explicit `const` generic parameter.
- **Compile-Time Assurances**: By replacing `Vec` with constant-capacity arrays, the system guarantees that all memory footprints are rigorously bounded and statically analyzed, preventing runtime out-of-memory (OOM) states or dynamic resizing loops.
- **No Unbounded Loops**: Avoiding `Vec` inherently avoids the hidden cyclic iteration required for runtime resizing, aligning with the **Radon Law ($CC=1$)**, which prohibits loop backedges in the hot path.

## Authoritative Memory Primitives

Since `Vec<T>` and the global allocator are absent, BCINR provides specialized branchless memory primitives mapped over these fixed capacities:

### 1. `LockFreeSlab<const N: usize>`
Used for heterogeneous, independently lived state.
- **Fixed-Size Capacity**: Pre-allocates memory using a fixed-width array `[u32; N]` for its internal freelist indices.
- **O(1) Branchless Allocation**: Acquires available slots via a strict loop-free Compare-And-Swap (CAS) over the atomic head, yielding mask-selected outcomes bounded at $\le 200$ ns.

### 2. `BumpArenaState`
Used for homogeneous, epoch-bound memory objects.
- **Pre-computed Capacity**: Tracks available space via static `offset` and `capacity` limits.
- **Mask-Based Refusal**: Allocations mathematically advance the offset up to the boundary. If the new offset exceeds the buffer, branchless bitwise masks force the transaction to refuse and yield zero, leaving the state precisely untouched without triggering a branch or panic.

## Enforcing Bounded Memory Execution

The substrate does not merely suggest compile-time memory boundaries; it legislatively enforces them:
1. **Object-Code Audits**: The release object code is mechanically verified to prove all authoritative functions are $CC=1$ and free of panic or bounds-checking paths. Any attempt to dynamically scale memory inherently creates branches or allocator dependencies, which blocks merge.
2. **Substrate Integrity Score (SIS)**: Any verified heap allocation in the hot path sets the SIS to 0, triggering an immediate quarantine under the `MaturityScrutiny` protocol.
