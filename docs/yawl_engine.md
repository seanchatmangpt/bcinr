# YAWL Routing Semantics Engine (`bcinr` Implementation)

## Overview
The codebase includes a branchless Binary YAWL engine (`BYawlEngine`), located in `playground/src/yawl.rs`. It executes the core routing semantics of YAWL (Yet Another Workflow Language), including all advanced split and join patterns, multiple instances, and cancellations, entirely without conditional control flow (`if`, `match`, `while`) or dynamic memory allocation.

Instead of branching, it evaluates execution rules using bitwise arithmetic masks, constant-time `popcount`, and 128-bit SIMD intrinsics (for counting multi-instances), conforming to `bcinr`'s deterministic mandate (`CC=1`).

## Data Structures
- **`BYawlTask`**: A cache-aligned struct (64-byte alignment) representing a YAWL transition. It defines the `join_type`, `split_type`, bounds for multi-instances (`min_instances`, `max_instances`, `threshold_instances`), and a series of u64 bitmasks: `consume_mask`, `produce_mask`, `cancellation_mask`, `condition_mask`, `reset_mask`, `reachability_mask`, and `interleaved_lock_mask`.
- **`BYawlEngine`**: The execution state container tracking up to 64 places. It holds `state_mask` (active tokens), `active_instances` (array of 64 bytes for multi-instance counts, manipulated via SIMD), `active_triggers` (transient events), `fired_joins_mask` (tracks completed complex joins), and `active_locks` (for interleaved routing).

## Routing Semantics (Joins)
The `join_type` specifies the pre-conditions for firing. All are computed simultaneously into masks:

- **XOR Join (Simple Merge):** Fires if *exactly one* incoming token is present.
  *Logic:* `nz_mask_u64(c) & z_mask_u64(c & (c - 1))`
- **AND Join (Synchronizing):** Fires if *all* required incoming tokens are present.
  *Logic:* `z_mask_u64((state_mask & consume_mask) ^ consume_mask)`
- **OR Join (Synchronizing Merge):** Fires when active incoming tokens are present and *no further tokens can reach the task* from upstream paths.
  *Logic:* Uses a pre-calculated `reachability_mask` (`aux & !val == 0`).
- **Complex Join (Discriminator / N-out-of-M):** Fires once a threshold is met (`popcount(consume_mask) >= threshold_instances`). Further tokens arriving after it has fired are vacuumed (consumed on bypass) by checking the `fired_joins_mask`.
- **Thread Merge Join:** Fires if *any* incoming token is present.

## Routing Semantics (Splits)
The `split_type` and its effects are applied conditionally via masks (using `& fired_mask`), rather than branching:

- **AND / XOR / OR Splits:** The engine bitwise ORs the `produce_mask` into the `state_mask`. (For XOR/OR splits, dynamic branch choices are typically modeled by evaluating state into the transition's `condition_mask` or mapping branches to distinct mutually exclusive tasks prior to the bitwise hot-path).
- **Multi-Instance Splits:** Spawns multiple instances (up to `max_instances`) at the target place. It uses `blend_u8x16` and `compare_eq_u8x16` SIMD intrinsics to update the `active_instances` array concurrently.
- **Implicit Termination:** Ends the case silently if no active tokens remain (the engine drops the `produce_mask` application).
- **Explicit Termination:** Annihilates all tokens in the case. It clears `state_mask`, `fired_joins_mask`, `active_locks`, and zeros out the `active_instances` array via SIMD `and_u8x16` masks.

## Advanced Patterns
- **Interleaved Routing (Mutex):** Controlled via `active_locks` and `interleaved_lock_mask`. A task can acquire a lock (blocking conflicting tasks) and release it when completed.
- **Cancellation Regions:** If a task fires, its `cancellation_mask` removes tokens from the specified places and zeros out their multi-instance counts.
- **Transient Triggers:** Supports events that satisfy a join condition dynamically (e.g., signals/messages), checked via `active_triggers`.
