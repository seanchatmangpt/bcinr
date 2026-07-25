I have searched the `crates/bcinr-cmca/src/` directory for `ObservatoryFlag::Drifting`. Here is the documentation detailing how this bit is set branchlessly:

In `crates/bcinr-cmca/src/observatory.rs`, the `ObservatoryFlag::Drifting` bit is managed by `ObservatoryFlagSet::BIT_DRIFT` and is determined branchlessly by comparing the evaluated drift metric (`d_js`) against the predefined threshold (`epsilon_drift`).

The mathematical inequality is evaluated as:
```rust
let d_js = artifact.drift;

// ...

#[cfg(not(feature = "mutant_9"))]
let is_drift = const_lt_u32(epsilon_drift.value_bits(), d_js.value_bits());
```

`is_drift` is a `u32` mask (evaluating to `1` if `epsilon_drift < d_js`, or `0` otherwise). This mask is passed to `ObservatoryFlagSet::from_conditions`:
```rust
let flags = ObservatoryFlagSet::from_conditions(
    is_numerically_uncertain,
    is_gram_degenerate,
    is_drift,
    is_scale_inert,
    is_unadmitted,
    is_recert,
);
```

Within `from_conditions`, the bit is set branchlessly using a bitwise polynomial:
```rust
(drift & 1).wrapping_mul(Self::BIT_DRIFT)
```
This isolates the least significant bit of the `drift` mask (making sure it's strictly `0` or `1`) and multiplies it by the bit value for `ObservatoryFlag::Drifting` (`1 << 2`). The result is then bitwise OR-ed with the other condition flags.
