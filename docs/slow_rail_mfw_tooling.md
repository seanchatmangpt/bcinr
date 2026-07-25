# Slow Rail `mfw` Tooling and Ontology Orchestration in BCINR

In the `bcinr` (BranchlessCInRust) architecture, there is a fundamental conflict between the dynamic, allocation-heavy nature of Semantic Web data and the absolute deterministic laws of the Authoritative Hot Path. 

The Hot Path is governed by the **Radon Law ($CC=1$)**, which prohibits data-dependent branches, loops, and heap allocations (`#![no_std]`). To process complex, unbounded RDF/Turtle semantic graphs without violating these constitutional laws, all semantic parsing and validation are strictly quarantined to the **Slow Rail**—an offline, asynchronous pipeline. 

The **`mfw`** (and specifically **`mfw-meaning`** / **`mfw-shacl`**) tooling plays the critical role of the Ahead-of-Time (AOT) producer in this separated topological boundary.

## 1. The Role of `mfw` / `mfw-meaning`

The `mfw` tooling acts as the gatekeeper and orchestration engine that ingests human-readable semantic data and prepares it for deterministic hardware consumption. Its primary responsibilities include:

- **Topological Disjointness**: It executes entirely out-of-band. The runtime crate (`bcinr-cmca`) is mathematically forbidden from compiling any `oxigraph`, SHACL engine, or variable-string parsers into its dependency graph. `mfw` ensures that all dynamic pointer chasing and variable-length parsing occur strictly on the Slow Rail.
- **Cycle Proofing & Topological Flattening**: Graph traversals and Kahn's Topological Sort (which rely on dynamic queues) are executed by `mfw` offline. It flattens execution dependencies from cyclic/acyclic graphs into fixed-width hardware bitmasks (e.g., `pred_mask`, `succ_mask`). 
- **Purging Variable-Length Allocations**: Before reaching the Hot Path, `mfw` interns all string labels (URIs) into static byte arenas and translates ontology values into exact Q16.16 fixed-point representations. It emits the logic as fixed-width, cache-aligned `#[repr(C, align(64))]` C-ABI Rust structs.

## 2. Orchestration Before the `admit_graph` Step

Before the formal `admit_graph` ingestion step can cryptographically seal the ontology, `mfw` tooling rigorously orchestrates the parsing and structural enforcement of the raw ontology:

### A. Mathematical Normalization (Canonical N-Quads)
The `mfw` tooling transforms dynamic `.ttl` (RDF/Turtle) files into an absolute, byte-stable representation (Canonical N-Quads). This ensures cryptographic stability by enforcing:
- Lexicographical sorting of quads in text form.
- Strict UTF-8 encoding with Unix (`\n`) line endings.
- Prohibition of insignificant whitespace or trailing blank lines.

### B. Structural and Semantic Enforcement (SHACL Pre-Checks)
Concurrent with ingestion, `mfw` drives the ontology through strict SHACL/ShEx validation constraints:
- **Property Conformance**: Missing required properties yield typed, structured refusals. The tooling is constitutionally forbidden from silently injecting defaults like `0` or empty strings.
- **Exact Decimal Arithmetic**: Floating-point conversions are explicitly banned. Numeric data is ingested through exact decimal arithmetic.
- **Strict Cycle Refusals**: Dependency cycles in the semantic graph immediately trigger typed refusals; `mfw` will never emit partial traversal results.
- **Index Invariants**: `mfw` verifies that semantic indices assigned to ontology objects are injective (unique), capacity-bounded, and contiguous.

### C. Cryptographic Binding and The Handoff
Once the tooling has successfully parsed, normalized, and validated the ontology, the `admit_graph` step formally locks the state:
1. **`rdf_digest`**: The canonical N-Quads are hashed using BLAKE3.
2. **`admission_digest`**: The results of the SHACL validation passes (shapes checked, their digests, and pass/fail states) are normalized via Canonical JSON and hashed.

These deterministic hashes are chained into the `cmca_generation_manifest.json` and `cmca_generation_receipt.json`. Finally, `mfw` hands off the verified, flattened, and fixed-width data as zero-allocation artifacts (`cmca_generated.rs`), safely crossing the `Gamma_CMCA` boundary for the `bcinr` Hot Path to consume via zero-copy embedded arrays.
