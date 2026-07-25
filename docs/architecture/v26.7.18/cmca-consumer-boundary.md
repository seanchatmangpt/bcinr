# Architecture Requirements Document: BCINR CMCA Consumer Boundary

## Context
**Coordinate:** v26.7.18
**Subproject:** `bcinr`
**Standing:** `UNKNOWN` (Candidate Declaration)

## Topological Invariants
The fundamental geometry of this subproject demands absolute isolation between the semantic source/generation boundary and the runtime consumer boundary.
* **One-Way Projection:** The artifact is generated upstream; `bcinr-core` is strictly a consumer.
* **Disjointness:** The dependency graph of `bcinr-core` must be topologically separated from all semantic parsing logic, RDF engines, or metadata resolution. 

## Exact Algebraic Falsifiers
* **Falsifier 1 (Structural Isolation):** A negative fixture (test) must exist proving that importing any semantic producer crate causes an unrecoverable algebraic contradiction at compile-time. The build must cleanly refuse to compile.
* **Falsifier 2 (Schema Integrity):** The numeric schema deserialization routine acts as a runtime algebraic invariant. Failure to align the checksum and schema version with the expected canonical representation must mathematically collapse to `UnsupportedSchema` or `MalformedPayload`.

## Bounded Construction
* **Location:** `bcinr-core` module.
* **Mechanism:** A pure parser that reads the pre-generated artifact.
* **Evidence Ladder:** Unit and integration tests that verify exact semantic isolation and perfect zero-loss struct recreation.
