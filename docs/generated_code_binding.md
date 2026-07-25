# Rule 21: Generated Code Cryptographic Binding Mechanism

Under Rule 21 (Generated-code law) of the BCINR Deterministic Substrate Constitution, generated authoritative code must structurally **"bind to source graph and certificate digests."** This mandate ensures that downstream fixed-execution mechanics are mathematically tied to upstream semantic models (e.g., RDF graphs or SHACL certificates) with zero variance, eliminating the possibility of "unexplained drift."

## The Artifact Structure

The cryptographic binding acts as a rigid contract between the generator (producer) and the `bcinr-cmca` runtime (consumer). This is implemented via a **three-file artifact unit**:

1. **`cmca_generated.rs`**: The generated Rust source containing branchless constants and tables.
2. **`cmca_generation_manifest.json`**: A tool-readable manifest containing provenance fields and a strict set of BLAKE3 hashes.
3. **`cmca_generation_receipt.json`**: A BLAKE3-chained receipt of the generation pipeline.

## The Six Canonical BLAKE3 Digests

The cryptographic binding itself is established within the `cmca_generation_manifest.json` using exactly six host-independent, deterministic BLAKE3 digests:

1. **`rdf_digest`**: Hash of the admitted RDF source graph (serialized as canonical N-Quads).
2. **`admission_digest`**: Hash of the canonical JSON encoding of the SHACL/ShEx validation result (the certificate digest).
3. **`generator_digest`**: Hash of the generator's own source code bytes.
4. **`numeric_profile_digest`**: Hash of the numerical configuration (precision, rounding mode, bounds).
5. **`formula_registry_digest`**: Hash of all formula/floor identity definitions in force.
6. **`generated_payload_digest`**: Hash of the exact emitted bytes of `cmca_generated.rs`.

Running the exact same generator on the same RDF input must yield identical digests across all machines or CI environments.

## Preventing Drift and Manual Tampering

This cryptographic linkage is strictly enforced in the CI/CD and build pipeline to prevent manual tampering:

### 1. `VerifyGeneratedProfile` / `verify-generated` Task
At build or test time, the `bcinr` substrate executes an independent verification step (without network access or Python tooling). It parses the manifest and recomputes the BLAKE3 hash of the local `cmca_generated.rs`. 

### 2. Immediate Failure on Digest Mismatch
If the recomputed hash does not perfectly match the `generated_payload_digest` stored in the manifest, the cryptographic chain is broken. This represents **"unexplained drift"** and triggers an immediate, hard typed build/test failure (e.g., `compile_error!`). Silent continuation or partial acceptance is strictly forbidden.

### 3. Prohibition of Hand-Editing
Hand-editing generated output is explicitly prohibited by Rule 21. If an agent or engineer modifies `cmca_generated.rs` manually:
- The file's hash changes, failing the local `VerifyGeneratedProfile` check.
- It bypasses the Hoare oracle, cheat scanners, and the verified mathematical contract of the generator.
- It acts as an immediate purge condition, quarantining the implementation.

By enforcing a strict byte-identical loop (*Clean generation → Digest output → Regenerate → Verify byte-identical output*), logical changes can only occur in the upstream generator or semantic models, never downstream in the unrolled Rust output.
