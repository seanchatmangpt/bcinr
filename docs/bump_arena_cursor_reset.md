# `bump_arena` Cursor Reset Mechanism in BCINR

In the BCINR deterministic substrate, memory management operates under a **Zero-Allocation Boundary (`#![no_std]`)** and the strict **Radon Law ($CC=1$)**. Traditional garbage collection algorithms and dynamic $O(N)$ `Drop` loops are completely prohibited due to their unpredictable timing and reliance on control-flow branches.

Instead, the `BumpArena` allocator relies on a **holistic, branchless cursor reset mechanism** to reclaim memory at epoch or tick boundaries.

## 1. Holistic $O(1)$ Memory Reclamation

Because allocations are merely sequential bumps of an integer `offset` against a fixed `capacity` boundary, individual objects are never individually freed. There are no destructors or $O(N)$ cleanup loops executed in the hot path. 

When an epoch boundary is reached (such as autonomic exhaustion or a cyclical phase transition), the entire arena is reclaimed holistically in $O(1)$ time by simply zeroing out the `offset` cursor. Any subsequent allocations overwrite the previously used buffer space. This phase-based allocation guarantees constant-time memory reuse without heap fragmentation.

## 2. Branchless Reset via Trigger Masks

In accordance with the Radon Law ($CC=1$), the reset mechanism avoids all `if` or `match` conditional jumps. Instead, it natively utilizes **bitwise polynomials (masking)** to conditionally clear the `offset` state.

As implemented in the `AutonomicExhaustionArena` (`crates/bcinr-logic/src/patterns/autonomic_arena.rs`), the reset is calculated branchlessly through the following sequence:

### Step A: Trigger Generation
A condition—such as a failed allocation mask or exceeding a stale bytes limit—generates a deterministic `0` or `1` boolean integer.
```rust
let trigger = ((self.stale_bytes >= self.healing_threshold) as u32) | failed_mask;
```

### Step B: Mask Expansion
The trigger flag is mathematically expanded into a full 32-bit width bitmask via wrapping arithmetic. 
```rust
let trigger_mask = 0u32.wrapping_sub(trigger & 1);
```
- If `trigger` is `1`, `trigger_mask` becomes `0xFFFFFFFF` (all ones).
- If `trigger` is `0`, `trigger_mask` remains `0x00000000` (all zeros).

### Step C: Cursor Clearing & State Rotation
The `offset` cursor is then bitwise AND-ed against the *negation* of the trigger mask. Simultaneously, the epoch rotates using a selection polynomial.
```rust
// 1. Rotate the Epoch
let next_epoch = self.epoch.epoch.wrapping_add(1) % 3;
self.epoch.epoch = (next_epoch & trigger_mask) | (self.epoch.epoch & !trigger_mask);

// 2. Clear the Cursor 
self.arena.offset &= !trigger_mask;
self.stale_bytes &= !trigger_mask as u64;
```
When a reset condition is met (`trigger_mask = 0xFFFFFFFF`):
- `!trigger_mask` becomes `0x00000000`.
- The `offset` is mathematically multiplied (AND-ed) by `0`, resetting it instantly to the start of the buffer.

When the reset condition is not met (`trigger_mask = 0x00000000`):
- `!trigger_mask` is `0xFFFFFFFF`.
- The `offset` is mathematically preserved (`offset & 0xFFFFFFFF == offset`).

## 3. Disassembly Guarantees

This architecture inherently prevents branch mispredictions, loops, or hidden panic paths. The compiler translates these bitwise statements into purely sequential machine instructions (e.g., `setae`, `neg`, `and`), guaranteeing that bounding, rejecting, and resetting memory operates within a hard realtime Worst-Case Execution Time (WCET) budget.
