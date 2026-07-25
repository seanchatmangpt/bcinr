# MAPE-K Autonomic Loop in BCINR

The BCINR deterministic computational substrate implements the MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge) autonomic loop using branchless, allocation-free operations. This ensures that the system manages itself without introducing timing side-channels, dynamic heap allocations, or conditional branching (enforcing the Radon Law, $CC=1$).

## Core Structures

### 1. `AutonomicSubstrate` (Knowledge & State Container)
**Location:** [`crates/bcinr-logic/src/autonomic/autonomic_substrate.rs`](file:///Users/sac/bcinr/crates/bcinr-logic/src/autonomic/autonomic_substrate.rs)

The `AutonomicSubstrate` serves as the generic container for the MAPE-K loop, holding the system's internal knowledge and reinforcement learning state without heap allocations.
```rust
pub struct AutonomicSubstrate<K, V, const N: usize> {
    pub knowledge: PackedKeyTable<K, V, N>,
    pub state: RlState,
}
```

### 2. `RlState` (Reinforcement Learning State)
**Location:** [`crates/bcinr-logic/src/autonomic/rl_state.rs`](file:///Users/sac/bcinr/crates/bcinr-logic/src/autonomic/rl_state.rs)

A strict 136-bit, stack-allocated representation designed to eliminate heap churn and serve as the deterministic state for the substrate.
```rust
#[repr(C)]
pub struct RlState {
    pub low: u64,   // bits 0-63
    pub high: u64,  // bits 64-127
    pub extra: u8,  // bits 128-135
}
```
State merging and manipulation is done using deterministic bitwise operations (e.g., XORing fields directly in the `merge` function) ensuring cyclical complexity remains $CC=1$.

## Branchless Implementation of the MAPE-K Loop

The formal interface for this self-managing cycle is defined in the `AutonomicKernel` trait (`crates/bcinr-logic/src/autonomic/kernel.rs`), and the phase transitions execute via constant-time, branchless logic:

### 1. Observe (Monitor)
- **Action:** Collect bit-level telemetry.
- **Branchless Execution:** The system ingests raw telemetry directly into fixed-size structures (`PackedKeyTable`) in constant time ($O(1)$) without dynamic memory allocation, parsing, or unbounded iteration.

### 2. Infer (Analyze)
- **Action:** Calculate `RlState` and telemetry metrics (e.g., `ObservatoryOutcome` in `bcinr-cmca`).
- **Branchless Execution:** Raw telemetry is transformed into safety indicators and internal `RlState` using strictly straight-line arithmetic, SWAR (SIMD Within A Register) mechanics, and fixed-point bitwise polynomials.
  - *Example in CMCA Observatory (`crates/bcinr-cmca/src/observatory.rs`):* Multiple numerical safety criteria (uncertainty, Gram degeneracy, non-stationary drift) are evaluated simultaneously using branchless bitwise OR/AND masks (`const_lt_u32`, `const_eq_u32`), aggregating into a full-set `ObservatoryFlagSet` without short-circuiting.

### 3. Propose (Plan)
- **Action:** Generate `AutonomicAction` masks.
- **Branchless Execution:** Instead of employing dynamic control flow or enqueuing tasks, constant-time, fixed-width execution masks are computed. These encode operational adjustments (like Repair, Optimize, or Scale) and prepare the system for unconditional execution.

### 4. Accept
- **Action:** Filter actions through the `PolicyGuard`.
- **Branchless Execution:** Action masks are evaluated against deterministic boundaries. Acceptance logic mathematically yields a full-width positive mask (e.g., all 1s) if accepted, or a zeroed-out mask if rejected, completely avoiding `if` statements and early returns.

### 5. Execute
- **Action:** Advance system state via constant-time transitions.
- **Branchless Execution:** The substrate's persistent state is transitioned purely via mask-based selection, which is mathematically equivalent to:
  ```rust
  next_state = (mask & proposed_state) | (~mask & current_state)
  ```
  This guarantees execution is bounded by deterministic time and devoid of data-dependent paths or back-edges. Rejected operations leave the persistent state bit-for-bit unchanged.
