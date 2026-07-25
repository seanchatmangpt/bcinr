Based on the `GEMINI.md` architectural laws and associated documentation in the `bcinr` codebase, here is a detailed breakdown of the **Infer** phase of the MAPE-K loop, focusing on how `RlState` is calculated using "branchless metrics" while strictly adhering to the CC=1 (Radon Law) constraint.

### MAPE-K Autonomic Loop Context
The MAPE-K loop in `bcinr` is designed for self-managing components utilizing `AutonomicSubstrate` building blocks. The loop follows these steps:
1. **Observe**: Collect bit-level telemetry.
2. **Infer**: Calculate `RlState` using branchless metrics.
3. **Propose**: Generate `AutonomicAction` masks.
4. **Accept**: Filter through the `PolicyGuard`.
5. **Execute**: Advance state via constant-time transitions.

### The "Infer" Phase: Calculating `RlState`
In the Infer phase, raw bit-level telemetry collected during the Observe phase is accumulated into the knowledge base, and the internal Reinforcement Learning state (`RlState`) is calculated. This must be done deterministically and without timing side-channels to preserve the "Zero-Allocation Boundary" and the $CC=1$ rule.

#### 1. Zero-Allocation `RlState` Structure
To comply with the Zero-Allocation Boundary (0 heap allocations in the hot path), `RlState` is implemented as a stack-allocated container utilizing exactly 136 bits:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(C)]
pub struct RlState {
    pub low: u64,   // bits 0-63
    pub high: u64,  // bits 64-127
    pub extra: u8,  // bits 128-135
}
```
This fixed-size memory footprint eliminates heap churn and ensures safety during hot-path execution.

#### 2. Branchless Metrics ($CC=1$ Compliance)
Under the **Radon Law ($CC=1$)**, no public primitive may contain an `if`, `match`, or data-dependent `loop`. Instead of relying on control flow constructs to analyze and categorize telemetry, the system derives its metrics strictly via straight-line arithmetic.

This is achieved by representing semantic state changes and conditions using:
* **Bitwise polynomials**
* **Arithmetic and bitwise masks**
* **SWAR (SIMD Within A Register) mechanics**

#### 3. Constant-Time State Transitions
When calculating or transitioning the `RlState`, the logic employs deterministic bitwise operations rather than conditional branching. This ensures execution in strictly $O(1)$ constant time with no data-dependent instruction paths or back-edges.

For example, merging two states involves a simple bitwise XOR logic:
```rust
pub const fn merge(&self, other: &Self) -> Self {
    Self {
        low: self.low ^ other.low,
        high: self.high ^ other.high,
        extra: self.extra ^ other.extra,
    }
}
```

Similarly, state integrity checks are reduced to single-instruction bitwise XOR operations (`low ^ high`), and state updates (reinforcement learning feedback) structurally mutate the fixed-width fields via arithmetic operations.

#### Verification and Safety
To satisfy the "Contract with Teeth," the logic supporting the `RlState` inference is proven against hostile counterfactual mutants (e.g., swapping `low ^ high` with `low & high`). This ensures any deviation from rigorous bitwise polynomials results in compile-time or test-time failure, keeping the Substrate Integrity Score pristine.

### Summary
By translating conditional telemetry inferences into fixed-width bitwise arithmetic and masked selections, the Infer phase computes the `RlState` while mathematically guaranteeing that the authoritative hot path contains zero branches.
