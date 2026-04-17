# Anti-Patterns in Branchless Calculus

## 1. Branching in Latency-Critical Loops
**Anti-Pattern:** Using `if` or `match` inside loops for performance-critical pathing.
**Consequence:** Branch predictor state pollution, pipeline stalls, and timing side-channels.
**Correction:** Utilize `Mask` or `Select` family primitives to map logical branches into arithmetic data-paths.

## 2. Unchecked Arithmetic in Physical Actuation
**Anti-Pattern:** Direct `+` or `-` operators for inputs controlling physical actuators.
**Consequence:** Arithmetic wrap-around (overflow) causes fatal physical failures (e.g., inverting thrust or cooling levels).
**Correction:** Use the `Fix` family primitives (`add_sat`, `sub_sat`) which are saturation-aware and branchless by design.

## 3. Cache-Line False Sharing
**Anti-Pattern:** Declaring shared buffers or transition tables without explicit 64-byte alignment (`#[repr(align(64))]`).
**Consequence:** High cache-invalidation traffic during concurrent access, leading to multi-lane latency spikes.
**Correction:** Use cache-line alignment on all shared/static state transition structures.

## 4. O(N) Spatial Queries
**Anti-Pattern:** Linear scanning of dense state arrays for bitmask operations.
**Consequence:** Throughput degradation at high scale ($>10^6$ agents).
**Correction:** Utilize `rank_u64` and `select_bit_u64` for constant-time $O(1)$ spatial coordinate resolution.
