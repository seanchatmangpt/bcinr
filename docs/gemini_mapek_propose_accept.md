# Propose and Accept in the MAPE-K Autonomic Loop

The BCINR "Deterministic Substrate" implements the MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) Autonomic Loop under absolute runtime laws, specifically **zero heap allocations** (`#![no_std]`) and the **Radon Law ($CC=1$)**. This guarantees that execution logic acts as an axiomatic calculus expressed purely through arithmetic, completely immune to timing side-channels.

## 1. The Propose Phase (Mask Generation)

In a traditional system, the "Plan/Propose" phase might allocate a task (e.g., a "Repair Task") and push it onto a variable-length queue for conditional execution. In BCINR, variable-length task queues and conditional logic are strictly prohibited.

Instead, the subsystem proposes operations by computing constant-time, fixed-width execution masks (**AutonomicAction masks**):

- **Mathematical Evaluation:** The system evaluates the `AutonomicState` (e.g., metrics like `drift_detected`, `integrity`, `throughput`, and `health`) using straight-line arithmetic, binary polynomials, and SWAR (SIMD Within A Register) mechanics.
- **Fixed-Width Masks:** The result of this evaluation is a wide integer mask for each potential operation (`Repair`, `Optimize`, `Scale`). 
  - An **active** (triggered) action results in a mask of all ones (e.g., `0xFFFFFFFFFFFFFFFF`).
  - An **inactive** (ignored) action results in a mask of all zeros (e.g., `0x0000000000000000`).

This ensures the system executes the exact same sequence of instructions regardless of whether an action is required or not.

## 2. The Accept Phase (PolicyGuard Filtering)

Before any proposed action mask is allowed to mutate the state in the "Execute" phase, it must pass through the **PolicyGuard**. The `PolicyGuard` strictly evaluates proposals against hard deterministic boundaries.

- **Branchless Mask Logic:** The `PolicyGuard` converts boolean validation checks into fixed-width masks branchlessly. For example, validating a metric against a threshold is implemented as:
  ```rust
  let check = (val > threshold) as u64;
  let mask = 0u64.wrapping_sub(check); 
  // Results in 0xFFFFFFFFFFFFFFFF if true, 0x0000000000000000 if false
  ```
- **Mask-Based Filtering:** If an action violates the deterministic boundaries or risk matrix, the guard applies a bitwise `AND` that effectively zeroes out the action mask. This rejects the operation completely without using speculative branching or early returns.

## 3. Mask Utilization (Execute Phase)

Finally, during the Execute phase, the system advances its persistent `RlState` using the accepted masks. The state transition Mathematically blends the proposed state and current state:

```text
next_state = (mask & proposed_state) | (~mask & current_state)
```

- If accepted (`mask == !0`), the system transitions perfectly to the `proposed_state`.
- If rejected or unneeded (`mask == 0x0`), the `proposed_state` is zeroed out, and the `current_state` is preserved bit-for-bit.

By utilizing `AutonomicAction` masks and the `PolicyGuard`, BCINR entirely eliminates timing side-channels, memory allocation variability, and branching complexities.
