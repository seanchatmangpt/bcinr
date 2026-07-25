I've found the branchless telemetry logic for `RecertificationSuggested` (represented in the code as `RecertificationCandidate` or `BIT_RECERTIFICATION_SUGGESTED`) in `crates/bcinr-cmca/src/observatory.rs`.

### Mathematical Inequality
The recertification candidate condition evaluates to true when:
1. The lower bound of the condition number meets or exceeds the safety threshold ($\kappa_{\text{under}} \ge \epsilon_{\text{on}}$).
2. The minimum positive Gram eigenvalue lower bound meets or exceeds the degeneracy threshold ($\gamma_{\text{min\_plus\_under}} \ge \epsilon_{\text{gram}}$).
3. The measurement artifact's proposal is proposing a delta rather than retaining the current mode ($\text{proposal} \ne \text{ModeDelta::Retain}$).

Mathematically, this implies:
$$(\kappa_{\text{under}} \ge \epsilon_{\text{on}}) \land (\gamma_{\text{min\_plus\_under}} \ge \epsilon_{\text{gram}}) \land (\text{proposal} \ne \text{Retain})$$

### Branchless Mask Evaluation
In `crates/bcinr-cmca/src/observatory.rs` (`evaluate_calibration`), this is implemented branchlessly as:

```rust
// 1. Check if condition number lower bound is >= epsilon_on
let kappa_under_on = const_lt_u32(epsilon_on.value_bits(), kappa_under.value_bits())
    | const_eq_u32(epsilon_on.value_bits(), kappa_under.value_bits());

// 2. Check if gram eigenvalue lower bound is < epsilon_gram (inverts the mathematical logic for composition)
let gamma_under_off =
    const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits());

// 3. Check if proposal is Retain
let is_unadmitted = const_eq_u32(artifact.proposal as u32, ModeDelta::Retain as u32);

// Compose using bitwise AND and bitwise NOT
let is_recert = kappa_under_on & (!gamma_under_off) & (!is_unadmitted);
```

This resulting `is_recert` mask is then passed directly into `ObservatoryFlagSet::from_conditions`, where it branchlessly sets `BIT_RECERTIFICATION_SUGGESTED` (the 5th bit):

```rust
// In ObservatoryFlagSet::from_conditions
| (recertification_suggested & 1).wrapping_mul(Self::BIT_RECERTIFICATION_SUGGESTED);
```
