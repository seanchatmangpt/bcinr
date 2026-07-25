# Slow Rail `numeric_profile_digest` in BCINR

In the `bcinr` architecture, the **Hot Path** (Authoritative Runtime) operates under absolute deterministic constraints (the Radon Law $CC=1$, zero heap allocation). It lacks the capability to parse complex semantic configurations dynamically or process floating-point mathematics. Instead, it relies on static, fixed-width arithmetic (such as Q16.16 fixed-point math).

To bridge this, the **Slow Rail** operates entirely offline and Ahead-of-Time (AOT), resolving all semantic profiles and numeric limits before emitting rigid constants for the Hot Path. The `numeric_profile_digest` is a critical cryptographic seal that guarantees the Hot Path executes mathematics bound exactly to the authorized numeric constraints.

## 1. The `NumericProfile` Schema
During the generation phase, the Slow Rail collects the semantic numeric parameters into a JSON object. This profile captures the exact bounds and behaviors of the fixed-point math to be used. According to the manifest schema, it contains:
- **`precision`**: Decimal precision limits.
- **`q16_16_scale`**: The explicit scale factor for fixed-point quantities (e.g., `65536`).
- **`rounding`**: The rounding mode policy (e.g., `"ROUND_HALF_EVEN"`).
- **Min/Max Boundaries**: Bounds for representable values such as `signed_max`, `signed_min`, `unsigned_max`, and `unsigned_min`.

## 2. Canonicalization (RFC 8785 JCS)
To prevent supply-chain drift and ensure the digest is deterministic across platforms and generation runs, the JSON metadata undergoes strict canonicalization before hashing, following **RFC 8785-style Canonical JSON (JCS)**:
- **Lexicographical Key Sorting**: Object keys are recursively sorted byte-wise using their UTF-8 representation.
- **Whitespace Stripping**: All insignificant whitespace (e.g., indentation, trailing newlines, spaces after colons/commas) is completely removed.
- **Strict Integer Representation**: Floating-point decimals and exponents are absolutely banned. Any fixed-point quantity (such as Q16.16 values) must be represented as a bare decimal integer literal (no leading zeros or plus signs), with its explicit scale factor separately anchored in the profile (e.g., `q16_16_scale`).

## 3. Cryptographic Binding (BLAKE3)
Once the `NumericProfile` JSON is canonicalized into a deterministic byte stream, the Slow Rail computes its **BLAKE3** hash. 
The result is formatted as a lowercase hex string with a prefix (e.g., `blake3:<64_hex_chars>`) and recorded as `numeric_profile_digest` inside the `cmca_generation_manifest.json` artifact, alongside other critical digests like `rdf_digest` and `generated_payload_digest`.

## 4. Hot Path Synchronization and Verification
To ensure absolute synchronization without injecting dynamic logic or runtime overhead into the Hot Path, the `bcinr-cmca` crate performs an isolated, offline check (`VerifyGeneratedProfile` within `artifact.rs`) at build or test time:
1. **Offline Validation**: The verifier reads the generated Rust source (`cmca_generated.rs`) and the `cmca_generation_manifest.json` directly, without invoking any external tooling (no Python, no network, no RDF parsers).
2. **Digest Verification**: It parses the manifest and ensures that the provided digests align with the expected structures and byte outputs, securely sealing the **Gamma_CMCA** contract.
3. **Typed Refusals**: Any missing constants, schema mismatches, or digest divergence immediately results in a typed structured compilation refusal (e.g., `compile_error!`). Silent best-effort acceptance or fallback defaults are strictly prohibited.

This rigorous verification chain guarantees that the constants and tables compiled into the branchless Hot Path accurately map back to the mathematically authorized semantic graph, safely isolating the runtime from rounding-error deviations, manual tampering, or numeric drift.
