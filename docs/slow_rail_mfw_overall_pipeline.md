# The Slow Rail `mfw` Compilation Pipeline in `bcinr`

In the `bcinr` (BranchlessCInRust) architecture, there is a fundamental conflict between the dynamic, allocation-heavy nature of Semantic Web data (RDF/Turtle) and the absolute deterministic laws of the Authoritative Hot Path (governed by the Radon Law, $CC=1$, `#![no_std]`, and zero heap allocations). To process complex semantic graphs without violating these constitutional laws, all semantic parsing, validation, and generation are strictly quarantined to the **Slow Rail**—an offline, asynchronous pipeline.

The `mfw` tooling suite orchestrates this Ahead-of-Time (AOT) pipeline, taking raw `.ttl` semantic ontologies and transforming them into the deterministic, zero-allocation `cmca_generated.rs` artifact safely across the `Gamma_CMCA` boundary. 

The unified pipeline consists of three core components:

## 1. `mfw-meaning` (Semantic Normalization and Constraint Enforcement)
`mfw-meaning` acts as the initial ingestion and normalization engine. It transforms raw, variable-length ontologies into fixed mathematical structures.
- **Mathematical Normalization**: Transforms dynamic `.ttl` files into an absolute, byte-stable representation (Canonical N-Quads). This enforces lexicographical sorting, strict UTF-8 with UNIX line endings, and prohibits insignificant whitespace for cryptographic stability.
- **URI Interning and Zero-Allocation**: Interns all variable-length string labels (global URIs) into static byte arenas, purging variable-length allocations.
- **Exact Decimal Arithmetic**: Translates unbounded ontology numeric literals into exact `Q16.16` fixed-point representations. Binary floating-point logic is explicitly banned to prevent representation errors.
- **Semantic Index Invariants**: Enforces that indices assigned to ontology objects are Injective (unique), Capacity-bounded (within declared limits), and Contiguous (no gaps for dense layouts).

## 2. `mfw-shacl` (Structural Validation and Cycle Proofing)
`mfw-shacl` performs rigorous structural and semantic enforcement using SHACL/ShEx constraints against the ontology, ensuring the graph is valid before hot path execution.
- **Property Conformance**: Ensures required properties are present ("Missing is not zero"). Missing properties yield typed, structured refusals. The tooling is constitutionally forbidden from silently substituting default values like `0`, empty strings, or empty collections to bypass validation.
- **Cycle Proofing & Topological Flattening**: Dependency graph traversals (e.g., Kahn's Topological Sort) are executed entirely offline. If any cycle exists within the consequence or derivation graph, it immediately triggers a typed refusal. Partial traversal results are never emitted.
- **Cryptographic Binding**: The canonicalized graph is hashed (`rdf_digest`), and the SHACL validation results are hashed (`admission_digest`). This prevents tampering and seals the environment into a one-way boundary before handoff.

## 3. `mfw-codegen` (Hardware Bitmask and C-ABI Generation)
`mfw-codegen` takes the validated, flattened data and produces strict, allocation-free C-ABI Rust logic that obeys the Radon Law.
- **Hardware Bitmasks (SWAR)**: Topological dependencies are flattened into fixed-width C-ABI hardware bitmasks (e.g., `u64`). Entities are mapped to strict `pred_mask` (execution prerequisites) and `succ_mask` (downstream consequences), allowing the hot path to resolve execution blindly via $O(1)$ SIMD-Within-A-Register (SWAR) bitwise operations.
- **C-ABI Struct Generation**: Emits the logic as fixed-width, cache-aligned `#[repr(C, align(64))]` C-ABI Rust structs (`pub const` arrays). Ontological entities are deterministically mapped to zero-indexed array offsets.
- **$CC=1$ Branchless Logic Generation**: Eliminates dynamic iteration. Emits purely straight-line sequential state transitions or generates macros (e.g., `unroll_n_static!`) so the compiler drops all loop backedges. The generated struct perfectly maps the domain without needing fallback initialization logic or `unwrap()` calls in the hot path.
- **Substrate Integrity Verification**: Embeds cryptographic hashes (`RDF_INPUT_DIGEST`, `GENERATOR_SOURCE_DIGEST`) directly into `cmca_generated.rs`. The output is passed through the `bcinr-cheat-scanner` (AST-level scan for hidden branches, magic constants, and scanner evasion tactics) and Object-Code Audits to physically demonstrate $CC=1$ and zero conditional jumps or panic paths in the final release assembly.

## The Handoff to `cmca_generated.rs`
Once the pipeline (Meaning $\rightarrow$ SHACL $\rightarrow$ Codegen) completes successfully, the deterministic hashes are chained into the `cmca_generation_manifest.json` and `cmca_generation_receipt.json`. Finally, `mfw-codegen` emits `cmca_generated.rs`—a purely fixed-width, zero-allocation artifact. This file safely crosses the `Gamma_CMCA` boundary, allowing the `bcinr` Hot Path to consume the entire semantic ontology via zero-copy embedded arrays without ever branching or allocating memory.
