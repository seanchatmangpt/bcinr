# Slow Rail and Hot Path Architectural Separation in BCINR

According to the `AGENTS.md` Constitution (Rule 6), the BCINR project enforces a strict, civilizational-scale dichotomy between the **Authoritative Runtime (Hot Path)** and the **Slow Rail**.

## Why RDF Parsing and SHACL Validation are on the Slow Rail
The Authoritative Runtime is governed by absolute deterministic constraints:
- **The Radon Law ($CC=1$)**: No branches (`if`, `match`, data-dependent loops).
- **Zero-Allocation**: `#![no_std]` environment with zero heap allocations.
- **Bounded Execution**: Fixed-width, deterministic execution.

Parsing Semantic Web data like RDF and validating SHACL constraints intrinsically violate these laws because they are unbounded, variable-length, and allocation-heavy. They require graph traversal, string resolution (URIs), and dynamic memory allocation. Thus, they are explicitly relegated to the **Slow Rail**, where branching, allocation, and unbounded iterations are permitted.

## Architectural Separation Mechanisms

The separation is achieved through strict isolation and transformation boundaries:

### 1. Topological Disjointness and Dependency Isolation
The Authoritative Runtime is strictly forbidden from importing any semantic logic into its execution path. If any RDF parser (like `oxigraph`), SHACL engine, or graph store is introduced into the release dependency graph of the authoritative crate, it constitutes a constitutional breach and triggers a `compile_error!`. All semantic parsing and validation happen entirely offline via Ahead-Of-Time (AOT) tooling (like Python scripts or `mfw-meaning`).

### 2. Flattening Graphs into Bitmasks
Because graph traversals (cycle detection, topological sorts) require dynamic data structures, the Slow Rail handles all graph resolution. Execution dependencies are flattened into pre-computed hardware bitmasks (e.g., `pred_mask`, `succ_mask`). The Hot Path never traverses a graph; it simply performs constant-time bitwise operations (like `AND` or `XOR`) on these masks to manage state transitions.

### 3. Banishing Variable-Length Allocations
Before data crosses into the Hot Path, the Slow Rail purges all variable-length data:
- **String Interning**: URIs and labels are interned into static byte arenas.
- **Numeric Conversion**: Decimals from RDF are converted to fixed-point (e.g., Q16.16) using exact decimal arithmetic.
- **Hardware-Aligned Payloads**: Complex semantic rules are serialized into fixed-width, cache-aligned `#[repr(C, align(64))]` C-ABI compliant structs.

### 4. Cryptographic Binding (The `Gamma_CMCA` Contract)
The transition from the Slow Rail to the Hot Path is strictly one-way and cryptographically sealed. 
- The Slow Rail emits artifacts bounded by content-identity BLAKE3 hashes (e.g., `RDF_INPUT_DIGEST`, `admission_digest` for SHACL).
- The Hot Path consumes the resulting payload as pure static constants (e.g., embedded Rust arrays like `cmca_generated.rs`) or zero-copy memory-mapped C-aligned structs.
- At build time, verification mathematically checks bounding invariants and digest equivalence without invoking any RDF parsing logic, structurally guaranteeing supply-chain integrity.
