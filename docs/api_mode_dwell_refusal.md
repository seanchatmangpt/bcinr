# Branchless Generation of `ModeDwellViolated`

In `bcinr-cmca` (specifically `crates/bcinr-cmca/src/allocator.rs`), the `ModeDwellViolated` typed refusal (represented internally as `RefusalSet::DWELL_UNSATISFIED`) is evaluated and enforced strictly without control-flow branches, conforming to the $CC=1$ rule.

### 1. Branchless Comparison
Instead of using branching (e.g., `if tau_d < MODE_DWELL_ROUNDS_MIN`), the elapsed dwell rounds (`tau_d`) are evaluated mathematically via the `const_lt_u32` polynomial comparator:
```rust
let dwell_err = const_lt_u32(
    tau_d,
    crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
) != 0;
```
`const_lt_u32` computes `a < b` using a bitwise polynomial over two's complement arithmetic. It extracts the sign bit without generating an assembly branch, producing a deterministic `1` (if violated) or `0` (if satisfied).

### 2. Error Aggregation
The `dwell_err` boolean flag is then folded directly into a unified bit-flag of control-plane errors:
```rust
let has_error =
    !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

### 3. Mask-Based Refusal Injection
The explicit `DWELL_UNSATISFIED` variant is seamlessly injected into the final refusal set via canonical masking. By casting `dwell_err` to `u32` and calling `.masked()`, it safely `union`s the refusal via bitwise OR only when the condition was violated:
```rust
let gated_refusals = RefusalSet::EMPTY
    // ...
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    // ...
    .masked(has_refusal as u32);
```

### 4. Mask-Based State Transition (The Gate)
Any accumulated error blocks the state transition by funneling into the `has_refusal` canonical mask. The state is then gracefully dropped or committed through fixed bitwise multiplexers (`const_select_u32` and `select_nnf`), successfully avoiding `if / else` blocks:
```rust
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],
            local_weights[v & 7][e & 7],
        );
    });
});
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
*prev_mode = const_select_u32(has_refusal as u32, *prev_mode, local_prev_mode);
```
If `dwell_err` was triggered, `has_refusal` resolves to an active mask (logically operating as `(mask & current) | (!mask & candidate)`). The state is seamlessly reverted bit-for-bit, and the refusal is surfaced upstream as a `ModeDwellViolated` equivalent, entirely respecting the $CC=1$ architectural mandate.
