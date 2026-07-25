# Slow Rail `cmca_generation_manifest.json` in BCINR

In the `bcinr` architecture, the **Hot Path** is bound by absolute deterministic constraints (the Radon Law, $CC=1$, and zero-allocation) and cannot parse complex Semantic Web data or dynamically traverse graphs. 

The **Slow Rail** bridges this gap by operating entirely offline and Ahead-of-Time (AOT), parsing the ontology and outputting rigid, branchless artifacts that the Hot Path can execute blindly. The `cmca_generation_manifest.json` acts as the definitive metadata record that seals this boundary, establishing the **`Gamma_CMCA` Contract**.

## Role of the Manifest

1. **Metadata and Digest Registry**: The manifest explicitly records the full suite of aggregated BLAKE3 content-identity digests computed during the Slow Rail generation pipeline.
2. **Dimension and Profile Declaration**: It formally declares the rigid dimension bounds (`N`, `F`, `K`, `Q`), numeric profiles, and the fixed generator source sequence used to produce the artifacts.
3. **Supply-Chain Seal**: It prevents supply-chain drift by cryptographically guaranteeing that the branchless Rust constants in the Hot Path (`cmca_generated.rs`) were mathematically derived from the authorized semantic ontology without manual tampering.

## Cryptographic Binding and Build-Time Verification

The manifest cryptographically binds the Slow Rail's semantic dimensions, input RDF digests, and rule registry digests into a single contract using strict canonicalization and **BLAKE3 hashing**:

### 1. The Six Canonical Digests
The manifest records exactly six binding digests (formatted as `blake3:<hex>`):
- **`rdf_digest`**: BLAKE3 hash of the admitted RDF graph, serialized as strict, lexicographically sorted canonical N-Quads.
- **`generator_digest`**: BLAKE3 hash of the generator source files, concatenated in a fixed, declared order.
- **`admission_digest`**: BLAKE3 hash of the canonical JSON encoding of the SHACL/ShEx validation result.
- **`numeric_profile_digest`**: BLAKE3 hash of the canonical JSON encoding of numeric parameters (precision, rounding mode, Q16.16 scale).
- **`formula_registry_digest`**: BLAKE3 hash of the canonical JSON encoding of formula and floor identity names paired with their defining rules.
- **`generated_payload_digest`**: The byte-for-byte BLAKE3 hash of the emitted `cmca_generated.rs` Rust source file.

*Note: Any JSON used for digesting is strictly canonicalized using RFC 8785-style Canonical JSON (JCS).*

### 2. Isolated Verification (`VerifyGeneratedProfile`)
To verify the seal without injecting dynamic logic into the Hot Path, `bcinr-cmca` performs an isolated, offline check at build or test time:
- **Independent Recalculation**: It independently parses the generated Rust source bytes and freshly recalculates its BLAKE3 digest, asserting it perfectly matches the `generated_payload_digest` recorded in the manifest.
- **Dimension Matching**: It asserts that the fixed dimension bounds (`N`, `F`, `K`, `Q`) recorded in the manifest exactly match the `pub const` array declarations emitted in `cmca_generated.rs`.
- **Schema Validation**: It ensures the manifest uses a recognized `schema_version`.

**Failure Discipline**: Any divergence (digest mismatch, bounds mismatch, or unrecognized schema) results in a typed, structured compilation refusal (`compile_error!`), eliminating the risk of unsealed state entering the final binary.
