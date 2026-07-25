# JTBD Readiness Report — v26.7.17 CMCA (independent rerun)

**Verifier:** independent rerun agent | **Branch:** `recovery/cmca-v26.7.17-c2` | **Date:** 2026-07-17

Scope: rerun of 5 speculative, explicitly-inferred JTBD scenario test files plus the two
`chicago-claims` claim files, and an audit for mock/stub use and for forbidden
version/git/publish side effects. No commits made. No `src/` files touched. No Cargo.toml
version fields or CHANGELOG.md touched by this task (a pre-existing, uncommitted
dev-dependency addition of `chicago-tdd-tools` to `crates/bcinr-cmca/Cargo.toml` was found
already present in the working tree before this task started — it adds a dev-dependency
only, not a version bump, and was not made or modified by this task).

These are speculative job-to-be-done scenarios, not confirmed product requirements. Each
result below states exactly what was tested and what was not.

## 1. `jtbd_safety_certified_adaptive_control`

**Command:** `cargo test -p bcinr-cmca --test jtbd_safety_certified_adaptive_control`
**Result:** PASS — 2/2 tests passed
(`jtbd_fully_certified_attempt_actually_changes_persistent_control_mode`,
`jtbd_uncertified_attempt_leaves_persistent_state_byte_identical`).

**Falsifiable property tested:** that a fully-certified adaptive-control attempt actually
changes persistent control-mode state, and — symmetrically — that an attempt which fails
certification leaves the real persistent state (`weights`/`last_switch_t`/`prev_mode`)
field-identical to its pre-attempt value, checked against the real production types.

**This validates:** that the current `bcinr-cmca` implementation exhibits the expected
certified-vs-uncertified state-mutation asymmetry for at least the one scenario each test
constructs, using real production entry points (no mock/stub of a CMCA internal — see
mock/stub audit below).

**This does NOT validate:** that every certification failure mode (not just the ones these
two tests construct) is covered; that this property holds under concurrent/adversarial
access; that "safety-certified" in any regulatory or domain-specific sense is satisfied —
that framing is the task's own inferred JTBD label, not a claim this test makes about
external safety standards.

## 2. `jtbd_auditable_adaptive_policy`

**Command:** `cargo test -p bcinr-cmca --test jtbd_auditable_adaptive_policy`
**Result:** PASS — 1/1 test passed
(`independent_reader_disambiguates_which_of_two_candidates_was_certified`).

**Falsifiable property tested:** that, given two candidate mode-switch proposals, an
independent reader of the real sealed receipt/certificate chain can disambiguate which one
was actually certified — i.e., the certificate identity is not ambiguous between candidates.

**This validates:** that the current sealed-type chain (as exercised by this one
two-candidate scenario) carries enough distinguishing information for this specific
disambiguation to succeed using only real production types.

**This does NOT validate:** general auditability across arbitrary numbers of candidates,
external/third-party audit tooling, non-repudiation guarantees, or any regulatory
"auditable" standard — again, "auditable" is the task's own inferred label, tested only in
the one narrow sense this single test exercises.

## 3. `jtbd_multi_agent_resource_governance`

**Command:** `cargo test -p bcinr-cmca --test jtbd_multi_agent_resource_governance`
**Result:** FAIL — 1 passed, 2 failed.

Failing tests:
- `n_competing_workloads_conserve_the_exact_unit_budget`
- `one_malformed_competing_workload_does_not_flip_the_global_refusal_flag`

Both fail on the same real assertion against real production output: for an 8-way competing
workload allocation, the real `NonNegativeFixed` shares returned by the real allocation path
summed to **65532**, not the exact required unit budget of **65536** (`ONE.value_bits()`).
This is a genuine, reproducible conservation-invariant violation observed against real
collaborators, not a test-authoring bug or a mock artifact — the panic message shows the
actual `NonNegativeFixed { val, faults }` values returned by production code.

**Falsifiable property tested:** that N competing workloads' allocated shares sum to exactly
the fixed-point unit budget (conservation, "Invariant 4"), and that one malformed workload
among N does not flip the global refusal flag for the others.

**This validates:** nothing affirmative — the conservation property does NOT currently hold
for this 8-way competing-workload scenario against the real implementation. This is a real,
useful negative result, not a test-authoring defect (rerun independently, same failure both
times conceptually consistent with the panic's own arithmetic).

**This does NOT validate:** any claim that multi-agent resource governance conservation
holds in general or in this specific 8-way case; it affirmatively falsifies that specific
claim for the exercised scenario. It does not indicate whether smaller N or different demand
distributions would also fail — only the tested case was exercised.

## 4. `jtbd_semantic_mechanical_compilation`

**Command:** `cargo test -p bcinr-cmca --test jtbd_semantic_mechanical_compilation`
**Result:** PASS — 10/10 tests passed (`artifact_under_test::tests::*` × 6,
`tampered_payload_digest_byte_is_refused`, `real_case_studies_artifact_is_accepted`,
`dependency_tree_excludes_semantic_toolchain_crates`,
`artifact_under_test::tests::smoke_test_against_real_mfw_artifact`).

**Falsifiable property tested:** that the real `artifact.rs` verification module correctly
accepts well-formed artifacts (including a real case-studies artifact and a real MFW
artifact) and refuses each of several distinct malformation classes (malformed digest
string, payload-digest mismatch, unknown schema version, non-conserved floor table, wrong
dimensions, single tampered payload-digest byte) — and that the crate's own dependency tree
excludes semantic-toolchain crates it should not need at compile time.

**This validates:** that the real `artifact.rs` verification logic, exercised against real
artifacts and real byte-level tampering, currently discriminates well-formed from
malformed/tampered input for every malformation class this test constructs.

**This does NOT validate:** that all possible malformation classes are covered (only the
listed ones were constructed); that the dependency-exclusion check is exhaustive against
future dependency additions; or any claim about "mechanical compilation" correctness beyond
artifact-verification accept/refuse behavior for these specific cases.

## 5. `chicago-claims` runs

**Commands (rerun directly against the published `chicago-claims` binary, not read from a
sibling's report):**

```
cargo run -p chicago-claims --bin chicago-claims-verify -- \
  /Users/sac/chicago-tdd-tools/crates/chicago-claims/claims/cmca-observatory-proposal-only.toml
cargo run -p chicago-claims --bin chicago-claims-verify -- \
  /Users/sac/chicago-tdd-tools/crates/chicago-claims/claims/cmca-rejection-invariance.toml
```

**Results:** both exited 0, both reported `Standing: Alive`.

- `cmca-observatory-proposal-only`: scan evidence confirms (syntax-level, source AST only)
  that `ObservatoryOutcome` exists, its `flags` field is private, and the forbidden
  construction `CertificateReceipt` is absent from the scanned scope.
- `cmca-rejection-invariance`: scan evidence confirms `RefusalSet` exists, its field `0` is
  private, and `union`/`is_empty`/`bits` methods exist.

**This validates:** that these two named source-level syntactic properties hold in the
current `bcinr-cmca` source, per `chicago-claims`' own AST-scan mechanism.

**This does NOT validate (per the tool's own printed disclaimer, reproduced verbatim from
its output, not paraphrased upward):** "object-code branchlessness of the scanned
implementation ... universal unforgeability or semantic correctness of any method body ...
absence of runtime allocation or any other runtime property." Both reports explicitly state
no mutant results were recorded for these two claims (`(no mutant results recorded)`), so
no mutation-testing evidence backs either "Alive" standing — it is syntax-presence evidence
only.

## 6. Mock/stub audit

Spot-read all four new JTBD test files
(`jtbd_safety_certified_adaptive_control.rs`, `jtbd_auditable_adaptive_policy.rs`,
`jtbd_multi_agent_resource_governance.rs`, `jtbd_semantic_mechanical_compilation.rs`).

**Finding: no mock or stub of a CMCA production type was found.** Every occurrence of
"mock"/"stub"/"fake" in these files is inside a doc comment explicitly disclaiming their use
(e.g. `jtbd_multi_agent_resource_governance.rs:7`: "No mock or stub of any CMCA internal is
used anywhere in this file."). Grep for `mock|stub|fake|Mock|Stub|Fake` across all four files
matched only comment lines, zero matches inside executable code. Each test constructs real
production types (`NonNegativeFixed`, `RefusalSet`, `ObservatoryOutcome`,
sealed-chain types from `proposal.rs`/`shadow.rs`/`jump.rs`/`stability.rs`/
`certification.rs`/`mode_switch.rs`, and calls the real `artifact.rs` verification
functions and the real allocation path) and asserts on their real returned state — this
satisfies the "real Chicago TDD, no mocks/stubs of CMCA internals" requirement for this
batch of 4 test files. (The `jtbd_multi_agent_resource_governance.rs` test failing in §3
above is a real, unmocked failure of production code — it is not evidence of a mock being
used; the panic message shows literal `NonNegativeFixed` values from the real allocator.)

## 7. Version/publish/git-touch audit

- No file in this batch of 5 test targets modifies `Cargo.toml` version fields or
  `CHANGELOG.md`.
- No test in this batch runs `git commit`, `cargo package`, `cargo publish`, or any
  git/package/publish subprocess.
- The one uncommitted `Cargo.toml` change present in the working tree
  (`crates/bcinr-cmca/Cargo.toml`, adding `chicago-tdd-tools` as a dev-dependency) was
  already present before this task ran, is a dependency addition (not a version bump of the
  `bcinr-cmca` package itself), and was not made by this task. It was not modified further
  by this task.
- This task made no commits and did not change git branch.

## Overall

Of the 5 JTBD scenarios, **4 have real, falsifiable, passing evidence** against real
production collaborators for the specific narrow scenario each test constructs: JTBD 1
(safety-certified state-mutation asymmetry), JTBD 2 (auditable disambiguation between two
candidates), JTBD 4 (semantic/mechanical artifact-verification accept/refuse behavior), and
JTBD 5 (the two `chicago-claims` source-level syntactic claims, explicitly scoped by the
tool itself to AST-presence evidence, not semantic or object-code proof). **JTBD 3
(multi-agent resource governance conservation) is currently FALSIFIED, not merely
speculative** — the real 8-way competing-workload allocation path returns shares summing to
65532 instead of the required 65536, a genuine conservation-invariant violation reproduced
in this independent rerun. None of the 5 passing/failing results should be read as a
blanket "production ready" claim: each validates one narrow, explicitly-scoped property
under the one or few scenarios its test constructs, not the full space of the JTBD's
speculative framing, and JTBD 3's failure is a real defect in the current allocator, not a
test artifact.
