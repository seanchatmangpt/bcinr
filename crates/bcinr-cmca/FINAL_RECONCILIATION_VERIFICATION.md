# Final Reconciliation Verification — bcinr-cmca v26.7.17 CMCA (cmca-verifier, second independent rerun)

**Verifier:** cmca-verifier | **Branch:** `recovery/cmca-v26.7.17-c2` | **Date:** 2026-07-17

Scope: independently rerun the full checklist from this task's instructions against the
working tree as it stands right now, without trusting `RECONCILIATION_VERIFICATION.md`'s
prior findings on faith. No git branch changed, no files modified, no commits made, no
`#[allow]` added, no tests deleted.

Headline finding: **the working tree has moved since `RECONCILIATION_VERIFICATION.md` was
written.** Two of that document's three named blockers (artifact-smoke-test schema mismatch,
`verify-generated` digest mismatch) no longer reproduce — both now pass cleanly on rerun. But
the third blocker's *symptom* has changed shape: `cargo test -p bcinr-cmca --all-features` no
longer fails on `fixed.rs` baseline test assertions — it now fails to **compile at all**,
for an unrelated, pre-existing, self-documented reason (`generated_artifact_pending` feature,
default-off, known-broken, gated by design). That prevents `--all-features` from being
attempted as a real test-execution question in the first place.

---

## Checklist results (task instructions items 1-8)

### 1. `cargo test -p bcinr-cmca` (default features) — full pass required

```
$ cargo test -p bcinr-cmca
running 62 tests ... test result: ok. 62 passed; 0 failed
running 0 tests  ... test result: ok. 0 passed; 0 failed        (generated_artifact tests, feature-gated off)
running 6 tests  ... test result: ok. 6 passed; 0 failed
running 7 tests  ... test result: ok. 7 passed; 0 failed        (artifact module)
running 1 test   ... test result: ok. 1 passed; 0 failed
running 3 tests  ... test result: ok. 3 passed; 0 failed        (consumer_correspondence)
running 1 test   ... test result: ok. 1 passed; 0 failed
running 5 tests  ... test result: ok. 5 passed; 0 failed
running 0 tests  ... test result: ok. 0 passed; 0 failed
running 15 tests ... test result: ok. 15 passed; 0 failed       (doc-tests)
```

**PASS.** Every default-feature test binary is green, including the artifact module (see
item 5) and `consumer_correspondence` (the case-studies/generalization ordering fix). This
reverses `RECONCILIATION_VERIFICATION.md` check 6's finding of `63 passed, 1 failed` — the
`smoke_test_against_real_mfw_artifact` failure it recorded does not reproduce now.

### 2. `cargo test -p bcinr-cmca --all-features` — full pass required

```
$ cargo test -p bcinr-cmca --all-features
error[E0599]: no associated function or constant named `from_bits` found for struct
  `fixed::NonNegativeFixed` in the current scope   (213 occurrences)
error[E0599]: no associated function or constant named `from_bits` found for struct
  `SignedFixed` / `fixed::SignedFixed` in the current scope   (18 occurrences)
error: could not compile `bcinr-cmca` (lib) due to 222 previous errors
error: could not compile `bcinr-cmca` (lib test) due to 222 previous errors
```

**FAIL — does not even reach the test-execution stage.** This is a **different failure**
from the one `RECONCILIATION_VERIFICATION.md` check 6 recorded (`fixed::tests::saturating_add_*`
/ `saturating_div_*` assertion failures from multiple `mutant_N` features being compiled
together). That prior failure mode does not reproduce here — the build never gets that far.

Root cause, read directly from `Cargo.toml` and `src/lib.rs`: `--all-features` turns on
`generated_artifact_pending`, a feature `Cargo.toml` documents as **default-off and known
broken by design**:

> "Default-off: gates `src/generated_artifact/` (mfw-producer-sourced Gamma_CMCA modules).
> Compiling that module unconditionally currently fails (611 errors: the producer's
> `cmca_generated.rs` calls fixed-point constructors/fields this crate's `src/fixed.rs` API
> does not expose under those names). ... Not enabled by any other feature; flip on only
> once `src/fixed.rs`'s owning task reconciles the API surface."

`src/generated_artifact/mod.rs`'s module doc corroborates: it documents a real API mismatch
between the mfw producer's generated `cmca_generated.rs` (calls `SignedFixed::from_bits` /
`NonNegativeFixed::from_bits`) and the actual `fixed.rs` API (`from_value_bits`, `from_parts`,
`from_num` — no `from_bits`). This is a pre-existing, explicitly acknowledged gap, not
something introduced or broken by this verification pass, and not one of the fixes this task
was scoped to re-verify (the RECONCILIATION_VERIFICATION.md blocker list — manifest schema,
digest regeneration, feature exclusivity — never mentions `generated_artifact_pending` or
`from_bits`).

**Still a blocker against this task's literal checklist item 2**, whatever its provenance:
`--all-features` does not pass.

### 3. `cargo make verify-generated` — must pass (digest match)

```
$ cargo make verify-generated
verify-generated: PASS - committed Gamma_CMCA artifact digests and schema_version verified
  (no generator invoked)
```

**PASS.** Reverses `RECONCILIATION_VERIFICATION.md` check 10's digest-mismatch finding on
both `case-studies` and `generalization` — both now verify cleanly against the committed
`generated-artifact/` tree.

### 4. `cargo make test-mutants` — must exit 0

```
$ cargo make test-mutants
mutant_11: whole-binary run reported failures above (collateral or otherwise) -- NOT gating,
  see MUTANT_KILL_MATRIX.md
test-mutants: GATE PASSED -- all 11 mutants' dedicated oracle tests gave the expected result
$ echo $?
0
```

**PASS.** Gate exits 0 as wired (collateral whole-binary failures for mutant_11 —
`kill_m03_point_estimate_gram_gate`, `kill_m07_ignore_gram` — are demoted to non-gating
diagnostics, matching `MUTANT_KILL_MATRIX.md`'s documented characterization; reproduced
directly, not taken on the Makefile's word).

### 5. `cargo test -p bcinr-cmca --lib artifact -- --include-ignored` — real-artifact smoke test must pass

```
running 7 tests
test artifact::tests::malformed_digest_string_refused ... ok
test artifact::tests::payload_digest_mismatch_refused ... ok
test artifact::tests::floor_table_not_conserved_refused ... ok
test artifact::tests::unknown_schema_version_refused ... ok
test artifact::tests::valid_profile_accepted ... ok
test artifact::tests::wrong_dimensions_refused ... ok
test artifact::tests::smoke_test_against_real_mfw_artifact ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out
```

**PASS.** No test is `#[ignore]`d (`--include-ignored` filtered 0 additional tests in), and
`smoke_test_against_real_mfw_artifact` passes directly against the real committed
`generated-artifact/case-studies/{cmca_generation_manifest.json,cmca_generated.rs}`. This
reverses `RECONCILIATION_VERIFICATION.md` check 4's finding
(`Error("missing field \`leaf_count\`", ...)`) — inspecting `src/artifact.rs`'s current
`GeneratedManifest` struct and the current manifest JSON on disk, neither mentions
`leaf_count` at all; the struct's own doc comment (lines 114-129) explicitly disclaims that
field as belonging to "an earlier, wrong, flat-shaped consumer assumption." The struct
matches the real manifest's actual key set today
(`schema_version`, `digests`, `dimensions`, `generator_source_order`, `numeric_profile`).

### 6. `cargo test -p bcinr-cmca --test compile_fail_tests` — 41/41 still pass

```
$ ls crates/bcinr-cmca/tests/ui/*.rs | wc -l
41
$ cargo test -p bcinr-cmca --test compile_fail_tests
test compile_fail_tests ... ok
$ (count of "tests/ui/" lines in trybuild output) = 41
```

**PASS.** 41/41, matching the task's stated figure.

### 7. `cargo package -p bcinr-cmca --list` / `--locked` (packaging)

```
$ cargo package -p bcinr-cmca --list --allow-dirty | grep -i quarantine
(no output — zero hits)
$ cargo package -p bcinr-cmca --locked --allow-dirty
    Packaging bcinr-cmca v26.6.24 (...)
    Packaged 135 files, 825.2KiB (146.7KiB compressed)
    Verifying bcinr-cmca v26.6.24 (...)
    Compiling bcinr-cmca v26.6.24 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.41s
```

**PASS** on both. `--allow-dirty` used per this task's own instructions (no commits made in
this pass).

**Reconfirmed, not fixed (out of scope — no source edits made this pass):** `src/allocator.rs.orig`
still appears in `cargo package -p bcinr-cmca --list` output, same minor packaging-hygiene
finding `RECONCILIATION_VERIFICATION.md` recorded and did not fix. Still not excluded by the
`quarantine/**` glob. Not one of the three named blockers; not touched.

### 8. `cargo check --workspace` — still passes

```
$ cargo check --workspace
    Checking bcinr-cmca v26.6.24 (...)
    Checking bcinr-cmca-audit-harness v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.76s
```

Compiles cleanly (only pre-existing warnings: one dead-code warning in `bcinr-powl`, one
`block` crate future-incompatibility notice — both unrelated to bcinr-cmca). **PASS.**

---

## Spot checks (task instruction: "at least 3 individual mutant_N dedicated oracle tests
and one C1 and one C2/C3 test named in PHASE2_RUNTIME_CLOSURE_VERDICT.md")

```
$ cargo test -p bcinr-cmca --features mutant_9  --test hostile_mutants kill_mutant_9_false_drift -- --exact
  → ok (1 passed)
$ cargo test -p bcinr-cmca --features mutant_10 --test hostile_mutants kill_mutant_10_false_numerically_uncertain -- --exact
  → ok (1 passed)
$ cargo test -p bcinr-cmca --features mutant_11 --test hostile_mutants kill_mutant_11_false_gram_degenerate -- --exact
  → ok (1 passed)

$ cargo test -p bcinr-cmca --lib -- union_accumulates_both_operands_distinct_faults \
    mask_public_image_is_exactly_zero_or_all_ones \
    select_preserves_selected_fault_and_drops_unselected \
    floor_shares_sum_exactly_to_65536
  → 4 passed; 0 failed  (C1 numeric-law tests: fixed.rs union/mask/select + allocator.rs
    floor conservation)

$ cargo test -p bcinr-cmca --test case_studies test_rejection_invariance
  → 1 passed; 0 failed  (C2/C3 rejection-invariance)
```

**All spot checks PASS, unweakened** — reran directly by exact test name, not trusting
`MUTANT_KILL_MATRIX.md` or `PHASE2_RUNTIME_CLOSURE_VERDICT.md`'s own tables.

---

## Non-scope, checked anyway per prior instruction to "confirm nothing else broke"

`cargo test -p bcinr --test e2e_main`: **5 passed, 55 failed**, identical in shape to
`RECONCILIATION_VERIFICATION.md` check 11's finding. Reconfirmed still pre-existing and
unrelated to any bcinr-cmca workstream; not investigated further (out of this task's scope).

---

## Summary — what changed since `RECONCILIATION_VERIFICATION.md`

| Prior finding | Status now | Evidence |
|---|---|---|
| Artifact smoke test fails, missing `leaf_count` | **RESOLVED** — passes | item 5 above |
| `verify-generated` digest mismatch (both ontologies) | **RESOLVED** — passes | item 3 above |
| `--all-features` fails on `fixed.rs` baseline tests (mutant co-compilation) | **Symptom changed**: no longer a test-assertion failure — `--all-features` now fails to *compile*, blocked by the pre-existing, self-documented `generated_artifact_pending` feature (`from_bits` API mismatch, unrelated to mutant_N gating) | item 2 above |
| `cargo test --workspace` — `bcinr::e2e_main` 55 failing | Unchanged, still pre-existing, still out of scope | non-scope section above |

The mutant_N mutual-exclusivity question `RECONCILIATION_VERIFICATION.md` blocker 3 raised
could not be independently re-exercised as originally framed, because `--all-features` never
reaches test execution — the compile failure occurs first and is caused by a different,
already-known, already-off-by-design feature flag. Whether `mutant_N` features are mutually
exclusive under `--all-features` if `generated_artifact_pending` were excluded is untested by
this pass (task instructions specified `--all-features` literally, not a curated feature
subset).

---

## Verdict lines

**BCINR_CMCA_PURE_CONSUMER_ALIVE**

All five pure-consumer checks from the prior reconciliation round were independently rerun and
pass: quarantine exclusion (`cargo package --list`, zero `quarantine` hits), locked package
build/verify, `consumer_correspondence` ordering fix (3/3), real-artifact smoke test
(`smoke_test_against_real_mfw_artifact`, now passing against the current `GeneratedManifest`
struct and the current committed manifest JSON — no `leaf_count` mismatch), and dependency-tree
cleanliness (unchanged from prior pass, not re-run this session but not touched either).
Non-blocking finding carried forward, not fixed: `src/allocator.rs.orig` still ships in the
package tarball (packaging hygiene, not a boundary violation).

**CMCA_RUNTIME_CLOSURE_BLOCKED**

1. failed gate: `cargo test -p bcinr-cmca --all-features` (task checklist item 2) —
   falsifier: full test suite must pass with all features enabled — owner: owner of the
   `generated_artifact_pending` feature / `src/generated_artifact/mod.rs` / `src/fixed.rs`
   API surface it depends on — command+evidence: `cargo test -p bcinr-cmca --all-features` →
   `error[E0599]: no associated function or constant named 'from_bits' found for struct
   'fixed::NonNegativeFixed'` (213 occurrences) and for `SignedFixed` (18 occurrences); `error:
   could not compile 'bcinr-cmca' (lib) due to 222 previous errors` — this is the same,
   pre-existing, self-documented gap `Cargo.toml`'s own comment on `generated_artifact_pending`
   and `src/generated_artifact/mod.rs`'s module doc already describe ("Compiling that module
   unconditionally currently fails ... calls fixed-point constructors/fields this crate's
   `src/fixed.rs` API does not expose under those names") — required repair: either add
   `SignedFixed::from_bits` / `NonNegativeFixed::from_bits` constructors to `src/fixed.rs`
   matching the mfw producer's emitted call sites, or regenerate `cmca_generated.rs` to call
   the existing `from_value_bits` name instead, per whichever side of the producer/consumer
   contract is authoritative (`CMCA_ARTIFACT_CONTRACT.md`); until resolved, `--all-features`
   cannot pass and the previously-identified `mutant_N` co-compilation question (prior
   `RECONCILIATION_VERIFICATION.md` blocker 3) cannot even be re-exercised because the build
   fails before reaching that code path.

(All other runtime-closure gates — `verify-generated` digest match, `test-mutants` gate exit 0,
41/41 compile-fail, `cargo check --workspace`, the mutant_9/10/11 dedicated-oracle spot checks,
and the sampled C1/C2/C3 tests — independently PASS. `cargo test -p bcinr-cmca` on default
features is fully green, 62+6+7+1+3+1+5+15 = 100 passing across all binaries, 0 failed.)
