# Slow Rail RDF Parsing Boundaries and Rules

In the `bcinr` (BranchlessCInRust) project, a strict civilizational-scale dichotomy exists between the **Hot Path** (Authoritative Runtime) and the **Slow Rail**. The Hot Path is governed by the **Radon Law ($CC=1$)**, requiring zero-allocation (`#![no_std]`), zero branching, and deterministic fixed-width execution. Because processing raw, unbounded Semantic Web data fundamentally violates these laws, all non-deterministic, unbounded RDF parsing is relegated to the **Slow Rail**.

The Slow Rail acts as an asynchronous, off-path environment that safely handles tasks like semantic parsing, SHACL validation, and artifact serialization. Before any RDF data is permitted to be transformed into deterministic static structures, the Slow Rail enforces a set of rigorous boundaries and constitutional invariants.

## 1. Highly Constrained Semantic Admission

The Slow Rail strictly regulates how raw ontology inputs (e.g., `cmca-rdf.ttl`) are ingested:

- **Bounded Parsing:** The parser explicitly rejects constructs that could introduce unbounded complexity or non-deterministic mapping, such as blank nodes, multiline literals, or language tags.
- **SHACL Shape Validation:** Structural compliance is mathematically guaranteed ahead-of-time. The input must conform to required properties, expected types, and topological bounds before any further processing occurs.

## 2. Invariants of the Semantic Admission Process

The generator process must abide by path-scoped constitutional laws (e.g., as defined in `cmca/rdf-generation.md`) during the semantic admission phase:

- **Invariant 1: Structured Refusals, Not Assertions:** Every semantic admission check (e.g., property presence, type conformance) must produce a typed, structured refusal value via ordinary control flow. These checks cannot be implemented as assertions (like `debug_assert!`) that compiler optimization flags could disable or elide.
- **Invariant 2: Missing is Not Zero:** The Slow Rail must never conflate a missing required property with a supplied zero or empty value. It is forbidden to substitute a default (like `0`, `""`, or `[]`) for a missing property; it must issue a structured typed refusal.
- **Invariant 3: Dependency Cycles Refuse:** A cycle in the derivation or consequence graph (e.g., via `cmca:dependsOn`) must terminate in a typed refusal. It cannot resolve to a silent zero, a partial result, or any arbitrary default.
- **Invariant 4: Injective and Bounded Indices:** When mapping ontology objects to array offsets, indices must be proven to be:
  1. *Injective* (no two distinct objects receive the same index).
  2. *Capacity-bounded* (indices fit within the pre-declared table capacity).
  3. *Contiguous* (no gaps in dense arrays).
  Failing any condition produces a distinct, typed refusal identifying the exact violation.
- **Invariant 5: Exact Decimal Fixed-Point Arithmetic:** Converting decimal literals from the ontology into fixed-point representations must use exact decimal arithmetic under a declared precision and rounding-mode profile. Binary floating-point arithmetic is strictly prohibited at any stage because it cannot represent common decimal fractions exactly.

## 3. Deterministic Flattening (Ahead-of-Time Resolution)

Because the Hot Path requires constant-time execution, the Slow Rail must perform all dynamic graph traversals **Ahead-of-Time (AOT)**.
- **Consequence Resolution:** Multi-dimensional relationships and dependencies are recursively explored, flattened, and pre-calculated (e.g., aggregating downstream consequence mass) into fixed-width state tables.

## 4. The Artifact Boundary (`Gamma_CMCA`)

The Slow Rail and the Hot Path never interact dynamically. Instead, the validated semantic state is translated into rigid, branchless static Rust intermediate representation (IR) and passed across a one-way deterministic artifact boundary called **`Gamma_CMCA`**.

- **One-Way Topological Isolation:** The `bcinr-cmca` runtime crate must be topologically isolated from any semantic parsing logic. Its dependency graph (especially the release-profile build) must contain **no** RDF parsers, graph stores, SHACL engines, or embedded scripting runtimes.
- **Schema and Digest Contracts:** The `Gamma_CMCA` artifact must carry a `schema_version` and strict content-identity digests covering:
  - The admitted RDF input graph (`RDF_INPUT_DIGEST`).
  - The validation pass and generator version.
  - The fixed-point numeric profiles applied.
- **Consumer Obligation:** The mechanical runtime consumer strictly acts as a reader of the artifact. It must verify the artifact's structure, `schema_version`, and digests at build/test time. On mismatch, it must issue a structured refusal (e.g., `UnsupportedSchema` or `MalformedPayload`) with zero fallback or silent re-interpretation.

## 5. Artifact Generation Rules and Compliance

- **Invariant 6: Byte-Identical Reproducibility:** Generation from a fixed, admitted input via a fixed generator version must be byte-for-byte identical across independent clean-state runs. Unstable iteration orders, timestamps, or host-specific paths are treated as defects that invalidate the generation.
- **Invariant 7: Explicit Manifests:** The generation manifest must plainly state what provenance/identity components it binds. It cannot silently omit required components while implying completeness.
- **Audit Exemption Law (Rule 21):** The generated code is not exempt from Substrate laws. It must pass the **cheat scanner** (detecting hidden branches, magic constants, or scanner evasion), achieve **source-level $CC=1$**, and pass physical **object-code disassembly audits** verifying the complete absence of conditional jumps, allocator calls, and loop backedges.
