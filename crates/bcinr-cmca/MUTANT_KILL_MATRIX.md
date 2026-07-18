# MUTANT_KILL_MATRIX.md — bcinr-cmca

Hostile mutation ledger per AGENTS.md §19 and §29. Regenerated for the WORKSTREAM D reconciliation
pass of the v26.7.17 CMCA release (see task context: "Isolate mutant oracles for mutants 9, 10, 11
so the aggregate gate does not fail on collateral, unrelated test breakage").

## Coordinate

- Branch: `recovery/cmca-v26.7.17-c2`
- Working tree: `cargo build -p bcinr-cmca` succeeds cleanly (0 errors) at the time of this run —
  this **supersedes** the previous version of this file, which recorded a 259-error whole-crate
  compile break (`NonNegativeFixed::from_bits` / `.val` API mismatch) blocking every mutant. That
  break no longer reproduces against the current tree; the crate and `tests/hostile_mutants.rs`
  both compile.
- Toolchain: `rustc`/`cargo` as configured in this workspace's `rust-toolchain`.
- Command pattern used per mutant, gating: `cargo test -p bcinr-cmca --features mutant_N --test hostile_mutants <dedicated_oracle_test_name> -- --exact`
- Command pattern used per mutant, non-gating diagnostic: `cargo test -p bcinr-cmca --features mutant_N --test hostile_mutants -- --nocapture` (whole binary)

## Root finding for this pass: mutants 9, 10, 11 mutate shared production code in `src/observatory.rs`

`mutant_9`, `mutant_10`, and `mutant_11` change the operand order of the drift /
`kappa_under_off` / `gamma_under_off` gates in `src/observatory.rs`. Those same gates are also
exercised — incidentally, not as their intended target — by four pre-existing baseline oracle
tests: `kill_m01_ignore_numeric_error`, `kill_m03_point_estimate_gram_gate`,
`kill_m05_ignore_drift`, `kill_m07_ignore_gram`. Running the bare `--test hostile_mutants` binary
(no test-name filter) under `mutant_9`/`10`/`11` fails one or more of those four baseline tests,
because the mutation corrupts shared code the baseline tests also depend on for their own,
unrelated assertions. This is collateral breakage, not evidence against the mutant's own intended
oracle:

| feature | dedicated oracle (isolated, exact-match) | whole-binary collateral failures observed |
|---|---|---|
| `mutant_9` | `kill_mutant_9_false_drift` — **PASS** | `kill_m05_ignore_drift` — FAILED (panics: "mutant M05 (zeroing d_js) should erase the Drifting flag that the true d_js would have set") |
| `mutant_10` | `kill_mutant_10_false_numerically_uncertain` — **PASS** | `kill_m01_ignore_numeric_error` — FAILED (panics: "mutant M01 (substituting kappa_hat for kappa_under) should erase the NumericallyUncertain flag that the true kappa_under would have set") |
| `mutant_11` | `kill_mutant_11_false_gram_degenerate` — **PASS** | `kill_m03_point_estimate_gram_gate` and `kill_m07_ignore_gram` — both FAILED (analogous Gram-gate assertion panics) |

The mutant's own dedicated oracle test passes cleanly in isolation for all three. The Makefile.toml
`test-mutants` task was restructured (see diff below) so the gating check runs ONLY each mutant's
own dedicated oracle test by exact name, and records the whole-binary collateral failures
separately as a non-gating diagnostic that never affects the task's exit code.

## Ledger — all 11 numbered mutants (isolated dedicated-oracle gating run)

| mutant id | source file(s) mutated | dedicated oracle test | isolated result | whole-binary collateral | classification |
|---|---|---|---|---|---|
| `mutant_1` | `src/allocator.rs` (per-measure index) | `kill_mutant_1_single_measure_collapse` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_2` | `src/allocator.rs` (Lens Q-value sign) | `kill_mutant_2_q_sign_inversion` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_3` | `src/allocator.rs` (weight normalization) | `kill_mutant_3_broken_normalization` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_4` | `src/allocator.rs` (`eta`/`zeta` identity) | `kill_mutant_4_rdf_identity_skew` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_5` | `src/allocator.rs` (`mu` clipping) | `kill_mutant_5_consequence_truncation` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_6` | `src/fixed.rs` (`saturating_add` overflow sense) | `kill_mutant_6_saturating_add_false_overflow` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_7` | `src/fixed.rs` (`const_eq_u32` zero-check sense) | `kill_mutant_7_saturating_div_false_zero` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_8` | `src/fixed.rs` (`log2` zero-domain refusal) | `kill_mutant_8_log2_false_zero` | PASS | none | KILLED_BY_INTENDED_ORACLE |
| `mutant_9` | `src/observatory.rs` (drift gate operand order) | `kill_mutant_9_false_drift` | PASS | `kill_m05_ignore_drift` FAILS | KILLED_BY_INTENDED_ORACLE (COLLATERAL_FAILURES_PRESENT) |
| `mutant_10` | `src/observatory.rs` (`kappa_under_off` operand order) | `kill_mutant_10_false_numerically_uncertain` | PASS | `kill_m01_ignore_numeric_error` FAILS | KILLED_BY_INTENDED_ORACLE (COLLATERAL_FAILURES_PRESENT) |
| `mutant_11` | `src/observatory.rs` (`gamma_under_off` operand order) | `kill_mutant_11_false_gram_degenerate` | PASS | `kill_m03_point_estimate_gram_gate`, `kill_m07_ignore_gram` FAIL | KILLED_BY_INTENDED_ORACLE (COLLATERAL_FAILURES_PRESENT) |

Baseline (no mutant feature), whole binary: all 5 tests in `hostile_mutants.rs`
(`kill_m01_ignore_numeric_error`, `kill_m03_point_estimate_gram_gate`, `kill_m05_ignore_drift`,
`kill_m07_ignore_gram`, `verify_correctness_baselines`) pass — `test result: ok. 5 passed; 0
failed`.

## Hand-written mutants m01/m03/m05/m07 (co-resident in `tests/hostile_mutants.rs`, no feature flag)

These are ordinary (non-`#[cfg(feature=...)]`-gated) test functions that inject their mutation
inline via local helper closures, not compile-time features. They run and pass under every mutant
feature except the one(s) whose production-code mutation they incidentally collide with (see
collateral table above) — they are not separately classified here since they are baseline
oracles, not `mutant_N` targets, but their pass/fail behavior under each feature is the mechanism
of the collateral-breakage finding above.

## Classification counts (final, this pass)

- `KILLED_BY_INTENDED_ORACLE`: **11** (all of `mutant_1` through `mutant_11`; `mutant_9`,
  `mutant_10`, `mutant_11` additionally carry a `COLLATERAL_FAILURES_PRESENT` annotation — their
  own oracle test passes; the whole-binary run shows unrelated collateral breakage in shared
  baseline tests, which is recorded as a non-gating diagnostic and does not change the KILLED
  classification).
- `KILLED_BY_SECONDARY_ORACLE`: 0.
- `MUTATION_GATE_FAILED`: 0.
- `SURVIVED`: 0.
- `INFRASTRUCTURE_BLOCKED`: 0.

This supersedes the prior ledger entry's `MUTATION_GATE_FAILED`-across-the-board finding, which was
tied to a `from_bits`/`val`-private API break in `src/fixed.rs`/`src/allocator.rs`/`src/lrc.rs`
against an earlier working-tree coordinate. That break does not reproduce against the current
tree; `cargo build -p bcinr-cmca` and `cargo test -p bcinr-cmca --test hostile_mutants` (baseline,
no feature) both succeed.

## Makefile.toml `test-mutants` task — isolation mechanism

Per WORKSTREAM D of this pass, `[tasks.test-mutants]` in `/Users/sac/bcinr/Makefile.toml` was
restructured into two passes:

1. **Gating pass**: for each `mutant_N` (1..11), runs
   `cargo test -p bcinr-cmca --features mutant_N --test hostile_mutants <dedicated_oracle_name> -- --exact`
   — i.e. a test-name filter that runs ONLY that mutant's own dedicated oracle test, not the whole
   `hostile_mutants` binary. The task's overall exit code is 0 only if all 11 of these isolated
   runs pass.
2. **Diagnostic pass** (non-gating): for each `mutant_N`, separately runs the full
   `--test hostile_mutants -- --nocapture` (whole binary) and logs its result. Failures here
   (i.e. the `mutant_9`/`10`/`11` collateral breakage documented above) are printed but never
   affect the task's exit code.

Verified run: `cargo make test-mutants` — **exit code 0** (gating pass: 11/11 dedicated oracle
tests passed; diagnostic pass: collateral failures logged for mutants 9/10/11 as expected, task
still exits 0 because the diagnostic pass is explicitly non-gating).

## Note on artifact class

This file is a superseding record of the current tree's mutation-gate standing for `bcinr-cmca`,
scoped to WORKSTREAM D of the v26.7.17 CMCA reconciliation pass. It must be regenerated again if
`src/observatory.rs`, `src/allocator.rs`, `src/fixed.rs`, or `tests/hostile_mutants.rs` change in
ways that affect any of the 11 mutants or their dedicated/baseline oracle tests.
