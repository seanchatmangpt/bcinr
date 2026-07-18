---
paths: ["crates/bcinr-cmca/generator.py", "crates/bcinr-cmca/ontology/**", "/Users/sac/mfw/crates/mfw-meaning/**", "/Users/sac/mfw/crates/mfw-shacl/**"]
---

# RDF-to-Rust Generator and Ontology Admission Invariants

Path-scoped law for the RDF/ontology-driven code generator and its ontology inputs. This
file states timeless invariants only. Current compliance status, file:line findings, and
defect reports belong exclusively in the release ledger, never here. See
`/Users/sac/bcinr/.claude/agents/hoare-oracle.md` for the verifier that checks these
invariants and `/Users/sac/bcinr/.claude/agents/turing-machine.md` /
`/Users/sac/bcinr/.claude/agents/armstrong-fault.md` /
`/Users/sac/bcinr/.claude/agents/von-neumann-bypass.md` for adjacent constitutional roles.
Do not duplicate their content; reference by path.

## Note — mfw destination (path-scoping limitation)

Architecture direction is to relocate CMCA RDF admission and generation into the separate
`/Users/sac/mfw` repository, connected to bcinr-cmca via a deterministic digest-bound artifact
boundary rather than a Cargo dependency. Claude Code path-scoped rules in this repository
cannot literally trigger on files in a different repository, so the `paths` frontmatter above
can only hold placeholders for the eventual mfw location — it does not make this rule fire on
mfw-side edits. An equivalent rule must eventually be authored natively in the mfw repository
once its CMCA ontology/generator location exists there. Until that rule exists, the invariants
in this file bind the interim bcinr-side `generator.py` and `ontology/` inputs, and the
invariant statements apply wherever RDF admission code for CMCA actually lives — the binding
location may change, the invariants do not.

## Invariant 1 — Semantic admission is a structured refusal, not a disableable assertion

Every semantic admission check the generator performs on ontology input — property
presence, index-bounds membership, type conformance — must produce a typed, structured
refusal value that propagates through ordinary control flow and cannot be removed by any
interpreter or compiler optimization flag.

**Falsifier:** an admission check that is implemented as a language assertion statement
(e.g. one governed by a runtime debug/optimization flag) such that running the identical
generator and ontology input under a different optimization setting causes the check to
stop firing and generation to proceed past the violation.

**Required evidence:** a demonstration that runs the same violating input under at least
two distinct interpreter/optimization configurations and observes identical refusal
behavior in both; plus a code-level trace showing the check's control-flow path does not
pass through any construct an optimization flag is documented to elide.

**Standing consequence:** any admission path reachable through an assertion a runtime flag
can disable is not an admission check and confers no admitted/refused standing on the
generator: the generator as a whole is BLOCKED for release until the check is reimplemented
as structured control flow.

## Invariant 2 — Missing is not zero

A missing required property and an explicitly-provided zero (or other minimal/valid-looking)
value are distinct conditions and must never be conflated. The generator must never
substitute a default of zero, empty string, empty collection, or any other looks-valid
placeholder for a required property that is absent from the ontology input.

**Falsifier:** an ontology input that omits a required property produces generated output
containing a concrete default value for that property, rather than a typed refusal citing
the missing property.

**Required evidence:** a paired test: (a) ontology input with the property explicitly set to
its zero/minimal value generates successfully with that value present in output; (b) the
same input with the property omitted entirely produces a typed refusal, not the same
output as (a).

**Standing consequence:** any observed default-on-missing behavior is a defect that
invalidates every generated artifact whose provenance cannot rule out silent defaulting;
such artifacts must be treated as UNVERIFIED until regenerated under a fixed generator.

## Invariant 3 — Dependency cycles refuse, never contribute a default

A cycle in the consequence/derivation graph over ontology objects must produce a typed
refusal identifying the cycle. It must never resolve to a silent zero, an arbitrary
traversal order's partial result, or any other default contribution standing in for the
unresolved cycle.

**Falsifier:** an ontology graph containing a derivation cycle generates output (of any
kind, including a zero or empty value at the cyclic node) instead of terminating in a typed
refusal.

**Required evidence:** a constructed minimal cycle (smallest possible cycle, e.g. two or
three mutually dependent objects) run against the generator, showing refusal on every
independent run, with no run reaching code generation.

**Standing consequence:** a generator that emits any output for a cyclic input is BLOCKED
for release; every artifact previously generated from an ontology later found to contain an
undetected cycle must be revoked as UNVERIFIED.

## Invariant 4 — Semantic indices are injective, capacity-bounded, and contiguous where required

Semantic indices used to place ontology objects into arrays or tables must satisfy, each as
a separately-checked and separately-refusing condition:

1. **Injectivity** — no two distinct semantic objects are assigned the same index.
2. **Capacity bounds** — every index falls within the declared capacity for its
   table/array.
3. **Contiguity** — where the generated layout's semantics require contiguous indices
   (e.g. a dense array with no permitted gaps), the assigned index set has no gaps.

**Falsifier:** an ontology input violating exactly one of the three conditions (constructed
so the other two hold) either (a) generates without any refusal, or (b) refuses with a
refusal reason that does not distinctly identify which of the three conditions failed.

**Required evidence:** three independent minimal counterexamples, one per condition,
each showing a refusal whose typed reason names that specific condition and not a generic
catch-all.

**Standing consequence:** collapsing the three checks into one non-distinguishing refusal,
or omitting any one of the three, is a defect; the indexing subsystem cannot be claimed
ALIVE until all three are independently demonstrated.

## Invariant 5 — Fixed-point numeric conversion is exact decimal arithmetic, not binary float

Conversion of a numeric literal from ontology input into a fixed-point representation must
proceed via an exact decimal-arithmetic path, operating under a declared and digested
precision/rounding-mode profile. Conversion via binary floating-point representation of the
decimal literal at any stage of the pipeline is not admissible for this conversion, because
binary floating-point cannot represent common decimal fractions exactly.

**Falsifier:** a decimal literal that binary floating-point cannot represent exactly (e.g.
one requiring more precision than an IEEE-754 double provides at the relevant scale)
produces a fixed-point result differing from the value obtained by exact decimal
arithmetic under the declared rounding-mode profile.

**Required evidence:** a conversion oracle comparison — the generator's output compared
against an independently-computed exact-decimal reference for a battery of literals chosen
to be adversarial to binary float representation — with the precision/rounding-mode
profile's digest recorded alongside the comparison.

**Standing consequence:** any conversion path shown to route through binary
floating-point is a defect that invalidates every fixed-point value in artifacts generated
through that path; the conversion routine cannot be claimed ALIVE until the exact-decimal
path is the only path exercised.

## Invariant 6 — Reproducibility: fixed input + fixed generator version implies byte-identical output

Given a fixed admitted ontology input and a fixed generator version, generation from a
clean state must be byte-for-byte identical across independent runs. Any observed
nondeterminism — unstable iteration order over ontology collections, embedded timestamps,
host-specific paths, or any other run-to-run varying artifact content — is a defect, not an
acceptable cosmetic difference.

**Falsifier:** two independent clean-state runs of the same generator version over the
same admitted input produce outputs that differ in any byte.

**Required evidence:** N ≥ 2 independent clean-state runs (distinct process invocations,
ideally distinct working directories/hosts where feasible) with a byte-level diff (or
content hash comparison) showing zero difference across all runs.

**Standing consequence:** any observed byte-level divergence between runs is a defect that
blocks the generator from ALIVE/reproducible standing regardless of how minor the diverging
bytes appear; the source of nondeterminism must be identified and eliminated, not
suppressed by pinning inputs that happen to avoid triggering it.

## Invariant 7 — The generation manifest states what it binds and what it omits, plainly

A generation manifest accompanying generated output must state, without implication of
completeness, exactly which of the required identity/provenance components (e.g. ontology
content identity, generator version identity, precision/rounding-mode profile digest, input
admission result) it actually binds, and must explicitly enumerate which required
components, if any, remain absent from the manifest.

**Falsifier:** a manifest omits a required identity/provenance component but is worded, or
structured, such that a reader would reasonably conclude the manifest fully establishes
provenance (e.g. by presenting only bound components with no indication that others exist
and are unbound).

**Required evidence:** a manifest schema review cross-referencing the full list of required
identity/provenance components against the manifest's fields, confirming every absent
required component has a corresponding explicit "absent" marker rather than silent
omission.

**Standing consequence:** a manifest that implies completeness while omitting a required
component is a defect strictly worse than an honestly partial manifest (which merely limits
what can be verified); the former misleads a consumer into overclaiming provenance and
blocks the manifest format from acceptance until every omission is made explicit.

## Invariant 8 — Runtime/producer boundary

The bcinr-cmca RUNTIME crate is a branchless numeric/authority-machine consumer of a
digest-bound artifact, not a semantic-admission producer. Its release-profile build must
contain no RDF parser, no RDF library, no graph store, and no SHACL/ShEx engine reachable as
a dependency, and must not embed a scripting-language runtime (e.g. Python) in any form. All
semantic admission and generation — parsing ontology input, validating it, deriving the
generated artifact — is a release-time producer-side concern that happens outside the
runtime crate's build and outside its dependency graph. Generation must never occur via a
`build.rs` step that regenerates RDF-derived output during `cargo build`: the checked-in
generated artifact is the unit that gets verified at build/test time — its digests, its
declared schema version, its declared dimensions — never regenerated inline as a side effect
of building.

**Falsifier:** either (a) a `cargo tree` or equivalent dependency-graph inspection of the
bcinr-cmca runtime crate's release-profile build finds an RDF parser, RDF library, graph
store, SHACL/ShEx engine, or embedded scripting-language runtime reachable as a dependency;
or (b) a `build.rs` (or equivalent build script) belonging to the runtime crate is found to
shell out to, invoke, or otherwise trigger a generator that regenerates the RDF-derived
artifact during `cargo build`.

**Required evidence:** a dependency-graph audit (e.g. `cargo tree` restricted to the
release-profile build of the runtime crate, or an equivalent whole-graph inspection) showing
the absence of any RDF/graph-store/SHACL-ShEx/scripting-runtime dependency, paired with a
review of the runtime crate's build script (if any) confirming it performs verification
(digest/schema-version/dimension checks against the checked-in artifact) and not generation.

**Standing consequence:** any RDF parser, RDF library, graph store, SHACL/ShEx engine, or
embedded scripting-language runtime found reachable from the bcinr-cmca runtime crate's
release-profile build, or any build.rs found to regenerate the artifact inline, is a boundary
violation that blocks the runtime crate from ALIVE/BRANCHLESS_ALIVE standing regardless of
whether the resulting artifact happens to be correct — the violation is architectural, not a
correctness defect, and cannot be waived by demonstrating the regenerated output matches the
checked-in one.

## Nonclaims

This rule does not claim that any current implementation of `generator.py` or the
`ontology/` inputs satisfies any invariant above. This rule does not specify the concrete
data structure, exception type, or refusal-code taxonomy used to implement structured
refusals — only that the mechanism must not be a disableable assertion. This rule does not
specify which precision/rounding-mode profiles are valid choices, only that one must be
declared and digested. This rule does not adjudicate performance, style, or any concern
other than the semantic-admission, determinism, and provenance properties stated above.
Verification of compliance, and any resulting standing (ALIVE/PARTIAL/BLOCKED/MOCKED/
REFUSED/UNSUPPORTED/UNVERIFIED), is the release ledger's responsibility, not this file's.

## See Also

- `/Users/sac/bcinr/.claude/agents/hoare-oracle.md`
- `/Users/sac/bcinr/.claude/agents/turing-machine.md`
- `/Users/sac/bcinr/.claude/agents/armstrong-fault.md`
- `/Users/sac/bcinr/.claude/agents/von-neumann-bypass.md`
- `/Users/sac/bcinr/AGENTS.md`
