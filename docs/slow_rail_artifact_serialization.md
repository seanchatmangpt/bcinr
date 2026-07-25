# Slow Rail Artifact Serialization in BCINR

In the `bcinr` (BranchlessCInRust) architecture, a strict civilizational-scale boundary exists between the non-deterministic, unbounded **Slow Rail** and the deterministic, constant-time **Hot Path** (Authoritative Runtime). Because the Hot Path is strictly bound by the **Radon Law ($CC=1$)** and a **Zero-Allocation Boundary** (`#![no_std]`, `0` heap allocations), traditional runtime parsing—like `serde_json`, Protocol Buffers, or variable-length decoders—is categorically banned.

To bridge this gap, the Slow Rail handles all complex graph parsing and validation Ahead-Of-Time (AOT), physically serializing the derived structures so the Hot Path can ingest them instantly via zero-copy `mmap` or safe pointer-casting.

## 1. Ahead-of-Time (AOT) Flattening
The Slow Rail (acting as the producer) ingests raw, unbounded Semantic Web data (RDF/SHACL). It resolves all multi-dimensional relationships, dependency cycles, and variable-length strings by transforming them into flattened, dense topological arrays.
- String labels are interned into static, bounded byte arenas (e.g., `LabelSlab`).
- Execution dependencies are translated into pre-computed bitmasks (e.g., `pred_mask`, `succ_mask`).

## 2. Hardware-Aligned, Fixed-Width Structs (`#[repr(C, align(64))]`)
To enable pointer-casting without parsing, the Slow Rail serializes the flattened data into binary payloads that perfectly mirror memory-mapped structs. These structures follow strict layout constraints:

*   **`#[repr(C)]`**: Enforces a strict, predictable C-ABI memory layout, preventing the Rust compiler from arbitrarily reordering fields.
*   **`align(64)`**: Forces the structs to perfectly align with CPU cache lines (64 bytes or 128 bytes), preventing false sharing and ensuring optimal memory access.
*   **Explicit Padding**: To prevent uninitialized bytes from leaking or breaking deterministic hashing, structs use explicitly sized padding arrays (e.g., `_pad: [u8; 36]`) to guarantee exact byte widths.

**Example: `Powl64Op`**
```rust
#[repr(C, align(64))]
pub struct Powl64Op {
    pub pred_mask: u64,     // Predecessor completion mask
    pub succ_mask: u64,     // Successor activation mask
    pub ctrl: u64,          // Control word
    pub op_kind: OpKind,    // Fixed enum byte
    pub choice_group: u8,   // XOR choice mapping
    pub depth: u8,          // Hierarchy nesting
    pub fan_out: u8,        // Outgoing edge count
    pub _pad: [u8; 36],     // Explicit padding to exactly 64 bytes
}
```

## 3. Zero-Copy Ingestion via Pointer Casting
Because the binary payload written by the Slow Rail is functionally identical to the in-memory array representation (`[Powl64Op; N]`), the Hot Path can entirely bypass the deserialization step.

1.  **Memory Mapping**: The Hot Path loads the artifact file into memory (e.g., via `mmap` or `include_bytes!`).
2.  **Pointer Casting**: The raw `&[u8]` byte slice is cast directly to a statically typed slice like `&[Powl64Op]`. This is typically done using safe, zero-cost abstractions like the `bytemuck` crate, which statically verifies that the target struct has no padding irregularities or invalid bit patterns.
3.  **O(1) Access**: The Hot Path immediately interacts with the struct fields natively. This ensures $O(1)$ initialization time, zero allocations, and zero branching.

## 4. Cryptographic Binding (The `Gamma_CMCA` Boundary)
To ensure the Hot Path does not cast a corrupted or malformed payload, the serialization is protected by the `Gamma_CMCA` artifact boundary. The generated payload is cryptographically bound using deterministic **BLAKE3 digests** (e.g., `generated_payload_digest`). Before ingestion, the Hot Path verifies the manifest digests deterministically. If a digest mismatch occurs, it triggers a structured `compile_error!` or typed startup refusal—never falling back to runtime recovery or speculative mutation.
