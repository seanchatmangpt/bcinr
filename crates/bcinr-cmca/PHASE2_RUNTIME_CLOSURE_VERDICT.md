# Phase 2 Runtime Closure Verdict — v26.7.17 CMCA (C1/C2/C3)

**Verifier:** cmca-verifier | **Branch:** `recovery/cmca-v26.7.17-c2` | **Date:** 2026-07-17

Scope: C1 (numeric) and C2/C3 (authority/adaptation-chain) runtime contradictions only.
C4/C6/product-proof/distributed/security closure are explicitly out of scope and do not
block this verdict. No commits made in this run (verification only).

All findings below are commands I ran myself in this session against the current working
tree, not carried forward from the prior REPORTED ledger entries.

## C1 — Numeric Law

| Falsifier | Status | Evidence |
|---|---|---|
| `NumericFaultSet` opaque, union-based | CLOSED | `fixed.rs:13` private tuple struct, `.union()` is `\|`. Test `union_accumulates_both_operands_distinct_faults` passed (`cargo test -p bcinr-cmca --lib`). No public `.val`/`.err` fields remain (private `0`/`val`/`faults` fields; compile-fail UI tests `fail_field_construct_non_negative_fixed.rs`, `fail_tuple_construct_non_negative_fixed.rs`, `fail_struct_update_non_negative_fixed.rs`, and the `signed_fixed`/`numeric_fault_set`/`refusal_set`/`canonical_mask` equivalents all pass under trybuild). |
| `CanonicalMask` sealed, image `{0, u32::MAX}` proven | CLOSED | Test `mask_public_image_is_exactly_zero_or_all_ones` passed. |
| Fixed types sealed | CLOSED | Private fields + trybuild negative-construction tests pass (41/41 `tests/ui/*.rs` cases green, see Compile-fail section). |
| Value-select preserves selected alternative's fault (spot check) | CLOSED (spot-checked, not exhaustive) | Test `select_preserves_selected_fault_and_drops_unselected` passed. I did not independently re-derive all ~10 `from_bits`/select call sites named in the prior REPORTED ledger entry; this is one direct unit test of the `select_faults` primitive itself, not a site-by-site audit of every call site in `allocator.rs`. |
| `AllocationOutcome` total vs Result, documented | CLOSED | `allocator.rs:589` doc comment: "Unlike the legacy `Result`-returning shape, `AllocationOutcome` is always [constructed via `new_internal`]" — it is a total return type (`fn allocate(...) -> AllocationOutcome`, not `-> Result<...>`), and this is documented in the struct's doc comment, satisfying "documented" either way. |
| `INVALID_NORMALIZATION` / `NO_LEAVES` emitted correctly | CLOSED | `allocator.rs:1830-1846` accumulates `NumericFaultSet::INVALID_NORMALIZATION` on `priced_sum == 0`; `allocator.rs:1931` unions `RefusalSet::NO_LEAVES` masked on `nl == 0`, independent of the certified/degraded gate. Read directly, not re-derived by a dedicated new test in this session. |
| Floor shares sum to exactly 65536 across leaf counts | CLOSED, real run | `cargo test -p bcinr-cmca --lib floor_shares_sum_exactly_to_65536_for_every_admitted_leaf_count` → **1 passed**. This is a real property-style test executed in this session, not a claim taken on faith. |
| Byte-level rejection invariance | CLOSED (field-equality, not literal byte-transmute) | `tests/case_studies.rs::test_rejection_invariance` passed (`cargo test -p bcinr-cmca --test case_studies`). It compares the full `weights` array, `last_switch_t`, and `prev_mode` via derived `PartialEq` before/after a forced refusal. This is complete-field equality (no ignored fields, no padding in these plain-old-data types) but is not a literal `unsafe`/byte-transmute comparison — stating this precisely rather than calling it "byte-level" in the stricter sense the prior ledger entry asked about. |

**C1 verdict: CLOSED**, with two precision notes above (select-site spot-check is partial, not exhaustive; rejection invariance is field-equality not byte-transmute) — neither is a violation of a required law, both are scope-of-evidence caveats.

## C2/C3 — Authority / Adaptation Chain

| Falsifier | Status | Evidence |
|---|---|---|
| Full chain Measurement→ModeProposal→AdmittedProposal→ShadowExecutionReceipt→JumpAnalysisReceipt→StabilityCandidate→CertificateReceipt→DwellSatisfied→CertifiedLearning→CertifiedModeSwitch→AtomicSwitch as sealed types | CLOSED | All named files now exist: `proposal.rs`, `shadow.rs`, `jump.rs`, `stability.rs`, `certification.rs`, `mode_switch.rs` (previously ABSENT per the ledger's REPORTED state — that gap is closed on disk now). 63 lib unit tests pass across these modules, including `proposal::tests::admits_when_every_binding_matches`, `shadow::tests::receipt_binds_all_four_identities`, `jump::tests::categories_are_mutually_exclusive_across_the_full_boundary`, `stability::tests::derives_candidate_when_witness_holds`, `mode_switch::tests::applies_when_every_binding_matches`. |
| Observatory proposes but does not certify | CLOSED | `grep -n CertificateReceipt observatory.rs` returns only doc-comment lines asserting the module *never* constructs one (lines 101, 211-212, 241) — no executable construction. `evaluate_calibration` now returns `ObservatoryOutcome`, not `Result<CertificateReceipt, ObservatoryFlag>`. |
| Full flag set preserved with documented priority projection | CLOSED | `observatory::flag_set_tests::full_set_survives_all_four_failure_conditions_simultaneously`, `full_set_survives_when_two_conditions_are_simultaneously_true`, and `primary_flag_priority_order_is_documented_and_tested` all pass. |
| `seal_certificate` verifies witness AND all 11 bindings (spot check ≥3) | CLOSED, exceeds the ask | Read `certification.rs:65-112` directly: checks `witness_holds` plus 11 individually-named field comparisons (`admitted_graph`, `generated_payload`, `kernel_specialization_identity`, `numeric_profile`, `q_registry`, `pricing_law`, `floor_law`, `control_mode`, `influence_state`, `comparison_derivation`, `round_identity`), each with its own typed `CertificationRefusal` variant. All 11 corresponding `refuses_solo_mismatch_*` tests plus `seals_when_witness_holds_and_all_bindings_match` and `refuses_when_witness_margin_insufficient` passed in the lib test run — spot-checked well beyond the requested 3. |
| Dwell is a token, not a bool | CLOSED | `mode_switch::tests::dwell_cannot_be_forged_for_a_different_round_or_transition` passed; `DwellSatisfied` is a sealed struct (trybuild negative tests `fail_field_construct_dwell_satisfied.rs`, `fail_tuple_construct_dwell_satisfied.rs`, `fail_struct_update_dwell_satisfied.rs` all pass), not a bare `bool`. |
| Exactly one production path constructs `AdaptiveUpdate<CertifiedLearning>` | CLOSED | `grep -rn 'fn admit_adaptive_update\|AdaptiveUpdate::admit_adaptive_update\|AdaptiveUpdate {' crates/bcinr-cmca/src/*.rs` shows exactly one function definition (`allocator.rs:950`) and the rest are doc-comment usages of that same function — one production constructor. |
| Rejected switch preserves bytes | CLOSED | `mode_switch::tests::rejection_cause_certificate_mismatch_leaves_state_untouched`, `rejection_cause_dwell_mismatch_leaves_state_untouched`, `rejection_cause_stale_admitted_state_leaves_state_untouched` all pass. |
| No actuation surface reachable | CLOSED (unchanged from prior report) | `grep -rn 'broker\|actuat' crates/bcinr-cmca/src/` returns no hits outside comments — re-confirmed in this session, not merely carried forward. |

**Note on `admit_adaptive_update`'s own binding count:** this function (distinct from `seal_certificate`) checks only 4 digest-equality terms (`state`/`cert`/`env`/`outcome`) plus 2 scalar thresholds. The prior ledger's "~0/11 categories" concern about this specific function is not resolved by this session's evidence — but the 11-category binding enforcement lives in `seal_certificate`, which mints the `CertificateReceipt` whose digest `admit_adaptive_update` then checks for equality. This is architecturally consistent (the certificate's digest already commits the 11 bindings), not a violation, but I did not independently prove the digest committed there necessarily changes if any one of the 11 fields changes (a full "committing hash" proof) — flagging as an evidence gap, not a blocker, since `seal_certificate`'s own tests directly falsify each of the 11 individually.

**C2/C3 verdict: CLOSED**, subject to the one noted evidence gap above (non-blocking).

## Compile-fail (`tests/ui/*.rs`)

Ran `cargo test -p bcinr-cmca --test compile_fail_tests`: **41/41 trybuild cases pass** (all
`ok`), covering tuple-construct, field-construct, and struct-update negative-construction
attempts across every sealed type named in scope. This is finite attack evidence over the
currently-pinned API surface — it demonstrates that these 41 specific illegal-construction
attempts are rejected by the compiler for the stated reason (trybuild diffs against
committed `.stderr` snapshots, which exist and are checked in for all 41 cases; I did not
manually re-read each `.stderr` file to confirm the rejection reason matches intent for
every one of the 41 — I spot-read none in this session beyond trusting trybuild's own
diffing, which is itself the standard mechanism this repo relies on). This is **not** a
universal theorem that no illegal construction is possible anywhere in the sealed API; it
is evidence that these 41 named attack surfaces are closed.

**Correction to the prior REPORTED claim:** the task brief stated "38 of 41 ... have no
committed .stderr baseline yet." That is **not what I found** — all 41 cases have baselines
and all 41 pass in a real `cargo test` run in this session. The prior REPORTED figure is
stale relative to current disk state.

## Consumer correspondence — REAL FAILING TEST (blocker)

`cargo test -p bcinr-cmca --tests` surfaces a real, currently-failing test not mentioned in
the task's prior-phase summary:

```
tests/consumer_correspondence.rs
FAILED: generalization_numeric_payload_matches_old_lawful_output_exactly
panicked at crates/bcinr-cmca/tests/consumer_correspondence.rs:93:5:
generalization from_value_bits(..) sequence order does not match the frozen
pre-migration fixture (multiset-equal: true) — this is a CORRESPONDENCE_REQUIRED
failure even though the underlying values are the same set
```

This is a real, reproducible failure (`cargo test -p bcinr-cmca --tests` exits nonzero) —
an ordering-only regression against the frozen `PRE_MIGRATION_BASELINE.md` correspondence
contract for the `generalization` ontology's generated payload. The values are a
multiset-equal match (same numbers) but sequence order differs from the frozen baseline,
which the test treats as a genuine violation per its own stated contract. This blocks a
full-green `cargo test -p bcinr-cmca --tests` run today.

## Mutation — `MUTANT_KILL_MATRIX`

Ran each of the 11 declared `mutant_N` features individually against
`tests/hostile_mutants.rs` (`cargo test -p bcinr-cmca --features mutant_N --test
hostile_mutants`, real runs, not read-only inspection):

| Mutant | Dedicated `kill_mutant_N_*` test | Classification | Note |
|---|---|---|---|
| 1 | passed | KILLED_BY_INTENDED_ORACLE | differential exact-value oracle against a named-wrong array |
| 2 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 3 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 4 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 5 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 6 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 7 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 8 | passed | KILLED_BY_INTENDED_ORACLE | " |
| 9 | passed | KILLED_BY_INTENDED_ORACLE, but collateral `kill_m05_ignore_drift` FAILED in same run | see below |
| 10 | passed | KILLED_BY_INTENDED_ORACLE, but collateral `kill_m01_ignore_numeric_error` FAILED in same run | see below |
| 11 | passed | KILLED_BY_INTENDED_ORACLE, but collateral `kill_m03_point_estimate_gram_gate` + `kill_m07_ignore_gram` FAILED in same run | see below |

For every mutant 1-11, the mutant's own dedicated named oracle test passes (proving the
mutant is wired and produces the specifically-predicted wrong behavior, per that test's
`assert_eq!`/`assert_ne!` pair). For mutants 9, 10, 11, the *same* `cargo test --features
mutant_N` invocation also fails one or two unrelated baseline tests
(`kill_m01_ignore_numeric_error`, `kill_m03_point_estimate_gram_gate`,
`kill_m05_ignore_drift`, `kill_m07_ignore_gram`) as collateral damage, because those
mutant features change shared code paths the baseline tests also exercise.

**This matters for the gate, not for the individual mutant's kill status.** The
`Makefile.toml` `test-mutants` task runs exactly `cargo test -p bcinr-cmca --features
mutant_N` per feature — the same command I ran. For `N` in `{9, 10, 11}` that command's
**overall exit code is nonzero** (FAILED), even though the intended oracle for that mutant
passed. Per this task's own instruction not to count `MUTATION_GATE_FAILED` as killed: the
gate command itself as currently wired in `Makefile.toml` fails for mutants 9, 10, 11. I am
recording this precisely rather than picking one classification and hiding the tension: the
**mutant is killed** (dedicated oracle assertion passes) but the **gate command as wired
fails** (collateral test breakage in the same binary). This is a wiring gap in
`hostile_mutants.rs`/`Makefile.toml` — the collateral-failing tests are not `#[cfg]`-gated
away from the mutant features that break them — not evidence the mutation itself survived.

No `SURVIVED` mutants were observed among the 11 declared features in this session.

## Allocation

`crates/bcinr-cmca/tests/alloc_gate.rs` exists (cfg-gated behind the `alloc-gate` feature,
not part of the default `--tests` run — this is why the earlier `--tests` invocation showed
"0 tests" for this file). Ran explicitly: `cargo test -p bcinr-cmca --features alloc-gate
--test alloc_gate -- --nocapture` → **1 passed**, asserting `allocs == 0` around a real call
to `allocator::allocate()` with a custom counting `#[global_allocator]`.

Classification: **NO_ALLOC_OBSERVED** (a real allocator-counting test ran clean, in this
session, over one representative call). This is **not** NO_ALLOC_PROVED — no transitive
call-graph/symbol audit (NO_ALLOC_CALL_GRAPH) was performed in this session; G6's
object-code audit tooling gap (no `objdump`/`cargo-asm` pipeline) noted in the prior ledger
was not re-investigated here as it is out of this task's C1/C2/C3 scope.

## Out-of-scope findings (recorded, not fixed)

- `crates/bcinr-logic/src/mask.rs::select_u32` (line 107) — contract-gate finding noted in
  the task brief. Confirmed the function exists at that location. Not modified; recorded as
  OUT OF SCOPE for this CMCA-focused workflow per explicit instruction.
- `crates/bcinr-cmca/tests/fixtures/pre_migration/case_studies.rs` — confirmed this frozen
  fixture is distinct from the live `tests/case_studies.rs` test file (different directory);
  I did not find evidence in this session that any scanner conflated the two. Not modified.

## Overall verdict rollup

- C1 (numeric): CLOSED, real evidence for every named falsifier, two precision notes
  (non-blocking).
- C2/C3 (authority/adaptation chain): CLOSED, real evidence for every named falsifier, one
  evidence gap on `admit_adaptive_update`'s indirect reliance on `seal_certificate`'s digest
  (non-blocking, architecturally consistent).
- Compile-fail: 41/41 blessed and passing (corrects a stale "38/41" figure from the prior
  phase) — finite attack evidence, not a universal theorem.
- Mutation: 11/11 mutants killed by their intended dedicated oracle; 3/11
  (mutants 9-11) additionally fail the literal `Makefile.toml` gate command due to
  collateral test breakage in the same test binary — a wiring gap, not a survived mutant.
- Allocation: NO_ALLOC_OBSERVED (not NO_ALLOC_PROVED).
- **Blocker found in this session that was not in the prior-phase summary:**
  `tests/consumer_correspondence.rs::generalization_numeric_payload_matches_old_lawful_output_exactly`
  is currently FAILING on this branch (ordering regression against the frozen migration
  baseline for the `generalization` ontology payload).

This real, reproducible test failure is why this verdict is BLOCKED rather than ALIVE: the
C1/C2/C3 falsifiers named in this task are individually closed, but the repository's own
`cargo test -p bcinr-cmca --tests` does not exit green today, and that green run is a
prerequisite this task's own falsifier list implicitly assumes ("differential.rs,
case_studies.rs, calibration.rs... need reconciling to the NEW sealed API" — they are now
reconciled to the new API and mostly pass, but one of them still fails on its own merits,
which is a distinct, real defect, not an API-reconciliation defect).

## Numbered blocker list

1. **File:** `crates/bcinr-cmca/tests/consumer_correspondence.rs:93`. **Law:** the
   `generalization` ontology's generated numeric payload must byte/order-match the frozen
   `PRE_MIGRATION_BASELINE.md` correspondence contract. **Evidence missing/failing:** the
   test fails today (`cargo test -p bcinr-cmca --test consumer_correspondence`) — the
   generated `from_value_bits(..)` sequence order for `generalization` differs from the
   frozen fixture even though the value multiset matches. Needs either a generator-ordering
   fix or a documented, re-blessed baseline update (not a hand-edit of the frozen fixture
   file itself, per `PRE_MIGRATION_BASELINE.md`'s own rule).
2. **File:** `Makefile.toml` `[tasks.test-mutants]` / `crates/bcinr-cmca/tests/hostile_mutants.rs`.
   **Law:** `make test-mutants` (equivalently, each `cargo test -p bcinr-cmca --features
   mutant_N` in the Makefile's dependency list) must exit 0 for a mutant to count as gated
   shut. **Evidence failing:** for `N` in `{9, 10, 11}`, that exact command exits nonzero
   today because an unrelated baseline test in the same binary (`kill_m01_ignore_numeric_error`,
   `kill_m03_point_estimate_gram_gate`, `kill_m05_ignore_drift`, `kill_m07_ignore_gram`)
   fails as collateral damage from the mutant feature flag. The mutant's own dedicated
   oracle test does pass — this is a test-wiring gap, not a survived mutant, but it does
   mean the literal CI gate command is red for 3 of 11 mutants today.

CMCA_RUNTIME_CLOSURE_BLOCKED
