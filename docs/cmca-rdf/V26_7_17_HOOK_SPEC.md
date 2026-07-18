# V26_7_17_HOOK_SPEC — bcinr-cmca Release Control Plane Hook/Enforcement Specification

## Status

This is a **specification only**. No `.claude/settings.json` hook, no `Makefile.toml` (cargo-make)
task, and no repository-relative enforcement script described below has been wired or created by
this document. Every script path named here is a *target path* for a future implementation step,
not an existing file. Nothing in this document has been run.

## Governing Law

Hook invocation frequency is inversely proportional to verification cost. A check that runs on
every keystroke-adjacent tool call (PreToolUse/PostToolUse) must be cheap enough to pay that price
without perceptible latency — grep, path-matching, or single-file AST inspection only. A check that
requires compiling, running a test binary, or disassembling a release artifact may only run at a
frequency proportional to its cost: on the file surface it actually verifies (Level 2), or at an
explicit release checkpoint invoked by a human or the release-integrator agent (Level 3). Placing
an expensive check at a cheap frequency is itself a violation of this spec, independent of whether
the check's content is correct.

This document defines exactly three enforcement levels. Every hook below is classified into exactly
one level. Every hook calls a version-controlled repository-relative script — never inline shell
that duplicates rule prose living elsewhere (rules live in `.claude/rules/*.md`; this document
governs *when* and *how cheaply* those rules are checked, not the rules' content).

## Relationship to Other Control-Plane Artifacts

- Rules the hooks below enforce are stated in `.claude/rules/*.md` (not duplicated here).
- Ownership of the files/gates these hooks protect is partitioned in `.claude/agents/*.md`,
  specifically `cmca-authority.md`, `cmca-numeric.md`, `cmca-semantics.md`, `cmca-verifier.md`,
  and `cmca-release-integrator.md`.
- Skills invoked by Level 2/3 scripts (`mutant-kill-protocol`, `object-code-audit`, `cheat-scan`,
  `evidence-report`) are defined under `.claude/skills/` and are referenced by path, not
  reproduced.
- Constitutional numbering (AGENTS.md §16-17 cheat-scan, §19 mutation protocol, §20 object-code
  audit, §29 evidence artifacts, §31 completion report) governs the *content* of what these hooks
  check; this document governs *trigger placement and cost tier* only.
- Any current pass/fail status, file:line finding, or "N of 11 features covered" fact belongs in
  the release ledger (e.g. `CURRENT_STATUS.md`, `AUDIT_REPORT.md`), not in this specification. Where
  this document notes a known gap (e.g. the mutant task coverage gap), that note is itself a ledger
  fact restated for context and remains REPORTED, not verified, until the extended task described in
  §New Tasks is implemented and independently run.

---

## LEVEL 1 — Immediate Blockers

**Trigger:** PreToolUse / PostToolUse, on every matching tool invocation. **Cost budget:** must
complete without perceptible latency — grep, path pattern match, or single-file AST parse only. No
compilation, no test execution, no workspace-wide scan.

### L1-1: Destructive git command rejection

- **Gate protected:** AGENTS.md fix-forward-only invariant (also stated in
  `/Users/sac/CLAUDE.md` under "CRITICAL: FIX FORWARD ONLY") — commits are immutable, history is
  never rewritten, no destructive git operation runs without explicit human override outside the
  hook path.
- **Trigger:** PreToolUse on Bash tool calls whose command matches a destructive-git pattern.
- **Script:** `scripts/hooks/reject-destructive-git.sh` (repository-relative, version-controlled).
  The hook calls this script with the proposed command string; the script owns the pattern list
  (`reset --hard`, `clean -fdx` / `clean -fd -x` variants, `push --force` / `push -f` without
  `--force-with-lease` to a protected branch, `rebase -i` combined with a public/pushed ref,
  `filter-branch`, `filter-repo`) and returns a block/allow exit code. The hook itself contains no
  inline pattern duplication.
- **Must pass:** `git commit -m "fix: ..."`, `git revert <sha>`, `git push --force-with-lease` to a
  private feature branch the script's allowlist recognizes as non-protected.
- **Must block/flag:** `git reset --hard HEAD~3`, `git clean -fdx`, `git push --force origin main`.

### L1-2: Direct edits to `crates/bcinr-cmca/src/generated/`

- **Gate protected:** generator-authoritative invariant — hand edits to generated output are
  never a valid source of truth; the generator (and its declared ontology input) is the only
  authoritative producer of files under `src/generated/`.
- **Trigger:** PreToolUse on Write/Edit tool calls whose target path matches
  `crates/bcinr-cmca/src/generated/**`.
- **Script:** `scripts/hooks/block-generated-edit.sh`, given the target path and (for Edit) the
  diff; the script owns the path-matching regex and any narrow allowlist (e.g. a generator-owned
  bootstrap commit tagged by an env var the script checks, not by trusting the caller's claim).
- **Must pass:** an edit to `crates/bcinr-cmca/generator.py` or its ontology source file (outside
  `src/generated/`), which regenerates the output through the normal pipeline.
- **Must block/flag:** a Write/Edit call targeting
  `crates/bcinr-cmca/src/generated/fixed.rs` (or any sibling) directly, even for a one-line fix.

### L1-3: New public field or safe-construction path on an authority-bearing type

- **Gate protected:** authority-containment invariant — an authority-bearing type's construction
  surface is closed except through the reviewed, gated constructors already present; adding a new
  `pub` field or a new `pub fn new`/`pub fn from_*`/`impl From` that yields a safe, unchecked
  construction path is a capability-widening change that must not land silently.
- **Trigger:** PostToolUse on Edit/Write to any file under `crates/bcinr-cmca/src/**` that defines
  or `impl`s a type tagged (by doc-comment marker, e.g. `/// AUTHORITY-BEARING`, or a
  project-maintained type list) as authority-bearing.
- **Script:** `scripts/hooks/detect-authority-surface-change.sh`, which does an AST-level diff
  (via `syn`-based parse or equivalent, not regex-over-text) of the type's field list and its
  `impl` block's public function signatures, before vs. after, and flags any addition.
- **Must pass:** adding a private field, or adding a new private helper method, to an
  authority-bearing type.
- **Must block/flag:** adding a new `pub` field to an authority-bearing struct, or adding a new
  `pub fn` constructor/conversion that does not route through the existing gated constructor.

### L1-4: Generated-output digest staleness

- **Gate protected:** generator/output correspondence invariant — every file under
  `src/generated/` embeds a digest of the ontology/generator source it was produced from; that
  digest must match the current declared source at all times the generated file is read as
  authoritative.
- **Trigger:** PostToolUse after any Edit/Write touching either `crates/bcinr-cmca/generator.py`,
  its ontology input file(s), or anything under `src/generated/`; also runnable as a PreToolUse
  check before a tool reads a generated file as an authority source.
- **Script:** `scripts/hooks/check-generated-digest.sh`, which recomputes the digest of the
  declared ontology/generator source and compares it to the digest embedded in the generated
  file's header comment, failing on mismatch.
- **Must pass:** a generated file whose embedded digest equals the freshly computed digest of its
  declared source (i.e. state immediately after a correct regeneration).
- **Must block/flag:** an ontology source file edited without re-running the generator, leaving
  the embedded digest in `src/generated/*.rs` stale relative to the new source content.

---

## LEVEL 2 — Surface-Specific Verification

**Trigger:** on relevant file change (staged-file hook) or pre-commit. **Cost budget:** moderate —
may compile and run a bounded, targeted test/tool invocation scoped to the changed surface; must
not run the full workspace suite or anything release-scale.

### L2-1: Focused numeric tests for `fixed.rs` / `allocator.rs`

- **Gate protected:** numeric-law invariants owned by `cmca-numeric.md` (fixed-point arithmetic
  correctness, allocator bound/overflow safety) — a change to either file must not silently break
  its existing proven numeric properties.
- **Trigger:** pre-commit, when the staged diff touches
  `crates/bcinr-cmca/src/**/fixed.rs` or `crates/bcinr-cmca/src/**/allocator.rs`.
- **Script:** `scripts/hooks/run-numeric-tests.sh`, which invokes
  `cargo test -p bcinr-cmca fixed:: ` / `allocator::` (the script owns the exact target-filter
  list, sourced from a maintained manifest rather than hardcoded twice).
- **Must pass:** a change to `fixed.rs` that keeps all existing fixed-point unit/property tests
  green.
- **Must block/flag:** a change to `allocator.rs` that causes an existing overflow-boundary test
  (e.g. an allocation-at-capacity property test) to fail.

### L2-2: Generator determinism check

- **Gate protected:** generator determinism invariant — running the generator twice from a clean
  state on the same ontology input must produce byte-identical output; this is required for the
  digest scheme in L1-4 to be meaningful and for generated output to be reproducible/auditable.
- **Trigger:** pre-commit, when the staged diff touches `crates/bcinr-cmca/generator.py` or its
  ontology input.
- **Script:** `scripts/hooks/verify-generator-determinism.sh`, which runs the generator twice into
  two temp output directories from a clean checkout state and byte-compares every produced file.
- **Must pass:** two consecutive clean-state runs of the generator producing byte-identical output
  trees.
- **Must block/flag:** a generator change that introduces nondeterminism (e.g. iterating an
  unordered map/set without a stable sort, or embedding a wall-clock timestamp) such that the two
  runs differ.

### L2-3: Compile-fail / trybuild suite

- **Gate protected:** the negative-compilation invariant — code that is supposed to fail to
  compile (e.g. an attempt to construct an authority-bearing type outside its gate, exercised by
  `trybuild`-style `.rs`/`.stderr` fixture pairs) must continue to fail to compile with the
  expected diagnostic.
- **Trigger:** pre-commit, when the staged diff touches any file under `crates/bcinr-cmca/src/**`
  or the compile-fail fixture directory itself.
- **Script:** `scripts/hooks/run-compile-fail-suite.sh`, which invokes the workspace's trybuild
  test binary scoped to `bcinr-cmca`'s compile-fail fixtures only.
- **Must pass:** an unrelated source change that does not affect the compile-fail fixtures,
  leaving the suite's pass/fail set unchanged.
- **Must block/flag:** a change that accidentally makes a fixture that should fail to compile
  start compiling successfully (i.e. the negative test loses its teeth).

### L2-4: Formatting of changed files only

- **Gate protected:** formatting-consistency invariant (`make fmt` per CLAUDE.md) — scoped to
  files actually touched, not a workspace-wide reformat that would produce unrelated diff noise.
- **Trigger:** pre-commit, on every commit touching any `*.rs` file.
- **Script:** `scripts/hooks/fmt-changed-files.sh`, which runs `cargo fmt` restricted to the set
  of staged `.rs` paths (via `--` file arguments or an equivalent scoped invocation) and fails the
  commit if formatting would change any of them.
- **Must pass:** a commit whose staged `.rs` files are already `rustfmt`-clean.
- **Must block/flag:** a commit introducing a staged `.rs` file with non-canonical formatting
  (e.g. inconsistent indentation) that `cargo fmt` would rewrite.

---

## LEVEL 3 — Release Gates

**Trigger:** integration/release checkpoints only, invoked explicitly by the release integrator
(see `cmca-release-integrator.md`) — **never** by a high-frequency PreToolUse/PostToolUse/pre-commit
hook. These are the most expensive checks in the control plane and their cost is exactly why they
sit at the lowest frequency this spec permits.

### L3-1: Full workspace test suite

- **Gate protected:** whole-workspace regression invariant — every crate's test suite passes
  together, not just the surface touched by the current change.
- **Trigger:** explicit invocation by the release integrator at an integration checkpoint.
- **Script:** `scripts/release/run-full-test-suite.sh`, wrapping `cargo test --workspace
  --all-features` (script owns any feature-matrix expansion needed).
- **Must pass:** a release candidate commit where every workspace crate's suite is green.
- **Must block/flag:** a release candidate where any crate outside `bcinr-cmca` regresses due to
  an unnoticed cross-crate effect of the cmca change.

### L3-2: Complete 11-feature mutant suite

- **Gate protected:** the hostile-mutation kill invariant (AGENTS.md §19,
  `mutant-kill-protocol` skill) — every one of the 11 declared `cfg` mutant features must be
  built and its mutants confirmed killed (typed refusal or oracle mismatch), not a subset.
- **Trigger:** explicit invocation by the release integrator at a release checkpoint.
- **Script:** `scripts/release/run-full-mutant-suite.sh`, invoking the mutant-kill-protocol
  machinery across all 11 declared features.
- **Known gap (ledger note, not a claim of current fix):** the existing `Makefile.toml`
  `test-mutants` task wires only `mutant_1..mutant_5`. This is a coverage gap to close — see
  §New Tasks — not a status to obscure. The current pass/fail state of any mutant feature is a
  release-ledger fact (REPORTED until independently reproduced), not stated here.
- **Must pass:** a release candidate where all 11 mutant features build and every injected
  mutant in each is confirmed killed by the protocol's oracle.
- **Must block/flag:** a release candidate where any of the 11 features is either not built at
  all by the invoked task, or built but contains a surviving (undetected) mutant.

### L3-3: Object-code audit

- **Gate protected:** the branchless-in-the-binary invariant (AGENTS.md §7/§13/§20,
  `object-code-audit` skill) — a primitive claimed `BRANCHLESS_ALIVE` must be branchless in the
  compiled release artifact, not merely in source.
- **Trigger:** explicit invocation by the release integrator at a release checkpoint.
- **Script:** `scripts/release/audit-object-code.sh` (does not exist today; this spec defines its
  required behavior for a future implementation step). It must:
  1. Build the release profile (`cargo build --release`, or the workspace's designated release
     invocation) with debug symbols retained for disassembly.
  2. Disassemble the resulting binary/artifact (e.g. via `objdump`/`cargo-show-asm` or equivalent)
     for each authoritative symbol named in a maintained manifest (the set of functions claimed
     branchless/authority-bearing).
  3. Per symbol, classify every instruction-level control-flow point into: conditional branches,
     loop back-edges, panic paths (calls into panic/unwind machinery), and allocator calls.
  4. Emit a per-symbol audit table (symbol name, counts per category, and the specific
     instruction offsets for any conditional branch found) to a version-controlled artifact path
     the release ledger can cite.
  5. Fail (non-zero exit) if any symbol claimed branchless contains a classified conditional
     branch, or if any symbol claimed allocation-free contains a classified allocator call.
- **Must pass:** a release build where every symbol in the manifest disassembles with zero
  conditional branches (only unconditional jumps/branchless CMOV-style sequences) and no
  allocator calls where none are declared.
- **Must block/flag:** a release build where a symbol claimed branchless in source compiles down
  to object code containing a conditional branch (e.g. the compiler failed to elide a
  predicated-store pattern), or a symbol claimed allocator-free calls into the allocator.

### L3-4: `cargo package`

- **Gate protected:** the packaging invariant — the crate must package cleanly under the
  registry's rules (manifest completeness, included-file set correctness) before any publish
  attempt.
- **Trigger:** explicit invocation by the release integrator, after L3-1/L3-2/L3-3 pass.
- **Script:** `scripts/release/package-crate.sh`, wrapping `cargo package -p bcinr-cmca` (and any
  sibling crates the release covers).
- **Must pass:** a manifest with all required metadata (`license`, `description`, `repository`,
  etc.) and a file set that packages without warning.
- **Must block/flag:** a manifest missing a required field, or a package attempt that silently
  excludes a file the crate needs at build time (e.g. an unlisted `include`).

### L3-5: `cargo publish --dry-run`

- **Gate protected:** the publish-readiness invariant — the final pre-flight check that a
  publish would succeed against the real registry rules, without actually publishing.
- **Trigger:** explicit invocation by the release integrator, as the last release-gate step.
- **Script:** `scripts/release/publish-dry-run.sh`, wrapping
  `cargo publish -p bcinr-cmca --dry-run`.
- **Must pass:** a packaged crate whose dry-run reports no registry-side rejection (version
  collision, missing required field, disallowed dependency spec).
- **Must block/flag:** a dry-run failure such as attempting to publish a version number that
  already exists on the registry, or a dependency pinned to a path/git source the registry
  rejects.

---

## New cargo-make Tasks This Spec Implies

These tasks are **not created or edited in this step**. They are named here so the release
ledger and the implementing PR can track them as concrete follow-up work, each backed by one of
the scripts above.

1. **`verify-generated`** — must invoke `scripts/hooks/verify-generator-determinism.sh` (L2-2)
   followed by `scripts/hooks/check-generated-digest.sh` (L1-4) in sequence, so a single task
   confirms both that the generator is deterministic and that checked-in generated output matches
   its declared source. Intended as a pre-commit-scale task, runnable standalone by a developer.

2. **`audit-object-code`** — must invoke `scripts/release/audit-object-code.sh` (L3-3) against
   the release build and fail the task if the emitted per-symbol audit table contains any
   disallowed classification (conditional branch on a claimed-branchless symbol, allocator call
   on a claimed-allocation-free symbol). Release-scale only; must not be added to any
   pre-commit or PreToolUse/PostToolUse hook path.

3. **`test-mutants` (extended)** — must supersede the current task's `mutant_1..mutant_5` scope
   with all 11 declared `cfg` mutant features, invoking
   `scripts/release/run-full-mutant-suite.sh` (L3-2). Closing the currently-known 5-of-11 gap is
   the explicit purpose of this task; until it exists and has been run, any claim that "the mutant
   suite passes" for features 6-11 is UNVERIFIED and belongs in the release ledger with that
   status, not asserted as fact in this document or elsewhere.

## References

- `/Users/sac/bcinr/AGENTS.md` — constitutional numbering for cheat-scan (§16-17), mutation
  protocol (§19), object-code audit (§7/§13/§20), evidence artifacts (§29), completion report
  (§31)
- `/Users/sac/bcinr/.claude/agents/cmca-authority.md`,
  `/Users/sac/bcinr/.claude/agents/cmca-numeric.md`,
  `/Users/sac/bcinr/.claude/agents/cmca-semantics.md`,
  `/Users/sac/bcinr/.claude/agents/cmca-verifier.md`,
  `/Users/sac/bcinr/.claude/agents/cmca-release-integrator.md` — ownership partitions these
  hooks protect
- `/Users/sac/bcinr/.claude/skills/mutant-kill-protocol/SKILL.md`,
  `/Users/sac/bcinr/.claude/skills/object-code-audit/SKILL.md`,
  `/Users/sac/bcinr/.claude/skills/cheat-scan/SKILL.md`,
  `/Users/sac/bcinr/.claude/skills/evidence-report/SKILL.md` — skill implementations Level 2/3
  scripts should call into rather than reimplement
- `/Users/sac/bcinr/docs/cmca-rdf/CURRENT_STATUS.md`,
  `/Users/sac/bcinr/docs/cmca-rdf/AUDIT_REPORT.md` — release-ledger location for any mutable
  status this spec's checks produce
