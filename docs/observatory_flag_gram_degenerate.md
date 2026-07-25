I have located where `ObservatoryFlag::GramDegenerate` is evaluated in `crates/bcinr-cmca/src/observatory.rs`.

The mathematical inequality that triggers this flag is:
$$ \kappa_{\text{under}} \ge \epsilon_{\text{on}} \quad \text{AND} \quad \gamma_{\text{min,under}}^{+} < \epsilon_{\text{gram}} $$

Where:
- **$\kappa_{\text{under}}$** (`kappa_under`) is the condition number lower bound (from `artifact.lower_bound`).
- **$\epsilon_{\text{on}}$** (`epsilon_on`) is the condition number stability threshold.
- **$\gamma_{\text{min,under}}^{+}$** (`gamma_min_plus_under`) is the minimum positive Gram eigenvalue lower bound (from `artifact.gram_lower_bound`).
- **$\epsilon_{\text{gram}}$** (`epsilon_gram`) is the Gram degeneracy threshold limit.

### Branchless Mask Evaluation
In the `evaluate_calibration` function, the condition is evaluated using constant-time bitwise operations to maintain $CC=1$:

```rust
// 1. Evaluate: kappa_under >= epsilon_on
let kappa_under_on = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits())
    | const_eq_u32(epsilon_on.value_bits(), kappa_under.value_bits());

// 2. Evaluate: gamma_min_plus_under < epsilon_gram
#[cfg(not(feature = "mutant_11"))]
let gamma_under_off =
    const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits());

// 3. Compose masks with bitwise AND
let is_gram_degenerate = kappa_under_on & gamma_under_off;
```

The resulting `is_gram_degenerate` flag is passed into `ObservatoryFlagSet::from_conditions`, where it's masked directly into the flag set using integer multiplication:
```rust
(gram_degenerate & 1).wrapping_mul(Self::BIT_GRAM_DEGENERATE)
```
