# ConstitutionIR — Claim Schema (v0 design draft)

Version/milestone marker: pre-v26.7.17 exploratory design, Constitutional Compiler v0.
Status: DESIGN ONLY — no compiler, no code generation, no execution exists yet. Nothing in
this document is a status claim about bcinr-cmca; every example claim instance below cites
the rule file it formalizes, and formalizes only the *statement* of that rule, not its
current compliance state in the codebase.

## Executive summary

ConstitutionIR is a proposed machine-readable intermediate representation for a single
"claim": a discrete, falsifiable statement of law (an invariant, a required evidence class,
an authority separation) currently hand-authored as prose in up to five places —
`AGENTS.md`, `.claude/rules/cmca/*.md`, `.claude/agents/*.md`, the release ledger, and the
artifact contract. The goal is for those five surfaces to become *projections* from one IR,
rather than five independently maintained hand-written documents that can drift out of
sync. This document defines the core claim schema only: field table, formal JSON Schema,
and worked examples grounded in real rules already in this repository.

## What this schema does NOT cover

- It does not execute anything. ConstitutionIR is a declarative fact base; a separate
  (unbuilt) compiler/projector would read it and emit prose, lint rules, or test scaffolds.
- It does not itself check compliance. A claim's `evidence_classes_required` names what
  *would* be dispositive; it is not itself a test runner or a ledger entry.
- It does not replace the release ledger. Current pass/fail/standing status for a concrete
  primitive remains exclusively the ledger's domain (per `verification.md` Invariant 3 and
  `authority-and-c3.md`'s repeated Nonclaims). A claim's `standing_transport` field says what
  conditions must hold for a *previously recorded* standing result to remain valid — it does
  not record a standing result itself.
- It does not model workflow, scheduling, or agent orchestration (that is POWL's domain in
  this codebase, a separate system).
- It does not cover versioning/migration of the IR itself, cross-claim conflict resolution,
  or a query/inference layer — those are open problems for a later design doc, not settled
  here.

## Field table

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string (stable slug) | yes | Unique, stable identifier for the claim, e.g. `cmca.authority.c3.no-collapse`. Never reused for a semantically different claim. |
| `scope` | list of path globs | yes | Filesystem paths (or path-scoped rule equivalents) the claim governs — mirrors the `paths:` frontmatter already used in `.claude/rules/cmca/*.md`. |
| `preconditions` | list of strings | yes (may be empty) | Conditions that must hold of the input/state before the postconditions are asserted to follow. Empty list means the claim is unconditional. |
| `postconditions` | list of strings | yes (min 1) | The observable properties that must hold given the preconditions — the actual content of the law. |
| `invariant_statement` | string | yes | One human-readable sentence stating the invariant in full, independent of the precondition/postcondition split — the canonical prose a document projector emits verbatim. |
| `falsifier_family` | list of objects `{description, expected_observation}` | yes (min 1) | Concrete, constructible violation scenarios. Each entry names a specific input/mutation and what observing it would prove. Mirrors the `**Falsifier:**` sections already present in the cmca rule files. |
| `owner` | agent role (enum/string) | yes | The role responsible for *implementing* or *proposing* satisfaction of this claim. |
| `verifier` | agent role (enum/string) | yes | The role responsible for *checking* satisfaction. **Schema constraint: `verifier != owner`, always** — this is the no-self-certification law encoded structurally, not left to prose discipline. |
| `evidence_classes_required` | list of enum | yes (min 1) | Which evidence classes are dispositive for this claim: `property_test`, `mutant_kill`, `compile_fail`, `byte_invariance`, `digest_check`, `call_graph_trace`, `construction_site_count`. |
| `standing_transport` | object `{fixed_conditions: [string], invalidated_by: [string]}` | yes | What must stay fixed (toolchain, numeric profile, commit coordinate, feature flags) for a previously-recorded standing result under this claim to remain valid without re-verification, and what change would invalidate it. |
| `dependencies` | list of claim ids | yes (may be empty) | Other claim ids this claim presupposes — e.g. an atomicity claim presupposing a byte-snapshot capability claim. |

Two fields are intentionally *not* part of the claim schema itself, to keep the IR
declarative rather than a status log: current pass/fail state, and file:line evidence
citations. Those belong in a separate `LedgerEntry` IR (out of scope for this document)
that references a claim `id` but is not merged into it — this separation is itself modeled
on `verification.md`'s repeated instruction that rule files "state timeless invariants
only... current status belongs exclusively in the release ledger."

## Formal schema (JSON Schema, draft 2020-12)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://bcinr.dev/schemas/constitution-ir/claim.schema.json",
  "title": "ConstitutionIR Claim",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "id",
    "scope",
    "preconditions",
    "postconditions",
    "invariant_statement",
    "falsifier_family",
    "owner",
    "verifier",
    "evidence_classes_required",
    "standing_transport",
    "dependencies"
  ],
  "properties": {
    "id": {
      "type": "string",
      "pattern": "^[a-z0-9]+(\\.[a-z0-9-]+)+$",
      "description": "Stable dotted slug, e.g. cmca.numeric.fault-join-semilattice"
    },
    "scope": {
      "type": "array",
      "minItems": 1,
      "items": { "type": "string" },
      "description": "Path globs this claim governs, mirroring rule-file `paths:` frontmatter"
    },
    "preconditions": {
      "type": "array",
      "items": { "type": "string" }
    },
    "postconditions": {
      "type": "array",
      "minItems": 1,
      "items": { "type": "string" }
    },
    "invariant_statement": {
      "type": "string",
      "minLength": 1
    },
    "falsifier_family": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["description", "expected_observation"],
        "properties": {
          "description": { "type": "string" },
          "expected_observation": { "type": "string" }
        }
      }
    },
    "owner": { "$ref": "#/$defs/agentRole" },
    "verifier": { "$ref": "#/$defs/agentRole" },
    "evidence_classes_required": {
      "type": "array",
      "minItems": 1,
      "items": {
        "enum": [
          "property_test",
          "mutant_kill",
          "compile_fail",
          "byte_invariance",
          "digest_check",
          "call_graph_trace",
          "construction_site_count"
        ]
      },
      "uniqueItems": true
    },
    "standing_transport": {
      "type": "object",
      "additionalProperties": false,
      "required": ["fixed_conditions", "invalidated_by"],
      "properties": {
        "fixed_conditions": {
          "type": "array",
          "minItems": 1,
          "items": { "type": "string" }
        },
        "invalidated_by": {
          "type": "array",
          "minItems": 1,
          "items": { "type": "string" }
        }
      }
    },
    "dependencies": {
      "type": "array",
      "items": { "type": "string", "pattern": "^[a-z0-9]+(\\.[a-z0-9-]+)+$" }
    }
  },
  "$defs": {
    "agentRole": {
      "type": "string",
      "enum": [
        "cmca-authority",
        "cmca-numeric",
        "cmca-semantics",
        "cmca-verifier",
        "cmca-release-integrator",
        "hoare-oracle",
        "turing-machine",
        "armstrong-fault",
        "von-neumann-bypass"
      ]
    }
  },
  "allOf": [
    {
      "$comment": "No-self-certification law: verifier must differ from owner. JSON Schema cannot express field inequality directly, so this is asserted via a not/const pairing per enum value — see accompanying validator note below.",
      "not": { "$comment": "Structural note, not enforceable in pure JSON Schema; see Implementation note." }
    }
  ]
}
```

**Implementation note on the `verifier != owner` constraint:** plain JSON Schema (draft-07
or 2020-12) cannot express a cross-field inequality constraint natively — there is no
`$data` reference in standard JSON Schema. Two conforming ways to enforce it, both
compatible with the schema above:

1. A second-pass structural validator (e.g. a small Rust or Python check run alongside
   schema validation) that rejects any claim document where `verifier == owner`. This is
   the recommended approach and is the one the schema's `allOf`/`$comment` above flags as
   required.
2. A JSON Schema draft that supports `$data` (e.g. via the `ajv` `$data` keyword extension)
   could express `"const": { "$data": "1/owner" }` negated — but this is a non-standard
   extension, not draft-07/2020-12 core, so it is noted here as an alternative rather than
   embedded in the canonical schema.

Either way, the no-self-certification law is a **schema-level obligation**, not a
convention left to the humans authoring claim YAML — a claim document that sets
`verifier` equal to `owner` is invalid ConstitutionIR, full stop, mirroring
`authority-and-c3.md` Invariant 1's requirement that proposing and certifying be
"independently implemented steps."

## Worked example claim instances (YAML)

### Example 1 — Fault accumulation is a join-semilattice

Formalizes: `.claude/rules/cmca/numeric-hot-path.md`, Invariant 1.

```yaml
id: cmca.numeric.fault-join-semilattice
scope:
  - "crates/bcinr-cmca/src/fixed.rs"
  - "crates/bcinr-cmca/src/allocator.rs"
preconditions:
  - "Two fault-bearing computation steps a and b are composed sequentially (a ; b)"
postconditions:
  - "faults(a ; b) equals the set union of faults(a) and faults(b)"
  - "The empty fault set is the identity element of composition"
invariant_statement: >
  The set of faults produced along a computation path forms a join-semilattice under
  union, with the empty fault set as the zero element; sequential composition of two
  fault-bearing steps must union their fault sets and must never collapse to
  first-fault-only or last-fault-only.
falsifier_family:
  - description: >
      Construct step a raising fault F1 only, and step b raising a distinct,
      non-overlapping fault F2 only; compose a ; b.
    expected_observation: >
      A violation is observed if faults(a ; b) is a strict subset of {F1, F2} —
      e.g. only {F1} (last-fault-dropped) or only {F2} (first-fault-dropped).
owner: cmca-numeric
verifier: hoare-oracle
evidence_classes_required:
  - property_test
  - mutant_kill
standing_transport:
  fixed_conditions:
    - "same toolchain (rustc version + target triple)"
    - "same numeric profile (fixed-point width/scale configuration)"
    - "same commit hash for fixed.rs and allocator.rs"
  invalidated_by:
    - "any change to the fault-set representation type"
    - "any change to the composition operator's implementation"
    - "toolchain upgrade that changes floating/fixed-point codegen"
dependencies: []
```

### Example 2 — Masked selection distributes over the fault-bearing pair

Formalizes: `.claude/rules/cmca/numeric-hot-path.md`, Invariant 2.

```yaml
id: cmca.numeric.masked-select-distributes-over-fault-pair
scope:
  - "crates/bcinr-cmca/src/fixed.rs"
preconditions:
  - "A branchless select operates over two (value, fault) alternatives A and B under mask m"
postconditions:
  - "select(m, (v_a, f_a), (v_b, f_b)) == (select(m, v_a, v_b), select(m, f_a, f_b))"
  - "The fault field of the unselected alternative never appears in the result (no contamination)"
  - "The fault field of the selected alternative is always preserved in the result (no erasure)"
invariant_statement: >
  Masked selection over (value, fault) pairs must distribute over the pair as a whole:
  the fault of the unselected alternative must never leak into the result, and the fault
  of the selected alternative must never be silently erased by a fresh no-fault
  reconstruction of the result value.
falsifier_family:
  - description: >
      Construct alternative A with a faulted value and a specific fault tag T, and
      alternative B with a clean value and no fault tag. Select B under the mask setting
      that should choose B.
    expected_observation: >
      Contamination is observed if the result carries fault tag T despite B (fault-free)
      being selected.
  - description: >
      Select A (the faulted alternative) under the complementary mask setting.
    expected_observation: >
      Erasure is observed if the result carries no fault tag despite A's fault tag T
      being the selected alternative's fault.
owner: cmca-numeric
verifier: hoare-oracle
evidence_classes_required:
  - property_test
  - mutant_kill
standing_transport:
  fixed_conditions:
    - "same toolchain and target triple"
    - "same mask type / canonical two-point image (see Example 3)"
    - "same commit hash for fixed.rs"
  invalidated_by:
    - "any change to the select operator's implementation"
    - "any change to the (value, fault) pair representation"
dependencies:
  - cmca.numeric.canonical-mask-two-point-image
```

### Example 3 — No self-certification: proposal and certification are distinct authorities

Formalizes: `.claude/rules/cmca/authority-and-c3.md`, Invariant 1.

```yaml
id: cmca.authority.c3.no-collapse
scope:
  - "crates/bcinr-cmca/src/observatory.rs"
  - "crates/bcinr-cmca/src/proposal.rs"
  - "crates/bcinr-cmca/src/certification.rs"
preconditions:
  - "A chain implements any hop of proposal -> admission -> certification -> actuation"
postconditions:
  - "No single function or type performs more than one of {proposal, admission, certification, actuation}"
  - "Certificate-shaped or admitted-state-shaped values are constructible only from the certification/admission module, never from the proposal/observation module"
invariant_statement: >
  Proposal, admission, certification, and actuation are four distinct authorities; no
  function or type may perform more than one of these roles, and in particular the
  observing/proposing component must never itself mint or admit a certified state.
falsifier_family:
  - description: >
      Trace every reachable code path from the observing/proposing module and check
      whether any path returns, constructs, or writes a certificate-shaped or
      admitted-state-shaped value without passing through the certification/admission
      module's own function.
    expected_observation: >
      A violation is observed if such a path exists, or if a single enum/struct's own
      contract is tagged as serving more than one of {proposal, admission,
      certification, actuation}.
owner: cmca-authority
verifier: cmca-verifier
evidence_classes_required:
  - construction_site_count
  - call_graph_trace
standing_transport:
  fixed_conditions:
    - "same module boundaries (no refactor merging proposal.rs and certification.rs)"
    - "same commit hash for observatory.rs, proposal.rs, certification.rs"
    - "same public API surface (no new re-export that bridges the two modules)"
  invalidated_by:
    - "any refactor that merges two of the four authority modules"
    - "any new public constructor for a certificate-shaped value outside the certification module"
dependencies: []
```

Note on Example 3's `owner`/`verifier` pairing: `cmca-authority` (implements/owns the
authority chain) and `cmca-verifier` (an agent role distinct from `cmca-authority`,
`cmca-numeric`, and `cmca-semantics` per the repo's existing agent role separation) satisfy
the schema's `verifier != owner` constraint by construction — this is the concrete instance
the constitution-wide no-self-certification law is meant to make un-skippable.

## Open questions (explicitly deferred, not answered here)

- Whether `owner`/`verifier` should be a single fixed agent-role enum (as modeled above,
  matching the concrete roles already defined under `.claude/agents/`) or a more general
  capability-tag system if roles proliferate.
- Whether `dependencies` should support a distinction between "presupposes" (must hold
  first) and "composes with" (independent, non-ordered) — collapsed to one list here.
- The `LedgerEntry` IR that would reference a claim `id` to record actual pass/fail
  standing is deliberately out of scope for this document and left to a follow-up design
  note (`02_LEDGER_ENTRY_SCHEMA.md`, not yet written).

## See Also

- `/Users/sac/bcinr/AGENTS.md` — constitution this IR is meant to eventually project from
- `/Users/sac/bcinr/.claude/rules/cmca/numeric-hot-path.md` — source of Examples 1-2
- `/Users/sac/bcinr/.claude/rules/cmca/authority-and-c3.md` — source of Example 3
- `/Users/sac/bcinr/.claude/rules/cmca/verification.md` — source of the ledger/rule
  separation principle this schema's Nonclaims section follows
- `/Users/sac/bcinr/docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` — read-only reference, not yet
  cross-walked against this schema (future work)
