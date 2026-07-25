Here is the documentation regarding `mutant_10` as requested:

### Location of `mutant_10` Implementation
The exact implementation of `mutant_10` is located in `crates/bcinr-cmca/src/observatory.rs` at lines 277-280, within the `evaluate_calibration` function.

```rust
    #[cfg(not(feature = "mutant_10"))]
    let kappa_under_off = const_lt_u32(kappa_under.value_bits(), epsilon_on.value_bits());
    #[cfg(feature = "mutant_10")]
    let kappa_under_off = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits()); // Mutated: inverted
```

### Mathematical Law Broken
The mutated code breaks the **boundary-uncertainty distinction** (a part of the numerical uncertainty bound check). 

According to the mathematical law, a measurement should be considered potentially `kappa_under_off` (and thus numerically uncertain when combined with `kappa_hat_on`) if the lower bound `kappa_under` is strictly less than the activation threshold `epsilon_on` (`kappa_under < epsilon_on`). 

`mutant_10` deterministically corrupts this law by completely inverting the comparison logic so that `kappa_under_off` resolves to true when `epsilon_on < kappa_under`.

### Expected Outcome / Refusal
The required hostile mutant verification is explicitly documented and tested in `crates/bcinr-cmca/tests/hostile_mutants.rs` within the `kill_mutant_10_false_numerically_uncertain` test. 

By inverting the comparison, the expected outcome is that the system will **falsely set the `ObservatoryFlag::NumericallyUncertain` bit in the `ObservatoryOutcome` flags** even when `kappa_under` is greater than or equal to `epsilon_on` (meaning it should have been completely secure from numerical uncertainty). The test mathematically proves this by asserting that the `NumericallyUncertain` flag is raised incorrectly despite providing valid bounding values where `kappa_hat == kappa_under == 131072 >= epsilon_on (65536)`.
