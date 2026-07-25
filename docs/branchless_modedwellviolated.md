# Branchless Generation of `ModeDwellViolated`

In `bcinr`, the deterministic Hot Path avoids standard timer conditionals (e.g., `if elapsed < min { return Err(...); }`) to strictly adhere to the project's $CC=1$ cyclomatic complexity mandate (the Radon Law). Instead of branching, the substrate uses pure bitwise polynomials and canonical masks to evaluate if an adaptive state is attempting to switch modes before fulfilling the required dwell time.

### 1. Branchless Comparison

The hot path evaluates whether the elapsed dwell rounds (`tau_d`) satisfy the `MODE_DWELL_ROUNDS_MIN` constraint without any control-flow branches. This is evaluated mathematically via the `const_lt_u32` polynomial comparator:

```rust
let dwell_err = const_lt_u32(
    tau_d,
    crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
);
```

Under the hood, `const_lt_u32` computes `a < b` using a proven bitwise polynomial over two's complement arithmetic, isolating the sign bit of the difference without generating an assembly branch:

```rust
// Polynomial branchless `<` comparison
let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
```

This returns a deterministic `1` (if violated, i.e., `tau_d < MODE_DWELL_ROUNDS_MIN`) or `0` (if satisfied).

### 2. Error Aggregation

The `dwell_err` boolean flag is then folded directly into a unified bit-flag of control-plane errors:

```rust
let has_error =
    !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

### 3. Mask-Based Refusal Injection

Once the `1` or `0` violation flag (`dwell_err`) is derived, it is expanded into a full-width canonical mask. The explicit `DWELL_UNSATISFIED` variant (which maps to `ModeDwellViolated`) is seamlessly injected into the final refusal set via canonical masking. By casting `dwell_err` to `u32` and calling `.masked()`, it safely unions the refusal via bitwise OR only when the condition was violated:

```rust
let gated_refusals = RefusalSet::EMPTY
    // ... other refusals
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    // ...
    .masked(has_refusal as u32);
```

If `dwell_err` is `1`, the mask is `0xFFFFFFFF`, and the `DWELL_UNSATISFIED` bit is logically OR'd into `gated_refusals`. If `0`, the mask is `0x00000000`, leaving the refusal bit untouched.

### 4. Mask-Based State Transition

Ultimately, the candidate state for the mode switch must be either committed or safely dropped. True to the mask-based execution law (Rule 9/10), this is executed using a bitwise multiplexer (`const_select_u32` / `select_nnf`) rather than an `if !has_refusal` block:

```rust
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```

If the dwell time was violated, the resulting `has_refusal` bit generates an active canonical mask. The `select` multiplexer then mathematically enforces `(mask & current) | (!mask & candidate)`. The state transition is gracefully reverted to the persistent state bit-for-bit, and the refusal is surfaced upstream as `ModeDwellViolated` (or its internal equivalent), successfully avoiding any architectural branching.
