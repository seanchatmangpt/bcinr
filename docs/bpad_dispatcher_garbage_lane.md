# BpadDispatcher Garbage Lane Mechanics

## Overview
The `BpadDispatcher` (Bit-Parallel Atomic Dispatcher) in the `bcinr` substrate strictly adheres to the **Radon Law ($CC=1$)**, meaning it must completely avoid control-flow branches (no `if`, `match`, or conditional execution) in its authoritative hot path. To securely handle failed atomic submissions—such as when the ring is full or there is Compare-And-Swap (CAS) contention—without introducing conditional logic to skip the memory write, it implements a **Garbage Lane** (a padding memory sink) at index 8.

## The Architecture
The dispatcher's internal storage is allocated with 9 elements instead of 8:
```rust
pub struct BpadDispatcher {
    pub occupancy: AtomicU8, // Bitmask tracking occupancy for lanes 0..7
    pub slots: [BpadSlot; 9], // 8 active lanes + 1 garbage lane (index 8)
}
```
* **Indices `0..7`**: Active parallel worker lanes.
* **Index `8`**: An unconstrained, write-only memory sink securely isolated from the read path.

## Branchless Absorption Mechanism
When operations are submitted (via `try_submit` or `fanout_pair`), the runtime employs a strict, constant-time execution model:

### 1. Destination Selection (`select`)
The runtime determines whether the CAS on the `occupancy` bitmask was successful. It then uses a purely bitwise `select` helper to evaluate the memory destination:
```rust
let dest_idx = select(is_ok, slot_idx as usize, 8);
```
If the allocation succeeds (`is_ok == true`), `dest_idx` resolves to the target lane (e.g., 0-7). If the allocation fails, it resolves to `8`.

### 2. Unconditional Store Execution
Wrapping a state mutation in an `if is_ok { ... }` block would violate the $CC=1$ cyclomatic complexity constraint. Instead, the runtime unconditionally executes the atomic memory store:
```rust
self.slots[dest_idx].op_index.store(op_idx, Ordering::Release);
```
The CPU issues the exact same memory instructions regardless of success or failure.

### 3. Harmless Dissipation
When the operation fails and `dest_idx` is `8`, the write securely sinks into the Garbage Lane:
* **Mathematical Inaccessibility**: The worker claim function (`try_claim`) strictly evaluates `slot_idx & 7` and reads from `occupancy`, which is an `AtomicU8`. Bit 8 mathematically cannot exist in the occupancy mask, ensuring consumers never attempt to read from the Garbage Lane.
* **Safe Overwriting in Fan-outs**: During an all-or-nothing two-slot fan-out (`fanout_pair`), if the dispatcher lacks the capacity for both slots, both destination variables evaluate to `8`. The code unconditionally writes `left` then `right` into the garbage lane. The second write simply clobbers the first in the isolated sink, completely preventing partial state corruption or the need for a complex "rollback" phase.

## Substrate Invariants Satisfied
This bit-parallel garbage lane approach strictly obeys the BCINR structural constitution:
1. **Zero Conditional Branches**: Straight-line, data-independent memory paths are maintained.
2. **Deterministic Latency**: Successful writes and failed writes traverse identical memory pathways and CPU instructions, entirely eliminating timing side-channels.
3. **Rollback-Free Atomicity**: Transient, partially written states are physically impossible in the operational lanes, effectively neutralizing data races observed in legacy dispatchers.
