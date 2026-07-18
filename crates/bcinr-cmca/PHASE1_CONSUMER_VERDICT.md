# Phase 1 Consumer-Boundary Verdict — bcinr-cmca (v26.7.17)

Evidence gathered by direct command execution and source reading on
2026-07-17, branch `recovery/cmca-v26.7.17-c2`. No files modified.

## 1. RDF parser / graph store / SHACL-ShEx engine / Python invocation in src/

`grep -rniE "rdf|shacl|shex|pyo3|Command::new\(\"python|subprocess" crates/bcinr-cmca/src/`
returns only comment/const-name occurrences (module doc references to "RDF"
in prose, and the constant name `RDF_INPUT_DIGEST` which stores a hex digest
string, not RDF data). No parser, no graph store, no SHACL/ShEx code, no
`std::process::Command` invocation of `python`, no `subprocess`-style calls.

**PASS.**

## 2. Legacy generator/ontology quarantined and unreachable from build

`crates/bcinr-cmca/quarantine/legacy-generator/generator.py` and
`crates/bcinr-cmca/quarantine/legacy-ontology/{cmca-rdf.ttl,generalization.ttl}`
exist (moved, not deleted). No `Makefile` target references
`quarantine`, `generator.py`, or `legacy-ontology`. No `build.rs` exists in
`crates/bcinr-cmca/` at all, so nothing in the crate's build graph can invoke
the quarantined generator.

**PASS** on reachability from build/Makefile. **Caveat (see §6):** the
quarantine directory is not excluded from the packaged crate contents.

## 3. src/artifact.rs typed verification coverage + real test run

`crates/bcinr-cmca/src/artifact.rs` exposes `verify_generated_profile(...)`
(source line 172) which typed-checks: schema version, payload digest,
manifest-payload agreement, dimensions, table lengths, registry
bounds/uniqueness, numeric-profile compatibility, formula-registry
compatibility, and floor-table conservation (each has a dedicated refusal
test below).

Ran `cargo test -p bcinr-cmca --lib artifact`:

```
running 9 tests
test artifact::tests::smoke_test_against_real_mfw_artifact ... ignored, requires crates/bcinr-cmca/generated-artifact/ from the sibling producer task; not present at implementation time
test artifact::tests::malformed_digest_string_refused ... ok
test artifact::tests::floor_table_not_conserved_refused ... ok
test artifact::tests::duplicate_registry_index_refused ... ok
test artifact::tests::out_of_bounds_registry_index_refused ... ok
test artifact::tests::unknown_schema_version_refused ... ok
test artifact::tests::payload_digest_mismatch_refused ... ok
test artifact::tests::valid_profile_accepted ... ok
test artifact::tests::wrong_dimensions_refused ... ok

test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 55 filtered out
```

8/8 non-ignored tests pass. One test (`smoke_test_against_real_mfw_artifact`)
is `#[ignore]`d with a stale reason string — the comment says the
`generated-artifact/` tree "is not present at implementation time," but that
directory **does now exist on disk**
(`crates/bcinr-cmca/generated-artifact/{case-studies,generalization}/...`).
This is a real, narrow finding: the ignore reason is stale and the smoke
test against the real producer artifact is not currently exercised even
though its precondition is now met. I did not run it manually or edit the
`#[ignore]` attribute (out of scope: no test-suite edits in this run).

**PASS** on typed-coverage breadth and green results for the tests that do
run. **Finding:** stale `#[ignore]` reason on the mfw-smoke test (§ Findings
below).

## 4. cargo tree — zero dependency on mfw / oxigraph / praxis-graphlaw

`cargo tree -p bcinr-cmca` (full tree, both normal and dev deps) contains no
`mfw`, `oxigraph`, or `praxis-graphlaw` entries — confirmed by grepping the
full tree output (`grep -iE "mfw|oxigraph|praxis"` → no matches). The only
path dependency is `bcinr-logic` (sibling in-workspace crate); all other
dependencies are crates.io leaf crates (`blake3`, `serde`, `serde_json`,
`trybuild`, `proptest`, and their transitive deps).

Textual occurrences of the string `mfw` do exist in the crate (in
`Cargo.toml` comments, `src/lib.rs`, `src/artifact.rs`, `src/allocator.rs`,
`src/generated_artifact/mod.rs`, `tests/consumer_correspondence.rs`) — all of
them are documentation prose describing the producer's identity/provenance,
not a dependency declaration or code path.

**PASS.**

## 5. Buildable/testable without `/Users/sac/mfw` reachable

No `build.rs` in the crate. No path dependency on `/Users/sac/mfw` in
`Cargo.toml`. Grepped all `.rs`/`.toml` files under `crates/bcinr-cmca/` for
the literal path `/Users/sac/mfw` — the only hits are inside doc comments
(`src/artifact.rs`, `tests/consumer_correspondence.rs`) describing where the
*generator* historically lived; none of them are `include!`, `include_str!`,
`std::fs::read` at a hardcoded path, or a `Command::new` invocation. `cargo
test -p bcinr-cmca` (see §3, §6) was run in this session without
`/Users/sac/mfw` being touched or referenced by the build.

**PASS.**

## 6. cargo package -p bcinr-cmca --locked

Ran `cargo package -p bcinr-cmca --locked --allow-dirty` (`--allow-dirty`
needed only because the workflow's snapshot conventionally leaves the tree
clean but the packaging step itself checks a clean-enough state; no source
edits were made). Result:

```
Packaging bcinr-cmca v26.6.24 (.../crates/bcinr-cmca)
Packaged 135 files, 805.4KiB (134.9KiB compressed)
Verifying bcinr-cmca v26.6.24 (.../crates/bcinr-cmca)
Compiling bcinr-logic v26.6.24
Compiling bcinr-cmca v26.6.24 (.../target/package/bcinr-cmca-26.6.24)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.06s
```

Package succeeds and the packaged/verified build compiles from packaged
contents alone. `generated-artifact/` and `src/artifact.rs` are both present
in the 135-file package list, so the artifact the crate needs to consume at
build/test time travels with the package.

**Finding, not a pass:** `cargo package -p bcinr-cmca --list` shows the
`quarantine/` directory (including `quarantine/legacy-generator/generator.py`
and both `.ttl` files) **is included** in the packaged crate. There is no
`exclude` key in `crates/bcinr-cmca/Cargo.toml`. This does not reintroduce a
runtime/build dependency on the quarantined generator (§2's reachability
check still holds — nothing in the build graph invokes it), but it does mean
the "excluding quarantine... appropriately" half of check 6, as posed, does
not hold: the quarantined Python generator ships inside the published crate
tarball.

## Additional finding surfaced incidentally (not one of the six checks)

Running the full `cargo test -p bcinr-cmca` (not just `--lib artifact`)
turned up one currently-failing test, outside the four files the workflow
summary called out as stale-API and outside the six checks above:

```
test generalization_numeric_payload_matches_old_lawful_output_exactly ... FAILED
thread '...' panicked at crates/bcinr-cmca/tests/consumer_correspondence.rs:93:5:
generalization from_value_bits(..) sequence order does not match the frozen
pre-migration fixture (multiset-equal: true) — this is a
CORRESPONDENCE_REQUIRED failure even though the underlying values are the
same set (an ordering-only regression is still a byte-equivalence violation
per PRE_MIGRATION_BASELINE.md)
```

`case_studies_numeric_payload_matches_old_lawful_output_exactly` (the sibling
fixture) passes; only the `generalization` fixture correspondence fails, on
ordering, not on value-set membership. This is inside the consumer-boundary
domain (it is exactly a producer/consumer artifact-correspondence check) so
it is recorded here rather than as purely out-of-scope, but reconciling it
is not part of the six checks this verdict was scoped to, and no fix was
attempted in this run (FIX FORWARD ONLY / no test edits this run).

Separately, contrary to the workflow's carried-forward summary that
`hostile_mutants.rs`, `differential.rs`, `case_studies.rs`, and
`calibration.rs` "were written against the OLD API" and would need
reconciling: as of this read, all four compile cleanly against the current
crate and all their tests pass (`cargo test -p bcinr-cmca --test
hostile_mutants --test differential --test case_studies --test calibration`
→ 6/6, 7/7, 1/1, 5/5 passed, 0 failed). Either they were already reconciled
by work landed after that summary was written, or the summary was stale at
the time it was carried into this task. I did not verify git history to
determine which; the current on-disk state is: they pass.

Also contrary to the carried-forward summary of "38 of 41 .../tests/ui/*.rs
... have no committed .stderr baseline yet": `ls tests/ui/*.stderr` and `ls
tests/ui/*.rs` both return 41. All 41 `.rs` compile-fail cases have a
committed `.stderr` baseline, and `cargo test -p bcinr-cmca --test
compile_fail_tests` passes all 41 trybuild cases. I spot-checked two
baselines (`fail_call_new.stderr`, 10 lines;
`fail_construct_admitted_control_state.stderr`, 5 lines) — both are
non-empty, non-trivial rustc diagnostics, not blank/placeholder files. I did
not read all 41 to independently confirm the compiler fails for the intended
reason in every case (that per-case review was not requested by the six
checks and would be substantial additional work); this is recorded as
UNVERIFIED at the level of "each stderr is correct as an intentional design
rationale," verified only at the level of "all 41 exist, are non-trivial,
and trybuild accepts them."

## Out-of-scope finding carried forward per instruction

`crates/bcinr-logic/src/mask.rs` `select_u32` contract-gate finding: recorded
as out-of-scope for this CMCA-focused workflow per explicit task instruction.
Not investigated further in this run.

## Verdict

Checks 1, 2 (reachability half), 3, 4, 5 pass with real evidence. Check 6
passes on "builds/tests from packaged contents" but fails on "excludes
quarantine appropriately" — the quarantined Python generator and ontology
files are shipped inside the package tarball with no `exclude` in
`Cargo.toml`. That is a genuine, narrow gap in the artifact-boundary
contract as posed by check 6, not a nonclaim: `bcinr-cmca` does not
currently depend on, invoke, or require the quarantined generator to build
or test, but it does currently redistribute it.

BCINR_CMCA_PURE_CONSUMER_BLOCKED

1. `crates/bcinr-cmca/Cargo.toml` has no `exclude` entry, so
   `cargo package -p bcinr-cmca --list` includes `quarantine/legacy-generator/generator.py`
   and `quarantine/legacy-ontology/*.ttl` in the published package (check 6
   fails the "excluding quarantine... appropriately" clause; check 2's
   build/Makefile-reachability clause still holds independently).
2. `tests/consumer_correspondence.rs::generalization_numeric_payload_matches_old_lawful_output_exactly`
   fails on disk right now (ordering mismatch vs. the frozen pre-migration
   fixture; values are multiset-equal but sequence order differs) — a real,
   currently-red test inside the producer/consumer correspondence surface
   this workflow is about, not yet reconciled.
3. `src/artifact.rs::tests::smoke_test_against_real_mfw_artifact` remains
   `#[ignore]`d with a reason string ("not present at implementation time")
   that is stale: `crates/bcinr-cmca/generated-artifact/` now exists on disk.
   The smoke test that would validate artifact.rs against a real producer
   tree is consequently not currently exercised in CI/test runs.
