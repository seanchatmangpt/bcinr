# AutonomicSubstrate: RlState, AutonomicAction, and PolicyGuard

Within the BCINR deterministic substrate, self-managing systems utilize the **AutonomicSubstrate** to implement a MAPE-K (Monitor-Analyze-Plan-Execute over a shared Knowledge base) autonomic loop. To strictly adhere to the project's absolute runtime laws—namely zero heap allocation and the Radon Law ($CC=1$)—the substrate relies on mathematically verifiable, branchless primitives.

This document explores how `RlState`, `AutonomicAction`, and `PolicyGuard` interact to enable autonomic behavior without conditional control flow.

## 1. `RlState`: Branchless Metrics Calculation

`RlState` (Reinforcement Learning State) acts as a strictly bounded, stack-allocated representation of the system's runtime learning condition. Designed to eliminate heap churn, it is packed into exactly 136 bits:
- `low`: 64 bits
- `high`: 64 bits
- `extra`: 8 bits

Rather than mutating state conditionally, `RlState` calculates transitions through fixed-width arithmetic and bitwise polynomials.

### Branchless Mechanics
To satisfy $CC=1$, state updates avoid all `if/else` logic. Instead, `RlState` provides constant-time algebraic functions. For example, merging two states relies on bitwise XOR:

```rust
pub const fn merge(&self, other: &Self) -> Self {
    Self {
        low: self.low ^ other.low,
        high: self.high ^ other.high,
        extra: self.extra ^ other.extra,
    }
}
```

Metrics and checksums are computed using a single instruction (e.g., `low ^ high`), translating semantic continuous learning into deterministic polynomials that are proven against counterfactual mutants.

## 2. `AutonomicAction`: Mask-Based Proposals

In a traditional architecture, an action (like a Repair or Optimize command) might be queued and conditionally dispatched. BCINR forbids dynamic queues and branching logic. Instead, self-correction operations are represented mathematically via `AutonomicAction` masks.

### The Proposal Phase
During the "Propose" step of the MAPE-K loop, the system evaluates the current `AutonomicState` (which includes metrics like integrity and health) and mathematically derives a full-width execution mask for potential actions.

- **Active Proposal**: If the mathematical evaluation requires an action, the result evaluates to `!0` (`0xFFFFFFFFFFFFFFFF`).
- **Inactive Proposal**: If no action is needed, the evaluation evaluates to `0x0`.

The execution step unconditionally calculates the result of the action, but only commits it to persistent state via mask-based selection. The state transition blends the candidate state and current state in purely constant time:

```text
next_state = (mask & proposed_state) | (~mask & current_state)
```

By expressing an `AutonomicAction` entirely as a selection mask, the CPU performs the exact same fixed amount of work whether the action is ultimately adopted or ignored, eliminating timing side-channels.

## 3. `PolicyGuard`: Structural Filtering Without Early Returns

Before an `AutonomicAction` is allowed to mutate the `RlState`, it must be authorized by the `PolicyGuard`. True to the deterministic mandate, the `PolicyGuard` enforces invariant boundaries and risk assessments entirely without conditional jumps or early returns.

### Generating Acceptance Masks
The `PolicyGuard` transforms safety checks (e.g., ensuring a metric exceeds a required threshold) into binary masks. It casts the result of a boolean comparison to an integer and structurally wraps it into a full mask:

```rust
pub fn mask_gt(val: u64, threshold: u64) -> u64 {
    let check = (val > threshold) as u64;
    0u64.wrapping_sub(check) 
}
```
- If `val > threshold`: `check` is `1`. `0u64.wrapping_sub(1)` evaluates to `0xFFFFFFFFFFFFFFFF` (`!0`).
- If `val <= threshold`: `check` is `0`. `0u64.wrapping_sub(0)` evaluates to `0x0000000000000000` (`0`).

### Mask-Based Enforcement
If an action violates the risk matrix, the `PolicyGuard` returns a `0x0` mask. This rejection mask is applied via a bitwise `AND` to the proposed `AutonomicAction` mask, completely zeroing it out. As a result, the operation is structurally rejected, preserving the existing state bit-for-bit without ever triggering a speculative branch, `if let` guard, or early return.
