# `LockFreeSlab` ABA-Free Mechanism

The `LockFreeSlab` (located in `crates/bcinr-logic/src/abstractions/lock_free_slab.rs`) achieves deterministic, ABA-free execution and complies with the project's strict branchless $CC=1$ mandate (the Radon Law) through several specific structural choices.

## 1. Monotonic Bump Allocation over Pointer Chasing
While standard lock-free freelists suffer from the ABA problem because freed indices are pushed back onto the `head` pointer directly, `LockFreeSlab`'s `alloc_t1` sidesteps this entirely. Rather than chasing a linked list via `next_indices[head]` (which would be vulnerable to an ABA race), the `freelist` head simply increments monotonically:
```rust
let next = (head.wrapping_add(1)) & can_alloc_mask | head & !can_alloc_mask;
```
Because the `AtomicU32` only goes forward (until wrapping at $2^{32}-1$), the atomic value itself acts as a continuously advancing generation counter. A thread suspended before a CAS will never experience a "false positive" success from an index being popped and pushed back, because the `head` value is never pushed backward.

## 2. Branchless Mask-Based Selection ($CC=1$)
All internal logic is executed using bitwise mask selection instead of conditional branches (`if`/`else`), preserving $CC=1$ compliance:
```rust
let is_empty = (head == 0xFFFFFFFF) as u32;
let can_alloc = (!is_empty) & 1;
let can_alloc_mask = 0u32.wrapping_sub(can_alloc);
```
The success and failure paths deterministically calculate the `next` value using `can_alloc_mask`. If the slab is exhausted (`0xFFFFFFFF`), the mask collapses to `0`, forcing `next` to safely equal `head` and failing the allocation without branching.

## 3. Omission of In-Band Deallocation
In the current authoritative implementation file, an explicit in-band `dealloc_t1` method does not exist. Because indices are generated monotonically and never directly pushed back onto the atomic `freelist` head, the classic ABA cycle is structurally impossible.

Memory reuse and reclamation are handled out-of-band. The system relies on physical array index mapping (`index % N` at the use-site) and external safe memory reclamation (supported by the `EpochState` in `epoch_reclamation.rs`), rather than interweaving freed indices back into the atomic allocation head. This strictly enforces the forward-progress-only state machine required for deterministic execution.
