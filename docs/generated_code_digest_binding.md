# Cryptographic Binding in Generated Authoritative Code (Rule 21)

Under Rule 21 (Generated-code law) of the BCINR Deterministic Substrate Constitution, generated authoritative code must "bind to source graph and certificate digests." This mandate ensures that the downstream fixed-execution mechanics are mathematically and structurally tied to the upstream semantic models (like SHACL or ontologies) with zero-variance.

## What Cryptographic Binding Entails

The binding acts as a rigid, cryptographic contract between the generator (producer) and the `bcinr-cmca` runtime (consumer). This is implemented via a three-file artifact structure, which are produced and consumed as a unit:

1. **`cmca_generated.rs`**: The generated Rust source containing constants and tables.
2. **`cmca_generation_manifest.json`**: A tool-readable manifest containing provenance fields and a strict set of BLAKE3 hashes.
3. **`cmca_generation_receipt.json`**: A BLAKE3-chained receipt of the generation pipeline.

The cryptographic binding itself is established within the manifest using exactly **six canonical BLAKE3 digests**:

* **`rdf_digest`**: Hash of the admitted RDF graph (serialized as canonical N-Quads).
* **`admission_digest`**: Hash of the canonical JSON encoding of the SHACL/ShEx validation result.
* **`generator_digest`**: Hash of the generator's own source code bytes.
* **`numeric_profile_digest`**: Hash of the numerical configuration (precision, rounding mode, bounds).
* **`formula_registry_digest`**: Hash of all formula/floor identity definitions in force.
* **`generated_payload_digest`**: Hash of the exact emitted bytes of `cmca_generated.rs`.

These identity digests must be completely host-independent and deterministic. Running the exact same generator on the exact same RDF input must yield identical digests across different machines or CI environments.

## Preventing Manual Tampering and Invalidating "Unexplained Drift"

This cryptographic linkage is strictly enforced to prevent manual tampering and ensure mathematical provability. 

### 1. The `VerifyGeneratedProfile` Check
At build or test time, `bcinr` executes an independent verification step without needing network access or the original Python/RDF tooling. It directly parses the manifest and recomputes the BLAKE3 hash of the local `cmca_generated.rs`. 

### 2. Immediate Failure on Digest Mismatch
If the recomputed hash does not perfectly match the `generated_payload_digest` stored in the manifest, the cryptographic chain is broken. This represents "unexplained drift"—non-deterministic inputs or manual edits have leaked into the code. 
Under the BCINR constitution, any digest mismatch results in an immediate, hard typed build/test failure (e.g., a `compile_error!`). Silent continuation or partial acceptance is forbidden.

### 3. Prohibition of Hand-Editing
Hand-editing generated output is strictly prohibited and immediately invalidates project standing. If an agent or user alters `cmca_generated.rs` manually:
* The file's hash changes, failing the local `VerifyGeneratedProfile` check.
* The file bypasses the Hoare oracle, cheat scanners, and the verified mathematical contract of the generator.
* It is treated as an "immediate purge condition," quarantining the implementation. 

By enforcing this bite-identical loop (Clean generation → Digest output → Regenerate → Verify byte-identical output), BCINR guarantees that logical changes can only be made in the upstream generator or semantic models, never downstream in the unrolled output.
