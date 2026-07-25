# JTBD Readiness Report — Round 2 — v26.7.17 CMCA (independent rerun)

**Verifier:** independent rerun agent | **Branch:** `recovery/cmca-v26.7.17-c2` | **Date:** 2026-07-17

Scope: independent reproduction of the round-2 conservation-bug fix plus the six new/changed
test files (five new JTBD categories plus the regression test and hostile-mutant baseline
update). Nothing in this pass is trusted from a sibling's self-report except the one item
explicitly named below (Track 5 environment-isolation check) — every other command in this
report was run fresh by this task.

## 1. Conservation bug — root cause and fix

**Bug (round 1):** an 8-way sibling `allocate()` call conserved shares summing to 65532
instead of the required 65536 (`NonNegativeFixed::ONE.value_bits()`), with `RANGE_VIOLATION`
set on every returned share.

**Root cause (per the comment added at `crates/bcinr-cmca/src/allocator.rs`, immediately
after the per-leaf commit-mask assignment loop):** the `nl_recip` term is exactly conserved
via the existing `q_floor`/`r_floor` base+residual scheme, but the price-normalized term
`p_mu` is a second, independent partition of the same unit budget computed via
`saturating_div` (which truncates toward zero) with no remainder redistribution of its own.
The subsequent `eta_actual * nl_recip + (1 - eta_actual) * p_mu` mix truncates a second time
per leaf (`saturating_mul` also floors). The pre-fix `floor_conservation_tests` only checked
the `q_floor`/`r_floor` formula in isolation and never exercised the live `eta < ONE` mixed
path, so the loss went undetected. Any non-degenerate mixing weight (e.g. the real
case-studies registry's `ETA = 0.5`) under-counts the returned shares by the accumulated
per-leaf truncation loss.

**Fix (verified by reading the diff, not trusted from a report):** after the existing mix
loop, the code now sums the actual returned `pi_res` bits over leaves, computes the exact
gap (deficit or surplus) against `NonNegativeFixed::ONE.value_bits()`, and redistributes that
gap across leaves using the same base-`q` + residual-`r` technique already used for
`q_floor`/`r_floor`, keyed off the same canonical `leaf_rank` ordering. It uses
`NonNegativeFixed::from_parts` (not `from_value_bits`) when bumping values up/down so that
any fault bits already accumulated on `pi_res[x]` (e.g. `SATURATION`/`RANGE_VIOLATION`) are
preserved rather than silently erased — this preserves numeric-hot-path.md Invariant 2. The
correction is applied unconditionally (`is_deficit`/`is_excess`/neither), so `sum(pi_res[x]
for is_leaf[x])` is now exactly `ONE.value_bits()` for any `eta`, `mu`, `costs`, or price
distribution.

`tests/hostile_mutants.rs`'s hardcoded baseline/mutant-output constants were updated in the
same diff to the real, unmodified post-fix `allocate()` output (verified: `CORRECT_BASELINE`
now sums to 65536, `CORRECT_MU_COST` now sums to 65536 instead of the pre-fix 32768/65532).

## 2–6, plus 12. Required commands — real pass/fail, rerun independently

| # | Command | Result |
|---|---------|--------|
| 1 | `cargo test -p bcinr-cmca --test jtbd_boundary_adversarial_inputs` | **PASS** 8/8 |
| 2 | `cargo test -p bcinr-cmca --test jtbd_sequential_state_evolution` | **PASS** 2/2 |
| 3 | `cargo test -p bcinr-cmca --test jtbd_artifact_lifecycle` | **PASS** 9/9 |
| 4 | `cargo test -p bcinr-cmca --test jtbd_bounded_under_pathological_input` | **PASS** 1/1 |
| 5 | `cargo test -p bcinr-cmca --test jtbd_conservation_regression` | **PASS** 1/1 |
| 6 | `cargo test -p bcinr-cmca --test jtbd_multi_agent_resource_governance` | **PASS** 3/3 (the two round-1 failing tests — `n_competing_workloads_conserve_the_exact_unit_budget`, `one_malformed_competing_workload_does_not_flip_the_global_refusal_flag` — now pass; a third test, `n_competing_workloads_never_get_a_share_out_of_bounds`, also passes) |
| 8a | `cargo test -p bcinr-cmca` (full default-feature suite) | **PASS** — all 19 test binaries/doc-test groups report `test result: ok`, 0 failed anywhere |
| 8b | `cargo test -p bcinr-cmca --all-features` | **1 genuine failure** — see finding below; every other binary passes |
| 8c | `cargo make test-mutants` | **PASS** — gate output: "GATE PASSED — all 11 mutants' dedicated oracle tests gave the expected result" |
| 8d | `cargo test -p bcinr-cmca --test compile_fail_tests` | **PASS** 1/1 (9 nested UI compile-fail cases inside it, all `ok`) |
| 8e | `cargo test -p bcinr-cmca --test jtbd_safety_certified_adaptive_control` | **PASS** 2/2 |
| 8f | `cargo test -p bcinr-cmca --test jtbd_auditable_adaptive_policy` (default features) | **PASS** 1/1 |
| 8g | `cargo test -p bcinr-cmca --test jtbd_semantic_mechanical_compilation` | **PASS** 10/10 |

### Finding: `--all-features` failure is a feature-flag artifact, not a round-2 regression

Under `cargo test -p bcinr-cmca --all-features`, `jtbd_auditable_adaptive_policy`'s
`independent_reader_disambiguates_which_of_two_candidates_was_certified` fails:

```
thread '...' panicked at crates/bcinr-cmca/tests/jtbd_auditable_adaptive_policy.rs:144:5:
fixture must clear the Observatory as RecertificationCandidate; got ObservatoryFlagSet(7)
```

Cause, confirmed by reading `crates/bcinr-cmca/Cargo.toml`'s `[features]` table: `mutant_1`
through `mutant_11` are eleven independent, default-off features, each of which corrupts one
specific step inside `allocate()` — designed and documented (see `hostile_mutants.rs`) to be
enabled **one at a time** for mutation testing. `--all-features` enables all eleven
simultaneously, which is not a supported or meaningful build configuration for this crate:
the combined corruption cascades into every consumer of `allocate()`, including the
certification/mode-switch path this test exercises, producing a different (still
internally-consistent, but different) `ObservatoryFlagSet` than the test's fixture assumes.
This same test passes cleanly (confirmed above, item 8f) under default features. This is a
pre-existing property of the mutant-feature-flag design (not introduced by this round's fix
or by the new test files) — `--all-features` was never a valid way to exercise this crate's
production path, and this is the first time in this round's audit that `--all-features` was
run against it. Flagging honestly per instructions rather than silently treating it as
equivalent to a real regression: it is a scope/tooling mismatch (running mutation-only
feature combinations together), not a defect in the conservation fix or in any new test.

## 7. Packaging/environment-reality checks (Track 5)

No packaging or environment-isolation test file exists anywhere in the working tree —
`git status --short` and a repo-wide search for packaging/environment-related filenames
under `crates/bcinr-cmca/` turned up nothing beyond the six jtbd test files and the report
files already listed. There is no "packaged-crate test" or "environment-isolation test" to
rerun or to trust from a sibling for this task's working tree at the time of this audit.
**This item could not be executed or verified — it is UNVERIFIED, not PASS, and not
"trusting the sibling's report,"** because no artifact for it was found to trust or rerun.
If a sibling task produced such a file in a different worktree or has not yet merged it here,
that is outside what this independent rerun can confirm.

## 9. Mock/stub audit — all six new/changed test files

Spot-read and grepped (`mock|stub|fake`, case-insensitive) all six files: the five new-round-2
test files plus `jtbd_multi_agent_resource_governance.rs` (the regression target confirmed
above) and cross-checked `jtbd_conservation_regression.rs` and `jtbd_bounded_under_pathological_input.rs`
as well.

**Finding: no mock or stub of a CMCA production type was found in any of the six files.**
Every match for `mock`/`stub`/`fake` is inside a doc comment explicitly disclaiming their use
(e.g. `jtbd_multi_agent_resource_governance.rs:7`: "No mock or stub of any CMCA internal is
used anywhere in this file."; `jtbd_boundary_adversarial_inputs.rs:1,4`;
`jtbd_sequential_state_evolution.rs:63`; `jtbd_artifact_lifecycle.rs:5`;
`jtbd_bounded_under_pathological_input.rs:12`; `jtbd_conservation_regression.rs:20`). Zero
matches occur inside executable code. Every file's `use` statements import real
`bcinr_cmca::{allocator, certification, fixed, generated, jump, stability}` production types
and the real `chicago_tdd_tools::test` harness, and (for `jtbd_artifact_lifecycle.rs`) the
real `artifact_under_test` module plus `std::process::Command` for a real subprocess-level
packaging/schema check. This satisfies the "real Chicago TDD, no mocks/stubs of CMCA
internals" requirement for all six files.

## 10. Git-safety confirmation

- **Branch:** `git branch --show-current` → `recovery/cmca-v26.7.17-c2`, unchanged throughout
  this task.
- **Commits:** `git log -3` shows the three most recent commits
  (`7e7e7cd5`, `ba74772b`, `7e91f37b`) all timestamped `2026-07-17 19:2x` — these are the
  separate release-integration workflow's own prior commits (version bump + CHANGELOG/ledger),
  not made by this task. This task made **zero** commits.
- **Cargo.toml version fields:** `git diff HEAD -- '**/Cargo.toml' | grep -i '^+version\|^-version'`
  → no output. The only uncommitted `Cargo.toml` change in the working tree is the
  `chicago-tdd-tools` dev-dependency addition (pre-existing, per the round-1 report, and
  unmodified further by this task) — no version-field edits.
- **CHANGELOG.md:** not present in `git status --short` output — untouched.

## 11. Overall production-readiness posture (all 10 JTBDs, rounds 1 + 2)

This is a scoped assessment, not a blanket claim. Round 1 covered five speculative JTBD
scenarios; four passed with narrow, real, falsifiable evidence, and one — multi-agent
resource governance conservation — was genuinely falsified. Round 2's fix directly targets
that falsification: the root cause was a real double-truncation defect in `allocate()`'s
price-mix term, and the fix (an unconditional, fault-preserving remainder-redistribution
pass keyed to the actual observed gap) makes the originally-failing 8-way scenario, and the
newly-added `jtbd_conservation_regression` and `jtbd_multi_agent_resource_governance` tests
(including a not-out-of-bounds check not present in round 1), pass under real production
code with no mocks. The five new round-2 categories (boundary/adversarial inputs, sequential
state evolution, artifact lifecycle, DoS-shape/bounded pathological input, and the
conservation regression itself) each pass for the specific scenarios their tests construct,
using real collaborators throughout, confirmed by an independent rerun of every command in
this report except Track 5's environment-isolation check (which could not be verified either
way because no such artifact exists in this working tree to run or to trust). The one new
negative finding this round is scoped and non-fatal: `--all-features` (all eleven
mutation-testing features enabled at once, a combination the crate's own feature design never
intends to be exercised together) breaks one certification-disambiguation test; the default
build and every individually-scoped feature configuration this round exercised pass cleanly,
and `cargo make test-mutants`'s one-at-a-time gate — the actually-intended way to exercise
those features — reports all 11 mutants killed. None of this should be read as "production
ready" across the full space of either round's speculative JTBD framings: each test validates
one narrow, explicitly-scoped property under the specific scenario(s) it constructs, the
Track 5 packaging/environment-reality claim from this round's task description has no
artifact in this working tree to substantiate, and no test in either round exercises
concurrent/adversarial access, arbitrary N beyond the tested values, or external/regulatory
"safety-certified"/"auditable" standards.
