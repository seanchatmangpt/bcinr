# Ecosystem Loop-Until-Dry Cycle 1

**Date:** 2026-08-13
**Branch:** `feat/powl-soundness-cli` (bcinr)

## 1. What "loop-until-dry" means here

Keep spawning parallel gap-finding agents across ecosystem repos in rounds; after each round,
apply any real fixes found, then spawn another round. Stop when a round turns up nothing new
(all finders come back dry) rather than after a fixed round count or agent budget — this way
effort tracks the actual remaining density of gaps in the ecosystem instead of an arbitrary
number of passes.

## 2. This cycle's real results

### Fix 1 — mfw generator `from_value_bits` → `from_bits`

Branch `agent/aggregate-2026-07-30`, commit `e20988ba7305b78ae21a98eb950a6dd49794805e`
(committed, **not pushed**). Root cause: `NonNegativeFixed::from_bits(bits: u32)` and
`SignedFixed::from_bits(bits: i32)` are the real constructors in
`crates/bcinr-cmca/src/fixed.rs`; `from_value_bits` does not exist there. Fixed 4 call sites
in the generator (lines 812, 894, 906, 982).

Verification performed:
- Regenerated `cmca_generated.rs` from the real ontology
  (`python3 generator.py .../cmca-rdf.ttl /tmp/mfw_fix_check`, exit 0) — 0 occurrences of
  `from_value_bits`, `from_bits` used throughout.
- `tools/cmca-generator/tests/test_negative_fixtures.py`: 12/12 PASS.
- Cross-checked against the real, currently-committed equivalent of the stale-path
  `VERIFICATION_REPORT.md` reference:
  `crates/bcinr-cmca/src/generated/consequence_mass/case_studies.rs` — 101/101 occurrences
  are `from_bits`, 0 are `from_value_bits`.
- Ticket updated: `docs/jira/v26.7.18/mfw-cmca-producer/FOLLOWUP-premise-false_2026-08-13.md`,
  recording that MFW-261718-002's premise was false.

### Fix 2 — bcinr `proposal.rs` stale API calls

Commit `c2e4d599` — "fix(cmca): repair proposal.rs's stale from_value_bits calls, resolve its
orphaned-module status".

- Sub-decision 1: fixed all 9 stale `from_value_bits` calls in
  `crates/bcinr-cmca/src/proposal.rs`.
- Sub-decision 2: **left the module unwired**, with a dead-code note added to its module
  doc-comment. Evidence for leaving it unwired: `proposal.rs` imports
  `crate::observatory::ObservatoryFlagSet`, a bitset type removed crate-wide in favor of the
  `ObservatoryFlag` enum; it also calls `SignedFixed::value_bits()`, which never existed.
  Actually attempting to wire it (`pub mod proposal;` in `lib.rs`) reproduced the failure
  directly: `cargo build -p bcinr-cmca --features std` failed with `E0432`
  (unresolved `ObservatoryFlagSet`) plus 3× `E0599` (`value_bits` not found). The wiring was
  reverted; `lib.rs` diff is empty.

Verification performed (module left unwired, as committed):
- `cargo build -p bcinr-cmca --features std` → clean.
- `cargo test -p bcinr-cmca --features std` → all suites 0 failed; one flake
  (`generator_ttl_comment_stripping::unsupported_language_tag_is_still_rejected`) observed
  once, reproduced identically against unmodified HEAD via `git stash`, and passes both in
  isolation and on repeat full-suite runs — pre-existing, unrelated to this change.
- `cargo clippy -p bcinr-cmca --features std -- -D warnings` → clean.
- `cargo fmt -p bcinr-cmca -- --check` → clean.

`crates/bcinr-pddl/src/validate.rs` was left untouched by this fix, per instructions.

### Round 1 findings (ggen, gymact, ggen-marketplace) — not dry, so round 2 ran

- **ggen** (`ggen-architecture` crate): confirmed 2269 lines of orphaned, never-`mod`-declared
  source across 3 files (`building_block.rs` 1103 lines, `self_play.rs` 1109 lines,
  `canonical.rs` 57 lines) sitting in the live `crates/` tree, silently untouched by
  `cargo build`/`cargo check`. `canonical.rs` may duplicate the inline
  `canonical_digest`/`canonicalize` functions `lib.rs` defines directly (lines 1418–1431) —
  worth a drift check. Not investigated further whether the files were ever wired and later
  dropped.
- **gymact**: found that `uv run pytest` in a base (no-`dspy`-extra) environment hard-fails
  collection (`Interrupted: 2 errors during collection`) instead of skipping, because
  `tests/test_dspy_signature_discipline.py` and `tests/test_epistemic_process_kernel_chicago.py`
  transitively hit an unguarded `import dspy` in `src/gymact/epistemic_dspy.py:37` — breaking
  the optional-dependency skip convention the rest of the suite follows. Mechanically fixable
  (guard the import or `importorskip`), not applied in this pass.
- **ggen-marketplace** (`fastmcp-pack`): the pack's `pack.toml` description claims
  "deterministic tool behavior," but the pack ships zero `tests/`/`qualification/` files, and
  its one admission gate (`gates/010_admission.rq`) checks structural naming only — nothing
  touches `fmcp:returnTemplate` content or determinism. The claim isn't obviously false (the
  generated template has no branching/randomness/IO), but nothing in the pack or the
  `qualify_packs.py` tooling actually checks it.

No repo in round 1 came back genuinely dry; all three surfaced real, previously-unreported
findings, which is why round 2 ran.

### Round 2 findings (ggen-legacy, autofde-lab)

- **ggen-legacy**: `docs/v26.8.1/90-legacy/93-capability-equivalence-matrix.md`'s "Disposition
  confidence" section is stale relative to its own generator source of truth
  (`tools/v26.8.1/legacy_archaeology.py`'s `CATALOG`), even though both shipped in the same
  commit (`07928bde`, 2026-08-02). The doc claims 12/15 individuals have a confident
  disposition and 3 remain `DISPOSITION_UNKNOWN`; `grep -c 'disposition="UNKNOWN"'` against the
  generator returns 0 — all 15 already carry confident dispositions as of that commit. This is
  doc-vs-generator drift, not an open `UNKNOWN`-standing gap (the separate `ggen:hasStanding`
  field is correctly still `UNKNOWN` on all 15; only the "Disposition confidence" prose is
  wrong). Cheap fix: regenerate that section's prose from `CATALOG`, or add a CI check for
  doc/CATALOG parity.
- **autofde-lab**: `uv run pytest` is broken by a stale console-script shebang —
  `.venv/bin/pytest` hardcodes `#!/Users/sac/scikit-decide/.venv/bin/python`, a deleted venv
  from a different, unrelated project. This produces `ModuleNotFoundError: No module named
  'autofde_lab'` across all 179 test files. `uv run python3 -m pytest` (module invocation,
  bypassing the broken shebang) collects and runs fine (e.g. `tests/test_utils.py` collects 12
  tests). Root cause confirmed; fix is `uv sync --reinstall` to rewrite the shebang. Not
  applied — this was a read-only gap search.

Both round 2 findings are real and confirmed by direct inspection, not fixed in this pass.

## 3. How to run the next cycle

Repos that have **not** yet had a focused gap-finding pass in this loop (as opposed to a prior,
shallower "medium breadth" check):

- **bcinr itself** — this session fixed one known bug (`proposal.rs`) but never ran a
  dedicated gap-finder round against bcinr's own crates the way round 1/2 did for ggen,
  gymact, ggen-marketplace, ggen-legacy, autofde-lab.
- **praxis** (praxis-graphlaw) — only had a build-failure/mass check, not a focused pass.
- **mfw** — only the generator fix's narrow root-cause investigation, not a focused pass.

Operational definition of "dry" for a future run: **2 consecutive all-repo rounds with zero
new real findings** (not just zero findings from a single round, since round 1 here found
issues in all 3 repos checked while a single quiet repo doesn't mean the ecosystem is dry).

## 4. Honest scope note

This was a small, time-boxed demonstration of the loop-until-dry mechanism — 2 real fixes plus
up to 2 rounds of 2–3 parallel finders each, not an exhaustive ecosystem sweep. Repos never
brought into this session's scope at all: **bcinr** (beyond the one targeted fix),
**unibit**, **dteam**, **ostar**, **wasm4pm**, and any other repos named in the original
2026-08-12 planning doc that this session did not touch. Findings recorded above are real and
verified by direct inspection/command output where stated; nothing here should be read as
"the ecosystem is now gap-free" — only as a record of what this one cycle actually found and
fixed.
