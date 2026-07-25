# Slow Rail SHACL Validation Protocol

In the BCINR architecture, there is a civilizational-scale dichotomy between the **Hot Path** (the Authoritative Runtime governed by strict deterministic laws, $CC=1$, and zero-allocation) and the **Slow Rail** (the asynchronous, off-path environment). Since raw Semantic Web data processing inherently requires branching, memory allocation, and graph traversal, all RDF parsing and Shapes Constraint Language (SHACL) validation is strictly isolated within the Slow Rail.

## The Role of SHACL in Semantic Admission

Before any ontology data (e.g., `cmca-rdf.ttl`) is permitted to cross the artifact boundary into the determinism of the Hot Path, the Slow Rail performs a rigorous **Semantic Admission Process**. This process utilizes SHACL (and ShEx) to guarantee the structural compliance of incoming RDF graphs ahead-of-time.

The Slow Rail utilizes SHACL to enforce:
- **Property Presence & Type Conformance:** Ensuring all required properties are present and match expected types.
- **Topological Bounds:** Verifying the graph structure does not exceed predetermined capacity limits.
- **Index Injectivity & Contiguity:** Guaranteeing that mapping objects to array offsets yields injective (unique), capacity-bounded, and, where necessary, contiguous indices.

## Core Invariants of the Validation Protocol

The Slow Rail's SHACL validation protocol is governed by non-negotiable constitutional invariants:

1. **Structured Refusals, Not Assertions:** Every semantic admission check (property presence, bounds membership, etc.) must produce a typed, structured refusal through ordinary control flow. They cannot be implemented as language assertions (e.g., `debug_assert!`) that compiler optimization flags could silently disable.
2. **Missing is Not Zero:** The Slow Rail must never conflate a missing required property with a supplied zero or empty value. It is strictly forbidden to substitute a default (like `0` or `""`) for missing data; doing so must trigger a typed refusal.
3. **Dependency Cycles Refuse:** If SHACL validation or the consequence derivation graph detects a cyclic dependency (e.g., via `cmca:dependsOn`), it must immediately produce a typed refusal identifying the cycle. It can never resolve to a silent zero or a partial result.
4. **Exact Decimal Fixed-Point Arithmetic:** When converting numeric decimal literals from the ontology into fixed-point representations, the conversion must use exact decimal arithmetic under a declared rounding-mode profile. Falling back to binary floating-point representation is prohibited, as it cannot represent common decimal fractions exactly.

## Deterministic Flattening and The `Gamma_CMCA` Boundary

Once the SHACL shapes have mathematically guaranteed the RDF graph's structural integrity, the non-deterministic Slow Rail flattens the graph. All dynamic traversals, consequence aggregation, and complex dependencies are recursively explored and pre-calculated into strict, fixed-width state tables.

These flattened, validated state tables are then passed across a one-way deterministic artifact boundary known as **`Gamma_CMCA`**.

- **Generated Static Artifacts:** The validated semantic state is translated into static, branchless Rust code (e.g., `src/generated/case_studies.rs`). 
- **Strict Digests:** The artifact carries a `schema_version` and strict content-identity digests covering the admitted RDF input graph, the validation pass, the generator version, and the numeric profiles applied.
- **Absolute Isolation:** The Hot Path consumes this `Gamma_CMCA` artifact purely as opaque, compiled constants, verifying the digests at build/test time. The Hot Path runtime crate is constitutionally forbidden from containing any RDF parser, graph store, or SHACL engine in its dependency tree.

By strictly enforcing these validation invariants in the Slow Rail, BCINR ensures that the authoritative Hot Path benefits from rich semantic ontologies without ever compromising its zero-allocation, branchless guarantees.
