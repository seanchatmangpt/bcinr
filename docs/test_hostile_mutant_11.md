# Analysis of `mutant_11`

**Location:** `crates/bcinr-cmca/src/observatory.rs` and `crates/bcinr-cmca/tests/hostile_mutants.rs`

## Mathematical Law Broken
The mutant corrupts the determination of whether a measurement is considered "Gram-degenerate." 

In the correct baseline implementation, the condition `gamma_under_off` is defined as:
```rust
const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits())
```
This correctly asserts that a measurement is Gram-degenerate if its Gram lower bound ($\gamma_{min+under}$) falls strictly below the accepted tolerance threshold ($\epsilon_{gram}$).

Under `mutant_11`, this condition is inverted:
```rust
const_lt_u32(epsilon_gram.value_bits(), gamma_min_plus_under.value_bits())
```
This breaks the mathematical law by triggering a Gram-degeneracy flag when the lower bound is strictly *greater* than the threshold ($\epsilon_{gram} < \gamma_{min+under}$).

## Expected Outcome & Refusal
Because the condition is inverted, the mutant will falsely flag mathematically sound measurements as degenerate and ignore genuinely degenerate cases.

The test `kill_mutant_11_false_gram_degenerate` in `hostile_mutants.rs` enforces the detection of this exact failure. It feeds the implementation an artifact where the `gram_lower_bound` (131072) is strictly greater than `epsilon_gram` (65536).

- **Correct baseline behavior**: The comparison $131072 < 65536$ is false, so the `GramDegenerate` flag is **not** set.
- **Mutant behavior**: The comparison $65536 < 131072$ is true, so it falsely sets the `GramDegenerate` flag.

The oracle test explicitly checks that the result incorrectly contains `bcinr_cmca::observatory::ObservatoryFlag::GramDegenerate`, proving the mutant's precise contractual violation.
