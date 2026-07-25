# Slow Rail Cryptographic Substrate Binding in BCINR

In the `bcinr` (BranchlessCInRust) architecture, the Authoritative Runtime (the "Hot Path") operates under absolute deterministic laws: $CC=1$ (Cyclomatic Complexity of 1, meaning strictly branchless), `#![no_std]`, and zero heap allocations. Since parsing and validating dynamic Semantic Web data (RDF/Turtle) fundamentally conflicts with these constraints, this complexity is quarantined to the **Slow Rail**—an offline, Ahead-of-Time (AOT) pipeline powered by the `mfw` suite.

To ensure the Hot Path executes logic mathematically bound to the authorized ontology without ever parsing it directly, the `mfw` pipeline employs a strict **Cryptographic Substrate Binding**. Here is how it seals the execution boundary into the `Gamma_CMCA` artifact.

## 1. Canonicalization and BLAKE3 Hashing

Before generating any code, the `mfw` suite strictly canonicalizes the inputs to prevent non-deterministic variations (such as insignificant whitespace or ordering differences) and computes content-identity digests using **BLAKE3**.

*   **`RDF_INPUT_DIGEST` (`rdf_digest`)**: The dynamic `.ttl` ontology is ingested and normalized by `mfw-meaning` into **Canonical N-Quads**. The quads are sorted lexicographically by their full quad string, UTF-8 encoded with UNIX line endings (LF), and stripped of trailing whitespace or blank lines. The BLAKE3 digest of this deterministic byte stream becomes the `RDF_INPUT_DIGEST`.
*   **`GENERATOR_SOURCE_DIGEST` (`generator_digest`)**: To prevent supply-chain drift in the generation logic itself, the source files of the generator script are concatenated in a fixed, declared order. Treated as pure UTF-8 bytes without post-hoc re-encoding, their BLAKE3 hash becomes the `GENERATOR_SOURCE_DIGEST`.
*   **Metadata Digests**: Any JSON-based configurations (like `admission_digest` or `numeric_profile_digest`) are canonicalized using RFC 8785-style Canonical JSON (JCS) before hashing.

All computed digests are formatted as lowercase hexadecimal strings with a `blake3:` prefix.

## 2. Generating the Gamma_CMCA Artifact Boundary

Once `mfw-shacl` proves there are no structural cycles or missing required properties, `mfw-codegen` translates the multi-dimensional semantic relationships into purely static, cache-aligned C-ABI structures (`#[repr(C, align(64))]`). 

The pipeline outputs three strictly bound artifact files, seamlessly sealing the environment into a one-way boundary called the `Gamma_CMCA` Contract:

1.  **`cmca_generated.rs`**: The generated Rust Intermediate Representation (IR). Instead of just emitting data tables, `mfw-codegen` structurally embeds the exact cryptographic hashes directly into the static output as `pub const` string slices:
    ```rust
    pub const RDF_INPUT_DIGEST: &str = "blake3:<64_hex_chars>";
    pub const GENERATOR_SOURCE_DIGEST: &str = "blake3:<64_hex_chars>";
    ```
2.  **`cmca_generation_manifest.json`**: Explicitly records the full suite of BLAKE3 digests, including a new `generated_payload_digest`, which is the byte-for-byte BLAKE3 hash of the generated `cmca_generated.rs` file itself. It also locks in the fixed dimension bounds (`N`, `F`, `K`, `Q`).
3.  **`cmca_generation_receipt.json`**: A hash-chained event record connecting the validation events to the generation run.

## 3. Build-Time Verification and Sealing

By embedding `RDF_INPUT_DIGEST` and `GENERATOR_SOURCE_DIGEST` directly into the code alongside the unrolled topological data tables, the generated Rust file acts as a cryptographically sealed, self-contained unit. 

To prove that the Hot Path code remains uncorrupted and mathematically bound to the original ontology, a strict offline verification step (`VerifyGeneratedProfile`) runs during the `bcinr` build or test phase:

*   **Isolated Recalculation**: It reads the `cmca_generated.rs` and the manifest JSON independently—without invoking Python, network requests, or RDF tooling.
*   **Hash Assertion**: It freshly recalculates the BLAKE3 digest of the `.rs` file and asserts that it perfectly matches the `generated_payload_digest` in the manifest. 
*   **Cheat Scanner**: The emitted source is also passed through the `bcinr-cheat-scanner` to ensure no hidden branches, magic constants, or scanner evasion tactics exist in the generated macros.

If there is even one bit of divergence between the generated constants and the cryptographic manifest, the verification matrix immediately issues a typed structural refusal (e.g., `GeneratedProfileRefusal::PayloadDigestMismatch`). 

Through this BLAKE3-backed `Gamma_CMCA` contract, the `bcinr` runtime is physically guaranteed to execute zero-allocation, branchless mathematics perfectly bound to the original semantic graph.
