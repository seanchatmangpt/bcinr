# Slow Rail `mfw-meaning` Tooling in BCINR

In the `bcinr` architecture, the dynamic and allocation-heavy nature of Semantic Web data (RDF/Turtle) is fundamentally incompatible with the Authoritative Hot Path, which is governed by the Radon Law ($CC=1$) and prohibits data-dependent branches, loops, and heap allocations.

To bridge this gap, the **Slow Rail** utilizes `mfw-meaning` (and related `mfw` tooling) as an Ahead-of-Time (AOT) orchestrator. `mfw-meaning` applies rigorous offline constraint enforcement against the semantic RDF ontology to bind global URIs into deterministic, fixed-width semantic indices before hot path execution. 

The specific offline constraint enforcements include:

## 1. Purging Variable-Length Allocations (URI Interning)
Before any data reaches the Hot Path, `mfw-meaning` interns all variable-length string labels (global URIs) into static byte arenas. It translates unbounded ontology values into exact Q16.16 fixed-point representations and emits the logic as fixed-width, cache-aligned `#[repr(C, align(64))]` C-ABI Rust structs.

## 2. Semantic Index Invariants
`mfw-meaning` rigorously validates the semantic indices assigned to ontology objects to ensure they are deterministic and fixed-width, checking three strict conditions:
- **Injective**: Every index must be unique to a single semantic object.
- **Capacity-bounded**: Indices must fall strictly within the declared fixed capacity of the target array/table.
- **Contiguous**: Where dense layouts are required, the assigned indices must not contain any gaps.
A failure in any of these conditions produces a specifically typed refusal.

## 3. Cycle Proofing & Topological Flattening
Graph traversals and Kahn's Topological Sort (which rely on dynamic queues) are executed entirely offline by `mfw-meaning`. It flattens execution dependencies from cyclic/acyclic semantic graphs into fixed-width hardware bitmasks (e.g., `pred_mask`, `succ_mask`). Any dependency cycles in the semantic graph immediately trigger typed refusals; partial traversal results are never emitted.

## 4. Exact Decimal Arithmetic
Conversion of numeric literals from the ontology into fixed-point numeric representations must be performed via exact decimal arithmetic. The tooling is constitutionally forbidden from using binary floating-point logic to prevent representation errors and precision loss.

## 5. Property Conformance
Missing required properties yield typed, structured refusals. `mfw-meaning` is forbidden from silently substituting default values like `0`, empty strings, or empty collections to bypass validation.

## 6. Cryptographic Binding and Canonicalization
To finalize the bindings, `mfw-meaning` transforms the dynamic `.ttl` files into an absolute, byte-stable representation (Canonical N-Quads). This canonicalized graph is hashed (`rdf_digest`), and the SHACL validation results are hashed (`admission_digest`). These deterministic hashes are chained into a cryptographic manifest and receipt, seamlessly sealing the environment into a one-way boundary called the `Gamma_CMCA` Contract before handing off the zero-allocation artifacts.
