# Mode Dwell Violation in `bcinr-cmca`

## Location and Naming
The refusal bitmask is named `RefusalSet::DWELL_UNSATISFIED` (which corresponds to the single-variant enum `StabilityRefusal::ModeDwellTimeViolated`). It is defined and evaluated in `crates/bcinr-cmca/src/allocator.rs`.

## Structural Condition
This refusal bit represents a violation of the mode dwell time constraint—specifically, attempting an allocation or state transition before the system has spent the minimum required rounds in the current control mode. 

The mathematical/structural condition is:
```rust
tau_d < crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN
```
where `tau_d` (passed into `allocate()`) represents the observed duration since the last mode switch.

## Branchless Evaluation and Accumulation
In strict compliance with the deterministic substrate constitution (the "Radon Law" requiring `CC=1` and "Mask-based execution law"), the violation is evaluated and added to the final `RefusalSet` using purely arithmetic, branchless operations.

### 1. Constant-Time Predicate Evaluation
Instead of using an `if` statement, the check utilizes a constant-time arithmetic intrinsic (`const_lt_u32`) to produce a `bool` without branching:
```rust
let dwell_err = const_lt_u32(
    tau_d,
    crate::generated::stability_profile::MODE_DWELL_ROUNDS_MIN,
) != 0;
```

### 2. Global Error Aggregation
The resulting `dwell_err` is then folded into a global aggregate boolean using a bitwise `|` alongside other constraint checks, avoiding any short-circuiting control flow (`||`):
```rust
let has_error =
    !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

### 3. Bitmask Construction (`masked` and `union`)
`RefusalSet` uses bit flags internally (e.g., `pub const DWELL_UNSATISFIED: Self = Self(1 << 7);`). To set the bit without branching, the code uses a mask-and-union pattern:
```rust
let gated_refusals = RefusalSet::EMPTY
    // ...
    .union(RefusalSet::DWELL_UNSATISFIED.masked(dwell_err as u32))
    // ...
    .masked(has_refusal as u32);
```

The `.masked(condition)` method expands the `u32` (0 or 1) into a full-width mask (all zeros or all ones) via arithmetic wrapping (`0u32.wrapping_sub(condition & 1)`), and performs a bitwise `&`. This returns the `DWELL_UNSATISFIED` bits if the error occurred, or `0` otherwise. The `.union()` method then logically `OR`s the result into the growing `RefusalSet`. 

Because no branching (`if` or `match`) is used, the instruction shape is fixed, and the exact same execution work occurs regardless of whether the dwell time was violated or satisfied.
