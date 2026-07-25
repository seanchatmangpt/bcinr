# Rule 21 (Generated Code Law): Structural Binding Mechanism

Under Rule 21 of the BCINR Deterministic Substrate Constitution, generated authoritative code is required to structurally "bind to source graph and certificate digests." This mandate acts as a rigid, cryptographic contract between the generator (producer) and the runtime (consumer) to ensure that downstream mechanics are tied to upstream semantics with zero variance.

## Artifact Structure

The structural binding mechanism is implemented via a **three-file artifact structure** that is produced and consumed as a unit:

1. **`cmca_generated.rs`**: The generated Rust source code containing constants and tables.
2. **`cmca_generation_manifest.json`**: A tool-readable manifest containing provenance fields and a strict set of BLAKE3 hashes.
3. **`cmca_generation_receipt.json`**: A BLAKE3-chained receipt of the generation pipeline.

## Cryptographic Binding (The Six Canonical Digests)

The cryptographic binding is established within the manifest (`cmca_generation_manifest.json`) using exactly six canonical, host-independent BLAKE3 digests:

1. **`rdf_digest`**: Hash of the admitted RDF source graph (serialized as canonical N-Quads).
2. **`admission_digest`**: Hash of the canonical JSON encoding of the SHACL/ShEx validation result (certificate digest).
3. **`generator_digest`**: Hash of the generator's own source code bytes.
4. **`numeric_profile_digest`**: Hash of the numerical configuration (precision, rounding mode, bounds).
5. **`formula_registry_digest`**: Hash of all formula/floor identity definitions in force.
6. **`generated_payload_digest`**: Hash of the exact emitted bytes of `cmca_generated.rs`.

These identity digests must be completely host-independent and deterministic.

## Verification and Enforcement

To prevent manual tampering, non-deterministic inputs, and "unexplained drift," the binding is enforced through the following mechanisms:

1. **`VerifyGeneratedProfile` Check**: At build or test time (e.g., via the `verify-generated` Makefile task), `bcinr` executes an independent verification step without needing network access or the original Python/RDF tooling. It directly parses the manifest and recomputes the BLAKE3 hash of the local `cmca_generated.rs`.
2. **Immediate Failure on Digest Mismatch**: If the recomputed hash does not perfectly match the `generated_payload_digest` stored in the manifest, the cryptographic chain is broken. This results in an immediate, hard typed build/test failure (e.g., a `compile_error!`). Silent continuation or partial acceptance is forbidden.
3. **Prohibition of Hand-Editing**: Hand-editing generated output is strictly prohibited. Manual alteration changes the hash, fails the local verification check, bypasses the Hoare oracle and cheat scanners, and represents an "immediate purge condition" that quarantines the implementation.

By enforcing a byte-identical loop (`Clean generation → Digest output → Regenerate → Verify byte-identical output`), logical changes can only be made in the upstream generator or semantic models, never downstream in the unrolled output.
