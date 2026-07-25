Here is exactly how `ObservatoryFlag::ModeDeltaUnadmitted` is set branchlessly within `crates/bcinr-cmca/src/observatory.rs`:

The flag is set during telemetry evaluation inside the `evaluate_calibration` function. The logic operates entirely via bitwise operations and arithmetic, avoiding any control-flow branching. 

Here are the three steps that make this possible:

### 1. Branchless Condition Evaluation
First, the current proposal is compared against `ModeDelta::Retain` using a branchless equality check `const_eq_u32`:
```rust
let is_unadmitted = const_eq_u32(artifact.proposal as u32, ModeDelta::Retain as u32);
```
`const_eq_u32` returns a `1` if the values are equal and `0` otherwise, avoiding conditional branches.

### 2. Passing the Condition Mask
This `is_unadmitted` mask is then passed along with all other independent telemetry condition masks to the flag set's branchless constructor:
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

### 3. Bitwise Construction 
Inside `ObservatoryFlagSet::from_conditions`, the bit representing `ModeDeltaUnadmitted` is toggled using integer arithmetic and bitwise ORs:
```rust
pub(crate) const fn from_conditions(
    // ... other parameters
    unadmitted: u32,
    // ...
) -> Self {
    let bits = // ... other conditions
        | (unadmitted & 1).wrapping_mul(Self::BIT_UNADMITTED)
        | // ... other conditions;
    Self(bits)
}
```
* The `(unadmitted & 1)` operation sanitizes the input mask, ensuring it's strictly `1` or `0`.
* Calling `.wrapping_mul(Self::BIT_UNADMITTED)` scales the binary `1/0` mask by the flag's position (`1 << 4`). This mathematically results in either `16` (if the flag is set) or `0` (if it isn't).
* This result is then composed into the final bitfield using a bitwise `|` alongside the evaluations of all the other flags.
