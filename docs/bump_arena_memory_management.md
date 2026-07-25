# `BumpArena` Memory Management in BCINR

## The Constraints: The Zero-Allocation Boundary

In the `bcinr` architecture, the authoritative hot path is subject to strict deterministic constraints. Under the **Zero-Allocation Boundary (`#![no_std]`)**, the use of the global memory allocator is strictly forbidden. Dynamic heap allocations (such as `Box`, `Vec`, or standard garbage collection) are entirely eliminated from the hot path. 

Furthermore, memory management must comply with the **Radon Law ($CC=1$)**, meaning no cyclomatic complexity. Data-dependent branches (`if`, `match`) and loop backedges (such as `while` or Compare-And-Swap spin loops) are prohibited to prevent timing side-channels and guarantee constant-time execution.

## How `BumpArena` Satisfies the Constraints

To adhere to these rules, `bcinr` employs `BumpArena`, a deterministic $O(1)$ memory allocator that operates on a fixed-capacity static buffer. Instead of dynamically requesting heap memory, it manages a pre-allocated contiguous byte space sequentially.

### 1. Branchless Sequential Allocation
`BumpArena` bumps a cursor forward for every allocation request. In its single-threaded form (`BumpArenaState`), the allocation (`try_alloc`) is handled via branchless bitwise operations rather than conditional bounds-checking:

```rust
pub fn try_alloc(&mut self, size: u32) -> (u32, u32) {
    let current_offset = self.offset;
    let next_offset = current_offset.wrapping_add(size);
    let success = (next_offset <= self.capacity) as u32;
    let mask = 0u32.wrapping_sub(success); // 0xFFFFFFFF if success, 0x00000000 if fail

    // Mask determines whether the offset is mutated or remains unchanged
    self.offset = (next_offset & mask) | (current_offset & !mask);
    (current_offset & mask, mask)
}
```

If an allocation request exceeds the remaining capacity, the bitwise mask resolves to zero, branchlessly dropping the allocation and rejecting it safely, ensuring no buffer overruns.

### 2. Loop-Free Concurrent Access (ACSBA)
In concurrent execution contexts, `BumpArena` scales via the **Atomic Concurrent-Safe Bump Arena (ACSBA)**. Traditional concurrent allocators rely on CAS (Compare-And-Swap) retry loops, violating the zero-loop-backedge rule. ACSBA avoids this entirely:
- **Optimistic Claim**: Threads claim space using a single `fetch_add` operation on an `AtomicU64` offset.
- **Branchless Validation**: Threads then branchlessly validate whether their specific claimed range is within bounds and free of integer overflow.
- **Write-Once Integrity**: If an allocation exceeds capacity, the space is simply marked invalid (rejected via a zero mask). The atomic pointer is not rolled back or looped, thereby preserving strict $CC=1$ sequential machine code execution.

### 3. Holistic Memory Lifetimes (No Individual Deallocation)
Because there is no dynamic global allocator or GC, individual memory slots in a `BumpArena` are never independently freed or recycled. `BumpArena` employs a **homogeneous lifecycle**:
- The memory is strictly **write-once and bump-only** during the hot path.
- Deallocation occurs holistically: the entire arena is reset (the offset cursor is zeroed) by a slow-rail mechanism at the conclusion of a computational epoch, transaction, or frame.
- For data requiring frequent individual recycling and heterogeneous lifetimes, `bcinr` uses a `LockFreeSlab` instead. `BumpArena` is strictly reserved for variable-sized elements, sequential log accumulation, or contiguous state aggregation bound by a single lifecycle epoch.

### Verification and Guarantees
Because `bcinr` requires a 100/100 Substrate Integrity Score (SIS), the allocators are proven at the machine-code level:
- Object-code audits verify that the generated assembly (e.g., on x86_64) translates the bounds-checking purely into constant-time sequential instructions (`setbe`, `setae`, `and`, `neg`) with zero conditional jumps (`je`, `jne`, `jb`).
- Memory safety is enforced through mathematical conservation constraints, effectively converting complex memory management problems into arithmetic logic.
