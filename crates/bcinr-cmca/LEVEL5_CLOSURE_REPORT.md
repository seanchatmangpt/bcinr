# Level-5 Closure Report — v26.7.17 CMCA, Track A/B/C/D/E Independent Verification

**Verifier:** independent verifier agent (this task) | **Branch:** `recovery/cmca-v26.7.17-c2`
(unchanged throughout) | **HEAD:** `7e7e7cd5` (unchanged throughout) | **Date:** 2026-07-18

Scope: independent reproduction of everything claimed by four sibling tracks (A: refusal
invariance, C: actuation evidence, B+E: packaging/sequencing, D: mutation-provider
generalization) that ran in parallel on this branch. Nothing below is taken on a sibling's
self-report — every command was run fresh by this task against the current working tree
(`git status --porcelain` at start: 7 modified + 22 untracked paths, listed and unchanged in
kind throughout this pass). No commits were made by this task. No `src/` files were edited by
this task (Bash-only verification). No test was deleted or weakened to make anything pass.

---

## 1. Full default suite — `cargo test -p bcinr-cmca`

**Command:** `cargo test -p bcinr-cmca` (no filters). **Exit code: 0.**

19 test binaries + one doc-test group, 20 `test result: ok` blocks, **147 tests passed, 0
failed, 0 ignored** (excluding the `alloc-gate`-feature-gated `alloc_gate.rs`, which is not part
of the default-feature run and is not claimed here). Full transcript captured at
`/tmp/full_test_suite.log` (this session's scratch, not committed). This includes
`tests/consumer_correspondence.rs` (3/3 passed) — the test `PHASE2_RUNTIME_CLOSURE_VERDICT.md`
recorded as the reason for its `CMCA_RUNTIME_CLOSURE_BLOCKED` verdict. That verdict is
confirmed stale/superseded on the current tree: the fix predates this round (present already at
the start of this task's work, not introduced by Track A/B/C/D/E), but is reconfirmed here by a
real run, not assumed from the task context's framing alone.

**Verdict: ALIVE** (AGENTS.md §28: "executes and passes all declared gates in the pinned
environment," scoped to the default-feature gate set).

## 2. Track A — refusal-invariance regression

**Commands and results:**
- `cargo test -p bcinr-cmca --test jtbd_refusal_invariance_regression` → **PASS, 8/8.**
- `cargo test -p bcinr-cmca --test jtbd_boundary_adversarial_inputs` → **PASS, 8/8.**

**Spot-read of the NO_LEAVES invariant, both sides:**
- `tests/jtbd_boundary_adversarial_inputs.rs:118-221`
  (`n_leaves_zero_fires_no_leaves_refusal_with_zeroed_commit_mask`): the module doc comment
  (lines 44-61) and the assertion's own inline comment (lines 179-198) explicitly record that
  this test **used to assert the defective behavior** (`weights` normalized 65536→32768 despite
  the refusal) and has been **flipped** to `assert_eq!(weights, weights_before, ...)` — the
  correct invariant. Confirmed by reading the executable assertion directly, not the comment
  alone: line 199-205 is `assert_eq!(weights, weights_before, "... must leave weights
  byte-identical to their pre-attempt value ...")`, and this test passed in the run above.
- `src/allocator.rs:1986` (production fix, read directly): `let has_refusal = (has_error |
  (nl_is_zero != 0)) & !degrade_to_certified_selection;` — `nl_is_zero` is now folded into the
  state-commit gate, branchlessly (`const_select_u32`/`select_nnf`/`unroll_8_static!`
  throughout, no new branches, no `unsafe`). Prior to this fix the gate was `has_error &
  !degrade_to_certified_selection` only, which is exactly the defect the JTBD readiness reports
  and `PHASE2_RUNTIME_CLOSURE_VERDICT.md` (numbered blocker note on `has_refusal`) both trace.
- `tests/jtbd_refusal_invariance_regression.rs` (new file, 432 lines, read in full):
  generalizes the single-cause NO_LEAVES fix into a permanent regression covering **every**
  `RefusalSet` variant — 4 reachable causes each get a dedicated single-cause test asserting
  full byte-invariance (`no_leaves_only_refusal_leaves_full_state_invariant`,
  `digest_mismatch_only_refusal_leaves_full_state_invariant`,
  `dwell_unsatisfied_only_refusal_leaves_full_state_invariant`,
  `proposal_rejected_only_refusal_leaves_full_state_invariant`); the 4 unreachable causes are
  handled honestly, not fabricated — `AUTHORITY_MISSING` gets a real targeted run proving its
  mask is unconditionally unsatisfiable (dead code, not fixed, documented as an authority/
  reporting-surface question out of this track's scope), and
  `CERTIFICATE_MISSING`/`CERTIFICATE_STALE`/`ROUND_MISMATCH` are documented via a grep-confirmed
  source-reading finding (no code path constructs them) plus a representative sweep test
  checking they never appear across 6 real scenarios. All 8 tests in this file passed.

**Track A verdict: CLOSED.** Real production fix, verified branchless by direct source read,
generalized regression coverage across all declared `RefusalSet` variants (not just the one
originally found), no test weakened — the previously-defective assertion was flipped to the
correct one and both old and new tests pass against the fixed code.

## 3. Track C — actuation-evidence closure

**Commands and results:**
- `cargo test -p bcinr-cmca --test jtbd_auditable_adaptive_policy` → **PASS, 2/2**
  (`independent_reader_disambiguates_which_of_two_candidates_was_certified` — the pre-existing
  certification-hop test — plus the new
  `independent_reader_disambiguates_which_candidate_was_actuated_via_public_evidence_only`).
- `cargo test -p bcinr-cmca --test compile_fail_tests` → **PASS, 1/1** (44 nested `trybuild`
  cases, all `ok` — up from the 41 recorded in `PHASE2_RUNTIME_CLOSURE_VERDICT.md`; the 3 new
  cases are `fail_field_construct_actuation_evidence.rs`,
  `fail_struct_update_actuation_evidence.rs`, `fail_tuple_construct_actuation_evidence.rs`, all
  confirmed present under `tests/ui/` and all passing with committed `.stderr` baselines).

**Spot-read for public-API-only actuation evidence (the specific ask):**
- `src/allocator.rs:872-874`: `pub struct CertificateReceipt { pub(crate) digest: u64 }` —
  confirmed by direct read, the digest field is `pub(crate)`, invisible outside the
  `bcinr-cmca` crate. `tests/jtbd_auditable_adaptive_policy.rs` lives under `tests/`, which
  Rust compiles as a separate crate — this is a real, compiler-enforced visibility boundary,
  not a self-imposed convention.
- `src/mode_switch.rs:181-186`: `ActuationEvidence::new` is a private (not even `pub(crate)`)
  associated function, with its doc comment stating the only call site is inside
  `apply_mode_switch` in the same module — confirmed by grep, no other construction site
  exists.
- `src/mode_switch.rs:270-320` (`apply_mode_switch`, read in full): signature changed from
  `expected_certificate_digest: u64` to `expected_certificate: CertificateReceipt`, and now
  returns `(Result<(), ModeSwitchRefusal>, ActuationEvidence)`. The certificate check is
  `certificate == expected_certificate` (derived `PartialEq` on the receipt), never a raw
  digest comparison the caller would need private-field access to construct.
- The new test's `external_auditor_identify_actuated_candidate` helper (lines 459-490) uses
  exactly: `ActuationEvidence::certificate_digest()`/`.outcome()` (both `pub fn` accessors),
  `CertificateReceipt::admit_certificate` (the pre-existing public constructor), and `==`. No
  `pub(crate)` item, no crate-internal `use`, no `#[cfg(test)]`-gated backdoor is referenced
  anywhere in the file. This was confirmed by reading the full 623-line file, not by trusting
  its own doc comments' claims about itself.
- 3 new UI compile-fail cases (read in full) attempt tuple-construction, field-construction, and
  struct-update-construction of `ActuationEvidence` from outside the crate; all 3 fail to
  compile with the expected `E0451`/`E0423` errors, confirmed via their committed `.stderr`
  files and the passing `compile_fail_tests` run above.

**Track C verdict: CLOSED.** The actuation-evidence test genuinely reaches through
`prepare_mode_switch`→`observe_dwell`→`apply_mode_switch` using only public API, confirmed by a
real Rust visibility boundary (integration-test crate separation), not by convention or
self-report. Both the certification-hop test and the new actuation-hop test pass; compile-fail
coverage grew from 41 to 44 or without regressing any of the original 41.

## 4. Mutation gate, generated-artifact gate, `--all-features`

- `cargo make test-mutants` → **exit 0.** Output: "GATE PASSED — all 11 mutants' dedicated
  oracle tests gave the expected result." All 11 `mutant_N` features run individually, each
  producing a clean, non-colliding pass (e.g. under `mutant_9` only 4 tests run, `kill_m05_*`
  is absent — cfg-gated out — rather than failing as collateral damage).
  `git diff crates/bcinr-cmca/tests/hostile_mutants.rs` shows only `CORRECT_*`/`WRONG_M*`
  constant updates this round (the conservation-fix baseline shift), not new `#[cfg]` gates —
  the `#[cfg(not(feature = "mutant_9/10/11"))]` gates that prevent the collateral failures
  `PHASE2_RUNTIME_CLOSURE_VERDICT.md` reported (mutants 9-11 additionally failing
  `kill_m01`/`kill_m03`/`kill_m05`/`kill_m07`) were already present in the tree before this
  round started — confirmed by grep — so that numbered blocker from `PHASE2_...VERDICT.md` is
  also stale/superseded, not newly fixed by Track A/C. Recorded honestly as a pre-existing-fix
  confirmation, not attributed to this round's tracks.
- `cargo make verify-generated` → **exit 0.** Output: "verify-generated: PASS — committed
  Gamma_CMCA artifact digests and schema_version verified (no generator invoked)."
- `cargo test -p bcinr-cmca --all-features` → **exit 101 (failing), same known root cause as
  `JTBD_READINESS_REPORT_ROUND2.md` documented, now observed more broadly.** Under `--fail-fast`
  (cargo's default), the first failing binary is `jtbd_auditable_adaptive_policy` — both of its
  tests fail with `fixture must clear the Observatory as RecertificationCandidate; got
  ObservatoryFlagSet(7)`, the identical panic round2 recorded for the one test that existed
  then. Re-run with `--no-fail-fast` to see the full picture: **new data point this round** —
  `jtbd_boundary_adversarial_inputs` (3 tests), `jtbd_bounded_under_pathological_input` (1),
  `jtbd_conservation_regression` (1), and `jtbd_multi_agent_resource_governance` (3) also fail
  under `--all-features`, none of which round2 tested against `--all-features` (they did not
  yet exist, or were only checked under default features). Root cause, read from
  `crates/bcinr-cmca/Cargo.toml`'s `[features]` table: `mutant_1..mutant_11` are eleven
  independent, default-off, mutually-corrupting cfg features, each intended to be enabled ONE
  AT A TIME (exactly what `cargo make test-mutants` above does correctly); `--all-features`
  enables all eleven simultaneously against `allocate()`'s numeric path, which no test file in
  this crate — old or new — is designed to tolerate. This is the same scope/tooling mismatch
  round2 named, not a new independent defect: the default-feature build (item 1 above) is fully
  green, and every individually-scoped `mutant_N` feature passes cleanly via `test-mutants`.
  Flagging the broader failure surface honestly per instructions, since this is new information
  this round even though the root cause is not new.

**Verdict on this item: no new regression traceable to Track A or Track C's own code changes.**
The `--all-features` failures are attributable to the pre-existing, documented,
intentionally-unsupported combination of all 11 mutation-testing features at once, now simply
observed against a larger set of numeric-sensitive test files than round2 checked. Default
features remain the crate's one supported/gated build configuration, and it is green.

## 5. `cargo make package-reality-check` (Track B)

**Command:** `cargo make package-reality-check`. **Exit code: 0.**

Real, fresh run (not the receipt file read passively): `bcinr-logic` packaged cleanly without
`--allow-dirty`; `bcinr-cmca` needed a dirty-tree `--allow-dirty` retry (its own directory has
uncommitted files from concurrent work) and then failed with **exactly** the known sequencing
blocker text (`failed to select a version for the requirement \`bcinr-logic = "^26.7.17"\`` +
`location searched: crates.io index`) — the script's own classifier (read at
`scripts/gates/package-reality-check.sh:151-160`) matches on both substrings before labeling a
`bcinr-cmca` failure "KNOWN" and non-gating; anything else is explicitly labeled UNEXPECTED and
sets `fail=1`. This run's summary line: `PACKAGE_REALITY_SUMMARY logic_result="PASS" ...
cmca_result="FAIL (KNOWN sequencing blocker)" ... fail=0` — matching the honest,
non-fabricated classification the task asked me to confirm. `crates/bcinr-cmca/
PACKAGE_REALITY_RECEIPT.md` was regenerated in full by this run (`Generated (UTC):
2026-07-18T03:40:32Z`, this session), correctly headed "MUTABLE RECEIPT — NOT A RULE," and
honestly states both the dirty-tree limitation (29 uncommitted paths at run time) and the
mfw-filesystem-absence limitation (dependency-graph absence proven elsewhere; true filesystem
absence not proven, by design, since that would require deleting a real directory outside this
task's scope).

**Track B verdict: CLOSED** for its own stated charter (produce a replayable, honest,
non-fabricated packaging-reality artifact distinguishing the known blocker from anything
unexpected). This does **not** close release gates G0/G1 themselves — those remain open exactly
as the ledger's own "Gate Closure Summary" states, unaffected by this round.

## 6. Publication Sequencing Decision (Track E)

Read `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` in full (909 lines). The "Publication Sequencing
Decision — 2026-07-18 (Track B+E...)" section exists (lines 720-909), and is honest on every
point checked:

- States plainly, twice, that actual `cargo publish` (without `--dry-run`) of either crate was
  **not** performed in this pass and is reserved for explicit separate authorization.
- Enumerates exactly 3 lawful options to resolve the `bcinr-logic`/`bcinr-cmca`
  publish-ordering blocker (real publish of `bcinr-logic` first; local-registry rehearsal;
  release-topology change) without recommending one — left as a `cmca-release-integrator`
  policy decision.
- Documents a real, empirical `[patch.crates-io]` local-path override test for `bcinr-logic`
  (tried, failed identically to the unpatched case, reverted — `git diff Cargo.toml` confirmed
  empty afterward by that pass) as informative negative evidence, not a fabricated success.
- Documents the full local-registry rehearsal (wholesale `[source.crates-io] replace-with`) as
  **assessed and explicitly not attempted**, with the sizing reasoning shown (16+ transitive
  packages would need hand-built index entries), rather than silently skipped or falsely
  claimed complete.
- A dedicated "What this section does not claim" subsection explicitly disclaims: rehearsal
  success, `[patch.crates-io]` being useless in general, `bcinr-logic` having been published,
  and closing G0/G1.

**Independent confirmation that no real publish occurred:** my own fresh
`cargo make package-reality-check` run (item 5 above, this session) reproduced the **identical**
sequencing-blocker error text. If `bcinr-logic` 26.7.17 had actually been published to the real
crates.io registry at any point (by this round or any other), that error would not reproduce —
the registry lookup would succeed. This is direct, fresh evidence, not a read of a prior claim.
A `find ~/.cargo/registry/cache -iname 'bcinr-*-26.7.17*'` also returned nothing.

**Track E verdict: CLOSED** for its own stated charter (produce an honest, complete decision
record with real evidence). Does not itself close G0/G1 — the ledger says so explicitly, and I
did not find evidence contradicting that.

## 7. `chicago-claims` sibling repo (Track D)

**Command:** `cargo test -p chicago-claims` (from `/Users/sac/chicago-tdd-tools`, branch
`main`). **Exit code: 0. 25 passed, 0 failed, 2 ignored** (the 2 ignored tests require the
`bcinr` checkout on disk and spawn real subprocesses — both were run explicitly, see below).

**Three claim verifications, rerun fresh:**

| Claim | Standing | Scan evidence | Mutant evidence |
|---|---|---|---|
| `cmca-fault-union.toml` | **Alive** | `NumericFaultSet` found; field `0` private: `Some(true)`; `union`/`is_empty`/`bits` found | 6/6 `KilledByIntendedOracle` (`first-wins`, `last-wins`, `left-only`, `right-only`, `empty-set`, `overwrite`) |
| `cmca-observatory-proposal-only.toml` | **Alive** | `ObservatoryOutcome` found; field `flags` private: `Some(false)`; forbidden `CertificateReceipt` construction absent | (no mutant results recorded) |
| `cmca-rejection-invariance.toml` | **Alive** | `RefusalSet` found; field `0` private: `Some(true)`; `union`/`is_empty`/`bits` found | (no mutant results recorded) |

All three reproduce **exactly** — same standing, same scan booleans, same mutant classification
(or honest absence thereof) as one would expect from `JTBD_READINESS_REPORT.md`'s prior run of
the latter two, plus the new third claim. `Delta: intent -> implementation: none;
implementation -> evidence: none` on all three.

**Precision note on `cmca-fault-union.toml`'s mutant evidence (read `mutate.rs` in full,
872 lines, to characterize this precisely rather than accept the "Alive" label at face value):**
`chicago-claims`'s `classify_mutant` dispatches on which fields a `[[mutants]]` entry sets —
`fixture_path` alone routes to `FixtureProvider` (which activates and tests a small, hand-written
local `.rs` fixture file modeling the described corruption, e.g.
`tests/fixtures/first_wins.rs`'s `union(a,b) = if a != EMPTY { a } else { b }`); the triple
`crate_path`+`feature`+`test_name` routes to `CargoFeatureProvider` (which spawns a real
`cargo test` subprocess against a real crate with a real cfg feature enabled). All six
`[[mutants]]` entries in `cmca-fault-union.toml` set only `fixture_path`. **The reported
"6/6 KilledByIntendedOracle" is therefore FixtureProvider evidence** — a hand-written analogous
`union` implementation exhibiting each named corruption pattern is confirmed to fail the
intended oracle — **not** literal evidence that `bcinr-cmca`'s real
`NumericFaultSet::union` was built six different corrupted ways and each was killed. The claim's
*scan* evidence (type/field/method presence) genuinely is against the real file
(`scope.file = "/Users/sac/bcinr/crates/bcinr-cmca/src/fixed.rs"`), confirmed by the file path
in the TOML itself; only the *mutant* evidence is fixture-based analogy. This is a precision
distinction the "Alive" standing label and the report's own text do not make explicit, so it is
recorded here rather than passed through uncritically.

**However, Track D's `CargoFeatureProvider` mechanism itself is genuinely real and was verified
live, not merely read as source:** ran
`cargo test -p chicago-claims mutate::tests::cargo_feature_provider_kills_real_bcinr_cmca_mutant_1_via_real_subprocess
-- --ignored --exact --nocapture` explicitly (it is `#[ignore]`d by default because it requires
the `bcinr` checkout and spawns a real subprocess — both true in this environment). **Result:
PASS, 1/1, 7.01s** — consistent with a genuine `cargo test -p bcinr-cmca --features mutant_1
--test hostile_mutants kill_mutant_1_single_measure_collapse` subprocess actually compiling and
running (a fixture comparison would not take 7 seconds). This confirms the provider
*abstraction* Track D built is real, working, and proven against the actual `bcinr-cmca` crate —
it is simply not what backs `cmca-fault-union.toml`'s specific evidence today.

**Track D verdict: CLOSED for the provider-generalization mechanism itself** (a real
`MutationProvider` trait with two working implementations, one proven end-to-end against the
real target crate). **The `cmca-fault-union.toml` claim's mutant evidence should be read
narrowly** — analogy-fixture evidence for the union-corruption pattern class, not literal
mutation of the production `NumericFaultSet::union` — a precision note, not a defect (nothing in
the claim's own report text or the TOML file misrepresents which provider ran; the distinction
just requires reading `mutate.rs`'s dispatch logic to see).

## 8. Mock/stub audit — Track A and Track C files

Grepped (case-insensitive) `mock|stub|fake` across all Track A/C new-or-modified test files:
`tests/jtbd_refusal_invariance_regression.rs` (new), `tests/jtbd_boundary_adversarial_inputs.rs`
(modified), `tests/jtbd_auditable_adaptive_policy.rs` (extended), and read
`tests/ui/fail_*_actuation_evidence.rs`/`.stderr` (new, Track C) in full.

**Finding: zero matches inside executable code.** Every match is inside a doc comment
explicitly disclaiming mock/stub use (e.g. `jtbd_refusal_invariance_regression.rs:59`: "No mock
or stub of any CMCA internal is used anywhere in this file."; `jtbd_boundary_adversarial_inputs
.rs:1,4`; `jtbd_auditable_adaptive_policy.rs:80,103`). Every test in these files constructs real
production types (`AdaptiveUpdate`, `AdmittedControlState`, `CertificateReceipt`,
`CertifiedLearning`, `RefusalSet`, `ObservatoryOutcome`, `StabilityCandidate`,
`ActuationEvidence`, etc.) through their real, sealed constructors, and calls the real
`allocator::allocate`/`observatory::evaluate_calibration`/`proposal::admit_proposal`/
`shadow::execute_shadow`/`jump::analyze_jump`/`stability::derive_stability_candidate`/
`certification::seal_certificate`/`certification::observe_dwell`/
`mode_switch::prepare_mode_switch`/`mode_switch::apply_mode_switch` functions — confirmed by
reading each file in full, not by trusting the disclaiming comments alone.

**No violation found.**

## 9. Git safety

- **Branch:** `recovery/cmca-v26.7.17-c2` at both the start and end of this task
  (`git branch --show-current`, checked twice).
- **Commits:** `git log --oneline -5` unchanged (`7e7e7cd5` still HEAD) before and after this
  task's work. `git reflog -10` shows the two most recent entries are `reset: moving to HEAD`
  (HEAD@{0}, HEAD@{1}) at the **same** commit `7e7e7cd5` — a no-op/unstage-type reset, not a
  commit or a destructive rewrite (the commit hash never changes) — most likely from an earlier
  session's `git status`/unstage activity, not from this task, which ran no `git` write command
  beyond the read-only ones cited here. Zero commits were made by this task or observed to have
  been made by any of the four sibling tracks during this round.
- **Cargo.toml version fields:** `git diff -- '**/Cargo.toml' | grep -iE '^\+version|^-version'`
  → empty. The only `Cargo.toml` change in the working tree is
  `crates/bcinr-cmca/Cargo.toml`'s `chicago-tdd-tools` dev-dependency addition, unchanged from
  what round 1's report already described and attributed to pre-existing work, not this round.
- **No actual publish:** confirmed two independent ways — (a) the ledger's own Publication
  Sequencing Decision section states plainly no real `cargo publish` occurred (§6 above); (b)
  this task's own fresh `cargo make package-reality-check` run reproduced the identical
  crates.io-registry-lookup failure for `bcinr-logic = "^26.7.17"`, which would not reproduce if
  that version had actually been published for real. Local cargo registry cache also has no
  `26.7.17` artifacts for either crate.

**No violations found.**

---

## Original 5 gaps — closed vs. open

The task context named 5 gaps from the prior operational assessment, mapped onto 4 parallel
tracks (B and E share one gap pair):

1. **NO_LEAVES-only refusal violates state-byte-invariance (Invariant 5)** — Track A —
   **CLOSED.** Real branchless fix in `src/allocator.rs`'s `has_refusal` gate; generalized
   regression test covers all 8 declared `RefusalSet` variants, not just the one found; the
   previously-defective assertion was flipped to the correct one, not deleted or weakened.
2. **External-auditor actuation gap (private digest field blocks driving `apply_mode_switch`
   from outside the crate)** — Track C — **CLOSED.** New `ActuationEvidence` public artifact,
   `apply_mode_switch` signature changed to compare full receipts, verified to use only public
   API via a real Rust integration-test crate-visibility boundary, not by convention.
3. **No replayable packaging-reality evidence artifact (prior evidence was ad-hoc prose)** —
   Track B — **CLOSED** for its own charter: `scripts/gates/package-reality-check.sh` +
   `cargo make package-reality-check` + a regenerated `PACKAGE_REALITY_RECEIPT.md` now exist,
   reproduced fresh in this session with an honest, correctly-classified result.
4. **Undocumented publication-sequencing decision** — Track E — **CLOSED** for its own charter:
   the ledger's Publication Sequencing Decision section exists, is complete, and is honest about
   what was and was not attempted, confirmed by independent reproduction (no real publish
   occurred).
5. **Mutation-provider generalization (mutant evidence limited to a single hardcoded
   mechanism)** — Track D — **CLOSED for the provider mechanism**, with a **precision caveat**
   on the specific new claim's evidence: the `MutationProvider` trait/`CargoFeatureProvider`
   path is real and was verified live against the actual `bcinr-cmca` crate (7.01s real
   subprocess run, real result); the `cmca-fault-union.toml` claim itself currently reports
   FixtureProvider (local-analogy) evidence, not literal production-crate mutation — worth
   distinguishing precisely, not a defect.

All 5 originally-identified gaps have real, reproduced closing evidence. None required a test
deletion or weakening to reach that state; in one case (Track A) an existing test's assertion
was correctly flipped from a defective-behavior assertion to the correct invariant, verified by
direct reading of both the diff and the passing run.

## New findings this round (beyond the 5 named gaps)

- `--all-features` fails more broadly than `JTBD_READINESS_REPORT_ROUND2.md` recorded — now
  touching 5 test files (10 individual test failures under `--no-fail-fast`), not just 1. Same
  pre-existing root cause (all 11 mutually-corrupting `mutant_N` features enabled at once is not
  a supported build configuration), newly observed against a larger surface because more
  numeric-sensitive JTBD tests now exist. Default features remain fully green. Not attributable
  to Track A or Track C's own changes.
- `PHASE2_RUNTIME_CLOSURE_VERDICT.md`'s two numbered blockers are both stale/superseded on the
  current tree, confirmed by fresh runs in this session, not merely by the task context's
  framing: `consumer_correspondence.rs` passes 3/3, and `cargo make test-mutants` passes 11/11
  with no collateral failures (the `#[cfg(not(feature = "mutant_9/10/11"))]` gates that prevent
  the previously-reported collateral failures already exist in the tree, predating this round).
- `cmca-fault-union.toml`'s "Mutant evidence" is FixtureProvider/analogy-based, not literal
  bcinr-cmca subprocess mutation — a precision distinction, not a defect (see §7).

## Overall standing

Consistent with the task's own framing, and unchanged by this round's evidence: the mandated
sentence remains correct — **"CMCA v26.7.17 is PARTIAL_ALIVE for the pinned bounded
configuration."** This round closes all 5 named gaps (4 fully, 1 — Track D — with a scoped
precision note on which provider backs which claim's evidence) without discovering anything that
would move C4/C6 off the standing the release ledger already carries:

- **C1 (numeric law), C2/C3 (authority/adaptation chain):** unaffected by this round in the
  negative direction — the default full suite (147/147 tests) remains green, including the two
  areas Track A/C touched, and the branchless masked-select style is preserved in both new
  production diffs read in full (`allocator.rs`, `mode_switch.rs`).
- **C4 (semantics/mfw boundary):** unaffected by this round — `cargo make verify-generated`
  passes, unchanged from prior evidence; PARTIAL_ALIVE stands (full SHACL/ShEx/QUDT closure
  remains explicitly fenced, per the ledger's own "Fenced-later-obligations").
- **C6 (object-code/branchlessness):** unaffected by this round — no disassembly audit was
  rerun by any of the 4 tracks or by this verification pass (out of scope for all of them);
  remains UNKNOWN/fenced exactly as `OBJECT_CODE_AUDIT.md` and the ledger's Gate Closure
  Summary already state.
- **G0 (version/metadata)/G1 (packaging hazards):** still open, exactly as the ledger already
  states — Track B's replayable evidence artifact does not itself resolve the
  `bcinr-logic`/`bcinr-cmca` publish-ordering blocker, only makes its state honestly
  re-verifiable (confirmed reproducible in this session).
- **No actual crates.io publish occurred** at any point checked in this session, confirmed by
  two independent methods (§6/§9 above), and zero commits were made by this verification pass
  or observed to have been made by the four sibling tracks during this round.

This report does not itself claim ALIVE/PARTIAL_ALIVE/BLOCKED for the release as a whole beyond
repeating the already-mandated sentence; per `.claude/rules/00-release-governance.md` (consulted
by path, not duplicated here), only `cmca-release-integrator` may emit the terminal
release-completion declaration, and this task is not that role.
