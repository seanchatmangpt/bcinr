# Semantic Artifact Digests in BCINR

In the BCINR project, the **Slow Rail** handles the transformation of flexible, semantic ontologies (RDF/Turtle) into a rigid, branchless Rust Intermediate Representation (IR). To preserve absolute traceability and prevent supply-chain drift across this boundary (the **`Gamma_CMCA`** contract), the system utilizes strict **content-identity digests**.

## Mathematical Generation of Digests

All artifact digests are computed using the **BLAKE3** hash algorithm and formatted as lowercase hex strings prefixed with `blake3:` (e.g., `blake3:<hex>`). 

The digests are mathematically generated with strict canonicalization rules:
- **`RDF_INPUT_DIGEST` (`rdf_digest`)**: The BLAKE3 digest of the admitted RDF graph. Before hashing, the graph is serialized as canonical N-Quads (quads sorted lexicographically by their full quad string, UTF-8 encoded, LF line endings, with no trailing whitespace or trailing blank lines).
- **`GENERATOR_SOURCE_DIGEST` (`generator_digest`)**: The BLAKE3 digest of the generator source files. The source files are concatenated in a fixed, declared order exactly as they are committed to the source tree (as UTF-8 bytes, with no post-hoc re-encoding or whitespace normalization).
- **Other Digests**: Digests representing JSON data (like `admission_digest`, `numeric_profile_digest`, `formula_registry_digest`) enforce RFC 8785-style Canonical JSON (JCS) before hashing. This means object keys are recursively sorted byte-wise, insignificant whitespace is stripped, and numeric types have strict representations (e.g., no floating-point decimals allowed).

## Embedding into the Gamma_CMCA Code

The Slow Rail generation pipeline (via `mfw-meaning` and Python generators) outputs three strictly bound artifact files, embedding the digests seamlessly:

1. **`cmca_generation_manifest.json`**: This manifest explicitly records the full suite of digests in a `digests` object block, including the `rdf_digest`, `generator_digest`, and a `generated_payload_digest` representing the byte-for-byte hash of the generated Rust source file.
2. **`cmca_generation_receipt.json`**: A BLAKE3 hash-chained event record of the generation run linking the various validation and generation steps.
3. **`cmca_generated.rs`**: The generated Rust static IR directly embeds these semantic digests as `pub const` string slice constants:
   ```rust
   pub const RDF_INPUT_DIGEST: &str = "blake3:...";
   pub const GENERATOR_SOURCE_DIGEST: &str = "blake3:...";
   ```

## Verification to Prevent Supply-Chain Drift

The Authoritative Runtime (Hot Path) requires execution to be completely allocation-free and branchless. Therefore, to ensure that the static tables compiled into the hot path accurately map back to the approved semantic graph without runtime overhead, verification is performed strictly **offline at build or test time**.

The validation is handled by **`VerifyGeneratedProfile`** (`crates/bcinr-cmca/src/artifact.rs`) functioning as a strict gate:
1. **Isolated Verification**: It reads the generated Rust source bytes and the manifest JSON directly, without invoking any RDF generation tools, Python, or network dependencies.
2. **Digest Re-computation**: It freshly computes the BLAKE3 hash of the `cmca_generated.rs` source bytes and asserts that it matches the `generated_payload_digest` declared in the manifest.
3. **Structural and Conservation Laws**: Beyond digests, it verifies that the fixed dimension bounds (`N`, `F`, `K`, `Q`) in the manifest exactly match the `pub const` declarations in the generated Rust. It also calculates and proves that mathematical conservation bounds hold across the arrays (e.g., `LEAF_FLOOR_BASE` and `LEAF_FLOOR_REMAINDER`).
4. **Typed Refusals**: Any unexplained drift, missing constants, digest mismatch, or unsupported schema immediately results in a typed refusal (e.g., `GeneratedProfileRefusal::PayloadDigestMismatch`). Silent best-effort acceptance or warnings are entirely prohibited by the BCINR constitution.

This rigorous chain ensures that the compiled branchless runtime is mathematically proven to be generated securely from the explicitly authorized semantic inputs.
