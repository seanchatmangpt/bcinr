# Rule 21: Reproducible Generation Pipeline in BCINR

Based on the documentation and CI configurations found in the `bcinr` codebase, Rule 21 strictly enforces that generated authoritative code is mathematically and structurally deterministic. It acts as an unbreakable bridge between high-level upstream semantic models (like RDF/SHACL ontologies) and downstream fixed-execution mechanics.

Here is how the reproducible generation pipeline ensures that the output is byte-identical, digest-bound, and completely free of fixture-specific identifiers before it reaches the `CC=1` audit.

## 1. The 4-Step Reproducibility Loop

The absolute law for code generation follows this exact process:
`clean generation → digest output → regenerate → verify byte-identical output`

To prevent "unexplained drift," generating code twice on a clean state must produce exact bit-for-bit replicas across different machines, CI runs, and developer environments.

## 2. Freedom from Fixture-Specific Identifiers

To fulfill the byte-identical requirement, the generators are legally prohibited from embedding any environment-dependent variables. The code must **not** contain:
- Timestamps
- Random seeds
- Temporary file paths
- Any identifiers specific to local test fixtures

If any of these leak into the output, the `verify byte-identical output` phase will fail, indicating unexplained drift, which forces the project's Substrate Integrity Score (SIS) to 0 and immediately quarantines the implementation.

## 3. Cryptographic Binding (Digest-Bound Artifacts)

Generated Rust code is never treated as a standalone source file. Instead, it is produced as a three-file artifact unit that binds the final mechanical instructions to the upstream source graph using BLAKE3 hashing.

1. **`cmca_generated.rs`**: The actual fixed-width unrolled source code.
2. **`cmca_generation_manifest.json`**: A manifest holding exactly six canonical BLAKE3 digests:
   - `rdf_digest`: Hash of the canonical N-Quads serialized RDF graph.
   - `admission_digest`: Hash of the SHACL validation result.
   - `generator_digest`: Hash of the generator script's own source code.
   - `numeric_profile_digest`: Hash of numerical configurations (precision, bounds).
   - `formula_registry_digest`: Hash of floor identity/formula definitions.
   - `generated_payload_digest`: Hash of the exact bytes of `cmca_generated.rs`.
3. **`cmca_generation_receipt.json`**: A chained receipt of the entire generation pipeline.

At build/test time (via `Makefile.toml` tasks like `verify-generated`), the system recomputes the BLAKE3 hash of `cmca_generated.rs` locally. If it doesn't perfectly match the `generated_payload_digest` in the manifest, the pipeline halts with a hard compile/build failure. This mathematically guarantees that hand-editing is impossible. 

## 4. Substrate Verification (`CC=1` and Beyond)

Once byte-reproducibility and cryptographic linkage are verified, the generated code acts like any other authoritative implementation in `bcinr` and passes through the remaining constitutional gates:

- **`scan-cheats`**: `bcinr-cheat-scanner` inspects the generated output to ensure prohibited operations are not hidden via macro expansion or arbitrary magic constants.
- **`contract-gate` (CC=1)**: Enforces the Radon Law by scanning the Abstract Syntax Tree (AST) to verify cyclomatic complexity is strictly 1 (no branches, no data-dependent loops).
- **`audit-object-code`**: Finally, the compiled object code is subjected to an exact, production-profile disassembly audit (using tools like `otool` or `objdump`). This proves that the generated code translates directly to a branchless machine-level instruction shape, free from optimization-induced jumps, panic bounds checks, or allocator calls.
