# `BumpArena` and Scratch Lifetimes in BCINR

## 1. Zero-Allocation Hot Path and `petri_tick` Scratch Space
In the BCINR substrate, the authoritative hot path operates under a strict **Zero-Allocation Boundary (`#![no_std]`)** and the **Radon Law ($CC=1$)**. 

While `BumpArena` is a primary tool for deterministic memory management in BCINR, the scratch space required to compute candidate states during a `petri_tick` transition does not actually use `BumpArena`. Instead, `petri_tick` completely eliminates the need for an arena allocator by utilizing **pure stack allocation** for its scratch structures. 

In `crates/bcinr-powl/src/scheduler_wired.rs`, the `petri_tick` function computes the candidate states and transition arrays using fixed-size, stack-allocated arrays:
```rust
// Build per-transition arrays for PriorityPetriEngine (stack-allocated, no heap).
let (inputs, outputs, op_indices) =
    build_transition_arrays(tape, candidates, state.choice_taken);
```
The output arrays (e.g., `[KBitSet<1>; 64]` and `[u32; 64]`) provide a fixed, bounded memory footprint. Because these are standard stack frames, this "scratch" memory is intrinsically deterministic and is safely and automatically reclaimed by the hardware stack pointer the moment the `petri_tick` function returns—eliminating the need for manual memory management, garbage collection, or arena resets on a per-tick basis.

## 2. Deterministic Fixed-Size Allocation in `BumpArena`
When variable-sized scratch space *is* required across multiple ticks or phases, BCINR employs `BumpArena`. `BumpArena` provides $O(1)$ allocations from a fixed-capacity buffer sequentially.

To remain branchless ($CC=1$) and avoid timing side-channels, `BumpArena` does not use `if/else` checks for capacity limits. Instead, it relies on mathematical bitwise masks:
```rust
let current_offset = self.offset;
let next_offset = current_offset.wrapping_add(size);
let success = (next_offset <= self.capacity) as u32;
let mask = 0u32.wrapping_sub(success); // 0xFFFFFFFF on success, 0x00000000 on fail

self.offset = (next_offset & mask) | (current_offset & !mask);
```
If the allocation fits, the cursor advances. If it exceeds capacity, the mask zeroes out the change and the arena predictably drops the request—ensuring execution continues in constant time without panicking or allocating to the heap.

## 3. Automatic and Holistic Memory Reclamation
Unlike a `LockFreeSlab` (which reclaims individual fixed-size slots via a freelist), `BumpArena` memory is **write-once and bump-only**. The arena does not deallocate or reclaim space per individual allocation. 

Space reclamation is handled **holistically** by resetting the arena cursor back to `0`. This is typically tied to an epoch, computational phase, or autonomic metric rather than an individual `petri_tick`. 

For example, in the `AutonomicExhaustionArena` (`crates/bcinr-logic/src/patterns/autonomic_arena.rs`), telemetry on allocation failures accumulates until it hits a healing threshold. When triggered, the arena resets branchlessly using another mask:
```rust
let trigger = ((self.stale_bytes >= self.healing_threshold) as u32) | failed_mask;
let trigger_mask = 0u32.wrapping_sub(trigger & 1);

// Pure state update (no side effects, no branches)
self.arena.offset &= !trigger_mask;
```
When `trigger_mask` is active (all 1s), `!trigger_mask` becomes `0`, instantly zeroing out the arena's `offset` and automatically reclaiming the scratch space for the next epoch, without violating the $CC=1$ rule.
