---
paths:
  - "crates/bcinr-cmca/src/generated/**"
  - "crates/bcinr-cmca/generator.py"
  - "crates/bcinr-cmca/ontology/**"
---

# CMCA Generated-Artifact Contract Boundary

Path-scoped law for the interim bcinr-side surfaces of the CMCA producer/consumer split:
the generator, its ontology inputs, and the generated output it emits into
`crates/bcinr-cmca/src/generated/`. This file states timeless invariants only — no
file:line findings, no current-compliance claims. Current status belongs exclusively in
the release ledger (`docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md`), tagged REPORTED unless
backed by direct tool evidence. This rule composes with, and does not replace,
`cmca/rdf-generation.md` (generator/ontology admission invariants) or `cmca/packaging.md`
(release packaging). An equivalent path-scoped rule belongs in the `/Users/sac/mfw` repo
once its CMCA projection module exists there — this file covers only the bcinr-side
locations relevant to the boundary until that relocation happens.

## Invariant — the artifact is the only channel

A deterministically generated, content-digest-bound artifact — call it `Gamma_CMCA` — is
the **only** channel of information between the semantic producer (RDF admission and
generation, whether housed in bcinr today or in mfw after relocation) and the mechanical
consumer (the bcinr-cmca runtime). Dependency direction is strictly:

```
producer (RDF admission + generation) --> Gamma_CMCA (artifact) --> consumer (bcinr-cmca runtime)
```

This direction is one-way and non-negotiable: consumer -> producer dependency is
forbidden, and so is any bidirectional coupling (shared mutable state, a callback from
consumer into producer, or a build-time dependency edge from the consumer crate onto any
producer-side RDF library). The artifact boundary exists precisely so the mechanical crate
never needs to know RDF exists.

## Contract shape Gamma_CMCA must carry

The concrete Rust type or file format of `Gamma_CMCA` is out of scope for this rule (see
Nonclaims below). Conceptually, regardless of representation, it must carry:

- a `schema_version` identifying the contract shape itself;
- a set of content-identity digests covering at minimum:
  - the admitted RDF graph,
  - the admission/validation pass applied to that graph,
  - the generator (the tool/version that performed the generation),
  - the numeric conversion profile in effect,
  - the formula/registry identities used,
  - the final generated payload itself;
- the bounded generated registries/tables the consumer needs (for example, floor
  base+remainder tables).

Representing these fields as static generated constants (rather than a runtime struct
instance) is an acceptable realization. What is not acceptable is omitting any of the
conceptual fields above from the generated output in some checkable form — a schema
version and each digest category must be present and inspectable, however they are
encoded.

## Consumer obligation

The mechanical crate (bcinr-cmca runtime) verifies the artifact's structure, its digests,
its `schema_version`, and its dimensions at build time or test time, and **refuses** —
via a typed refusal, never best-effort parsing and never a silent fallback — on any
mismatch or unrecognized `schema_version`. The consumer never reinterprets or re-derives
meaning from RDF itself: it treats `Gamma_CMCA` as opaque, checked data, not as a hint to
re-run semantic inference over.

## Producer obligation

Generation from a fixed admitted input and a fixed generator version must be
byte-for-byte reproducible. This obligation composes with, and does not replace, the
determinism invariant already stated in `cmca/rdf-generation.md`; it restates that
invariant here as a precondition for the artifact boundary to mean anything (a
non-reproducible producer makes the consumer's digest check vacuous).

## Falsifier

Any of the following falsifies the boundary:

- an artifact is consumed by the mechanical crate without its digests being checked;
- a consumer falls back to a default value, or otherwise proceeds, on a `schema_version`
  mismatch instead of issuing a typed refusal;
- any RDF crate or library is reachable from the bcinr-cmca consumer crate's build
  dependency graph (directly or transitively).

## Required evidence

- a build-time or test-time verification routine in the consumer crate that checks
  artifact structure, digests, `schema_version`, and dimensions, and demonstrably refuses
  on at least one constructed mismatch case;
- a dependency-graph audit (e.g. `cargo tree` or equivalent) of the bcinr-cmca consumer
  crate showing no RDF crate or library appears in its build graph.

## Nonclaims

- This rule does not specify the exact Rust type, file format, or serialization of
  `Gamma_CMCA`. That concrete contract is defined in
  `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md`, which is mutable/current documentation, not
  this rule, and is the place to look for today's actual shape.
- This rule does not assert whether the current codebase already satisfies this
  boundary. Whether the artifact exists, whether the consumer checks it, and whether the
  RDF-reachability falsifier currently holds or fails are ledger facts, to be recorded in
  `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` with REPORTED status unless backed by direct
  tool evidence.

## See Also

- `cmca/rdf-generation.md` — generator/ontology admission invariants and the determinism
  invariant this rule composes with
- `cmca/packaging.md` — release packaging invariants
- `docs/cmca-rdf/CMCA_ARTIFACT_CONTRACT.md` — the current, mutable concrete contract for
  `Gamma_CMCA`
- `docs/cmca-rdf/V26_7_17_RELEASE_LEDGER.md` — current compliance status and findings
