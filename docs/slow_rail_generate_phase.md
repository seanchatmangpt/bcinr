# Slow Rail `generate` Phase in BCINR: Flattening RDF to Static Bitmasks

In the `bcinr` architecture, the Authoritative Runtime (Hot Path) operates under strict deterministic constraints ($CC=1$, `#![no_std]`, and zero heap allocation). Dynamic graph traversal and dependency resolution are strictly prohibited at runtime. To resolve this, the **Slow Rail** performs ahead-of-time (AOT) dependency flattening during the `generate` phase, crossing into the Hot Path via the rigid `Gamma_CMCA` boundary.

Following the `validate` step (which seals the input via `rdf_digest` and `admission_digest`), the Slow Rail invokes the topological generator to translate the unbounded semantic ontology into static C-ABI bitmasks.

## 1. Ahead-of-Time Application of Kahn's Algorithm

Kahn's Topological Sort fundamentally violates Hot Path runtime laws because it relies on dynamic queue allocations and data-dependent loop termination. Therefore, it is strictly confined to the Slow Rail. 

The generator reads the validated semantic dependencies (e.g., `cmca:dependsOn`) and executes Kahn's algorithm offline to verify the graph is acyclic and to establish a deterministic execution order.

## 2. Deterministic Array Index Mapping

Before bitmasks can be generated, the semantic entities must be mapped to rigid array offsets. The generator uses explicitly defined properties (like `cmca:measureIndex`) to map nodes to fixed zero-indexed positions. 
This establishes rigid, fixed sequence bounds (such as `K` for Measure Heads and `Q` for Lenses) required for zero-allocation state arrays.

## 3. Flattening Dependencies into Hardware Bitmasks

With the entities strictly ordered and indexed, the dependency chains are flattened into fixed-width C-ABI hardware bitmasks (typically `u64`). 
For each entity mapped to an index `i`:
- **`pred_mask` (Predecessor Mask):** A bitwise mask where the `j`-th bit is set if entity `j` must complete before `i` can execute.
- **`succ_mask` (Successor Mask):** A bitwise mask where the `k`-th bit is set if entity `k` is a downstream consequence of `i`.

These masks represent the entire execution topology in an $O(1)$ SWAR (SIMD Within A Register) compatible format, eliminating all need for pointer-chasing or dynamic lookups in the Hot Path.

## 4. Emitting `cmca_generated.rs`

The generated hardware bitmasks, along with pre-calculated static constants and fixed-point data (such as consequence tables and the `LAMBDA` Matrix), are formatted as static Rust IR (`pub const` arrays and C-ABI structs).

This output is emitted directly into `cmca_generated.rs`. This design guarantees that:
- The Hot Path executes blindly and branchlessly over the bitmasks.
- No dynamic memory is allocated during execution.
- The generated payload is cryptographically sealed in the manifest (recording the `generated_payload_digest` alongside the bounds) to prevent supply-chain drift, thereby enforcing the one-way **`Gamma_CMCA` Contract**.

Through this pipeline, the Slow Rail guarantees the Hot Path receives a perfectly branchless execution structure, mathematically proven to be acyclic and safely executable in deterministic, constant time.
