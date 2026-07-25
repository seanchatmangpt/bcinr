# Slow Rail `validate` (SHACL) Pipeline in BCINR

In the BCINR architecture, there is a strict separation between the **Hot Path** (the Authoritative Runtime governed by deterministic, zero-allocation laws) and the **Slow Rail** (the asynchronous, off-path environment). Since raw Semantic Web data processing inherently requires branching, memory allocation, and graph traversal, all RDF parsing and Shapes Constraint Language (SHACL) validation is rigorously isolated within the Slow Rail.

Following the initial **`admit_graph`** step—which parses the RDF/Turtle ontology, normalizes it into Canonical N-Quads, and seals it with the cryptographic `rdf_digest`—the Slow Rail pipeline advances to the **`validate`** phase. 

## Enforcing Ontological Constraints with SHACL

Before any ontology data is permitted to cross the artifact boundary into the determinism of the Hot Path, the Slow Rail performs a rigorous **Semantic Admission Process** using SHACL (and ShEx) to guarantee structural compliance ahead of time.

The Slow Rail utilizes SHACL to enforce the following core constraints:

1. **Property Presence & Type Conformance:** Ensures all required properties are strictly present and match expected types. The protocol dictates that a missing required property must never be conflated with a supplied zero or empty value (e.g., defaulting to `0` or `""` is strictly forbidden).
2. **Topological Bounds:** Verifies the graph structure does not exceed predetermined, bounded capacity limits required by the memory-aligned Hot Path.
3. **Index Injectivity & Contiguity:** Guarantees that mapping objects to array offsets yields injective (unique), capacity-bounded, and (where necessary) contiguous indices.
4. **Cycle Detection:** Actively scans for cyclic dependencies in the graph (e.g., via `cmca:dependsOn`). A detected cycle cannot resolve to a silent zero or partial result; it immediately triggers a refusal.
5. **Exact Decimal Fixed-Point Arithmetic:** When converting numeric literals from the ontology into fixed-point representations, the conversion must use exact decimal arithmetic under a declared rounding-mode profile. Falling back to binary floating-point representation is strictly prohibited.

### Core Invariant: Structured Refusals

A non-negotiable constitutional invariant of this pipeline is that every semantic admission check (property presence, bounds membership, cycle detection) must produce a **typed, structured refusal** through ordinary control flow upon failure. Implementations cannot rely on language assertions (e.g., `debug_assert!`) that compiler optimization flags could silently disable.

## Culmination: The Cryptographic `admission_digest`

Once the SHACL shapes have mathematically guaranteed the RDF graph's structural integrity, the non-deterministic Slow Rail flattens the graph (pre-calculating dynamic traversals into strict, fixed-width state tables). 

The outcome of this validation pass—including which SHACL shapes were checked, their content digests, and the precise pass/fail states per shape—is mathematically normalized. To ensure the representation is byte-stable regardless of formatting:
- The validation state is serialized using **RFC 8785-style Canonical JSON (JCS)** (recursively sorting object keys byte-wise and stripping insignificant whitespace).
- This canonical JSON payload is then hashed using the **BLAKE3** algorithm to produce the **`admission_digest`** (formatted as a lowercase hex string prefixed with `blake3:`).

The `admission_digest`, alongside the `rdf_digest`, effectively locks the exact validated semantic state in place. Both digests are recorded in the **`cmca_generation_manifest.json`** identity record and hash-chained securely into the **`cmca_generation_receipt.json`**. 

This allows the flattened semantic state to be emitted as static, C-ABI aligned Rust structs (`cmca_generated.rs`), safely crossing the **`Gamma_CMCA`** artifact boundary into the Hot Path, where the digests are strictly verified at build/test time to prevent supply-chain drift.
