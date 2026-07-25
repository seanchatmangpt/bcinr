I have located and analyzed `mutant_11` within the `crates/bcinr-cmca` codebase. 

### Location in the Codebase

- **Test File**: `crates/bcinr-cmca/tests/hostile_mutants.rs` (in the function `kill_mutant_11_false_gram_degenerate`)
- **Production File**: `crates/bcinr-cmca/src/observatory.rs` (in the function `evaluate_calibration`)

### Mathematical Law Broken

`mutant_11` targets the condition that determines if the Gram matrix is degenerate (`is_gram_degenerate`), specifically the calculation of the `gamma_under_off` predicate.

In the baseline branchless implementation, `gamma_under_off` is defined mathematically as:
`gamma_min_plus_under < epsilon_gram`
(Where `gamma_min_plus_under` corresponds to the artifact's `gram_lower_bound`).

```rust
// Baseline (Correct)
#[cfg(not(feature = "mutant_11"))]
let gamma_under_off =
    const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits());
```

Under `mutant_11`, the operand order of this comparison is maliciously inverted, changing the mathematical law to:
`epsilon_gram < gamma_min_plus_under`

```rust
// Mutated
#[cfg(feature = "mutant_11")]
let gamma_under_off =
    const_lt_u32(epsilon_gram.value_bits(), gamma_min_plus_under.value_bits()); 
```

### Expected Outcome / Refusal

The dedicated oracle test (`kill_mutant_11_false_gram_degenerate`) verifies this exact breach.

It supplies a test case where `gram_lower_bound` (131072) is strictly greater than `epsilon_gram` (65536). Under the correct law, `gamma_under_off` should be `FALSE`, and the `GramDegenerate` flag should **not** be set. 

Because `mutant_11` inverts the logic, it breaks this contract and produces a false positive. The expected test outcome is that the `evaluate_calibration` function falsely flags the result as degenerate. The oracle explicitly asserts that the `ObservatoryFlag::GramDegenerate` flag is incorrectly present in the returned bitmask.

Additionally, as documented in `MUTANT_KILL_MATRIX.md`, running the entire test suite with `mutant_11` active causes collateral failures in baseline tests (`kill_m03_point_estimate_gram_gate` and `kill_m07_ignore_gram`) because they depend on the shared correct behavior of the Gram gate that `mutant_11` corrupts.
