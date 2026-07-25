# Slow Rail Formula Registry Digest in BCINR

In BCINR, the **Slow Rail** is responsible for transforming flexible, semantic ontologies (RDF/Turtle) and mathematical evaluation parameters into a rigid, branchless Rust Intermediate Representation (IR). To prevent supply-chain drift and ensure complete determinism across the boundary into the Authoritative Runtime (Hot Path), the system uses strict content-identity digests under the **`Gamma_CMCA`** contract.

One of the most critical of these is the `formula_registry_digest`.

## What is the `formula_registry_digest`?

The `formula_registry_digest` is one of the six authoritative digests defined in the `Gamma_CMCA v1` artifact contract. It is a **BLAKE3 hash (`blake3:<hex>`)** of the canonicalized JSON encoding of the mathematical formula parameters in force. 

Specifically, it pairs every **formula/floor identity name** with its defining expression or table-generation rule (e.g., `UniformLeafFloor`, `UniformLeafFloorQ16Residual`).

## How Mathematical Parameters are Canonicalized 

Before mathematical parameters (dynamic thresholds, weights, control constants) are hashed, they must be formatted into **Canonical JSON (RFC 8785-style / JCS)**. This canonicalization strips away any host-specific or non-deterministic artifacts:

- **Strict Key Ordering**: Object keys are recursively sorted lexicographically (byte-wise on UTF-8) at every nesting level.
- **Array Order Preservation**: Arrays inside the JSON (e.g., registration order, leaf application order) carry semantic meaning and are **never sorted**.
- **No Floating-Point Values**: Floating-point decimals and exponents are completely banned. Any fixed-point quantity (e.g., Q16.16) must be represented as an integer, with its explicit scale factor separately anchored in the `numeric_profile_digest`.
- **Bare Numerics**: Integers are expressed as bare decimal literals with no leading zeros (except `0` itself) and no leading `+`.
- **No Whitespace**: All insignificant whitespace (spaces after colons/commas, indentation, trailing newlines) is stripped.
- **Host Independence**: File paths must be relative and host-independent (no usernames, process IDs, or hostnames).

This ensures that the exact same semantic mathematical rules processed on two different machines will produce a byte-identical `formula_registry_digest`.

## Injection into the Hot Path `Gamma_CMCA` Contract

The `formula_registry_digest` bridges the semantic logic of the Slow Rail and the branchless fixed-point tables of the Hot Path. 

### 1. Generation in the Slow Rail
The Slow Rail tooling (such as `mfw-meaning` and Python generators) processes the semantic graph and emits three files forming the artifact pair:
1. `cmca_generation_manifest.json`: Holds the authoritative digests (`formula_registry_digest`, `rdf_digest`, `generated_payload_digest`) and dimension bounds (`N`, `F`, `K`, `Q`).
2. `cmca_generation_receipt.json`: A hash-chained, timestamped event record of the generation steps.
3. `cmca_generated.rs`: The generated Rust source that implements the formulas via bounded arrays (e.g., `LEAF_FLOOR_BASE` and `LEAF_FLOOR_REMAINDER`).

### 2. Offline Verification (`VerifyGeneratedProfile`)
The Hot Path execution (`bcinr-cmca`) is strictly `#![no_std]`, zero-allocation, and mathematically branchless (Radon Law, `CC=1`). It cannot parse RDF or JSON at runtime. Instead, verification occurs **offline at build or test time** via `crates/bcinr-cmca/src/artifact.rs`.

The verification routine acts as a strict constitutional gate:
- It reads the generated Rust source text (`cmca_generated.rs`) and the manifest JSON.
- It verifies that the `generated_payload_digest` matches a freshly computed BLAKE3 hash of the generated Rust source.
- It proves that the array structures generated for the formulas perfectly respect the dimensions (`N`, `F`, `K`, `Q`) bound by the manifest.
- It mathematically verifies table conservation bounds declared by the formulas (e.g., validating that the Q16.16 residual formulas encoded in `LEAF_FLOOR_BASE` and `LEAF_FLOOR_REMAINDER` sum back exactly to `65536`).

### 3. Typed Refusal
If there is any inconsistency between the `formula_registry_digest` semantics (as defined in the manifest) and the generated artifact (e.g., a missing rule mapping or incorrect array dimension), the build gate issues a **typed refusal** (such as `GeneratedProfileRefusal::FormulaRegistryMismatch` or `TableLengthMismatch`). Silent fallback is prohibited.

Through this discipline, the Hot Path `Gamma_CMCA` contract executes mathematical rules dynamically, branchlessly, and safely, while guaranteeing 100% cryptographic provenance back to the original semantic configuration.
