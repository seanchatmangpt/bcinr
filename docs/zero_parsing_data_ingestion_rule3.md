# BCINR Zero-Parsing Data Ingestion Pattern

## 1. The Core Mandate (Rule 3)
Under the constitutional rules of `AGENTS.md`, the `bcinr` hot path (the Authoritative Runtime) is governed by absolute deterministic constraints:
* **The Radon Law ($CC=1$)**: No data-dependent branches, `if` statements, or dynamic loop terminations.
* **The Zero-Allocation Boundary**: Execution must be `#![no_std]` and strictly perform zero heap allocations (no `String`, `Vec`, etc.).
* **Rule 3 - No Runtime Parsing**: Standard data ingestion practices like decoding JSON, parsing XML/RDF graphs, or reading variable-length buffers are mathematically prohibited because they are inherently unbounded and allocation-heavy.

## 2. Topologically Disjoint Boundaries (The Slow Rail)
To ingest complex data without runtime parsing, `bcinr` utilizes an architectural division known as the **Slow Rail**. All variable-length, graph-based parsing (like Semantic Web/RDF/Turtle processing, Kahn's Topological Sort, or SHACL validation) is quarantined here and performed offline/Ahead-Of-Time (AOT). 

The runtime crate simply consumes statically checked-in artifacts. No parsing dependencies (e.g., `serde_json`, `oxigraph`) are ever allowed into the hot path's dependency graph. 

## 3. Graph Flattening and Mask-Based State Selection
Because pointer-chasing and dynamic traversal inherently branch and vary in execution time, the Slow Rail transforms arbitrary relational graphs into flat hardware bitmasks.
* **Dense 0-Indexing**: Source nodes are densely packed so their ID matches their direct array index.
* **Masked Wiring**: Predecessor and successor execution edges are flattened into integer fields (e.g., `pred_mask: u64`, `succ_mask: u64`). 
* **Branchless Execution**: The hot path NEVER walks the tree; it solely computes state transitions in constant time via bitwise `AND`/`OR`/`XOR` masks on these `u64` integers.

## 4. Hardware-Aligned, Fixed-Width Structs
Variable-length data is aggressively banished from the payload:
* **String Interning**: Unbounded string labels are stripped out and interned into bounded byte arenas (e.g., `LabelSlab` of `[u16-len-le][utf8-bytes]`).
* **Cache-Aligned Structs**: Data is compiled into exact, hardware-aligned structs. For example, `Powl64Op` uses `#[repr(C, align(64))]` to consume exactly one 64-byte CPU cache line.
* **Explicit Padding**: All structs use explicitly sized padding arrays (e.g., `_pad: [u8; 36]`) to prevent any uninitialized bytes in the C-ABI memory layout.

## 5. Safe, Zero-Cost Casting (`bytemuck`)
Without a parser to read the incoming byte streams, the hot path performs **Zero-Copy Ingestion** using the `bytemuck` crate. 
* Rather than looping over fields and dynamically instantiating structs, raw `&[u8]` byte slices (loaded via `mmap` or `include_bytes!`) are directly cast into typed slices like `&[Powl64Op]`. 
* `bytemuck` guarantees memory safety by verifying statically at compile time that the target structures contain no padding irregularities or invalid bit patterns (implementing traits like `Pod` and `Zeroable`).
* This achieves **$O(1)$ initialization** with zero allocations and zero branching validation logic.

## 6. Cryptographic Binding (The `Gamma_CMCA` Contract)
Because the runtime is blindly casting byte payloads into hardware structs, `bcinr` protects structural integrity via cryptographic boundaries rather than runtime structural checks.
* The Slow Rail emits artifacts bound by **BLAKE3 Digests** representing the canonical origin source (e.g., `RDF_INPUT_DIGEST`).
* Before casting via `bytemuck`, the hot path mathematically checks the schema bounds and digest equivalences. If any corruption or dimensional divergence is detected, it results in a typed structured refusal, establishing a secure chain of custody between semantic parsing and branchless execution.
