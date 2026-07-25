# Analysis of `mutant_4`

Based on a review of the codebase (`crates/bcinr-cmca/tests/hostile_mutants.rs` and `crates/bcinr-cmca/src/allocator.rs`), here is the documentation for `mutant_4`:

### Mathematical Law Broken
`mutant_4` violates the **explore/exploit mixing law**:
`val = eta * nl_recip + (1 - eta) * p_mu`

It breaks this law by substituting `zeta` for `eta` during the calculation of `eta_actual`. This swaps in the wrong admitted identity for the exploration-floor weight (skewing the RDF identity hash masking).

### Expected Outcome / Refusal
The expected outcome is that the mutant diverges from the mathematically correct outcome (`CORRECT_BASELINE`). Specifically, the test `kill_mutant_4_rdf_identity_skew` verifies that `mutant_4` produces exactly the corrupted allocation array represented by the constant `WRONG_M4_RDF_IDENTITY_SKEW`. If it produces this precise incorrect output, the mutant is successfully detected (killed) by the test oracle.
