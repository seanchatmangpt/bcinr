# Hostile Mutation Protocol for `bcinr-cmca` (Rule 19)

Based on the research, the project manages the hostile mutation protocol and mutant ledger for `bcinr-cmca` as follows:

## 1. The Mutants Ledger (`MUTANT_KILL_MATRIX.md`)
The official ledger for `bcinr-cmca` is maintained in `crates/bcinr-cmca/MUTANT_KILL_MATRIX.md`. Instead of strictly following the 9-column format outlined in Rule 19, it utilizes adapted columns focusing on isolating "collateral breakage":
*   **mutant id:** e.g., `mutant_1` to `mutant_11`
*   **source file(s) mutated:** Includes a brief description of the **changed law** (e.g., `src/allocator.rs` (Lens Q-value sign)).
*   **dedicated oracle test:** The exact test name (e.g., `kill_mutant_2_q_sign_inversion`).
*   **isolated result:** The result when testing *only* the dedicated oracle test (e.g., `PASS`).
*   **whole-binary collateral:** Identifies if other baseline tests failed due to shared code corruption (the **actual detection** of collateral breakage).
*   **classification:** The standing, usually `KILLED_BY_INTENDED_ORACLE`, occasionally with `(COLLATERAL_FAILURES_PRESENT)`.

## 2. Changed Law, Exact Mutation, and Actual Detection
The specifics of the "changed law", "exact mutation", and "actual detection" are richly documented within the test suite (`crates/bcinr-cmca/tests/hostile_mutants.rs`) and integrated directly into the verification process rather than fully detailed in the markdown matrix:

*   **Changed Law:** Documented comprehensively in test comments. For example, mutant 1 changes the law that "each of the K measures must independently weight the allocation" by pinning the per-measure index to 0. Mutant 3 breaks the "flat-share normalization law".
*   **Exact Mutation:** Injected via compile-time Cargo features (`#[cfg(feature = "mutant_N")]`) into the production codebase (like `src/allocator.rs` or `src/observatory.rs`), which corrupts the exact code path via the real build path. There are also hand-written mutants (m01, m03, m05, m07) implemented inline via closures that substitute variables (e.g., passing `kappa_hat` instead of `kappa_under`).
*   **Expected / Actual Detection:** Tests assert exact typed refusals, numeric faults, or strict equality to specifically corrupted mathematical arrays (avoiding weak assertions like `assert_ne!`), strictly fulfilling Rule 19's requirement for independent oracles. For example:
    *   Mutant 1 output must exactly match a predefined `WRONG_M1_MEASURE_COLLAPSE` array.
    *   Mutant 6 must set exact numeric fault bits: `OVERFLOW` and `SATURATION`.
    *   Mutant 10 must erroneously trigger the `NumericallyUncertain` flag.

## 3. Hostile Mutation Verification Process
The automated verification process is formalized in `Makefile.toml` under the `test-mutants` task. It uses a **two-pass architecture** to resolve "collateral breakage" (where one mutation corrupts shared code and breaks unrelated baseline tests):

1.  **Gating Pass (Isolated Run):**
    *   Iterates through all 11 mutants.
    *   Runs a highly targeted cargo command: `cargo test -p bcinr-cmca --features mutant_N --test hostile_mutants <dedicated_oracle_name> -- --exact`.
    *   This ensures the mutant is successfully killed *only* by its dedicated oracle test. This pass governs the task's exit code (exits 0 if all 11 isolated runs pass).
2.  **Diagnostic Pass (Whole-Binary Run):**
    *   Iterates through all 11 mutants again.
    *   Runs the full test binary without a test-name filter: `cargo test -p bcinr-cmca --features mutant_N --test hostile_mutants -- --nocapture`.
    *   This intentionally captures and logs any collateral failures in other baseline tests caused by the mutation. It is purely non-gating (does not change the exit code) but informs the "whole-binary collateral" diagnostic column in the ledger.
