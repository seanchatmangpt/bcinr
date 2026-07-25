# Slow Rail BLAKE3 Manifest Generation in BCINR

In the `bcinr` architecture, the **Hot Path** (Authoritative Runtime) is subjected to absolute deterministic constraints (the Radon Law $CC=1$ and zero-allocation). It cannot parse complex Semantic Web data or traverse arbitrary graphs. Instead, it relies on strict `u32` equality arrays and fixed-width tables. 

To bridge the gap between flexible ontology inputs and rigid execution logic, the **Slow Rail** operates entirely offline and Ahead-of-Time (AOT). It performs all parsing and graph traversals and creates rigid artifacts that the Hot Path can execute blindly.

To prevent supply-chain drift and ensure the Hot Path executes mathematics mathematically bound to the authorized ontology, the Slow Rail relies on **content-identity digests** computed via the **BLAKE3** hash algorithm.

## Calculating the Digests

The Slow Rail applies strict canonicalization rules before hashing the inputs to prevent non-deterministic variations (like whitespace or element ordering):

1. **`RDF_INPUT_DIGEST` (`rdf_digest`)**: 
   - **Canonicalization**: Before hashing, the semantic graph is serialized into **canonical N-Quads**. Quads are sorted lexicographically by their full quad string, UTF-8 encoded, with LF line endings, and with all trailing whitespace and trailing blank lines stripped out.
   - **Calculation**: The BLAKE3 digest of this deterministic N-Quads byte stream is computed.

2. **`GENERATOR_SOURCE_DIGEST` (`generator_digest`)**:
   - **Sequencing & Concatenation**: The source files of the generator script are concatenated in a fixed, declared order exactly as they appear in the source tree. 
   - **Canonicalization**: The concatenated source code is treated as pure UTF-8 bytes with no post-hoc re-encoding or whitespace normalization.
   - **Calculation**: The BLAKE3 digest of this concatenated byte stream is computed.

3. **Other State Digests** (`admission_digest`, `numeric_profile_digest`, `formula_registry_digest`):
   - **Canonicalization**: Any JSON-based metadata uses **RFC 8785-style Canonical JSON (JCS)**. Object keys are recursively sorted byte-wise, insignificant whitespace is stripped, and numeric types strictly prohibit representations like floating-point decimals.
   - **Calculation**: The BLAKE3 digest is computed from the canonicalized JSON byte stream.

All final digests are aggregated and formatted as lowercase hex strings prefixed with `blake3:` (e.g., `blake3:<64_hex_chars>`).

## Aggregating and Sealing the Artifact (Gamma_CMCA)

Once the data is computed, sorted, and flattened into rigid numeric arrays, the Slow Rail generation pipeline (e.g., via `mfw-meaning`) outputs three strictly bound artifact files, seamlessly sealing the environment into a one-way boundary called the **`Gamma_CMCA` Contract**:

1. **`cmca_generation_manifest.json`**: 
   - This manifest explicitly records the full suite of aggregated BLAKE3 digests in a `digests` block.
   - Alongside `rdf_digest` and `generator_digest`, it also aggregates a `generated_payload_digest`, which is the byte-for-byte BLAKE3 hash of the generated Rust source file.
   - It records the declared dimensions, numeric profiles, and the fixed `generator_source_order`.

2. **`cmca_generated.rs`**: 
   - The compiled branchless Rust Intermediate Representation (IR). 
   - Multi-dimensional properties are mapped to deterministic array offsets (like the `LAMBDA` Matrix or `OBJECT_REGISTRY`). 
   - It directly embeds the calculated digests as `pub const` string slices:
     ```rust
     pub const RDF_INPUT_DIGEST: &str = "blake3:...";
     pub const GENERATOR_SOURCE_DIGEST: &str = "blake3:...";
     ```

3. **`cmca_generation_receipt.json`**: 
   - A BLAKE3 hash-chained event record of the generation run connecting validation events.

## Isolated Verification in the Hot Path Build

To verify that the offline Slow Rail correctly constructed the semantic state without injecting dynamic logic into the Hot Path, an offline validation check runs at build or test time. 

The **`VerifyGeneratedProfile`** logic checks this seal isolated from the Slow Rail (without any network access or Python logic):
- It independently parses the generated Rust source bytes and the manifest JSON.
- It freshly recalculates the BLAKE3 digest of the `cmca_generated.rs` bytes and asserts it perfectly matches the `generated_payload_digest` recorded in the manifest.
- It asserts that the fixed dimension bounds (`N`, `F`, `K`, `Q`) exactly match the emitted `pub const` declarations. 
- Any divergence results in a typed structured refusal, eliminating the risk of unsealed state entering the final binary.
