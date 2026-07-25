# Zero-Allocation Memory Management: `BumpArena` in BCINR

## 1. Architectural Constraints (The Radon Law)
Memory management in the BCINR hot path operates under strict axiomatic constraints to guarantee deterministic computational logic:
* **Zero-Allocation Boundary (`#![no_std]`)**: The codebase strictly forbids the use of the global memory allocator. Dynamic heap allocations (`Box`, `Vec`, etc.) are completely eliminated.
* **Radon Law ($CC=1$)**: Cyclomatic complexity must remain exactly 1. Data-dependent branches (`if`, `match`, `while`) and loop backedges are entirely prohibited to prevent timing side-channels.

## 2. `BumpArenaState`: Single-Threaded Base
Located in `crates/bcinr-logic/src/abstractions/bump_arena.rs`, `BumpArenaState` provides a deterministic $O(1)$ memory allocator using a fixed-capacity buffer.

Instead of conditional logic to check if there is enough space, `try_alloc` utilizes branchless bitwise masking:
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
If the requested size exceeds capacity, the mask isolates the current offset, preventing mutation while returning a predictable, branchless rejection `(0, 0)`.

## 3. ACSBA: Atomic Concurrent-Safe Bump Arena
To scale `BumpArena` for concurrent execution, BCINR innovates with the **ACSBA**, detailed in `docs/innovations/atomic_concurrent_safe_bump_arena.md`.

Traditional concurrent allocators use Compare-And-Swap (CAS) loops, which introduce loop backedges and conditional retry branches. This violates the Radon Law. ACSBA solves this by utilizing a **loop-free atomic fetch-add** strategy:
1. **Optimistic Claim**: Threads claim space using a single `fetch_add` operation on an `AtomicU64` offset. 
2. **Branchless Validation**: 
   ```rust
   let old_offset = self.offset.fetch_add(size, Ordering::SeqCst);
   let next_offset = old_offset.wrapping_add(size);
   
   let within_capacity = (next_offset <= self.capacity) as u64;
   let no_overflow = (next_offset >= old_offset) as u64;
   
   let success = within_capacity & no_overflow;
   let mask = 0u64.wrapping_sub(success);
   ```
3. **Write-Once Integrity**: Rejection conditions (capacity limit or integer wrapping) return `0`, safely dropping the invalid allocation request. Space is not reclaimed on failure since the bump arena is reset holistically by a slow-rail mechanism.

## 4. Disassembly and Verification
Because `bcinr` aims for a perfect Substrate Integrity Score (SIS), the allocators are proven at the machine-code level:
* **Object-Code Audits**: Disassembly (e.g., `x86_64`) verifies that zero conditional jumps (`je`, `jne`, `jb`) are generated. The compiler translates the Rust logic natively into `setbe`, `setae`, `and`, and `neg` instructions, resulting in sequential constant-time assembly.
* **Hostile Mutation Testing**: Mutants (e.g., inverting bounds inequalities or dropping integer overflow checks) are systematically injected to verify that the Hoare-logic constraints rigorously block corrupted memory states.
