# Cryptographic Substrate Binding and Verification in BCINR

In the `bcinr` architecture, the Authoritative Runtime (the "Hot Path") operates under absolute deterministic laws ($CC=1$, zero allocation, no branching). Since parsing complex Semantic Web Data (RDF) conflicts with these constraints, all ontology translation happens entirely offline in an Ahead-of-Time (AOT) pipeline known as the **Slow Rail**. 

To bridge this gap securely, the Slow Rail relies on **content-identity digests** computed via the **BLAKE3** hash algorithm.

## 1. How Bindings are Created (The Slow Rail)

Before generating any code, the `mfw` generator canonicalizes inputs and computes their BLAKE3 digests:
- **`RDF_INPUT_DIGEST`**: Computed from the canonicalized N-Quads of the dynamic `.ttl` ontology.
- **`GENERATOR_SOURCE_DIGEST`**: Computed from the concatenated source code of the generator script itself.

These are formatted as `blake3:<64_hex_chars>`.

The pipeline outputs three strictly bound artifact files, seamlessly sealing the environment into a boundary called the **`Gamma_CMCA` Contract**:
1. **`cmca_generated.rs`**: The compiled branchless Rust Intermediate Representation. The calculated digests are embedded directly into this file as `pub const` string slices.
2. **`cmca_generation_manifest.json`**: Records the suite of BLAKE3 digests. Crucially, it records the **`generated_payload_digest`**, which is the byte-for-byte BLAKE3 hash of `cmca_generated.rs`.
3. **`cmca_generation_receipt.json`**: A hash-chained event record connecting validation events.

## 2. How Bindings are Verified (The Hot Path / Consumer)

To guarantee that the Hot Path executes logic mathematically bound to the original ontology, a strict offline verification step runs via the **`cargo make verify-generated`** task.

This gate runs the `VerifyGeneratedProfile` test (found in `crates/bcinr-cmca/src/artifact.rs`) completely independently—without invoking Python, network requests, or RDF tooling. The verification enforces the following:

1. **Payload Digest Verification**: It independently calculates the BLAKE3 digest of the bytes in `cmca_generated.rs` and asserts it perfectly matches the `generated_payload_digest` recorded in `cmca_generation_manifest.json`.
2. **Structure Verification**: It verifies that the fixed dimension bounds (`N`, `F`, `K`, `Q`) recorded in the manifest match the actual `pub const` array lengths unrolled in the Rust artifact.
3. **Floor Table Conservation**: It re-derives the scalar pairs `LEAF_FLOOR_BASE` and `LEAF_FLOOR_REMAINDER` from the generated code, asserting they perfectly sum to the conservation target (`LEAF_FLOOR_CONSERVATION_TARGET`) across all states.

## 3. Ensuring Supply-Chain Integrity

This system physically prevents several classes of supply-chain attacks and architectural drift:

- **Prohibits Hand-Editing**: Hand-editing generated output is strictly prohibited by Rule 21. If an engineer manually alters `cmca_generated.rs`, the file's hash will drift from `generated_payload_digest`, instantly failing the `verify-generated` CI gate with a typed structural refusal (`GeneratedProfileRefusal::PayloadDigestMismatch`).
- **Cryptographic Binding**: By embedding the `RDF_INPUT_DIGEST` and `GENERATOR_SOURCE_DIGEST` into the artifact's struct declarations, the final binary execution is computationally tied directly to the exact ontology and exact pipeline state that built it.
- **Deterministic Reproducibility**: The verification doesn't rely on arbitrary scripts. The 4-step Rule 21 process (`clean generation → digest output → regenerate → verify byte-identical output`) ensures exact replica standing regardless of environmental drift.
- **Enforces Core Runtime Laws**: Passing the artifact payload verification also passes it on to downstream static checks like `bcinr-cheat-scanner` and `audit-object-code` (`otool`/`objdump`), eliminating the risk of branches or allocator logic hiding within macro expansions.
