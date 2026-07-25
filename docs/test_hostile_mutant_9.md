I have investigated `mutant_9` in the codebase. Here are my findings:

### Mathematical Law Broken
`mutant_9` corrupts the threshold comparison logic for detecting Jensen-Shannon drift ($d_{js}$). 
The authoritative law states that drift is detected when the measured drift strictly exceeds the threshold: $d_{js} > \epsilon_{drift}$.

In [crates/bcinr-cmca/src/observatory.rs](file:///Users/sac/bcinr/crates/bcinr-cmca/src/observatory.rs#L267-L271), the branchless substrate implements the correct comparison mask like this:
```rust
#[cfg(not(feature = "mutant_9"))]
let is_drift = const_lt_u32(epsilon_drift.value_bits(), d_js.value_bits());
```

`mutant_9` breaks this by explicitly inverting the comparison direction:
```rust
#[cfg(feature = "mutant_9")]
let is_drift = const_lt_u32(d_js.value_bits(), epsilon_drift.value_bits()); // Mutated: drift check inverted
```
This mathematical corruption will falsely flag the system as "drifting" whenever $d_{js} < \epsilon_{drift}$.

### Expected Outcome / Refusal
The `evaluate_calibration` routine is supposed to compute the full `ObservatoryFlagSet` as a bitwise-OR of independent condition masks.

For an admitted artifact where $d_{js} = 0$ and $\epsilon_{drift} > 0$, the correct execution must **not** set the `Drifting` flag. Under `mutant_9`, the inverted comparison will evaluate to true, falsely raising the `ObservatoryFlag::Drifting` bit.

The oracle explicitly designed to catch and document this refusal is `kill_mutant_9_false_drift` located in [crates/bcinr-cmca/tests/hostile_mutants.rs](file:///Users/sac/bcinr/crates/bcinr-cmca/tests/hostile_mutants.rs#L644-L684):
```rust
assert!(
    result
        .flags
        .contains(bcinr_cmca::observatory::ObservatoryFlag::Drifting),
    "Mutant 9 (inverted drift comparison) should falsely set Drifting for drift=0"
);
```
