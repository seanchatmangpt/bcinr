# Rule 21: Generated Code Integrity Architecture

According to `AGENTS.md` and the internal documentation, Rule 21 mandates that generated code bridges high-level semantics to downstream fixed mechanics with absolute determinism. It enforces a strict **4-step Reproducibility Process**:
`Clean generation → Digest output → Regenerate → Verify byte-identical output`

## How Byte-Identical Output is Verified

The verification is structured as a **consumer-side strict check** rather than running the generator on the fly. This is primarily handled by the `verify-generated` task (in `Makefile.toml`) and the cryptographic binding mechanism:

1. **The Three-File Artifact**: Code generation produces an immutable unit comprising:
   - `cmca_generated.rs` (the Rust payload)
   - `cmca_generation_manifest.json` (provenance and hashes)
   - `cmca_generation_receipt.json` (BLAKE3-chained receipt)
2. **Quarantined Generator**: The actual Python/RDF generator script has been deliberately quarantined. The authoritative Rust consumer crate (`bcinr-cmca`) must never invoke the generator itself.
3. **BLAKE3 Hash Recomputation**: During CI via the `verify-generated` task, the pipeline uses `b3sum` to independently recompute the BLAKE3 hash of `cmca_generated.rs`. 
4. **Manifest Comparison**: The script parses `cmca_generation_manifest.json` using Python's standard library (with no network or external tool access) and strictly compares the newly computed hash against the declared `generated_payload_digest`.

## What Happens if Unexplained Drift Occurs?

> [!CAUTION]
> **Hand-editing generated output is strictly prohibited.** Manual alteration represents an immediate purge condition.

If a file experiences unexplained drift or an engineer manually edits the generated `.rs` code:
1. **Immediate Hash Mismatch**: The recomputed BLAKE3 hash diverges from the manifest's `generated_payload_digest`.
2. **Gate Failure**: The `verify-generated` CI task immediately returns a hard failure. Silent continuation or partial acceptance is structurally impossible.
3. **Invalidation of Standing**: `AGENTS.md` strictly dictates: *"Generated files with unexplained drift invalidate standing."* Any absolute failure forces the Substrate Integrity Score (SIS) to `0`.
4. **MaturityScrutiny Protocol**: This triggers an immediate feature freeze, quarantines the affected code, and mandates a complete root-cause repair, full regeneration of dependent artifacts, and reissuance of a standing receipt. 

## Generated Code Integrity Architecture

Generated code is not exempt from the core substrate strictures; it is treated with the same hostility as hand-written code. The architecture enforces this through two main pillars:

### 1. Cryptographic Binding (The Six Canonical Digests)
To satisfy the rule's requirement to *"bind to source graph and certificate digests"*, the manifest cryptographically binds the downstream output back to its upstream semantic inputs using 6 host-independent BLAKE3 digests:
- `rdf_digest`: Hash of the admitted RDF source graph (N-Quads).
- `admission_digest`: Hash of the canonical SHACL/ShEx validation certificate.
- `generator_digest`: Hash of the generator's source code.
- `numeric_profile_digest`: Hash of numerical configurations (bounds, precision).
- `formula_registry_digest`: Hash of all formula identity definitions.
- `generated_payload_digest`: Hash of the exact emitted bytes.

### 2. Continuous Integration Gates
Generated artifacts are physically wired into the substrate's verification loop to ensure they adhere to all computational laws:
- **`verify-generated`**: Enforces hash equality and structural reproducibility (as detailed above).
- **`scan-cheats`**: Inspects generated output for prohibited algorithmic cheats, macro indirection, and magic constants.
- **`contract-gate`**: Parses the generated Abstract Syntax Tree to ensure `CC=1` (Cyclomatic Complexity of 1, enforcing the Radon Law with zero conditional branches).
- **`audit-object-code`**: Assembles a raw `otool`/`objdump` disassembly of the generated output compiled in release mode. This ensures the compiler didn't inject hidden jumps, loop backedges, or allocator calls.
