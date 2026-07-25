# Generic Artifact Boundary — Gamma_D Template

**Status:** exploratory design note for a post-v26.7.17 milestone ("Constitutional Compiler v0").
Not a contract. Nothing here binds any producer or consumer. Written read-only against
`docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` (Gamma_CMCA v1); that document remains the sole
authoritative artifact contract until a real Gamma_D instantiation is proposed and reviewed on
its own terms.

## 1. Purpose

`docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` defines one concrete boundary: a semantic-input domain
(admitted RDF graph) projected, through a declared generator, into a mechanical artifact
(`cmca_generated.rs` + manifest + receipt) that a downstream consumer (`bcinr-cmca`) verifies
without ever touching the semantic domain's own tooling. This note asks whether that shape is
domain-specific to CMCA or a reusable pattern, and if reusable, what the generic template is and
what stays domain-specific at each instantiation. This is a UNVERIFIED design hypothesis: no
second instantiation has been built, so "genuinely reusable" is a claim about structural
similarity across sketches, not about a working second producer/consumer pair.

## 2. The generic pattern, Gamma_D

For a domain `D`, define:

```
Gamma_D : SemanticDomain_D -> MechanicalArtifact_D
```

`SemanticDomain_D` is whatever authored, admitted, human-legible input the domain's own tooling
understands (an RDF graph, a PDDL domain file, a POWL workflow registry, ...). `MechanicalArtifact_D`
is the fixed-shape output a downstream consumer checks and consumes **without** depending on the
semantic domain's own toolchain. The boundary is enforced by never letting the consumer link
against, import, or invoke anything from the semantic domain's tooling — only the artifact files
themselves are read.

### 2.1 Generic contract fields

| Generic field | Role |
|---|---|
| `schema_identity` | Version tag for this instantiation of Gamma_D; increments on any incompatible change to the file set, digest list, serialization rules, or verification obligations. |
| `semantic_input_digest` | Digest of the canonicalized semantic-domain input itself (the admitted graph / file / registry), under a domain-declared canonical serialization. |
| `admission_digest` | Digest of the **admission/validation pass result** over that input (which checks ran, per-check content digest, pass/fail per check and overall) — not the input again. |
| `generator_digest` | Digest of the generator source, in a fixed declared concatenation order, exactly as committed (no re-encoding). |
| `numeric_or_physical_profile_digest` | Digest of whatever fixed numeric/physical parameters the generated artifact bakes in (precision, rounding, ranges, scale factors — or, in a non-numeric domain, whatever the analogous fixed-parameter surface is). |
| `payload_digest` | Digest of the exact bytes of the emitted mechanical artifact itself. |
| `manifest` | Human/tool-readable JSON carrying `schema_identity`, dimension/shape bounds, every named digest, and provenance (generator invocation, generation-run identifiers). |
| `generation_receipt` | Hash-chained, timestamped **event** record of the generation run — expected to vary run-to-run, unlike the identity-bearing manifest/payload digests it references by digest. |
| `consumer_verification_law` | The consumer's obligation: check schema recognition, check declared shape bounds against actual emitted shapes, check digest self-consistency (esp. payload digest against freshly hashed bytes) — and refuse (typed failure, never silent fallback) on any mismatch or unrecognized schema version. |

The generic shell adds two structural rules inherited from Gamma_CMCA, restated domain-agnostically:

- **Identity vs. event separation**: every digest in the manifest, and the manifest as a whole,
  must be reproducible byte-for-byte from the same semantic input and generator toolchain
  coordinate, regardless of machine, wall-clock time, or run. The receipt alone is permitted —
  expected — to vary run-to-run; verification never treats receipt-to-receipt variation as a
  defect, only receipt internal well-formedness and its reference to manifest digests.
- **Gap rule**: if some semantic-input identity is not transitively bound into one of the
  declared digests, a new named digest must be added (with a `schema_identity` bump), never
  silently folded into an existing digest's assumed scope.

## 3. CMCA as one instantiation of Gamma_D

| Generic field | Gamma_CMCA v1 concrete field |
|---|---|
| `schema_identity` | `schema_version: u32` (recognized set `{1}`) |
| `semantic_input_digest` | `rdf_digest` — BLAKE3 of canonical N-Quads |
| `admission_digest` | `admission_digest` — BLAKE3 of canonical JSON of the SHACL/ShEx pass result |
| `generator_digest` | `generator_digest` — BLAKE3 of concatenated generator source (Python, this release) |
| `numeric_or_physical_profile_digest` | `numeric_profile_digest` — BLAKE3 of Decimal precision, rounding mode, range, Q16.16 scale |
| `payload_digest` | `generated_payload_digest` — BLAKE3 of `cmca_generated.rs` bytes |
| `manifest` | `cmca_generation_manifest.json` |
| `generation_receipt` | `cmca_generation_receipt.json` (BLAKE3-chained, admit_graph → validate → generate → emit_artifact) |
| `consumer_verification_law` | `VerifyGeneratedProfile` (§4 of Gamma_CMCA): schema recognition, `N`/`F`/`K`/`Q` bound match, payload digest recheck, typed-failure-on-mismatch |

Gamma_CMCA also carries a fifth digest with no generic slot listed above — `formula_registry_digest`
(BLAKE3 of every formula/floor identity paired with its defining rule) — and a §8 correspondence
domain classification (`CORRESPONDENCE_REQUIRED` / `DEFECTIVE_BEHAVIOR_QUARANTINED` /
`NEW_LAW_REQUIRED`). Both are called out explicitly in §5 below as domain-specific, not part of
the generic shell.

## 4. Two other domain sketches (illustrative, not specified)

**PDDL domain definitions (`bcinr-pddl`).** `SemanticDomain_D` would be an authored PDDL domain +
problem pair (predicates, actions, durative actions) as admitted by `pddl_admit_domain` /
`pddl_parse_domain`. `semantic_input_digest` would cover the canonicalized PDDL text (requirement
list, predicate signatures, action preconditions/effects in a fixed serialization — analogous to
canonical N-Quads but for PDDL S-expressions). `admission_digest` would cover the admission
witness `pddl_admit_domain` already returns (requirement_count, action_count, durative_action_count,
witness) rather than re-parsing. `numeric_or_physical_profile_digest`'s analogue here is a
**temporal/resource profile digest**: duration bounds, numeric-fluent ranges, resource-key
capacities — the same conceptual slot as CMCA's Decimal/Q16.16 profile, just bound to planning
semantics instead of numeric-law semantics. `payload_digest` would cover whatever the generator
emits for a consumer that must plan/schedule without re-invoking the PDDL parser — e.g. a
precompiled action-applicability table or grounded operator set. This sketch is UNVERIFIED: no
generator of this shape exists in `bcinr-pddl` today; the mapping above is a plausibility
argument, not a proposal to build it.

**POWL workflow registries (`bcinr-powl`).** `SemanticDomain_D` would be an authored POWL
topology/op registry (the Sequence/XorChoice/Loop constructs `powl_compile_sequence` and
`powl_compile_choice` accept as labels, plus the topology-admission rules `powl_admit_context`
enforces). `semantic_input_digest` would cover the canonicalized op/label sequence and topology
kind. `admission_digest` would cover the capability-mask and topology-admission check result
(`powl_capability_check`'s granted/required comparison, `powl_admit_context`'s topology
classification) rather than the raw registry. The numeric/physical-profile analogue is a
**capacity/capability profile digest**: resource_load bounds, urgency-tier range, capability-mask
width — the fixed parameter surface a compiled tape's execution depends on. `payload_digest`
would cover the compiled op tape bytes themselves (what `powl_plan_to_tape` emits). Again
UNVERIFIED and illustrative only — `bcinr-powl` has no manifest/receipt-bearing artifact boundary
today; this is a sketch of how one *could* instantiate the same shell, not a claim that it does.

## 5. What is domain-specific vs. generic

**Domain-specific (must be re-derived per instantiation, never copied from CMCA):**

- The exact digest list and its cardinality. Gamma_CMCA's list is *six*, closed by the gap rule
  in its own §3.1; a PDDL or POWL instantiation might need five, seven, or a different
  partition entirely (e.g. PDDL may need a separate `temporal_profile_digest` distinct from a
  `numeric_profile_digest` if both duration bounds and Decimal-style numeric bounds are present
  simultaneously). The generic template above names *slots*, not a fixed count.
- The correspondence-domain classification (§8 of Gamma_CMCA: `CORRESPONDENCE_REQUIRED` /
  `DEFECTIVE_BEHAVIOR_QUARANTINED` / `NEW_LAW_REQUIRED`) is specific to migrations that have a
  pre-existing baseline generator to compare against. It has no generic slot in §2.1 above
  because it presupposes migration history CMCA happens to have (a legacy Python generator being
  replaced); a from-scratch Gamma_D instantiation with no predecessor artifact would not need
  this category at all.
- The canonical serialization rules for the semantic input (N-Quads sort order for RDF; whatever
  the PDDL or POWL analogue would be) are domain-specific by construction — there is no
  domain-agnostic canonical form for "PDDL text" versus "RDF graph."
- Which concrete files realize `manifest` / `generation_receipt` / the payload, and the exact
  directory layout convention (Gamma_CMCA's `generated-artifact/` versus `src/generated/`
  reasoning in its §5) is a per-repository packaging decision, not part of the shell.

**Generic (the reusable shell, per §2.1):** the nine-slot field table itself; the
identity-vs-event separation rule; the gap rule; and the shape of the consumer's verification
law (recognize schema, check declared bounds against actual emitted shapes, recheck payload
digest, refuse-don't-fallback on any mismatch).

## 6. Non-scope

This document does not propose implementing Gamma_D for PDDL or POWL, does not modify
`CMCA_ARTIFACT_CONTRACT.md` or any file under `crates/bcinr-cmca/`, and does not claim the
six-digest CMCA list or the two domain sketches above have been built or run. It is a structural
comparison exercise only.

## See Also

- `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` — the sole authoritative artifact contract (Gamma_CMCA
  v1) this note generalizes from, read-only.
- `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` — current release status (not modified here).
