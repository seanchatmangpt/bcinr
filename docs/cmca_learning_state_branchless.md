# Branchless Derivation of Metrics and RL States

Based on an analysis of `crates/bcinr-cmca/src/observatory.rs` and `crates/bcinr-logic/src/autonomic/rl_state.rs`, the system derives metrics (like divergence/drift, condition numbers) and Reinforcement Learning states (`RlState`) from raw observations using purely deterministic, branchless mechanics ($CC=1$) and zero heap allocations. This is achieved through the following architectural patterns:

## 1. Compile-Time Loop Unrolling
Variable-bound iteration is entirely prohibited. Iterations over datasets or node structures are performed using static unrolling macros like `unroll_8_static!(I, { ... })` and `unroll_4_static!(K_IDX, { ... })`. This generates straight-line code at compile time, completely eliminating loop backedges.

## 2. Bitwise Masking for Bounds Checks
To prevent the compiler from inserting implicit, branch-bearing bounds checks, array indices are statically constrained using bitwise AND masks before access. For example:
- `x[I & 7]` guarantees the index is always $\le 7$ for an array of size $N=8$.
- `node_masses[K_IDX & 3]` constrains the index within the valid $K=4$ bounds.
This provides mathematical certainty that bounds will never be violated, preventing panic branches.

## 3. Branchless Selection Logic
Instead of `if/else` control flow, logical decisions are converted into full-width masks (0 or 1).
- Conditions are evaluated using `const_eq_u32` or `const_lt_u32`.
- The results form a mask that drives constant-time selection via `const_select_u32(mask, true_val, false_val)`.
- For example, conditionally isolating children metrics is done using: `let x_safe = const_select_u32(is_child, x[J & 7] as u32, i32::MIN as u32)`.

## 4. Arithmetic Aggregation over Control Flow
Metrics are accumulated across all items blindly rather than short-circuiting. For instance, when aggregating exponentials over valid subset nodes, the subset mask is used to either yield the exponential value or `0`:
```rust
let is_child = const_eq_u32(parent[J & 7] as u32, v as u32);
let exp_val = SignedFixed::from_value_bits(a_prime).exp2();
sum_exp_meas += NonNegativeFixed::from_value_bits(const_select_u32(is_child, exp_val.value_bits(), 0));
```

## 5. Parallel State Flags
Calibration evaluations (`evaluate_calibration`) evaluate multiple conditions simultaneously (numerical uncertainty, drift, scale inertia, etc.) using bitwise masks. These are compiled into an `ObservatoryFlagSet` via bitwise-OR composition rather than early-returning on the first failure. This guarantees that all simultaneously-true telemetry conditions are preserved in a deterministic timeline.

## 6. RL State Structural Integrity
`RlState` is strictly bounded and stack-allocated (repr(C), 136 bits via two `u64`s and one `u8`), preventing any heap churn. Merging states or transitioning configurations utilizes simple bitwise XOR logic (`self.low ^ other.low`), completely avoiding branches while retaining zero-cost abstractions.
