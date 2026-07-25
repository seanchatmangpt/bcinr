Here is the documentation for `ScaleInert` based on `crates/bcinr-cmca/src/observatory.rs`:

### Mathematical Equality
The **Scale Inertia** condition triggers when the measured scale ($s_{\text{meas}}$) exactly matches the target leaf scale ($s_{\text{leaf}}$):
$$s_{\text{meas}} = s_{\text{leaf}}$$
According to the module documentation, this indicates "a collapse of informative scaling variance" (zero scaling update information).

### Branchless Mask Evaluation
The telemetry condition is evaluated branchlessly using a constant-time equality check on the raw underlying fixed-point bit representations of the parameters:

```rust
let is_scale_inert = const_eq_u32(s_meas.value_bits(), s_leaf.value_bits());
```

This `is_scale_inert` mask evaluates to `1` if true and `0` if false. It is then provided to `ObservatoryFlagSet::from_conditions`, where the specific telemetry bit (`BIT_SCALE_INERT`) is set branchlessly using arithmetic multiplication and bitwise OR:

```rust
let flags = ObservatoryFlagSet::from_conditions(
    is_numerically_uncertain,
    is_gram_degenerate,
    is_drift,
    is_scale_inert, // Passed in here
    is_unadmitted,
    is_recert,
);
```

Inside `from_conditions`, the condition evaluates as follows:
```rust
// ...
| (scale_inert & 1).wrapping_mul(Self::BIT_SCALE_INERT)
// ...
```
