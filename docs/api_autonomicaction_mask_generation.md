Here is the documentation on how `AutonomicAction` masks are generated and mutated branchlessly during the Propose phase in the BCINR codebase:

# `AutonomicAction` and Branchless Mask Generation

In the BCINR Deterministic Substrate, self-managing components follow a strict MAPE-K (Monitor-Analyze-Plan-Execute) autonomic loop defined in `crates/bcinr-logic/src/autonomic/kernel.rs`. Under the project's **Radon Law ($CC=1$)** and **Zero-Allocation Boundary**, the system is forbidden from using conditional branching (e.g., `if/else`, `match`), dynamic loops, or heap-allocated task queues. 

Instead of generating enums or pushing boolean-gated tasks to a queue, the system triggers `AutonomicAction` operations (Repair, Optimize, Scale) by generating and evaluating **fixed-width bit masks**.

## 1. Mask Generation (Propose Phase)

During the **Propose** (Plan) phase, the system evaluates the inferred `AutonomicState` (which tracks telemetry like drift, integrity, and throughput). System constraints and thresholds are evaluated mathematically to yield full-width masks (e.g., 64-bit integers):
* **Active / Triggered:** Evaluates to all 1s (`0xFFFFFFFFFFFFFFFF` or `!0`)
* **Inactive / Ignored:** Evaluates to all 0s (`0x0000000000000000`)

To achieve this without branching, boolean comparisons are explicitly cast to integers, and integer underflow (`wrapping_sub`) is used to expand `1` into a full-width bitwise mask:

```rust
// Translating a threshold check directly into a mask branchlessly
let check = (val > threshold) as u64; // Yields 1 if true, 0 if false
let mask = 0u64.wrapping_sub(check);  // Yields !0 (all 1s) if true, 0 if false
```

## 2. Branchless Mutation (Execute Phase)

Because branching is prohibited (`CC=1`), the CPU executes the exact same mathematical workload whether an action is needed or not. The generated action masks dictate whether the results of that work are *committed* to the persistent memory. 

The framework mathematically blends the `proposed_state` and the `current_state` in constant time using SIMD Within A Register (SWAR) mechanics. 

```rust
// Mask-based state transition (CC = 1)
let next_state = (mask & proposed_state) | (!mask & current_state);
```

* If the action is triggered and accepted (`mask == 0xFFFFFFFFFFFFFFFF`), `!mask` becomes `0x0`, perfectly adopting the `proposed_state`.
* If the action is unneeded or rejected (`mask == 0x0`), the `proposed_state` is logically zeroed out, and the `current_state` is preserved bit-for-bit.

This exact mathematical approach guarantees uniformity in execution latency and eliminates timing side-channels, fully satisfying the deterministic requirements of the substrate.
