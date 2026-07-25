# `mutant_9` Analysis

I have inspected the `bcinr` codebase and found the implementation and test for `mutant_9`. Here is the requested documentation on the mathematical law it breaks and the expected outcome.

## Locations
- **Implementation:** [`crates/bcinr-cmca/src/observatory.rs`](file:///Users/sac/bcinr/crates/bcinr-cmca/src/observatory.rs#L267-L271) (in `evaluate_calibration`)
- **Hostile Test:** [`crates/bcinr-cmca/tests/hostile_mutants.rs`](file:///Users/sac/bcinr/crates/bcinr-cmca/tests/hostile_mutants.rs#L644-L684) (function `kill_mutant_9_false_drift`)

## Mathematical Law Broken
The mutant violates the **Drift Thresholding Law** (`d_js > epsilon_drift`). 

In the deterministic substrate, an artifact is only considered to be "drifting" if its measured Jensen-Shannon drift (`d_js`) strictly exceeds the authorized epsilon threshold (`epsilon_drift`). `mutant_9` explicitly breaks this law by inverting the comparison direction:

```rust
// Lawful implementation: Drift must be strictly greater than epsilon_drift
#[cfg(not(feature = "mutant_9"))]
let is_drift = const_lt_u32(epsilon_drift.value_bits(), d_js.value_bits());

// mutant_9 implementation: Inverted logic
#[cfg(feature = "mutant_9")]
let is_drift = const_lt_u32(d_js.value_bits(), epsilon_drift.value_bits());
```
This mutation creates a critical fault where perfectly stable artifacts (e.g., zero drift) are penalized, while genuinely divergent artifacts may be accepted.

## Expected Outcome & Refusal
The verification test `kill_mutant_9_false_drift` constructs a `MeasurementArtifact` with absolutely zero drift (`drift: NonNegativeFixed::ZERO`) and evaluates it against an `epsilon_drift` of `65536` bits.

Under the correct laws of arithmetic, `is_drift` would be false. However, because `mutant_9` evaluates whether `0 < 65536`, it incorrectly returns true.

The **expected refusal** is that the evaluation framework will explicitly (but falsely) raise the `ObservatoryFlag::Drifting` bit in the resulting `ObservatoryOutcome`. The mutant is successfully "killed" when this typed refusal is asserted in the adversarial test suite:

```rust
assert!(
    result
        .flags
        .contains(bcinr_cmca::observatory::ObservatoryFlag::Drifting),
    "Mutant 9 (inverted drift comparison) should falsely set Drifting for drift=0"
);
```
