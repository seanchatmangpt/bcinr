# The Gamma_CMCA Contract: The Unbreachable Boundary in BCINR

In the `bcinr` architecture, the **Hot Path** is constrained by absolute deterministic laws (the Radon Law, $CC=1$, zero-allocation, branchless mechanics). It cannot parse semantic web data, dynamically traverse graphs, or evaluate unbounded inputs. Conversely, the **Slow Rail** (`mfw`) operates ahead-of-time (AOT), parsing complex RDF graphs and evaluating semantics.

The **`Gamma_CMCA` Contract** establishes the deterministic, digest-bound artifact boundary that physically and topologically separates these two worlds. It ensures that the deterministic Hot Path (`bcinr-cmca`) never links against, imports, or invokes the unbounded tooling of the Slow Rail (e.g., `oxigraph`, SHACL/ShEx, Python).

## The Three-File Artifact Suite

The contract is materialized as exactly three files, produced by the Slow Rail and consumed by the Hot Path as an atomic unit:

1. **`cmca_generated.rs` (The Payload)**
   A pure-Rust source file containing exclusively branchless, fixed-size constants, mathematical bounds, and flat lookup tables (e.g., base-`q` and residual-`r` bounded object registries). It replaces runtime semantic evaluation with static $O(1)$ memory access.
2. **`cmca_generation_manifest.json` (The Identity Record)**
   A metadata registry that strictly defines the shape bounds (`N`, `F`, `K`, `Q`) and cryptographically binds the semantic provenance into six canonical BLAKE3 digests.
3. **`cmca_generation_receipt.json` (The Event Record)**
   A hash-chained run receipt proving the chronological execution of the Slow Rail pipeline (`admit_graph` -> `validate` -> `generate` -> `emit_artifact`).

## Cryptographic Binding (The Six Digests)

To prevent supply-chain drift, the manifest seals the exact state of the Slow Rail at generation time using six mandatory `blake3:<hex>` digests. Any JSON or text digested is first aggressively normalized (e.g., Canonical JSON / JCS, lexicographically sorted N-Quads, LF endings). 

1. **`rdf_digest`**: The admitted, canonicalized N-Quads of the RDF graph.
2. **`admission_digest`**: The validation pass result (SHACL/ShEx).
3. **`generator_digest`**: The source code of the generator script itself.
4. **`numeric_profile_digest`**: Fixed parameters like precision, ranges, and Q16.16 scales.
5. **`formula_registry_digest`**: Formula and floor identity names mapped to their defining rules.
6. **`generated_payload_digest`**: The byte-for-byte hash of the emitted `cmca_generated.rs`.

**The Gap Rule:** If any semantic input influences generation but is not transitively bound by these six digests, the contract dictates that a seventh digest must be added (accompanied by a schema version bump). No implicit bounds are allowed.

## Consumer Obligation: The `VerifyGeneratedProfile`

The Hot Path enforces the boundary via a rigorous, offline build/test time check. Without invoking any RDF/generation tools, the `bcinr-cmca` crate performs the following:

- **Schema Check**: Validates the `schema_version` of the manifest.
- **Dimension Check**: Asserts that the bounds (`N`, `F`, `K`, `Q`) in the manifest perfectly match the lengths of the arrays emitted in `cmca_generated.rs`.
- **Payload Recalculation**: Freshly hashes the bytes of `cmca_generated.rs` and asserts it matches the `generated_payload_digest` in the manifest.

**Strict Refusal:** If any of these checks fail (or a schema is unrecognized), it triggers a typed compilation/test failure (e.g., `compile_error!`). Silent fallbacks or best-effort acceptances are constitutionally prohibited.

## Identity vs. Event Separation

The contract fundamentally separates mathematical identity from generation events:
- **Identity (Manifest & Payload):** Given the same admitted graph and generator, these files must be byte-for-byte identical across runs, machines, or time. They are devoid of timestamps, hostnames, or relative paths.
- **Event (Receipt):** The receipt tracks run-specific timestamps and local hash chains. It is expected to vary between runs. The consumer evaluates the receipt's structural validity, but its variation is never treated as a determinism defect. 

Through this contract, `bcinr` achieves absolute mathematical certainty: the Slow Rail processes all semantic complexity, while the Hot Path blindly and deterministically executes the sealed physical consequences.
