Here is the documentation on how `RlState` collects and calculates metrics branchlessly:

# RlState: Reinforcement Learning State

**Location in Codebase:**
- `RlState` definition: [`crates/bcinr-logic/src/autonomic/rl_state.rs`](file:///Users/sac/bcinr/crates/bcinr-logic/src/autonomic/rl_state.rs)
- Integration: [`crates/bcinr-logic/src/autonomic/autonomic_substrate.rs`](file:///Users/sac/bcinr/crates/bcinr-logic/src/autonomic/autonomic_substrate.rs)

The MAPE-K Autonomic Loop manages the system through constant-time, branchless operations, adhering to the project's **Radon Law (CC=1)** and **Zero-Allocation Boundary**.

## Observe Phase: Collecting Bit-Level Telemetry
During the Observe phase, the system collects telemetry without utilizing branching logic or memory allocations:
* **No Branching or Allocation**: Ingestion avoids dynamic memory allocation and parsing logic that would introduce conditional branches or timing side-channels.
* **Constant-Time Storage**: Raw bit-level data is ingested in constant time ($O(1)$) directly into fixed-width, pre-allocated memory structures, such as the `PackedKeyTable` within the `AutonomicSubstrate`.

## Infer Phase: Calculating `RlState` Branchlessly
During the Infer phase, the system analyzes telemetry to calculate the internal Reinforcement Learning state (`RlState`):
* **The `RlState` Structure**: `RlState` is a strict 136-bit, stack-allocated, zero-allocation container designed to eliminate heap churn. It is structurally represented as two 64-bit integers (`low`, `high`) and an 8-bit integer (`extra`).
* **Branchless Metrics**: Telemetry categorization is performed strictly through straight-line branchless metrics rather than control flow (`if`/`else` or data-dependent loops). The calculations utilize bitwise polynomials, SWAR (SIMD Within A Register) mechanics, and arithmetic masks.
* **Constant-Time State Merging**: Transitions and internal updates of the `RlState` use deterministic bitwise operations (e.g., XORing fields directly in the `merge` function or using fieldwise masked selection). This preserves structural integrity and guarantees cyclical complexity remains precisely $CC=1$.
