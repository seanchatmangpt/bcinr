# Reconciliation Verification — bcinr-cmca v26.7.17 CMCA (cmca-verifier, independent rerun)

**Verifier:** cmca-verifier | **Branch:** `recovery/cmca-v26.7.17-c2` | **Date:** 2026-07-17

Scope: independently rerun every claim from the four sibling reconciliation workstreams
(package boundary, order correspondence, artifact smoke test, mutant isolation). No commits
made. No files modified. All commands below were executed directly in this session; none of
the sibling workstreams' self-reports were taken on faith.

---

## Pure-consumer checks

### 1. `cargo package -p bcinr-cmca --list` excludes quarantine/, includes generated-artifact/ + src/**

Ran `cargo package -p bcinr-cmca --list --allow-dirty` (working tree has 120 uncommitted
files per `git status`; `--allow-dirty` needed purely to run the list command, same caveat
the prior-phase verdict recorded).

- `grep -i quarantine` on the file list: **zero hits.** `Cargo.toml` now has
  `exclude = ["quarantine/**"]` (line 14). **PASS** — this fixes the Phase-1 finding that
  quarantine/ (including the legacy Python generator and `.ttl` ontology files) previously
  shipped inside the published tarball.
- `generated-artifact/{case-studies,generalization}/*` (manifest/receipt/generated.rs):
  **present.** `src/*.rs` (including `src/artifact.rs`, `src/lib.rs`, all sealed-type
  modules): **present.** **PASS.**
- **New finding, not one of the four named blockers:** the package list also includes
  `src/allocator.rs.orig` (53,652 bytes), a stray backup file left in the working tree. It is
  not excluded by the `quarantine/**` glob and ships inside the crate tarball. This is a
  packaging hygiene defect (an accidental `.orig` artifact reaching a published crate), not a
  boundary-architecture violation — recorded here as a finding, not fixed (no source edits
  in this verification pass).

### 2. `cargo package -p bcinr-cmca --locked` succeeds

Ran `cargo package -p bcinr-cmca --locked --allow-dirty`:

```
Packaging bcinr-cmca v26.6.24 (/Users/sac/bcinr/crates/bcinr-cmca)
    Updating crates.io index
    Packaged 134 files, 802.9KiB (139.7KiB compressed)
   Verifying bcinr-cmca v26.6.24 (/Users/sac/bcinr/crates/bcinr-cmca)
   Compiling bcinr-cmca v26.6.24 (/Users/sac/bcinr/target/package/bcinr-cmca-26.6.24)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.07s
```

**PASS.** Packages and verifies from packaged contents alone.

### 3. `cargo test -p bcinr-cmca --test consumer_correspondence` passes

```
running 3 tests
test defective_paths_not_exercised_by_current_fixtures_is_a_nonclaim ... ok
test case_studies_numeric_payload_matches_old_lawful_output_exactly ... ok
test generalization_numeric_payload_matches_old_lawful_output_exactly ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**PASS.** This reverses the Phase-1/Phase-2 blocker: the `generalization`
`from_value_bits(..)` ordering fix actually works, independently reproduced.

### 4. Real-artifact test in `src/artifact.rs` is no longer `#[ignore]`d, and passes

`cargo test -p bcinr-cmca --lib artifact`:

```
running 9 tests
test artifact::tests::malformed_digest_string_refused ... ok
test artifact::tests::duplicate_registry_index_refused ... ok
test artifact::tests::floor_table_not_conserved_refused ... ok
test artifact::tests::payload_digest_mismatch_refused ... ok
test artifact::tests::out_of_bounds_registry_index_refused ... ok
test artifact::tests::unknown_schema_version_refused ... ok
test artifact::tests::valid_profile_accepted ... ok
test artifact::tests::wrong_dimensions_refused ... ok
test artifact::tests::smoke_test_against_real_mfw_artifact ... FAILED

thread 'artifact::tests::smoke_test_against_real_mfw_artifact' panicked at
crates/bcinr-cmca/src/artifact.rs:411:51:
real manifest must parse: Error("missing field `leaf_count`", line: 1, column: 944)

test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 55 filtered out
```

**Half-fixed, half-BLOCKED.** The `#[ignore]` attribute was correctly removed (the stated
Phase-1 finding is fixed — the test is no longer skipped). But un-ignoring it surfaced a
**real, currently-failing assertion**: the committed
`crates/bcinr-cmca/generated-artifact/{case-studies,generalization}/cmca_generation_manifest.json`
does not deserialize into whatever manifest struct `artifact.rs` expects — it is missing a
`leaf_count` field the parser requires. This is not a stale-ignore issue anymore; it is a
genuine schema/fixture mismatch between the committed generated-artifact manifests and the
consumer's manifest struct. **This is a blocker**, not a caveat.

### 5. `cargo tree -p bcinr-cmca` shows zero RDF/Python/mfw/oxigraph/praxis-graphlaw dependency

`cargo tree -p bcinr-cmca | grep -iE "mfw|oxigraph|praxis|rdf|python"` → **zero hits.**
**PASS.**

**Pure-consumer rollup:** 4 of 5 checks pass cleanly (quarantine exclusion, locked package,
consumer-correspondence ordering fix, dependency tree). One check (real-artifact smoke test)
is un-ignored as intended but now fails on a genuine data/schema mismatch — this is the
"artifact smoke test" workstream's claim not holding up under independent rerun.

---

## Runtime-closure checks

### 6. `cargo test -p bcinr-cmca` (full) and `--all-features`

Full run: **63 passed, 1 failed** — the same `smoke_test_against_real_mfw_artifact` failure
from check 4 above (identical panic, identical location).

`--all-features` run: **61 passed, 3 failed** — the same artifact-smoke failure, **plus two
additional failures that only appear under `--all-features`** (i.e. with one or more
`mutant_N` feature simultaneously active alongside the default build):

```
fixed::tests::saturating_add_overflow_reports_overflow_and_saturation
  assertion `left == right` failed
  left: 0
  right: 65

fixed::tests::saturating_div_by_zero_reports_divide_by_zero_and_invalid_domain
  attempt to shift left with overflow
```

**Both checks FAIL.** `--all-features` is materially worse than the default profile: it
activates `mutant_6`/`mutant_7`-style code paths (the mutant features mutate
`saturating_add`/`const_eq_u32` in `src/fixed.rs`, per `MUTANT_KILL_MATRIX.md`) simultaneously
with the ordinary baseline tests, corrupting `fixed.rs` baseline test assertions that assume
no mutant feature is active. This is a real, reproducible defect: the crate's `--all-features`
build is not lawful (baseline fixed-point tests fail when mutant features are compiled in
together), even though each mutant is individually gated correctly one at a time.

### 7. 41/41 compile-fail cases

`cargo test -p bcinr-cmca --test compile_fail_tests`: **41/41 trybuild cases `ok`.** **PASS.**
Matches the prior-phase verdict's corrected figure.

### 8. `cargo make test-mutants` exits 0; spot-check mutants 9, 10, 11

`cargo make test-mutants` output tail: `test-mutants: GATE PASSED -- all 11 mutants' dedicated
oracle tests gave the expected result`. Exit code 0. **PASS** on the gate as currently wired
(the Makefile.toml task was restructured, per its own in-file changelog comment, to gate only
on each mutant's own dedicated oracle test by exact name, with whole-binary collateral
failures demoted to a non-gating diagnostic pass in the same task run).

Spot-checked mutants 9, 10, 11 by rerunning their dedicated oracle commands directly (not
trusting `MUTANT_KILL_MATRIX.md`'s table):

```
cargo test -p bcinr-cmca --features mutant_9  --test hostile_mutants kill_mutant_9_false_drift -- --exact
  → test kill_mutant_9_false_drift ... ok  (1 passed)
cargo test -p bcinr-cmca --features mutant_10 --test hostile_mutants kill_mutant_10_false_numerically_uncertain -- --exact
  → test kill_mutant_10_false_numerically_uncertain ... ok  (1 passed)
cargo test -p bcinr-cmca --features mutant_11 --test hostile_mutants kill_mutant_11_false_gram_degenerate -- --exact
  → test kill_mutant_11_false_gram_degenerate ... ok  (1 passed)
```

Also reran the whole-binary diagnostic for mutant_11 to confirm the documented collateral
failure still reproduces exactly as `MUTANT_KILL_MATRIX.md` claims: `kill_m03_point_estimate_gram_gate`
and `kill_m07_ignore_gram` both FAILED in that run, consistent with the ledger. **PASS** — the
isolation fix for mutants 9/10/11 is real and independently reproduced; the ledger's
characterization of collateral-vs-intended-oracle failure is accurate.

### 9. No production authority reopened / no fault propagation weakened / no chain stage bypassed

Reran a sample of the C1/C2/C3 tests named in `PHASE2_RUNTIME_CLOSURE_VERDICT.md` directly
(not re-trusting the prior verdict's pass/fail claims):

```
cargo test -p bcinr-cmca --lib -- union_accumulates_both_operands_distinct_faults \
  mask_public_image_is_exactly_zero_or_all_ones \
  select_preserves_selected_fault_and_drops_unselected \
  floor_shares_sum_exactly_to_65536
  → 4 passed; 0 failed

cargo test -p bcinr-cmca --test case_studies test_rejection_invariance
  → 1 passed; 0 failed
```

**PASS** on this spot check — the sampled C1 numeric-law and rejection-invariance tests still
hold. (Note: this is a spot check of 5 named tests, not an exhaustive re-audit of every
falsifier in the C1/C2/C3 table; the fuller `cargo test -p bcinr-cmca` run in check 6 above
already confirms all `proposal`/`shadow`/`jump`/`stability`/`certification`/`mode_switch`
module tests pass in the default-feature build.)

### 10. `cargo make verify-generated`

```
MISMATCH: crates/bcinr-cmca/generated-artifact/case-studies generated_payload_digest
  declared=blake3:308b5a92b83a91355150ebde5541215a9996f6d124dcd136c19266995a06e4ce
  recomputed=blake3:437c7ad4349021a714040ab531cb4464e774423eaac2f4a1dc8936c65f5d83d1
MISMATCH: crates/bcinr-cmca/generated-artifact/generalization generated_payload_digest
  declared=blake3:6d443ce9c27072901a54fea3b1de160be002fb650ce3c30d6a95d44f4b4f23af
  recomputed=blake3:51fdd20b41d1892c3f060906ddd100a78be2df4a740e2e9c61764d862d2bb451
verify-generated: FAIL - artifact digest/schema check failed
```

**FAILS. This is a blocker.** The committed `generated-artifact/{case-studies,generalization}`
payload digests do not match the recomputed digest of the checked-in generated Rust source.
Given check 4's finding (the manifest is also missing `leaf_count`, causing the smoke test to
fail), this strongly suggests the `generated-artifact/` tree currently checked in is stale or
was regenerated inconsistently relative to either its own manifest schema or its own declared
digest — the artifact-boundary contract's own self-check does not hold today.

### 11. `cargo check --workspace` and `cargo test --workspace`

`cargo check --workspace`: succeeds cleanly (warnings only: one dead-code warning in
`bcinr-powl`, one future-incompatibility notice for the `block` crate; no errors). **PASS.**

`cargo test --workspace --no-fail-fast`: **two test-binary failures**, both real:

1. `bcinr-cmca --lib` (artifact module): 63 passed / 1 failed — the same
   `smoke_test_against_real_mfw_artifact` failure as check 4/6.
2. `bcinr --test e2e_main`: 5 passed / 55 failed. This failure is **outside bcinr-cmca
   entirely** (top-level `bcinr` crate's tier2 e2e harness — clippy/fmt/contract-gate/bench
   scenario tests). I did not investigate root cause since it is out of this task's declared
   scope (bcinr-cmca C1/C2/C3/package-boundary reconciliation), but it means the literal
   command `cargo test --workspace` does **not** exit green today, independent of anything
   this reconciliation pass touched. Flagging it because the task explicitly asked to
   "confirm nothing else broke" — recording rather than silently omitting.

All other workspace test binaries (bcinr-logic, bcinr-pddl, bcinr-powl, bcinr-mcp, etc.)
passed cleanly in the same run.

---

## Explicit non-promotions (unchanged from prior phase verdicts, not touched in this pass)

- `NO_ALLOC_OBSERVED` is not promoted to `NO_ALLOC_PROVED` — not re-investigated here.
- 41/41 compile-fail is finite attack evidence, not a universal theorem — unchanged framing.
- Source-shape evidence is not promoted to object-code branchlessness claims — not
  re-investigated here.

---

## Summary of blockers found by independent rerun

1. **Artifact smoke test genuinely fails, not just previously-ignored.**
   File: `crates/bcinr-cmca/src/artifact.rs:411`. Falsifier: `smoke_test_against_real_mfw_artifact`
   must parse the real committed manifest and pass. Evidence: `cargo test -p bcinr-cmca --lib artifact`
   → panics with `Error("missing field \`leaf_count\`", line: 1, column: 944)` when parsing
   `crates/bcinr-cmca/generated-artifact/{case-studies,generalization}/cmca_generation_manifest.json`.
   Owner: whoever owns the `generated-artifact/` producer-side manifest schema and/or the
   consumer's manifest struct in `artifact.rs`. Required repair: reconcile the manifest JSON
   schema and the consumer parser — either the manifest needs a `leaf_count` field or the
   parser's expected struct needs to stop requiring one, decided by which side of the
   producer/consumer contract is authoritative per `CMCA_ARTIFACT_CONTRACT.md`.

2. **`cargo make verify-generated` fails — committed artifact digests do not match recomputed digests.**
   Files: `crates/bcinr-cmca/generated-artifact/case-studies/*`,
   `crates/bcinr-cmca/generated-artifact/generalization/*`. Falsifier: the committed
   `generated_payload_digest` in each manifest must equal the BLAKE3 digest of the committed
   generated Rust source. Evidence: `cargo make verify-generated` reports both digests
   mismatched (declared vs. recomputed, both ontologies). Owner: owner of the
   `generated-artifact/` producer pipeline / whoever last regenerated or hand-edited these
   files. Required repair: regenerate `generated-artifact/` consistently (source, manifest,
   and digest all from the same producer run) or, if the checked-in tree is intentionally
   ahead of its manifest, rebless the manifest's digest field through the producer, not by
   hand-editing the tarball.

3. **`cargo test -p bcinr-cmca --all-features` fails on baseline `fixed.rs` tests, not just the artifact smoke test.**
   File: `crates/bcinr-cmca/src/fixed.rs` (tests `saturating_add_overflow_reports_overflow_and_saturation`,
   `saturating_div_by_zero_reports_divide_by_zero_and_invalid_domain`). Falsifier: `--all-features`
   must pass in full per this task's checklist. Evidence: both tests fail only under
   `--all-features` (multiple `mutant_N` features compiled in simultaneously corrupt shared
   `fixed.rs` code paths that baseline tests assume are unmutated). Owner: owner of
   `tests/hostile_mutants.rs` / the `mutant_N` feature-gating scheme in `Cargo.toml`. Required
   repair: either make the `mutant_N` features mutually exclusive at compile time (a
   `compile_error!` if more than one is active) or make baseline `fixed.rs` tests
   `cfg`-conditional on no `mutant_*` feature being active, mirroring the isolation already
   done for `hostile_mutants.rs` itself.

4. **`cargo test --workspace` does not exit green (pre-existing, outside bcinr-cmca).**
   File: `crates/bcinr/tests/e2e_main.rs` (`e2e::tier2::*`, 55 failing). Not investigated
   (out of this task's scope), but recorded per the task's explicit "confirm nothing else
   broke" instruction. This failure is unrelated to any of the four CMCA reconciliation
   workstreams and was not touched or caused by this verification pass.

## Minor packaging finding (non-blocking)

- `src/allocator.rs.orig` (a stray backup file) ships inside the `cargo package` tarball;
  not excluded by the `quarantine/**` glob. Cosmetic/hygiene issue, not an authority or
  boundary violation.

---

## Verdict lines

BCINR_CMCA_PURE_CONSUMER_BLOCKED
1. failed gate: real-artifact smoke test (`cargo test -p bcinr-cmca --lib artifact`) — falsifier: `smoke_test_against_real_mfw_artifact` — owner: generated-artifact manifest schema / artifact.rs parser owner — command+evidence: `cargo test -p bcinr-cmca --lib artifact` → panics `Error("missing field \`leaf_count\`", line: 1, column: 944)` — required repair: reconcile manifest JSON schema with consumer struct (add `leaf_count` to manifest or relax parser), per CMCA_ARTIFACT_CONTRACT.md.
(quarantine exclusion, locked package, consumer_correspondence ordering, and dependency-tree cleanliness all independently PASS.)

CMCA_RUNTIME_CLOSURE_BLOCKED
1. failed gate: `cargo make verify-generated` — falsifier: committed `generated_payload_digest` must equal recomputed BLAKE3 digest of committed generated source — owner: generated-artifact producer-pipeline owner — command+evidence: `cargo make verify-generated` → MISMATCH on both `case-studies` and `generalization` (declared vs recomputed digests differ) — required repair: regenerate `generated-artifact/` consistently from one producer run, or rebless manifest digest through the producer.
2. failed gate: `cargo test -p bcinr-cmca --all-features` — falsifier: full test suite must pass with all features enabled — owner: `mutant_N` feature-gating scheme owner (Cargo.toml / hostile_mutants.rs) — command+evidence: `cargo test -p bcinr-cmca --all-features` → `fixed::tests::saturating_add_overflow_reports_overflow_and_saturation` and `fixed::tests::saturating_div_by_zero_reports_divide_by_zero_and_invalid_domain` FAILED (multiple mutant_N features compiled simultaneously corrupt shared fixed.rs code) — required repair: make mutant_N features mutually exclusive at compile time or cfg-gate baseline fixed.rs tests against any mutant_* feature being active.
3. failed gate: real-artifact smoke test (shared with pure-consumer blocker #1 above) — same evidence, same required repair — also blocks a full-green `cargo test -p bcinr-cmca`.
4. non-blocking-for-CMCA but explicitly checked: `cargo test --workspace` — falsifier: nothing else in the workspace should be broken — command+evidence: `cargo test --workspace --no-fail-fast` → `bcinr --test e2e_main` 55/60 FAILED, unrelated to any of the four CMCA workstreams, pre-existing and out of this task's scope, recorded per explicit instruction not to omit it.
(41/41 compile-fail, `cargo make test-mutants` gate-as-wired, mutants 9/10/11 dedicated-oracle spot checks, C1/C2/C3 sampled tests, and `cargo check --workspace` all independently PASS.)
