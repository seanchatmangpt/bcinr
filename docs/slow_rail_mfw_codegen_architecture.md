# Slow Rail `mfw-codegen` Architecture in `bcinr`

In the BranchlessCInRust (BCINR) ecosystem, the Authoritative Hot Path is strictly bound by the **Radon Law ($CC=1$)**, `#![no_std]`, and absolute zero heap allocations. Semantic Web data (RDF/Turtle), characterized by dynamic graphs, variable-length URIs, and dynamic typing, natively violates all of these constraints. 

To bridge this gap, the **Slow Rail** utilizes a strict Ahead-of-Time (AOT) pipeline composed of `mfw-meaning`, `mfw-shacl`, and `mfw-codegen`. The final step—`mfw-codegen`—is responsible for safely emitting a purely deterministic static Rust IR (`cmca_generated.rs`) that can safely cross the `Gamma_CMCA` boundary.

## 1. Prerequisites: The Semantic Pipeline
Before `mfw-codegen` generates output, the graph must be mathematically stable:
1. **`mfw-meaning`**: Canonicalizes the dynamic RDF graph, interning string URIs, normalizing values, and enforcing that sequence properties (like `cmca:measureIndex`) are injective, contiguous, and capacity-bounded.
2. **`mfw-shacl`**: Structurally enforces the presence of required mapping properties and ensures the absence of topological cycles via offline dependency proofs.

## 2. Deterministic Object Mapping: Dynamic URIs to $O(1)$ Indices
To ensure the Authoritative Hot Path never allocates HashMaps or performs dynamic string comparisons, `mfw-codegen` must resolve all URIs into fixed array offsets offline.

1. **Explicit Index Properties:** Rather than assigning arbitrary IDs, `mfw-codegen` relies on ontological layout definitions. For instance, `cmca:measureIndex` defines exactly where a `cmca:MeasureHead` lives in memory.
2. **Offline Sorting:** The generator extracts these semantic properties, deterministically sorting the entities by their indices. 
3. **Array Bounds Definition:** By counting the sorted entities, fixed compile-time capacity bounds are established (e.g., $K$ for measures, $Q$ for lenses) guaranteeing that sequences are bounded without runtime calculation.
4. **Static Constants Emission:** The dynamic URIs are then baked directly into the static Rust IR as zero-indexed `usize` constants:
   ```rust
   pub const K: usize = 4;
   pub const Q: usize = 4;
   pub const MEASURE_RETRIEVAL: usize = 2; // Derived offline from URI + cmca:measureIndex
   ```

## 3. Emitting `cmca_generated.rs` (The Static IR)
With URIs replaced by deterministic integer offsets, `mfw-codegen` emits memory-safe, C-ABI compliant logic:

### Fixed-Size Multi-Dimensional Arrays
Complex relationships are mapped precisely using the generated indices. A cross-dimensional property linking a measure and a lens becomes a deterministic $O(1)$ lookup in a two-dimensional fixed-point matrix:
```rust
#[repr(C, align(64))]
pub static LAMBDA: [[NonNegativeFixed; Q]; K] = [
    // Pre-calculated fixed-point vectors
    [NonNegativeFixed::from_bits(26214), /* ... */], 
    // ...
];
```

### Topological Flattening and SWAR Bitmasks
Relationships (like `cmca:dependsOn`) cannot be resolved at runtime using pointer traversal or recursion. `mfw-codegen` runs **Kahn's Topological Sort** offline. 
The semantic graph dependencies are then flattened into fixed-width C-ABI hardware bitmasks (e.g., `u64`):
- **`pred_mask`:** Indicates execution prerequisites using the mapped indices.
- **`succ_mask`:** Indicates downstream consequences using the mapped indices.
At runtime, execution progresses purely via $O(1)$ SIMD-Within-A-Register (SWAR) bitwise operations.

### Enforcing the Radon Law ($CC=1$)
To guarantee zero looping, `mfw-codegen` eliminates dynamic iterators. It produces generated macros (e.g., `unroll_k_static!`, `unroll_q_static!`) or purely sequential, straight-line state transitions, ensuring that the rust compiler unrolls all loop backedges and strictly adheres to cyclomatic complexity 1.

## 4. Cryptographic Sealing
To uphold Substrate Constitution Rule 21 ("Generated-code law"), `mfw-codegen` cryptographically seals the mapped data. Exact cryptographic hashes (`RDF_INPUT_DIGEST` and `GENERATOR_SOURCE_DIGEST`) are embedded directly into `cmca_generated.rs`. This ensures an unbroken chain of custody, proving that every physical index offset accessed in $O(1)$ time structurally traces back to the securely validated Semantic ontology.
