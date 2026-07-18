# bcinr-cmca Reconciliation Verification V2 (cmca-verifier, independent rerun)

Date: 2026-07-17
Branch: recovery/cmca-v26.7.17-c2 (unchanged, no commits made)

## Summary

The producer-side rename to `from_value_bits` is confirmed applied: `crates/bcinr-cmca/src/fixed.rs`
exposes only `from_value_bits` (no `from_bits`), and both generated artifacts
(`generated-artifact/case-studies/cmca_generated.rs`, `generated-artifact/generalization/cmca_generated.rs`)
call `NonNegativeFixed::from_value_bits` / `SignedFixed::from_value_bits` exclusively — the compile
blocker described in the task (222 errors, `from_bits` not found) is gone.

`cargo test -p bcinr-cmca --all-features` now COMPILES. It does **not** fully PASS: 10 of 15
`hostile_mutants` tests fail, plus 1 doctest (`src/lib.rs` line 28) fails. This is a genuine
runtime/test-content problem, not a compile problem — it is a distinct, still-open issue from
the one this task was scoped to fix. The `Cargo.toml` `generated_artifact_pending` feature
comment ("Compiling that module unconditionally currently fails... flip on only once
src/fixed.rs's owning task reconciles the API surface") is now stale/inaccurate — the module
does compile — but should not be read as evidence the runtime issue is also resolved.

## Step-by-step results

1. **`cargo test -p bcinr-cmca --all-features`** — COMPILES. Does NOT pass in full.
   - unit tests (`src/lib.rs`): 60/60 pass
   - `alloc_gate`: passed on second run; failed once earlier in this session with
     "allocator::allocate() performed 5 heap allocation(s) (956 bytes)" — appears
     nondeterministic/flaky, not investigated further per task scope.
   - `calibration`, `case_studies`, `differential`, `reference`: 0 tests each (no-op)
   - `compile_fail_tests`: 1/1 (trybuild harness) pass — see item 6
   - `consumer_correspondence`: 3/3 pass
   - **`hostile_mutants`: 5 passed, 10 FAILED** (kill_m01_ignore_numeric_error,
     kill_m03_point_estimate_gram_gate, kill_m05_ignore_drift, kill_m07_ignore_gram,
     kill_mutant_1_single_measure_collapse, kill_mutant_2_q_sign_inversion,
     kill_mutant_3_broken_normalization, kill_mutant_4_rdf_identity_skew,
     kill_mutant_5_consequence_truncation, kill_mutant_8_log2_false_zero)
   - **doctest `src/lib.rs` line 28: FAILED** — panicked on
     `assertion failed: outcome.flags.contains(ObservatoryFlag::RecertificationCandidate)`
   - **Verdict: NOT fully green under `--all-features`.**

2. **`cargo test -p bcinr-cmca` (default features)** — PASS. 62 unit tests, plus all other
   test binaries (alloc_gate, calibration, case_studies, compile_fail_tests,
   consumer_correspondence, differential, hostile_mutants — 15/15, reference) all green.

3. **`cargo make verify-generated`** — PASS:
   `verify-generated: PASS - committed Gamma_CMCA artifact digests and schema_version verified
   (no generator invoked)`

4. **`cargo make test-mutants`** — exits 0 / GATE PASSED:
   `test-mutants: GATE PASSED -- all 11 mutants' dedicated oracle tests gave the expected result`.
   Note: this task's own harness runs each mutant under its own single feature (not
   `--all-features`) and explicitly treats the whole-binary `--all-features` collateral
   failures as non-gating (see its own inline note: "mutant_11: whole-binary run reported
   failures above (collateral or otherwise) -- NOT gating, see MUTANT_KILL_MATRIX.md"). This
   is consistent with the `--all-features` failures being pre-existing/known and out of this
   gate's scope, but it also means `test-mutants` passing does not contradict item 1's failure.

5. **`cargo test -p bcinr-cmca --lib artifact -- --include-ignored`** — PASS. 7/7
   (`malformed_digest_string_refused`, `floor_table_not_conserved_refused`,
   `unknown_schema_version_refused`, `payload_digest_mismatch_refused`,
   `wrong_dimensions_refused`, `valid_profile_accepted`,
   `smoke_test_against_real_mfw_artifact`).

6. **`cargo test -p bcinr-cmca --test compile_fail_tests`** — PASS. The trybuild harness
   reports as a single `test compile_fail_tests ... ok` wrapping all UI cases; stdout lists
   14 `tests/ui/fail_tuple_construct_*.rs` cases individually, all `ok`. (Task description
   said "41/41" — this run's harness shows 14 named UI fixtures under one passing harness
   test; did not find a separate 41-count breakdown. Reporting exactly what was observed
   rather than assuming the 41 figure.)

7. **`cargo package -p bcinr-cmca --list`** — quarantine/ NOT present in the file list (grep
   for "quarantine" against the list output returned nothing).
   **`cargo package -p bcinr-cmca --locked --allow-dirty`** — PASS: "Packaged 136 files,
   840.9KiB (150.7KiB compressed)", verify build succeeded.

8. **`cargo check --workspace`** — PASS (warnings only: unused `full_mask` in bcinr-powl,
   future-incompat notice for `block v0.1.6` dependency; no errors).

9. **Spot-check `kill_mutant_6_saturating_add_false_overflow` /
   `kill_mutant_7_saturating_div_false_zero` under their own single feature** —
   both PASS individually:
   `cargo test -p bcinr-cmca --test hostile_mutants --features mutant_6
   kill_mutant_6_saturating_add_false_overflow -- --exact` → ok
   `cargo test -p bcinr-cmca --test hostile_mutants --features mutant_7
   kill_mutant_7_saturating_div_false_zero -- --exact` → ok
   Confirms the `--all-features` fix did not weaken these two specifically.

10. **No `from_bits` shim in `fixed.rs`** — confirmed. `grep -n "fn from_bits("
    crates/bcinr-cmca/src/fixed.rs` matches nothing. Only `from_value_bits` (and the
    unrelated internal `from_bits_raw` on `NumericFaultSet`, a different type, pre-existing,
    not part of this contract) exist. The fix is entirely on the generator/consumer
    (generated-artifact) side, matching the stated decision.

## Verdicts

- **BCINR_CMCA_PURE_CONSUMER_ALIVE**: not re-litigated by this task (stated as already
  achieved); nothing observed in this pass contradicts it.
- **CMCA_RUNTIME_CLOSURE**: the specific compile blocker this task targeted (`from_bits` vs
  `from_value_bits`, 222 errors) is resolved — `--all-features` now compiles. However,
  "compile AND pass in full" (task step 1's exact bar) is not met: 10 hostile-mutant tests
  and 1 doctest fail under `--all-features`. This is a distinct, currently-open blocker.

## Blockers (for CMCA_RUNTIME_CLOSURE_BLOCKED)

1. `cargo test -p bcinr-cmca --all-features` — 10/15 `hostile_mutants` tests fail:
   `kill_m01_ignore_numeric_error`, `kill_m03_point_estimate_gram_gate`,
   `kill_m05_ignore_drift`, `kill_m07_ignore_gram`, `kill_mutant_1_single_measure_collapse`,
   `kill_mutant_2_q_sign_inversion`, `kill_mutant_3_broken_normalization`,
   `kill_mutant_4_rdf_identity_skew`, `kill_mutant_5_consequence_truncation`,
   `kill_mutant_8_log2_false_zero`. Not diagnosed further (out of this task's scope — task
   explicitly says not to start new work in this pass); root cause unknown, UNVERIFIED
   whether this is a feature-interaction artifact (multiple `mutant_N` features enabled
   simultaneously by `--all-features`, each individually gating a different code mutation,
   colliding) or a genuine regression.
2. `cargo test -p bcinr-cmca --all-features` doctest failure at `src/lib.rs` line 28:
   `assertion failed: outcome.flags.contains(ObservatoryFlag::RecertificationCandidate)`.
   Same caveat — not diagnosed, UNVERIFIED root cause.

Both blockers are consistent with `--all-features` simultaneously enabling all 11
`mutant_N` features (each designed to inject exactly one hostile mutation into shared code
paths) — plausible but UNVERIFIED without reading `hostile_mutants.rs`'s cfg-gating in
detail, which was out of scope for this verification pass.
