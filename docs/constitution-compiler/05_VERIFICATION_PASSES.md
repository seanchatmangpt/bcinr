# 05 — Verification Passes

**Status:** design/reframing document for the future "Constitutional Compiler v0" milestone.
Nothing in this file changes, edits, or supersedes the four live agent definitions
(`hoare-oracle.md`, `turing-machine.md`, `armstrong-fault.md`, `von-neumann-bypass.md`) or
`AGENTS.md` §4. Those files remain the authoritative, hand-authored, currently-in-force
constitution for v26.7.17. This document is UNVERIFIED as an implementation — no code here has
been built or run — and is scoped to *reframing*, i.e. proposing how the same authority
boundaries could be projected from a single machine-readable IR in a later milestone.

## 1. Purpose

`AGENTS.md` §4 currently defines four constitutional roles as autonomous *agents*, each an owner
of specific artifact files with exclusive authority over a domain of judgment (contracts,
structural audits, mutation adequacy, implementation). That framing conflates two things that a
Constitutional Compiler IR needs to keep separate:

1. **A verification capability** — a specific, bounded kind of check (does this satisfy its
   contract? is it structurally bounded? does it survive hostile mutation? does it bypass
   authority boundaries?) — which is domain-agnostic and could in principle be invoked against
   any crate, not only against bcinr-logic primitives.
2. **A named, tool-scoped, persona-bearing agent** — `@hoare_oracle`, `@turing_machine`,
   `@armstrong_fault`, `@von_neumann_bypass` — which is one *particular current binding* of that
   capability to a Claude Code subagent definition with its own `tools:`, `model:`, and prose
   voice.

This document proposes reframing the four roles as four **verification passes** — callable,
composable checks with an input/output contract — and describes a `verification_profile` format
a crate could declare to say which passes apply to it. The four current agent `.md` files would,
in a later implementation milestone, become one *rendering target* of the pass definitions, not
the source of truth. That migration is out of scope here; this file only specifies the target
shape.

## 2. The four passes

Each pass below names its source role, its check function, its required inputs, its required
output, and the artifact file(s) it currently owns per `AGENTS.md` §4 and the four agent `.md`
files.

### ContractPass (Hoare-oracle style)

- **Checks:** does a stated Hoare triple `{P(x)} f(x) {Q(x,f(x))}` exist and is it internally
  consistent — valid input domain, output range, conservation law, monotonicity law where
  applicable, overflow behavior, invalid-input refusal, determinism, state-mutation boundary,
  numeric error envelope, and — per `AGENTS.md` §15 — an oracle that is structurally and
  logically distinct from the implementation under test (not a line-by-line translation, not an
  import-and-wrap).
- **Input:** a primitive's declared signature + the proposed contract.
- **Output:** `PROVEN` / `INVARIANT` / `UNKNOWN` (bounded vocabulary, `AGENTS.md` §28) attached to
  the contract, plus the contract artifact itself.
- **Owns today:** `CONTRACT.md`, `HOARE_TRIPLES.md`, `ORACLE_INDEPENDENCE.md`.
- **Source role:** `hoare-oracle.md`.

### BoundednessPass (Turing-machine style)

- **Checks:** cyclomatic complexity `CC=1` at source *and* in the disassembled object code for
  the declared release target; whole transitive call graph coverage (private functions, macro
  expansions, generic monomorphizations, generated code, build-script output); no reachable
  panic symbol; no reachable allocator symbol; no unexpected branch instruction; no loop
  backedge; no floating-point/division instruction unless explicitly admitted; cheat-scanner
  policy (`CHEAT[rule-id]` findings, no baseline suppression without an admitted waiver); gate
  jurisdiction (which scanners/files a check actually covers).
- **Input:** a release object-code artifact for a named target + the source under audit.
- **Output:** a per-symbol table (`Symbol | CC | Conditional jumps | Loop backedges | Panic path
  | Allocator | Standing`) and a `BRANCHLESS_ALIVE` / `SOURCE_BRANCHLESS_PARTIAL` / `UNKNOWN`
  standing per symbol.
- **Owns today:** `SOURCE_AUDIT.md`, `OBJECT_CODE_AUDIT.md`, `AUTHORITATIVE_CALL_GRAPH.md`,
  `GATE_JURISDICTION.md`.
- **Source role:** `turing-machine.md`.

### FaultPass (Armstrong-fault style)

- **Checks:** at least three independent, syntactically plausible mutants per authoritative
  implementation file, each altering a load-bearing law (sign inversion, dropped factor,
  incorrect mask, normalization omission, index skew, stale digest acceptance, premature state
  mutation, table truncation, bypassed refusal, incorrect clamp, unsupported fallback); each
  mutant must be killed by a **typed** refusal or oracle mismatch derived from ContractPass's
  independent contract — never a bare `assert_ne!(baseline, mutant)`.
- **Input:** the implementation under test + ContractPass's contract (mutants are judged against
  the contract, not against the implementation's own behavior).
- **Output:** the mutant ledger (mutant id, source file, changed law, mutation, expected
  detection, actual detection, test name, receipt digest, standing); any surviving mutant forces
  project standing to `MUTATION_GATE_FAILED`.
- **Owns today:** the mutant ledger, `MUTANT_KILL_MATRIX.md`.
- **Source role:** `armstrong-fault.md`.

### BypassPass (von-Neumann-bypass style)

- **Checks:** whether an implementation observes the mask-based execution discipline (§9),
  performs no speculative/partial state mutation before complete admission (§10), and returns
  only bounded typed refusals (§18) rather than panicking, silently clamping outside admitted
  policy, dropping a factor, falling back to a simpler algorithm, or returning a plausible
  default — i.e. whether it *bypasses* an authority boundary (a checkpoint, a gate, another
  pass's exclusive jurisdiction) that it is not entitled to cross.
- **Input:** implementation source + the checkpoint order (`AGENTS.md` §30) it claims to satisfy.
- **Output:** a bypass/no-bypass finding per checkpoint transition, plus (in the domain-agnostic
  generalization below) a finding on whether any actor approved its own output across a
  constitutional-authority boundary.
- **Owns today:** implementation source under `crates/*/src` (as the thing produced, which the
  *other three* passes then check — BypassPass's own audit output is the boundary-crossing
  finding, not the implementation itself).
- **Source role:** `von-neumann-bypass.md`.

Read together, `AGENTS.md` §4's four roles are a fixed binding of {ContractPass, BoundednessPass,
FaultPass, BypassPass} to bcinr-logic's specific domain (branchless numeric primitives). The
Constitutional Compiler's job in a later milestone would be to keep the pass *definitions*
domain-agnostic and let each crate declare, via a `verification_profile`, which subset applies
and with what parameters.

## 3. `verification_profile` format (proposed, not implemented)

A crate would declare a profile in a new `.claude/verification-profile.toml` (project-scoped,
sibling to `.claude/agents/`) or an equivalent `[package.metadata.verification_profile]` table in
that crate's `Cargo.toml`. Nothing under `crates/bcinr-cmca/**` is being created or edited by this
task — the block below is illustrative TOML, not a file this task writes into that path.

```toml
# .claude/verification-profile.toml (proposed schema, v0)

[profile]
crate = "bcinr-cmca"
schema_version = "0.1.0-draft"

# Each pass entry: required | optional | not_applicable, plus the artifact(s) it must produce.
# "verifier" names the role/agent binding permitted to execute this pass for this crate.
# CONSTRAINT (see §4 below): verifier must never equal any name listed under
# [profile.domain_agents] for the same crate.

[profile.passes.contract]
requirement = "required"
verifier = "hoare-oracle"
artifacts = ["CONTRACT.md", "HOARE_TRIPLES.md", "ORACLE_INDEPENDENCE.md"]

[profile.passes.boundedness]
requirement = "required"
verifier = "turing-machine"
artifacts = ["SOURCE_AUDIT.md", "OBJECT_CODE_AUDIT.md", "AUTHORITATIVE_CALL_GRAPH.md"]
release_targets = ["x86_64-unknown-linux-gnu"]

[profile.passes.fault]
requirement = "required"
verifier = "armstrong-fault"
artifacts = ["MUTANT_KILL_MATRIX.md"]
min_mutants_per_file = 3

[profile.passes.bypass]
requirement = "required"
verifier = "von-neumann-bypass"
# BypassPass here audits *other* implementers' boundary crossings, never its own binding's
# implementation output — see §4.
checkpoint_order_ref = "AGENTS.md#section-30"

# Domain agents this crate actually runs, for the no-self-certification check in §4.
[profile.domain_agents]
names = ["cmca-numeric", "cmca-authority", "cmca-semantics", "cmca-verifier",
         "cmca-release-integrator"]
```

### Worked example: `bcinr-cmca`

Per `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` and the five `cmca-*` mission agents' scope notes
(reproduced in all four constitutional agent files under "Scope relative to the v26.7.17 release
mission"), the four constitutional passes are currently **consultative/review capability only**
for `bcinr-cmca` — they own no release gate in `{G0..G9}`; those gates are owned solely by the
five `cmca-*` mission agents. A `bcinr-cmca` profile reflects that split explicitly rather than
leaving it as prose scattered across four files:

```toml
[profile]
crate = "bcinr-cmca"
schema_version = "0.1.0-draft"

[profile.passes.contract]
requirement = "required"
verifier = "hoare-oracle"
release_gate_authority = false   # consultative only; does not close G0..G9

[profile.passes.boundedness]
requirement = "required"
verifier = "turing-machine"
release_gate_authority = false

[profile.passes.fault]
requirement = "required"
verifier = "armstrong-fault"
release_gate_authority = false
# a surviving mutant still forces MUTATION_GATE_FAILED project-wide even though this
# pass does not itself close a release gate

[profile.passes.bypass]
requirement = "required"
verifier = "von-neumann-bypass"
release_gate_authority = false

[profile.domain_agents]
names = ["cmca-numeric", "cmca-authority", "cmca-semantics", "cmca-verifier",
         "cmca-release-integrator"]
gate_authority = ["G0", "G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9"]
```

This profile is a *reframing* of what is already stated in prose across the four agent files and
`AGENTS.md` §4 — it does not grant the four constitutional passes any release-gate authority they
do not already (explicitly, per their own files) disclaim, and it does not remove any authority
from the five `cmca-*` mission agents.

## 4. Invariant: no-self-certification is a schema constraint, not prose

`AGENTS.md` §27 states this as a rule about roles: "the implementation agent may not be the final
approver for... mathematical correctness, branchlessness, oracle independence, mutant adequacy,
object-code compliance, standing... Agent agreement is not evidence." Every one of the four
current agent files restates this in its own "No self-certification" section.

A prose restatement in five places is exactly the failure mode the Constitutional Compiler exists
to remove. The schema-level form of the same invariant is a **disjointness constraint** between
two sets in the IR, checkable without reading any prose:

```
verifier(pass, crate) ∉ domain_agents(crate)   for every pass ∈ {contract, boundedness, fault, bypass}
                                                and every crate with a declared profile
```

Equivalently, in the worked `bcinr-cmca` example above: `{hoare-oracle, turing-machine,
armstrong-fault, von-neumann-bypass} ∩ {cmca-numeric, cmca-authority, cmca-semantics,
cmca-verifier, cmca-release-integrator} = ∅`. A profile that violated this — e.g. one that set
`verifier = "cmca-numeric"` on the `fault` pass for `bcinr-cmca` while `cmca-numeric` is also
listed under `domain_agents` — would be rejected by a compiler that checks set membership, not by
a reviewer who has to notice the prose contradicts itself. This is the property a
machine-readable IR buys over five hand-authored files: the constraint becomes a validation rule
over the IR's own data, not a convention four independent authors have to remember to restate
correctly every time a fifth file is added.

This document does not implement that validator. It states the constraint at the level a future
`ConstitutionIR` schema would need to enforce it, and names the exact set-disjointness check a
`cargo xtask verify-profile` (or equivalent) command would run.

## 5. What this document is not

- It is not an edit to `hoare-oracle.md`, `turing-machine.md`, `armstrong-fault.md`,
  `von-neumann-bypass.md`, or `AGENTS.md`. Those files are unchanged and remain authoritative for
  v26.7.17.
- It is not a new `.claude/verification-profile.toml` file, and no such file has been created
  under `crates/bcinr-cmca/**` or `.claude/`. The TOML above is illustrative schema text inside
  this design document only.
- It does not change gate ownership for `G0..G9`, which remains with the five `cmca-*` mission
  agents per `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md`.
- It is a design/reframing document for a future milestone. No pass validator, IR parser, or
  profile-compiler has been built or run; every claim above is a proposal, not a verified
  artifact.

## See Also

- `/Users/sac/bcinr/AGENTS.md` §4 (Roster of Transcendent Constructs), §27 (No self-certification),
  §28 (Standing vocabulary), §30 (checkpoint order)
- `/Users/sac/bcinr/.claude/agents/hoare-oracle.md`
- `/Users/sac/bcinr/.claude/agents/turing-machine.md`
- `/Users/sac/bcinr/.claude/agents/armstrong-fault.md`
- `/Users/sac/bcinr/.claude/agents/von-neumann-bypass.md`
- `/Users/sac/bcinr/docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` (read-only reference; not edited by
  this task)
