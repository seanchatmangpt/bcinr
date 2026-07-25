# Setting `ObservatoryFlag::NumericallyUncertain`

In `crates/bcinr-cmca/src/observatory.rs`, the `ObservatoryFlag::NumericallyUncertain` bit is set branchlessly within the `evaluate_calibration` function.

The bit is set by the `is_numerically_uncertain` mask, which is evaluated based on the following mathematically equivalent inequalities:

1. **`kappa_hat >= epsilon_on`**
2. **`kappa_under < epsilon_on`**

Where:
- `kappa_hat` is the `point_estimate` from the `MeasurementArtifact`.
- `kappa_under` is the `lower_bound` from the `MeasurementArtifact`.
- `epsilon_on` is the threshold limit passed as an argument.

### Branchless Mask Evaluation

The conditions are evaluated using bitwise masks and `const_*` functions to guarantee branchless execution:

```rust
let kappa_hat_on = const_lt_u32(epsilon_on.value_bits(), kappa_hat.value_bits())
    | const_eq_u32(epsilon_on.value_bits(), kappa_hat.value_bits());

#[cfg(not(feature = "mutant_10"))]
let kappa_under_off = const_lt_u32(kappa_under.value_bits(), epsilon_on.value_bits());

#[cfg(feature = "mutant_10")]
let kappa_under_off = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits()); // Mutated: inverted

let is_numerically_uncertain = kappa_hat_on & kappa_under_off;
```

The resulting `is_numerically_uncertain` mask (which will be `1` if both conditions are met, and `0` otherwise) is then passed into `ObservatoryFlagSet::from_conditions()` to construct the final bitset.

*(Note: If the `mutant_10` feature is enabled, the second condition is deliberately inverted for mutation testing purposes.)*
