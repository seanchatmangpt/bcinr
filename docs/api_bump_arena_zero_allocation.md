Here is the documentation on `BumpArena` found in `crates/bcinr-logic/src/abstractions/bump_arena.rs` and its related file `crates/bcinr-logic/src/patterns/autonomic_arena.rs`.

# BumpArena and AutonomicExhaustionArena

The `BumpArena` mechanism in BCINR operates exclusively as a branchless, deterministic abstraction that manages a fixed-capacity memory boundary. By adhering to the overarching constitutional requirement of `#![no_std]` and zero heap allocation, it achieves $CC=1$ cyclomatic complexity.

## Zero Heap Allocation Enforcement
The zero heap allocation boundary is strictly enforced structurally:
1. **No Allocator Dependencies:** The implementation avoids any standard library memory management primitives (no `Box`, `Vec`, or standard `Allocator` traits) and uses no runtime allocation calls.
2. **Pre-allocated Bounded Capacity:** The `BumpArenaState` struct consists solely of simple `offset` and `capacity` integers (`u32`). It does not allocate memory on the heap itself; rather, it issues abstract offsets mapped against a pre-allocated chunk.
3. **Strict Bounds Checking without Panicking:** The arena has a fixed capacity limit. Operations mathematically fail gracefully if the allocation limit is breached without ever invoking language-level `panic` paths or triggering dynamic allocation routines to grow the backing memory.

## Branchless Techniques Used
All semantic decisions within `BumpArena` and `AutonomicExhaustionArena` are compiled down to mathematically equivalent bitwise polynomials:
1. **Boolean-to-Mask Conversion:** Branchless masks are derived by evaluating a bounds condition as a boolean, casting it to `u32`, and subtracting it from `0` to saturate bits:
   ```rust
   let success = (next_offset <= self.capacity) as u32;
   let mask = 0u32.wrapping_sub(success); // 0xFFFFFFFF if true, 0x00000000 if false
   ```
2. **Bitwise State Selection:** Control flow for assigning the new offset is handled completely via masking rather than if-else branches:
   ```rust
   self.offset = (next_offset & mask) | (current_offset & !mask);
   ```
   The allocation returns `(current_offset & mask, mask)`.
3. **Branchless Word-Alignment:** Calculations forcing aligned sizes bypass conditional modulo arithmetic using pure bitwise alignment logic:
   ```rust
   let aligned_size = (size + 7) & !7;
   ```
4. **Mask-Triggered Side-Effect Free Transitions:** In the higher-level `AutonomicExhaustionArena`, state transitions triggered by capacity exhaustion avoid branches entirely. They construct a boolean trigger mask and apply bitwise AND/OR logic to reset memory values seamlessly.
   ```rust
   let trigger = ((self.stale_bytes >= self.healing_threshold) as u32) | failed_mask;
   let trigger_mask = 0u32.wrapping_sub(trigger & 1);
   
   // Pure state update (no side effects, no branches)
   let next_epoch = self.epoch.epoch.wrapping_add(1) % 3;
   self.epoch.epoch = (next_epoch & trigger_mask) | (self.epoch.epoch & !trigger_mask);
   self.arena.offset &= !trigger_mask;
   self.stale_bytes &= !trigger_mask as u64;
   ```
