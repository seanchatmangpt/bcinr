# Observatory Telemetry Flag Accumulation

The telemetry machinery in `bcinr` uses a strict $CC=1$ branchless design to record multiple simultaneous failure modes without early termination or short-circuiting. Both `ObservatoryFlag` and `ObservatoryOutcome` are defined in `crates/bcinr-cmca/src/observatory.rs`.

## 1. Branchless Condition Derivation

Within `evaluate_calibration()`, numeric metrics are checked against safety thresholds using branchless comparison functions like `const_lt_u32` and `const_eq_u32`. These functions return raw `1` (true) or `0` (false) as `u32` values.

For `Drifting`, the drift distance (`d_js`) is checked against the boundary (`epsilon_drift`):
```rust
let is_drift = const_lt_u32(epsilon_drift.value_bits(), d_js.value_bits());
```

For `NumericallyUncertain`, multiple masks are derived and then combined using bitwise AND (`&`):
```rust
let kappa_hat_on = const_lt_u32(epsilon_on.value_bits(), kappa_hat.value_bits())
    | const_eq_u32(epsilon_on.value_bits(), kappa_hat.value_bits());
let kappa_under_off = const_lt_u32(kappa_under.value_bits(), epsilon_on.value_bits());

let is_numerically_uncertain = kappa_hat_on & kappa_under_off;
```
Because no `if` or `&&` statements are used, both expressions are unconditionally evaluated.

## 2. Bitwise Accumulation Without Short-Circuiting

The independent `u32` indicator flags (`1` or `0`) are passed to `ObservatoryFlagSet::from_conditions`. The set preserves all simultaneously-true conditions using arithmetic wrapping and bitwise OR (`|`).

```rust
pub(crate) const fn from_conditions(
    numerically_uncertain: u32,
    gram_degenerate: u32,
    drift: u32,
    scale_inert: u32,
    unadmitted: u32,
    recertification_suggested: u32,
) -> Self {
    let bits = (numerically_uncertain & 1).wrapping_mul(Self::BIT_NUMERICALLY_UNCERTAIN)
        | (gram_degenerate & 1).wrapping_mul(Self::BIT_GRAM_DEGENERATE)
        | (drift & 1).wrapping_mul(Self::BIT_DRIFT)
        | (scale_inert & 1).wrapping_mul(Self::BIT_SCALE_INERT)
        | (unadmitted & 1).wrapping_mul(Self::BIT_UNADMITTED)
        | (recertification_suggested & 1).wrapping_mul(Self::BIT_RECERTIFICATION_SUGGESTED);
    Self(bits)
}
```

- Each condition mask is constrained to the least significant bit (`& 1`).
- The mask is conditionally shifted into the correct bit position by multiplying it by the flag's constant (e.g., `BIT_DRIFT` = `1 << 2`). `wrapping_mul` is used to satisfy substrate laws against runtime panics.
- All terms are combined using bitwise OR (`|`). Since `|` evaluates its entire operand chain (unlike `||`), there is no short-circuiting.

## 3. Inclusion in the Outcome State

The accumulated `bits` value is wrapped in an `ObservatoryFlagSet` struct. This opaque bitset is then assigned directly into the final `ObservatoryOutcome`:

```rust
let flags = ObservatoryFlagSet::from_conditions(
    is_numerically_uncertain,
    is_gram_degenerate,
    is_drift,
    // ...
);

ObservatoryOutcome { proposal, flags }
```

By tracking the status as an accumulated bitset rather than an `enum`, the system ensures that multiple critical telemetry signals—such as concurrent `Drifting` and `NumericallyUncertain`—can be safely preserved and propagated upwards without losing data to control-flow branches.
