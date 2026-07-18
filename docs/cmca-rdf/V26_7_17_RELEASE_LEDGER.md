# V26.7.17 CMCA Release Execution Ledger

Version tag: 26.7.17 | Branch: `recovery/cmca-v26.7.17-c2` | Last updated: 2026-07-17

This is the mutable release ledger for the bcinr-cmca v26.7.17 release gate. Per the
control-plane architecture rule, this is the ONLY artifact permitted to carry file:line
references, current defect status, and mutable release progress. Rules live in
`.claude/rules/*.md` (timeless invariants only); agent authority lives in
`.claude/agents/*.md`; enforcement mechanics live in `V26_7_17_HOOK_SPEC.md`.

**Standing discipline:** every entry below is tagged **REPORTED**. REPORTED means: derived
from a live-repo Explore-agent survey (5 surveys of `/Users/sac/bcinr` on this branch), not
yet mechanically reproduced by an independent verifier (`cmca-verifier` or
`cmca-release-integrator` re-running the cited command against the cited file:line). No
entry here may be read as CONFIRMED, ALIVE, or settled until that reproduction happens and
the entry is re-tagged with the reproducing command's actual output.

Ownership matrix used below: G0/G1/G7/G8/G9 → `cmca-release-integrator`; G2 →
`cmca-numeric`; G3/G4 → `cmca-authority`; G5 → `cmca-semantics`; G6 → `cmca-verifier`.
Verifier column is `cmca-verifier` for every gate except G6, whose reproduction is done by
`cmca-release-integrator` (a verifier cannot self-verify its own gate).

Consult, do not duplicate: `.claude/agents/hoare-oracle.md`, `.claude/agents/turing-machine.md`,
`.claude/agents/armstrong-fault.md`, `.claude/agents/von-neumann-bypass.md`, `AGENTS.md`.

---

## G0 — Release Identity

- **Owner:** cmca-release-integrator
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** the workspace and crate version strings do not read `26.7.17` at tag time,
  or a publishable crate lacks `publish = false` where required, or required package
  metadata is missing at publish time.
- **Sub-obligations (REPORTED, needing reproduction):**
  1. `crates/bcinr-cmca/Cargo.toml` version reads `26.6.24`, not `26.7.17`. REPORTED —
     reproduce via `grep '^version' crates/bcinr-cmca/Cargo.toml`.
  2. `crates/bcinr-cmca/Cargo.toml` has no `publish = false`; crate is publishable as-is.
     REPORTED — reproduce via `cargo publish --dry-run -p bcinr-cmca` (do not actually
     publish).
  3. `crates/bcinr-cmca/Cargo.toml` is missing `readme`/`repository`/`keywords` fields.
     REPORTED — reproduce via manual diff against a template `Cargo.toml` with those fields.
  4. Path dependency `bcinr-logic` is itself version `26.6.24`, also publishable (no
     `publish = false`). REPORTED — reproduce via `grep '^version' crates/bcinr-logic/Cargo.toml`.
  5. `rust-toolchain.toml` pins `nightly` while both crates declare `rust-version = "1.70"`
     — a stated-vs-pinned toolchain tension. REPORTED — reproduce via reading
     `rust-toolchain.toml` next to both `Cargo.toml` `rust-version` fields.
- **Required evidence/commands:** `cargo publish --dry-run -p bcinr-cmca`,
  `cargo publish --dry-run -p bcinr-logic`, `grep -rn '^version' crates/bcinr-cmca/Cargo.toml
  crates/bcinr-logic/Cargo.toml`, diff of `rust-toolchain.toml` vs `rust-version` fields.
- **Blocker:** version bump and toolchain-pin decision are open (see Hypotheses section
  below) — G0 cannot close until `cmca-release-integrator` decides and the decision is
  independently reproduced.

## G1 — Workspace/Packaging Hazards

- **Owner:** cmca-release-integrator
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** `cargo publish --dry-run` fails, or a dirty-tree scratch file is packaged
  into the release tarball, or the `[patch.crates-io]` override silently changes resolved
  dependency behavior between dev and release builds.
- **Sub-obligations (REPORTED, needing reproduction):**
  1. Workspace root `Cargo.toml` declares `[patch.crates-io] wasm4pm-compat` pointing to an
     absolute external path `/Users/sac/wasm4pm-compat`. REPORTED — reproduce via
     `grep -A2 '\[patch.crates-io\]' Cargo.toml`.
  2. Dirty-tree risk files present at repo/crate root: `crates/bcinr-cmca/src/allocator.rs.orig`
     (stale backup), `err_list.txt`, `errors.json`, assorted root patch/fix/resolve scratch
     scripts. REPORTED — reproduce via `git status --porcelain` and `find . -maxdepth 2
     -name '*.orig' -o -name 'err_list.txt' -o -name 'errors.json'`.
  3. Loose root artifacts `bcinr-cmca.s`, `cmca_dump.txt`, empty `objdump.txt` exist outside
     any documented audit pipeline. REPORTED — reproduce via `ls -la bcinr-cmca.s
     cmca_dump.txt objdump.txt` at repo root.
- **Required evidence/commands:** `cargo publish --dry-run -p bcinr-cmca` on a machine
  without `/Users/sac/wasm4pm-compat` present (to test whether the absolute-path patch
  blocks a foreign-machine dry run), `git status --porcelain`, `git clean -ndx` (dry-run
  only, list untracked/ignored files, do not execute a real clean).
- **Blocker:** whether the absolute-path patch blocks dry-run packaging on another machine
  is UNVERIFIED — see Hypotheses section; G1 cannot close until reproduced on a clean
  checkout.

## G2 — Numeric Law (fixed-point, allocator, floor)

- **Owner:** cmca-numeric
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** a masked/branchless payload-select changes the selected value's error
  channel in a way inconsistent with the pre-select fault of the *selected* operand; or
  `const_eq_u32` produces different result types/semantics at two call sites for the same
  logical predicate; or the floor computation's output violates the conservation identity
  it is claimed to satisfy (none currently proven — see below).
- **Sub-obligations (REPORTED, needing reproduction), all in `crates/bcinr-cmca/src/fixed.rs`
  and `crates/bcinr-cmca/src/allocator.rs`:**
  1. `NonNegativeFixed { pub val, pub err }` and `SignedFixed` (same shape) expose the fault
     channel `err` as a public field — no encapsulating `NumericFaultSet`/`RefusalSet` type
     exists anywhere in the crate. REPORTED — reproduce via `grep -n 'pub err' fixed.rs` and
     `grep -rn 'NumericFaultSet\|RefusalSet' crates/bcinr-cmca/src/`.
  2. `err: u32::MAX` is the OK sentinel; `branchless_err_acc` is documented/observed as
     FIRST-ERROR-WINS, not a union of faults. REPORTED — reproduce via reading
     `branchless_err_acc`'s body and a targeted unit test asserting union vs first-wins
     semantics under two simultaneous fault-producing inputs.
  3. `CanonicalMask { pub val: u32 }` exposes raw bits publicly; no encapsulating type
     enforces canonical-mask invariants at the API boundary. REPORTED — reproduce via
     `grep -n 'pub val' fixed.rs` (CanonicalMask definition).
  4. `allocate(...)` returns `Result<[NonNegativeFixed; N], StabilityRefusal>` — no
     `AllocationOutcome` struct wraps the success/fault channel. REPORTED — reproduce via
     `grep -n 'fn allocate' allocator.rs` and its return-type signature.
  5. `from_bits(const_select_u32(...))` resets `err` to `OK`, observed at approximately 10
     hot-path call sites — a masked payload-select that silently drops the operand's
     pre-existing fault. REPORTED — reproduce via `grep -n 'from_bits(const_select_u32'
     allocator.rs fixed.rs` and count/enumerate the sites, then a targeted test asserting a
     faulted operand's error survives selection.
  6. Floor is a hand-written `LEAF_RECIP` rounded-reciprocal lookup table, not a
     base-q + residual-r decomposition; no conservation proof exists for it. REPORTED —
     reproduce via reading the `LEAF_RECIP` table definition and searching for any
     accompanying proof file/comment (`grep -rn 'LEAF_RECIP' crates/bcinr-cmca/src/`).
  7. When `priced_sum == 0`, code branchlessly substitutes a `1.0` denominator with no
     fault recorded; when `nl == 0`, floor contributes `0` with no panic. REPORTED —
     reproduce via targeted unit tests constructing `priced_sum == 0` and `nl == 0` inputs
     and asserting on the resulting `err` field.
  8. Two conflicting `const_eq_u32` semantics exist: `fixed.rs`'s returns `CanonicalMask`;
     `allocator.rs`'s returns a `0/1 u32`. REPORTED — reproduce via
     `grep -n 'fn const_eq_u32' fixed.rs allocator.rs` and diff the two signatures.
  9. Rejection invariance: a masked-rollback code path exists at approximately
     `allocator.rs:1281-1286`; `case_studies.rs:304` tests `weights`/`last_switch_t`/
     `prev_mode` fields but does not assert full byte-level state equality on rollback.
     REPORTED — reproduce via reading `allocator.rs` lines 1275-1295 and
     `case_studies.rs` line ~304, then a byte-level (`unsafe` transmute-free, e.g.
     `PartialEq`-derived struct comparison or serialized-bytes comparison) rollback test.
- **Required evidence/commands:** targeted `cargo test -p bcinr-cmca` cases per
  sub-obligation above; `cargo asm`/`objdump` is NOT required for G2 (that is G6); line
  numbers above are REPORTED approximate locations from survey, to be pinned exactly by the
  reproducing verifier run.
- **Blocker:** no `NumericFaultSet`/`RefusalSet` type exists — closing sub-obligations 1, 3,
  4 requires a design decision (new type vs. documented-and-tested current shape) before
  `cmca-numeric` can propose a fix; conservation proof for the floor (sub-obligation 6) has
  no existing artifact to check against.

## G3 — Authority: Certification and Sealing

- **Owner:** cmca-authority
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** any code path outside the sealed constructors (`admit_*` functions) can
  produce a `CertificateReceipt` or other sealed authority type; or `Observatory`'s
  `evaluate_calibration` mints a certificate as a side effect of evaluation rather than
  through a distinct, separately-verifiable seal step.
- **Sub-obligations (REPORTED, needing reproduction), in
  `crates/bcinr-cmca/src/observatory.rs` and `allocator.rs`:**
  1. `Observatory::evaluate_calibration` returns
     `Result<CertificateReceipt, ObservatoryFlag>` — it mints certificates directly inside
     evaluation, which is the specific violation under review; result carries a single
     priority flag, not an `ObservatoryFlagSet`/`ObservatoryOutcome`. REPORTED — reproduce
     via `grep -n 'fn evaluate_calibration' observatory.rs` and reading its body for a
     direct `CertificateReceipt` construction.
  2. No `seal_certificate` function exists anywhere in the crate. REPORTED — reproduce via
     `grep -rn 'fn seal_certificate' crates/bcinr-cmca/src/`.
  3. `admit_adaptive_update` binds approximately 0/11 required categories — only a
     4-receipt digest equality check plus 2 profile scalars (temperature ceiling,
     distinguishability floor) are enforced. REPORTED — reproduce via reading
     `admit_adaptive_update`'s body and enumerating exactly which of the 11 categories (as
     specified in the constitutional/AGENTS.md authority model) are checked vs. asserted by
     survey.
- **Required evidence/commands:** `grep -rn 'CertificateReceipt\|ObservatoryFlag\|
  seal_certificate' crates/bcinr-cmca/src/`, a targeted test attempting to construct a
  `CertificateReceipt` from outside `observatory.rs`'s module boundary (should fail to
  compile — trybuild `tests/ui/*.rs` may already partially cover this; confirm which).
- **Blocker:** whether the 3 existing trybuild UI compile-fail tests + 5 compile_fail
  doctests already cover the "cannot mint outside sealed constructor" property, or only
  cover field-privacy, is UNVERIFIED — needs enumeration against the 11-category model
  before `cmca-authority` can claim partial coverage.

## G4 — Authority: Mode-Switch Lifecycle (Proposal/Shadow/Jump/Stability/Dwell)

- **Owner:** cmca-authority
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** a mode switch is observed to occur through any path other than a typed,
  constructor-admitted token; or a dwell/cooldown condition is bypassed by direct field
  mutation rather than through a checked constructor.
- **Sub-obligations (REPORTED, needing reproduction):**
  1. The following files are ABSENT from the crate: `proposal.rs`, `shadow.rs`, `jump.rs`,
     `stability.rs`, `certification.rs`, `mode_switch.rs`. REPORTED — reproduce via
     `find crates/bcinr-cmca/src -iname 'proposal.rs' -o -iname 'shadow.rs' -o -iname
     'jump.rs' -o -iname 'stability.rs' -o -iname 'certification.rs' -o -iname
     'mode_switch.rs'` (expect empty).
  2. The following types are ABSENT anywhere in the crate: `ModeProposal`,
     `AdmittedProposal`, `ShadowExecutionReceipt`, `JumpAnalysisReceipt`,
     `StabilityCandidate`, `DwellSatisfied`, `CertifiedModeSwitch`. REPORTED — reproduce
     via `grep -rn 'ModeProposal\|AdmittedProposal\|ShadowExecutionReceipt\|
     JumpAnalysisReceipt\|StabilityCandidate\|DwellSatisfied\|CertifiedModeSwitch'
     crates/bcinr-cmca/src/` (expect empty).
  3. The following sealed types DO exist with private fields and `admit_*` constructors:
     `CertificateReceipt`, `AdmittedControlState`, `EnvelopeReceipt`, `OutcomeReceipt`,
     `CertifiedLearning`, `CertifiedSelectionOnly`, `AdaptiveUpdate<Mode>`. REPORTED —
     reproduce via `grep -rn 'struct CertificateReceipt\|struct AdmittedControlState\|
     struct EnvelopeReceipt\|struct OutcomeReceipt\|struct CertifiedLearning\|
     struct CertifiedSelectionOnly\|struct AdaptiveUpdate' crates/bcinr-cmca/src/` plus
     field-visibility check on each.
  4. Dwell is an inline boolean (`can_switch` check) at the point of use, not a distinct
     type. REPORTED — reproduce via `grep -n 'can_switch' allocator.rs`.
  5. Mode switching itself is an inline masked-select inside `allocate()`, not routed
     through a `CertifiedModeSwitch` token. REPORTED — reproduce via reading `allocate()`'s
     body in `allocator.rs` for the mode-select expression.
  6. No broker/actuation surface exists anywhere in the crate — "slow-rail nonactuation" is
     structurally not applicable because there is nothing to actuate. This is recorded
     honestly as a scope-boundary fact, not fabricated as a passing broker check. REPORTED
     — reproduce via `grep -rn 'broker\|actuat' crates/bcinr-cmca/src/` (expect empty or
     only comments).
- **Required evidence/commands:** the `find`/`grep` commands above, run against a clean
  checkout of this branch by an independent verifier.
- **Blocker:** the full proposal→shadow→jump→stability→dwell→certified-switch lifecycle
  described in the constitutional model has no corresponding types in this codebase at all;
  closing G4 requires either implementing that lifecycle or a scope decision that the
  inline-masked-select design is the accepted v26.7.17 shape (open policy question, not
  decided here).

## G5 — Semantics: RDF/Ontology Generator

- **Owner:** cmca-semantics
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** the generator silently produces a non-zero, non-refused numeric output for
  an input that should have triggered a refusal (missing required property, cyclic
  consequence-mass reference, malformed Turtle line); or two independent runs of the
  generator over the same ontology input produce different generated Rust source bytes.
- **Sub-obligations (REPORTED, needing reproduction), in
  `crates/bcinr-cmca/generator.py` and `crates/bcinr-cmca/ontology/*.ttl`:**
  1. `validate_shapes` uses Python `assert` for semantic validation, which is stripped
     under `python -O`. REPORTED — reproduce via `grep -n 'assert' generator.py` in
     `validate_shapes` and a run comparison `python generator.py` vs `python -O
     generator.py` over a deliberately-invalid input.
  2. Pervasive `.get(subject, {}).get(prop, 0-or-0.0)` fallback pattern — a missing required
     property silently becomes zero rather than triggering refusal. REPORTED — reproduce
     via `grep -n '\.get(.*{}).get(' generator.py` and a targeted input omitting a required
     property, asserting on generated output.
  3. `get_consequence_mass` cycle detection returns `0.0` silently on a detected cycle, no
     refusal raised. REPORTED — reproduce via `grep -n 'def get_consequence_mass'
     generator.py` and a constructed cyclic-reference ontology fixture.
  4. Q16.16 fixed-point conversion is `float(val_str)` then `int(round(val * 65536))` —
     binary float, not `Decimal`, introducing rounding-path risk. REPORTED — reproduce via
     `grep -n 'round(.*65536)' generator.py` and a differential test against a `Decimal`-
     based reference conversion for boundary values.
  5. The custom line-based Turtle parser loudly refuses (raises `ValueError`) unsupported
     constructs — multiline literals, language tags, blank nodes, collections, named
     graphs, non-prefixed IRIs, repeated properties — but silently continues past malformed
     lines with fewer than 3 parts. REPORTED — reproduce via constructing a `.ttl` fixture
     with a 2-part malformed line and asserting the parser does not raise.
  6. No `AdmissionError` type exists; refusals are string-coded `ValueError`s (e.g.
     `CMCA_OBJECT_COUNT_EXCEEDED`). REPORTED — reproduce via `grep -n 'raise ValueError'
     generator.py`.
  7. No formula-registry / `UniformLeafFloor` / `WeightedLeafCountFloor` identity exists in
     `ontology/*.ttl`. REPORTED — reproduce via `grep -rn 'UniformLeafFloor\|
     WeightedLeafCountFloor' crates/bcinr-cmca/ontology/`.
  8. Generation is manual — no `build.rs`, not wired into `Makefile.toml`; generated output
     is checked into git; generation is already observed deterministic (sorted iteration,
     no timestamps/host paths in output). REPORTED — reproduce via `grep -n 'generator'
     Makefile.toml` (expect no hit) and two successive `python generator.py` runs diffed
     byte-for-byte.
  9. Generated headers carry `RDF_INPUT_DIGEST` and `GENERATOR_SOURCE_DIGEST`; no
     numeric-profile digest is present. REPORTED — reproduce via `grep -n 'DIGEST'
     crates/bcinr-cmca/src/generated/*.rs`.
  10. `src/generated/stability_profile.rs` has different, unexplained provenance — it is
      not produced by `generator.py`. REPORTED — reproduce via checking for
      `RDF_INPUT_DIGEST`/`GENERATOR_SOURCE_DIGEST` headers in that specific file (absence
      would corroborate) and `git log --follow` on the file.
  11. Acronym conflict: `lib.rs` documents CMCA as "Covariance Monitoring and Calibration
      Assessment"; the ontology/generator models consequence-mass allocation over semantic
      objects — two unrelated domains sharing one acronym. REPORTED — reproduce via
      `grep -n 'Covariance Monitoring\|Calibration Assessment' crates/bcinr-cmca/src/lib.rs`.
- **Required evidence/commands:** `python crates/bcinr-cmca/generator.py` run twice with
  byte-diff, targeted fixture `.ttl` files per sub-obligation, `grep`/`find` commands above.
- **Blocker:** whether the acronym conflict (sub-obligation 11) is a documentation defect
  or a genuine scope/design ambiguity is a policy question for `cmca-release-integrator`,
  not resolvable by `cmca-semantics` alone.

## G6 — Verifier: Object-Code / Branchlessness Closure

- **Owner:** cmca-verifier
- **Verifier:** cmca-release-integrator (reproduction) — G6 is the one gate where the
  Verifier column deviates: `cmca-verifier` cannot self-verify its own gate, so
  `cmca-release-integrator` independently reproduces G6's claims.
- **Standing:** REPORTED
- **Falsifier:** an `objdump`/`cargo-asm` disassembly of the release build shows a
  conditional branch on a value derived from a claimed-branchless input, for any function
  claiming `BRANCHLESS_ALIVE` standing.
- **Sub-obligations (REPORTED, needing reproduction):**
  1. No object-code audit tooling exists — no `objdump`/`cargo-asm` scripts are present in
     the repo; only loose, non-pipelined root artifacts `bcinr-cmca.s`, `cmca_dump.txt`, and
     an empty `objdump.txt`. REPORTED — reproduce via `find . -maxdepth 1 -iname
     '*.s' -o -iname 'cmca_dump.txt' -o -iname 'objdump.txt'` and `wc -l objdump.txt`
     (expect 0).
  2. `Makefile.toml` tasks `scan-cheats`, `contract-gate`, `test-mutants` exist;
     `test-mutants` only wires `mutant_1..mutant_5` of the 11 declared mutant features (not
     all 11). REPORTED — reproduce via `grep -n 'mutant_' Makefile.toml
     crates/bcinr-cmca/Cargo.toml` and counting declared vs. wired features.
  3. `ci` task's dependency list does NOT include `test-mutants`. REPORTED — reproduce via
     reading the `[tasks.ci]` `dependencies` array in `Makefile.toml`.
  4. Tasks `audit-object-code` and `verify-generated` are declared nowhere in
     `Makefile.toml`. REPORTED — reproduce via `grep -n 'audit-object-code\|
     verify-generated' Makefile.toml` (expect empty).
  5. `trybuild` dev-dependency plus `tests/compile_fail_tests.rs` and `tests/ui/*.rs` exist;
     `tests/hostile_mutants.rs` exists with `mutant_1..11` cfg features plus hand mutants
     `m01..m07`. No `MUTANT_KILL_MATRIX.md` file exists anywhere in the repo. REPORTED —
     reproduce via `find . -iname 'MUTANT_KILL_MATRIX.md'` (expect empty) and
     `grep -c 'mutant_' crates/bcinr-cmca/tests/hostile_mutants.rs`.
  6. A counting (non-aborting) global allocator exists only in `bcinr-pddl` behind a
     `dhat-heap` feature — it is NOT present in `bcinr-cmca`. REPORTED — reproduce via
     `grep -rn 'dhat-heap\|GlobalAlloc' crates/bcinr-pddl/ crates/bcinr-cmca/`.
- **Required evidence/commands:** the `mutant-kill-protocol` and `object-code-audit`
  skills' own procedures (`.claude/skills/mutant-kill-protocol/SKILL.md`,
  `.claude/skills/object-code-audit/SKILL.md`) must actually be run against a release build
  before G6 can move past REPORTED; this ledger records that they have NOT yet been run in
  this task.
- **Blocker:** no object-code audit tooling exists to run — G6 is currently un-runnable as
  specified until `object-code-audit` tooling is built or the skill's manual procedure is
  executed by hand.

## G7 — Documentation/Standing Consistency

- **Owner:** cmca-release-integrator
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** a status document claims a standing (e.g. ALIVE, 100/100 SIS) for a gate
  that this ledger's own REPORTED entries contradict, without cross-reference.
- **Sub-obligations (REPORTED, needing reproduction):**
  1. `docs/cmca-rdf/CURRENT_STATUS.md` currently claims "100/100 SIS" for C4 (semantics/
     generator) only, and does not mention C1 (numeric), C2/C3 (authority), or C6
     (object-code) at all. REPORTED — reproduce via reading
     `docs/cmca-rdf/CURRENT_STATUS.md` in full and diffing its scope against this ledger's
     G2-G6 coverage.
  2. No v26.7.17 release document existed prior to this ledger. REPORTED — reproduce via
     `git log --follow -- docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` (expect this file's own
     initial commit as the only history at time of writing).
- **Required evidence/commands:** full read of `CURRENT_STATUS.md`, `BASELINE.md`,
  `AUDIT_REPORT.md`, `ARCHITECTURE.md`, `AGENT_DISPOSITION.md` for cross-consistency against
  this ledger.
- **Blocker:** `CURRENT_STATUS.md` needs a decision — supersede/update in place, or mark
  deprecated in favor of this ledger for v26.7.17 scope — before G7 can close.

## G8 — Constitutional/Config Surface

- **Owner:** cmca-release-integrator
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** a required constitutional artifact (rule, agent definition, skill) is
  referenced by `AGENTS.md` but does not exist on disk, or exists with content that
  violates the control-plane architecture rule (e.g. a rule file containing a file:line
  reference).
- **Sub-obligations (REPORTED, needing reproduction):**
  1. `.claude/rules/` directory does not yet exist in the bcinr repo (distinct from the
     user's global `~/.claude/rules/`). REPORTED — reproduce via
     `ls /Users/sac/bcinr/.claude/rules/ 2>&1` (expect "No such file or directory" at time
     of writing, pending this workflow's other artifacts).
  2. `.claude/agents/hoare-oracle.md`, `turing-machine.md`, `armstrong-fault.md`,
     `von-neumann-bypass.md` already exist (created earlier this session), with tools
     matching `AGENTS.md` §4. REPORTED — reproduce via `ls .claude/agents/` and diffing
     each file's YAML frontmatter `tools:` list against `AGENTS.md` §4.
  3. `.claude/skills/mutant-kill-protocol/SKILL.md`, `object-code-audit/SKILL.md`,
     `cheat-scan/SKILL.md`, `evidence-report/SKILL.md` already exist. REPORTED — reproduce
     via `find .claude/skills -name SKILL.md`.
- **Required evidence/commands:** `ls`/`find` commands above against a clean checkout.
- **Blocker:** none identified beyond standard reproduction; G8 is expected to close once
  an independent verifier confirms the `ls`/`find` outputs match.

## G9 — Standing/Evidence Rollup

- **Owner:** cmca-release-integrator
- **Verifier:** cmca-verifier
- **Standing:** REPORTED
- **Falsifier:** the release is declared with any standing stronger than PARTIAL_ALIVE, or
  the mandated final-standing sentence (below) is altered, or the literal token
  `CMCA_RDF_PARTIAL_ALIVE` is claimed present somewhere it does not actually appear.
- **Sub-obligations (REPORTED, needing reproduction):**
  1. `ORIGINAL_REQUEST.md` (repo root) mandates the exact final standing sentence quoted in
     the Mandated-final-standing section below. REPORTED — reproduce via
     `grep -n 'PARTIAL_ALIVE' ORIGINAL_REQUEST.md`.
  2. The literal token `CMCA_RDF_PARTIAL_ALIVE` does not exist in any file in the repo as
     of this ledger's writing. REPORTED — reproduce via
     `grep -rn 'CMCA_RDF_PARTIAL_ALIVE' /Users/sac/bcinr` (expect empty, or only this
     ledger once it is committed, which would then need updating to note its own mention is
     not itself the establishing artifact).
  3. This ledger (G0-G8 above) is the aggregation point; no gate above is CONFIRMED — every
     one is REPORTED pending independent reproduction. This is itself the G9 rollup fact.
- **Required evidence/commands:** the `evidence-report` skill's checklist
  (`.claude/skills/evidence-report/SKILL.md`) run against this ledger before any
  completion report is issued for v26.7.17.
- **Blocker:** G9 cannot close (i.e. the release cannot be declared PARTIAL_ALIVE with
  standing evidence) until G0-G8's REPORTED entries are independently reproduced by
  `cmca-verifier` and/or `cmca-release-integrator` and re-tagged accordingly.

---

## Fenced-later-obligations (explicitly OUT of v26.7.17 scope)

The following are named so they are not silently assumed closed by this ledger's silence.
They are fenced OUT of v26.7.17 unless separately implemented and independently verified
before the release is tagged:

1. **Full SHACL/ShEx/unit closure** over the ontology inputs — this ledger's G5 entries
   cover the generator's observed behavior on ad hoc fixtures, not a complete shape-
   validation closure proof.
2. **Full C6 physical object-code closure** — if still UNKNOWN at release time (see G6
   blocker: no audit tooling exists to run), it remains UNKNOWN, not assumed passing.
3. **Product/equal-budget proof** for the allocator's numeric claims — no such proof
   artifact was found in survey; not fenced in.
4. **Distributed/security closure** — no distributed-systems or security-boundary review
   was performed as part of this survey; explicitly out of scope for v26.7.17.

## Mandated-final-standing

`ORIGINAL_REQUEST.md` (repo root) mandates the following exact final standing sentence for
this release:

> CMCA v26.7.17 is PARTIAL_ALIVE for the pinned bounded configuration.

REPORTED note: the literal token `CMCA_RDF_PARTIAL_ALIVE` does not yet exist in any file in
the repository as of this ledger's writing. Its absence is itself a REPORTED fact requiring
reproduction (`grep -rn 'CMCA_RDF_PARTIAL_ALIVE' /Users/sac/bcinr`), not an assumption.

## Hypotheses-for-the-release-integrator

The following are hypotheses for `cmca-release-integrator` to decide — they are NOT settled
policy and nothing in this ledger should be read as having decided them:

1. **Version-bump synchronization hypothesis:** should `bcinr-cmca` and `bcinr-logic` share
   a synchronized version bump to `26.7.17` (both currently `26.6.24`), or may they diverge
   (e.g. only `bcinr-cmca` bumps while `bcinr-logic` stays at its own cadence)? Open.
2. **Packaging-hazard hypothesis:** does the workspace `[patch.crates-io]` absolute-path
   override for `wasm4pm-compat` (pointing at `/Users/sac/wasm4pm-compat`) actually block
   `cargo publish --dry-run` packaging on a machine other than this one, or is it inert for
   dry-run purposes and only a hazard at actual publish time? Open — untested on a foreign
   checkout as of this ledger.

## Architecture Correction (Accepted)

This section records a decisive, accepted architecture call amending v26.7.17 scope: CMCA
RDF/ontology admission and generation moves out of `bcinr-cmca` and into `/Users/sac/mfw`
("mfw"); `bcinr-cmca` becomes a pure generated-artifact consumer, connected to mfw via a
deterministic digest-bound ARTIFACT boundary, not a Cargo dependency. Everything below is
tagged REPORTED — this amendment does not move any code; it records the decision, splits G5's
gate, and starts migration steps 1-2 only.

### G5 split into three sub-obligations

The former single G5 ("Semantics: RDF/Ontology Generator") gate is split into three
independently-owned, independently-falsifiable sub-obligations. G5's existing 11
sub-obligations above remain REPORTED findings against the current (pre-migration)
`bcinr-cmca` generator and are not retired by this split — they describe the state being
migrated away from.

#### C4_mfw_admission

- **Owner:** cmca-semantics
- **Standing:** REPORTED
- **Falsifier:** an RDF graph that mfw's `mfw-meaning` admits as valid contains a
  SHACL/ShEx-shape violation, a malformed Turtle construct, or a semantic defect (missing
  required property, cyclic consequence-mass reference) that the admission step should have
  refused; or two independent admission runs over the same input graph produce different
  admitted-graph digests.
- **Required evidence:** a run of mfw's admit-graph → validate → receipt pipeline
  (`mfw-meaning`, oxigraph + praxis-graphlaw SHACL/ShEx/N3) over the CMCA ontology inputs,
  producing a BLAKE3 digest of the admitted graph, reproducible by an independent verifier
  re-running the same command against the same input files.

#### C4_projection

- **Owner:** cmca-semantics
- **Standing:** REPORTED
- **Falsifier:** generation from the admitted graph to the `Gamma_CMCA` artifact is observed
  non-deterministic across two runs on identical input (different output bytes for the same
  admitted-graph digest), or two distinct admitted graphs are observed to project to the same
  `Gamma_CMCA` artifact (a non-injective collision).
- **Required evidence:** two independent generation runs from the same admitted-graph digest,
  byte-diffed for determinism; a documented argument or test corpus supporting injectivity
  (distinct admitted graphs -> distinct artifacts), reproducible by an independent verifier.

#### C4_bcinr_consumption

- **Owner:** cmca-verifier (verifies all three sub-obligations); packaging of the consumed
  artifact into `bcinr-cmca` is executed by `cmca-numeric`/`cmca-authority` via
  `cmca-release-integrator` packaging, per the existing G2/G3/G4 numeric/authority ownership.
- **Standing:** REPORTED
- **Falsifier:** `bcinr-cmca` is observed to parse, reinterpret, or re-derive meaning from raw
  RDF/Turtle/ontology source at build or run time (rather than only verifying and consuming
  the already-generated `Gamma_CMCA` artifact); or the artifact's digest is not checked before
  use; or generated output committed into `bcinr-cmca` does not match the digest recorded for
  the mfw generation run that produced it.
- **Required evidence:** a verifier run confirming `bcinr-cmca` contains no RDF/Turtle parsing
  or ontology-interpretation code path post-migration, only digest verification plus
  consumption of the committed generated artifact; digest-equality check between the
  committed artifact and the mfw-side generation receipt.

### Accepted facts (REPORTED)

1. `praxis-graphlaw` (identified as mfw's strongest RDF/SHACL/ShEx engine) path-depends on
   `bcinr-pddl` and `bcinr-powl`. This is accepted as a release-time/toolchain-graph coupling
   on the producer (mfw) side only — it is explicitly NOT a `bcinr-cmca` runtime dependency.
   REPORTED, pending independent reproduction of the dependency graph.
2. `ggen` is an external, currently-uninstalled CLI whose templates target Lean, not Rust — so
   RDF-to-Rust codegen for CMCA is NOT ready today via `ggen` and is a fenced-later item,
   unless the Python-generator-relocation path is used instead. The Python-generator-
   relocation path (moving the existing, upgraded `generator.py` into mfw) is the path chosen
   for v26.7.17. REPORTED.
3. QUDT/unit validation is absent in mfw and stays fenced — not part of v26.7.17 scope.
   REPORTED.
4. mfw has zero existing CMCA references as of this amendment. REPORTED — reproduce via a
   grep for CMCA-related terms across `/Users/sac/mfw`.

### Migration sequence

The following 10-step sequence is the accepted migration plan. Only steps 1-2 are
in-progress-this-round; steps 3-10 are not-yet-started.

1. Freeze regression fixtures. **(in progress this round)**
2. Define artifact contract (the `Gamma_CMCA` digest-bound boundary between mfw and
   `bcinr-cmca`). **(in progress this round)**
3. Port ontology/generator into mfw. **(not yet started)**
4. Generate byte-equivalent payloads where lawful. **(not yet started)**
5. Resolve known semantic defects in mfw (the G5 sub-obligations 1-11 findings above,
   re-scoped to the mfw-hosted generator). **(not yet started)**
6. Replace bcinr generator ownership with artifact verification. **(not yet started)**
7. Remove RDF/parser dependencies from bcinr. **(not yet started)**
8. Quarantine/delete old bcinr semantic source only after correspondence passes. **(not yet
   started)**
9. Continue C1-C3 mechanics (numeric, authority) in bcinr unchanged. **(not yet started as a
   distinct migration step — ongoing under existing G2-G4 gates)**
10. Dry-run publish with the split boundary in place. **(not yet started)**

### Terminal release graph (updated)

The release graph description is updated to:

> mfw CMCA generation command -> generated CMCA payload committed into bcinr -> bcinr-logic
> -> bcinr-cmca -> `cargo publish --dry-run`

`mfw` need not be in the Cargo dependency chain, since it is a release-time producer: its
output is a committed, digest-verified artifact, not a compiled dependency of `bcinr-cmca` or
`bcinr-logic`.

## See Also

- `.claude/agents/hoare-oracle.md`, `.claude/agents/turing-machine.md`,
  `.claude/agents/armstrong-fault.md`, `.claude/agents/von-neumann-bypass.md` — constitutional
  agent definitions this ledger's ownership matrix maps onto.
- `docs/cmca-rdf/V26_7_17_HOOK_SPEC.md` — enforcement-level spec for gates referenced here.
- `docs/cmca-rdf/CURRENT_STATUS.md` — prior standing report, scope-inconsistent with this
  ledger per G7 above; superseded in place by a new Section 0 covering C1-C6 (dated
  2026-07-17, see Gate Closure Summary below).
- `AGENTS.md` — constitution defining REPORTED/CONFIRMED/ALIVE/PARTIAL_ALIVE vocabulary and
  the 33-section rule set this ledger operates under.

---

## Gate Closure Summary — 2026-07-17 (dated addendum, appended, not overwriting the REPORTED-era entries above)

This section records the FINAL standing of each gate G0-G9 as of this dated pass, per
independent reproduction gathered across `PHASE1_CONSUMER_VERDICT.md`,
`PHASE2_RUNTIME_CLOSURE_VERDICT.md`, `RECONCILIATION_VERIFICATION.md`,
`FINAL_RECONCILIATION_VERIFICATION.md`, `FINAL_RECONCILIATION_VERIFICATION_V2.md`,
`FINAL_RECONCILIATION_VERIFICATION_V3.md`, `MUTANT_KILL_MATRIX.md`, and
`OBJECT_CODE_AUDIT.md`. Entries above this section (the original G0-G9 REPORTED bodies) are
left untouched — where a gate closes here, treat the standing below as authoritative and the
REPORTED entry above as its superseded starting point, not as deleted history.

Per `.claude/rules/00-release-governance.md`, only `cmca-release-integrator` may emit the
terminal release-completion declaration, and only after a reproduced-evidence record exists
for every gate and a `cargo publish --dry-run` exit-0 transcript exists against the final
integrated coordinate. **Neither of those two conditions is established by this section.**
This section records per-gate technical standing only; it is not, and must not be read as, the
terminal release declaration.

| Gate | Final standing | Citation (doc + command/finding) |
|---|---|---|
| **G0 — Release Identity** | **REPORTED (open)** | No evidence in any doc cited by this pass shows `crates/bcinr-cmca/Cargo.toml`'s version bumped to `26.7.17`, `publish = false` decided, or `readme`/`repository`/`keywords` metadata added. This is explicitly the parallel sibling task's scope (metadata changes), not reproduced here — remains open pending that task's own evidence. |
| **G1 — Workspace/Packaging Hazards** | **PARTIALLY REPRODUCED** | Quarantine-exclusion sub-obligation: REPRODUCED → resolved. `RECONCILIATION_VERIFICATION.md` item 1 (`cargo package -p bcinr-cmca --list --allow-dirty`, `grep -i quarantine` → zero hits; `Cargo.toml` now has `exclude = ["quarantine/**"]`). Remaining sub-obligations open: the `[patch.crates-io] wasm4pm-compat` absolute-path hazard is still UNVERIFIED on a foreign checkout (unchanged from the original REPORTED entry above); `RECONCILIATION_VERIFICATION.md`'s "Minor packaging finding" also surfaces a new, not-yet-fixed defect — `src/allocator.rs.orig` (a stray backup file) still ships inside the package tarball, uncaught by the `quarantine/**` glob. Per `.claude/rules/cmca/packaging.md`, all `cargo package`/`cargo publish --dry-run` evidence cited above used `--allow-dirty` against an uncommitted tree, so none of it is admissible as release-closing evidence — only as an interim smoke check. G1 remains open. |
| **G2 — Numeric Law** | **REPRODUCED → ALIVE** | `PHASE2_RUNTIME_CLOSURE_VERDICT.md` C1 table (every sub-obligation from the original G2 REPORTED list — opaque union-based fault set, sealed `CanonicalMask` with proven `{0,u32::MAX}` image, sealed fixed types, floor conservation — closed with real test runs) + `FINAL_RECONCILIATION_VERIFICATION_V3.md` item 1 (`cargo test -p bcinr-cmca --all-features` 100% green, independently rerun in this session). Two non-blocking precision notes carried forward (select-site spot-check is partial; rejection invariance is field-equality not byte-transmute) — neither is a law violation. |
| **G3 — Authority: Certification and Sealing** | **REPRODUCED → ALIVE** | `PHASE2_RUNTIME_CLOSURE_VERDICT.md` C2/C3 table: `Observatory::evaluate_calibration` confirmed (via `grep -n CertificateReceipt observatory.rs`) to never construct a `CertificateReceipt` directly; `seal_certificate` exists (`certification.rs:65-112`) and independently checks the sealing witness plus all 11 named bindings, each with a dedicated typed refusal and passing test — exceeding the original G3 falsifier's "spot-check ≥3" bar. |
| **G4 — Authority: Mode-Switch Lifecycle** | **REPRODUCED → ALIVE** | `PHASE2_RUNTIME_CLOSURE_VERDICT.md` C2/C3 table: the six files named ABSENT in the original G4 REPORTED entry (`proposal.rs`, `shadow.rs`, `jump.rs`, `stability.rs`, `certification.rs`, `mode_switch.rs`) now exist with 63 passing lib unit tests; `DwellSatisfied` is a sealed struct, not a bare bool (trybuild negative tests pass); rejected switches preserve bytes (`mode_switch::tests::rejection_cause_*_leaves_state_untouched`, 3 tests pass). The "no broker/actuation surface" sub-obligation is re-confirmed (not merely carried forward) via a fresh `grep -rn 'broker\|actuat'` in this pass — still structurally not applicable, recorded honestly rather than fabricated as a passing check. |
| **G5 (split) — C4 sub-obligations** | **REPRODUCED → PARTIAL_ALIVE** | `C4_mfw_admission`/`C4_projection`/`C4_bcinr_consumption` (the three-way split recorded in this ledger's own "Architecture Correction" section above) are each individually evidenced: `PHASE1_CONSUMER_VERDICT.md` checks 1, 4, 5 (no RDF/SHACL/parser code in `src/`, zero `mfw`/`oxigraph`/`praxis-graphlaw` in `cargo tree`, buildable without `/Users/sac/mfw` reachable) plus `FINAL_RECONCILIATION_VERIFICATION_V3.md` item 3 (`cargo make verify-generated` PASS) — together giving `BCINR_CMCA_PURE_CONSUMER_ALIVE`. Full closure (SHACL/ShEx/QUDT validation over the ontology inputs) remains explicitly fenced per this ledger's own "Fenced-later-obligations" §1/§3 below — hence PARTIAL_ALIVE, not ALIVE. The original (pre-split) G5's 11 sub-obligations describe the pre-migration, now-quarantined `bcinr-cmca`-hosted generator; they are superseded by this split, not independently closed. |
| **G6 — Verifier: Object-Code/Branchlessness Closure** | **REPRODUCED (attempted) → UNKNOWN, fenced** | `OBJECT_CODE_AUDIT.md`, quoted exactly: *"Standing: BLOCKED — crate does not compile in this git state (pre-existing, not introduced by this audit)"*; harness built at `tools/bcinr-cmca-audit-harness/`, `cargo build -p bcinr-cmca` / `--release` both failed with 259 pre-existing errors at audit time; disassembly step (`otool -tv`, Darwin) never reached; *"Per-symbol table: Not produced. Standing: UNKNOWN."* This audit has **not been rerun** against the current tree since the `from_bits`/`from_value_bits` fix (documented in `FINAL_RECONCILIATION_VERIFICATION.md`/`_V2`/`_V3`, none of which re-invoke the harness) — G6 is not newly closed by those later docs; it remains exactly what it was: a compile-time blocker recorded once, never re-measured. Object-code-level branchlessness is UNKNOWN for this release, not proven and not disproven. |
| **G7 — Documentation/Standing Consistency** | **REPRODUCED → resolved by this pass** | The G7 falsifier named `docs/cmca-rdf/CURRENT_STATUS.md` claiming "100/100 SIS" for C4 only, with no mention of C1/C2/C3/C6. This pass's own edit to `CURRENT_STATUS.md` (new Section 0, dated 2026-07-17) adds explicit per-component standing for C1, C2/C3, C4, and C6 with citations, and states the mandated final-standing sentence verbatim. This ledger section is the second half of that same G7 resolution. |
| **G8 — Constitutional/Config Surface** | **Unchanged — REPORTED** | Out of this pass's scope; no new `.claude/rules/`, `.claude/agents/`, or `.claude/skills/` evidence was gathered in this documentation-only pass. Left as originally REPORTED above. |
| **G9 — Standing/Evidence Rollup** | **Partially reproduced — mandated sentence now present verbatim in two documents** | The mandated final-standing sentence — "CMCA v26.7.17 is PARTIAL_ALIVE for the pinned bounded configuration." — is now present verbatim in `CHANGELOG.md`'s v26.7.17 entry and `CURRENT_STATUS.md`'s new Section 0 (both edited in this pass). G9 itself is **not** closed by this: per `.claude/rules/00-release-governance.md`, the terminal release declaration may only be emitted by `cmca-release-integrator`, and only after a reproduced `cargo publish --dry-run` exit-0 transcript exists against the final integrated coordinate. No such transcript was produced in this documentation-only pass — G0 (version/metadata) is also still open (see above). The release remains PARTIAL_ALIVE at the technical-gate level (G2-G5 ALIVE/PARTIAL_ALIVE, G6 UNKNOWN, G0/G1/G8 open, G9 pending the terminal dry-run gate), not yet ready for a terminal completion declaration. |

### What this summary does not claim

- It does not claim `cargo publish --dry-run` was run in this pass. It was not.
- It does not claim G0 (version bump, Cargo.toml metadata) is closed — that is explicitly the
  parallel sibling task's scope, tracked separately.
- It does not upgrade G6 (object-code) beyond UNKNOWN — the audit's compile-blocker finding is
  reported honestly as stale-and-unrerun, not silently reinterpreted as passing now that the
  crate compiles again elsewhere.
- It does not alter or delete any REPORTED-era entry above; it appends a dated closure record
  alongside them.
