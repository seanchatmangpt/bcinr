# Slow Rail RDF Parsing Isolation in BCINR

## The Fundamental Conflict

In the `bcinr` (BranchlessCInRust) architecture, the core execution environment—the **Hot Path** (Authoritative Runtime)—is governed by absolute deterministic constraints:
- **The Radon Law ($CC=1$)**: No `if`, `match`, or data-dependent loops are permitted. Execution must be modeled entirely via bitwise polynomials and mask-based state selection.
- **The Zero-Allocation Boundary**: The hot path operates under `#![no_std]` and strictly bans heap allocations (e.g., `String`, `Vec`, `Box`).
- **Bounded Execution**: Unbounded iterations or dynamic pointer chasing are mathematically prohibited.

By contrast, parsing **Semantic Web/RDF** data (e.g., RDF/Turtle, SHACL validation) is intrinsically **unbounded, variable-length, and allocation-heavy**. Parsing an RDF ontology requires traversing arbitrary cyclic graphs, resolving variable-length string labels (URIs), and dynamically allocating memory for tree nodes. 

To resolve this conflict, `bcinr` relies on an architectural division called the **Slow Rail**. All semantic processing is quarantined to this Slow Rail, guaranteeing that the Authoritative Hot Path achieves cognitive and semantic complexity without violating its deterministic, allocation-free constraints.

## 1. Topologically Disjoint Boundaries

The isolation is achieved through absolute **Topological Disjointness**:
- **Offline / Ahead-of-Time (AOT) Producer**: Semantic admission, RDF parsing, SHACL validation, and index assignment are performed entirely offline (or out-of-band) on the Slow Rail. This is managed via python generator scripts (`generator.py`), legacy generators in quarantine boundaries, or external tooling (`mfw-meaning`).
- **Strict Dependency Isolation**: The runtime `bcinr-cmca` consumer crate is forbidden from compiling any semantic logic into its execution path. If any RDF parser (`oxigraph`), SHACL/ShEx engine, or graph store is introduced into the release dependency graph of the authoritative crate, it constitutes an architectural breach, triggering a gate violation and a `compile_error!`.
- **No Dynamic Generation (`build.rs`)**: Generation must never occur dynamically as a side-effect of `cargo build` in the runtime. The runtime crate solely reads statically checked-in artifacts. 

## 2. Flattening Graph Traversals into Bitmasks

Because graph traversal (e.g., chasing node pointers, finding topological ordering) inherently branches and requires variable time depending on the graph's depth, the Slow Rail fundamentally transforms the graph's geometry:
- **Cycle Proofing**: The Slow Rail executes Kahn’s Topological Sort to detect cycles and validate execution dependencies. (Kahn's algorithm uses dynamic queues, making it illegal on the Hot Path).
- **Execution Dependencies to Masks**: Once the graph is validated and topological order is established, execution relationships are flattened into pre-computed hardware bitmasks. For example, dependencies become `pred_mask` (predecessors that must be completed) and `succ_mask` (successors to activate). 
- **Consequence**: The Hot Path never "walks" a graph; it simply performs a bitwise `AND` or `XOR` operation on these fixed-width integer masks (e.g., `u64`) to determine state transitions.

## 3. Banishing Variable-Length Allocations

The Slow Rail actively purges all variable-length data (like URI strings and dynamic property values) before producing the final payload:
- **Interning Strings**: String labels are interned into static, bounded byte arenas (e.g., `LabelSlab`), preventing runtime `String` allocations.
- **Exact Numeric Conversion**: Numeric values from the RDF ontology are strictly converted into Q16.16 fixed-point representations via exact decimal arithmetic in the Slow Rail, removing the need for floating-point or dynamic numeric parsing.
- **Hardware-Aligned Payloads**: Complex semantic rules are serialized into fixed-width, cache-aligned `#[repr(C, align(64))]` structs, carefully padded with exact byte arrays (`_pad: [u8; 36]`) to ensure predictable C-ABI memory layouts.

## 4. Cryptographic Binding (The `Gamma_CMCA` Contract)

The transition of the processed logic from the Slow Rail to the Hot Path is strictly governed by the **`Gamma_CMCA` Contract**.
To ensure that the static tables compiled into the hot path accurately map back to the approved semantic graph:
- **BLAKE3 Digests**: The Slow Rail emits artifacts bounded by content-identity hashes. These include the `RDF_INPUT_DIGEST` (hash of canonical N-Quads of the RDF graph), `GENERATOR_SOURCE_DIGEST`, and a `generated_payload_digest`.
- **Zero-Copy Ingestion**: The Hot Path consumes the resulting payload either via direct `pub const` embedded Rust arrays (e.g., `cmca_generated.rs`) or zero-copy pointer casting from an `mmap` byte slice to the strict C-aligned structs (e.g., `&[Powl64Op]`). 
- **Isolated Offline Verification**: At build time, the runtime's `VerifyGeneratedProfile` parses the generated manifest and mathematically checks the bounding invariants (dimension bounds, layout schema, digest equivalence) *without* invoking any RDF parsing logic. Any divergence results in a typed structured refusal, eliminating the risk of supply-chain drift.

## Summary

By strictly performing RDF parsing as an AOT operation, isolating its dependencies, mapping graph traversals to bounded topological bitmasks, and generating explicitly padded `#[repr(C)]` memory layouts bounded by BLAKE3 identity hashes, `bcinr` guarantees that the semantic state is cleanly injected into the Authoritative Hot Path while leaving behind all strings, branches, cycles, and heap allocations.
