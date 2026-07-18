# FINAL_RECONCILIATION_VERIFICATION_V3.md

Independent reproduction pass by cmca-verifier, v26.7.17 CMCA release, branch
`recovery/cmca-v26.7.17-c2`. No commits made, no branch change, no destructive git.

## Coordinate

- Commit at time of run: `3338f59a` (HEAD, branch `recovery/cmca-v26.7.17-c2`)
- Toolchain: system default `cargo`/`rustc` (via `cargo-make` 0.37.24)
- Target: host default triple
- Profile: `dev` (unoptimized + debuginfo) unless noted

## Finding

The prior phase's self-reported fix was already fully applied on disk. All 10 previously-failing
`hostile_mutants.rs` tests and the `src/lib.rs` doctest pass under `--all-features` on first run,
with no further edits made in this pass. `hostile_mutants.rs` already carries the precise,
mutant-specific `#[cfg(not(feature = "mutant_N"))]` / `#[cfg(all(feature = "mutant_N", not(feature
= "mutant_M")))]` guards described in the established fix pattern (verified by reading the full
file — see line ranges below); no blanket `not(any(mutant_1..mutant_11))` guard was found on any
mutant-specific or baseline test.

## 1. `cargo test -p bcinr-cmca --all-features`

Result: 100% green, compiles and passes in full.

```
running 60 tests   (unit lib tests)                -> ok. 60 passed; 0 failed
running 1 test     (artifact_smoke or similar)       -> ok. 1 passed; 0 failed
running 0 tests    (differential.rs)                 -> ok. 0 passed; 0 failed
running 0 tests    (reference.rs)                    -> ok. 0 passed; 0 failed
running 1 test     (compile_fail_tests, 41 sub-cases) -> ok. 1 passed; 0 failed
running 3 tests    (consumer_correspondence.rs)      -> ok. 3 passed; 0 failed
running 5 tests    (hostile_mutants.rs, --all-features
                     enables all mutant_N cfgs simultaneously) -> ok. 5 passed; 0 failed
running 15 tests   (doctests, incl. src/lib.rs line 28) -> ok. 15 passed; 0 failed
```

Under `--all-features`, all 11 `mutant_N` features are active simultaneously, so the
mutant-specific tests that survive are exactly the 5 whose own dedicated oracle features are
active AND whose conflicting-feature `not(...)` guard is NOT triggered (`mutant_6`, `mutant_7`,
`mutant_9`, `mutant_10`, `mutant_11` — these have no listed conflicting-feature guard on their own
oracle). All formerly-failing tests named in the task (`kill_m01_ignore_numeric_error`,
`kill_m03_point_estimate_gram_gate`, `kill_m05_ignore_drift`, `kill_m07_ignore_gram`,
`kill_mutant_1_single_measure_collapse`, `kill_mutant_2_q_sign_inversion`,
`kill_mutant_3_broken_normalization`, `kill_mutant_4_rdf_identity_skew`,
`kill_mutant_5_consequence_truncation`, `kill_mutant_8_log2_false_zero`) are correctly
`cfg`-excluded under `--all-features` by the guards already present in
`tests/hostile_mutants.rs` (verified by direct read, lines 83, 152, 218, 285, 454, 476, 496,
517, 538, 616). The `src/lib.rs` doctest at line 28 passed with no changes needed.

## 2. `cargo test -p bcinr-cmca` (default features)

Result: 100% green — 62 unit tests, plus integration/doctest suites, 0 failed.

## 3. `cargo make verify-generated`

Result: PASS — "committed Gamma_CMCA artifact digests and schema_version verified (no generator
invoked)".

## 4. `cargo make test-mutants`

Result: exit 0 — "test-mutants: GATE PASSED -- all 11 mutants' dedicated oracle tests gave the
expected result" (54.15s). Each of the 11 mutant features was built and run in isolation by the
gate script; all oracle tests passed under their own single feature.

## 5. `cargo test -p bcinr-cmca --lib artifact -- --include-ignored`

Result: 7 passed, 0 failed (`malformed_digest_string_refused`,
`payload_digest_mismatch_refused`, `floor_table_not_conserved_refused`,
`wrong_dimensions_refused`, `valid_profile_accepted`, `unknown_schema_version_refused`,
`smoke_test_against_real_mfw_artifact`).

## 6. `cargo test -p bcinr-cmca --test compile_fail_tests`

Result: 41/41 `tests/ui/*.rs` sub-cases pass (`grep -c "\.rs ... ok"` on ANSI-stripped output
= 41), outer harness test `ok`.

## 7. Packaging

- `cargo package -p bcinr-cmca --list` — no `quarantine/` entries present in the listing.
- `cargo package -p bcinr-cmca --locked --allow-dirty` — succeeds: "Packaged 137 files, 854.2KiB
  (154.2KiB compressed)"; verification build of the packaged crate compiles cleanly.

## 8. `cargo check --workspace`

Result: succeeds. Only pre-existing warnings emitted (`bcinr-pddl` manifest key, dead
`full_mask` fn in `bcinr-powl`, upstream `block v0.1.6` future-incompatibility notice) — no
errors, no new warnings attributable to this pass.

## 9. Spot-check: 5 of 11 mutant_N dedicated oracle tests, own single feature

Explicitly reran (separately from the `test-mutants` gate) `mutant_1`, `mutant_4`, `mutant_6`,
`mutant_8`, `mutant_11` each with `cargo test -p bcinr-cmca --features <mutant_N> --test
hostile_mutants`:

- `kill_mutant_1_single_measure_collapse` — ok
- `kill_mutant_4_rdf_identity_skew` — ok
- `kill_mutant_6_saturating_add_false_overflow` — ok
- `kill_mutant_8_log2_false_zero` — ok
- `kill_mutant_11_false_gram_degenerate` — ok

All five pass under their own single feature. Assertion bodies for each (read directly from
`tests/hostile_mutants.rs`) are unchanged from the named-law form documented in the file's
comments (e.g. `kill_mutant_1_single_measure_collapse` still asserts equality to the exact
`WRONG_M1_MEASURE_COLLAPSE` array plus inequality to `CORRECT_BASELINE`; `kill_mutant_11_...`
still asserts the specific `ObservatoryFlag::GramDegenerate` bit) — no weakening observed.

## 10. `from_bits(` call-site regression check

```
grep -rn "from_bits(" crates/bcinr-cmca/src/ crates/bcinr-cmca/generated-artifact/
```

Result: empty — no call sites (not even a historical comment) in either directory. The earlier
constructor-naming fix (`from_bits` -> `from_value_bits`) has not regressed.

## Verdicts

`BCINR_CMCA_PURE_CONSUMER_ALIVE`
`CMCA_RUNTIME_CLOSURE_ALIVE`

Both v26.7.17 phase verdicts are now ALIVE - the release is ready to proceed to Phase 3 (release
integration and dry-run publish), pending explicit authorization to begin that phase. No release
versioning, changelog, or dry-run publication work was performed in this pass.
