# AutonomicSubstrate and the MAPE-K Loop

## Definition

The `AutonomicSubstrate` is defined in `crates/bcinr-logic/src/autonomic/autonomic_substrate.rs`. It serves as a generic container for holding a system's internal knowledge and reinforcement learning state.

```rust
pub struct AutonomicSubstrate<K, V, const N: usize>
where
    K: Copy + Default + PartialEq,
    V: Copy + Default,
{
    pub knowledge: PackedKeyTable<K, V, N>,
    pub state: RlState,
}
```

## Branchless MAPE-K Loop Orchestration

Following the project's stringent zero-allocation (`#![no_std]`) and Radon Law ($CC=1$) mandates, the Monitor-Analyze-Plan-Execute (over a shared Knowledge base) loop is orchestrated to be an axiomatic calculus immune to timing side-channels. The loop is formally interfaced via the `AutonomicKernel` trait (`crates/bcinr-logic/src/autonomic/kernel.rs`) and orchestrates the five phases as follows:

### 1. Observe
- **Mandate:** Collect bit-level telemetry.
- **Branchless Implementation:** The system ingests raw telemetry directly into fixed-size structures (the `PackedKeyTable` knowledge base) without dynamic memory allocation, parsing, or unbounded iteration.

### 2. Infer
- **Mandate:** Calculate `RlState` using branchless metrics.
- **Branchless Implementation:** Raw telemetry is transformed into an internal `RlState` using strictly straight-line arithmetic, SWAR (SIMD Within A Register) mechanics, and bitwise polynomials. No conditional branching is used for metric calculation.

### 3. Propose
- **Mandate:** Generate `AutonomicAction` masks.
- **Branchless Implementation:** Instead of enqueuing variable tasks or employing dynamic control flow, constant-time, fixed-width execution masks are computed. These encode operational adjustments (like Repair, Optimize, or Scale) and prepare the system for unconditional execution.

### 4. Accept
- **Mandate:** Filter through the `PolicyGuard`.
- **Branchless Implementation:** Action masks are evaluated against deterministic boundaries. Acceptance logic mathematically yields a full-width positive mask if accepted, or a zeroed-out mask if rejected, completely avoiding `if` statements and early returns.

### 5. Execute
- **Mandate:** Advance state via constant-time transitions.
- **Branchless Implementation:** The substrate's persistent state is transitioned purely via mask-based selection, equivalent to `next_state = (mask & proposed_state) | (~mask & current_state)`. This guarantees execution bounded by deterministic time and devoid of data-dependent execution paths or back-edges.
