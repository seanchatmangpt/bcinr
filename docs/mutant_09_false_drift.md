# `mutant_9` Analysis

I have inspected the `bcinr` codebase and found the implementation and adversarial test for `mutant_9`.

## Locations
- **Test:** [`crates/bcinr-cmca/tests/hostile_mutants.rs`](file:///Users/sac/bcinr/crates/bcinr-cmca/tests/hostile_mutants.rs#L644-L684) (function `kill_mutant_9_false_drift`)
- **Implementation:** [`crates/bcinr-cmca/src/observatory.rs`](file:///Users/sac/bcinr/crates/bcinr-cmca/src/observatory.rs#L267-L271) (function `evaluate_calibration`)

## Mathematical Law Broken
The mutant violates the **Drift Thresholding Law**, which dictates that an artifact is only considered to be "drifting" if its measured Jensen-Shannon drift (`d_js`) strictly exceeds the authorized epsilon threshold (`epsilon_drift`).

`mutant_9` explicitly breaks this law by inverting the comparison direction:

```rust
// Correct Lawful implementation: epsilon_drift < d_js
#[cfg(not(feature = "mutant_9"))]
let is_drift = const_lt_u32(epsilon_drift.value_bits(), d_js.value_bits());

// mutant_9 implementation: Inverted logic (d_js < epsilon_drift)
#[cfg(feature = "mutant_9")]
let is_drift = const_lt_u32(d_js.value_bits(), epsilon_drift.value_bits()); // Mutated: drift check inverted
```
This mutation creates a critical fault where a perfectly stable artifact (e.g., zero drift) is falsely flagged as drifting, while a genuinely divergent artifact might be incorrectly accepted.

## Expected Outcome / Typed Refusal
The verification test `kill_mutant_9_false_drift` constructs a `MeasurementArtifact` with exactly zero drift (`drift: NonNegativeFixed::ZERO`) and evaluates it against an `epsilon_drift` of `65536` bits.

Under the correct laws of arithmetic, `is_drift` should evaluate to false. However, because `mutant_9` evaluates whether `0 < 65536`, it incorrectly resolves the condition to true.

The **expected refusal** is that the evaluation framework will erroneously raise the `ObservatoryFlag::Drifting` bit in the resulting `ObservatoryOutcome`. The mutant is considered successfully "killed" when this exact typed refusal flag is asserted by the adversarial test suite:

```rust
assert!(
    result
        .flags
        .contains(bcinr_cmca::observatory::ObservatoryFlag::Drifting),
    "Mutant 9 (inverted drift comparison) should falsely set Drifting for drift=0"
);
```
