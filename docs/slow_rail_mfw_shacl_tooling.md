# Slow Rail `mfw-shacl` Tooling in BCINR

In the `bcinr` architecture, semantic parsing and validation of RDF/Turtle ontologies are isolated to the **Slow Rail** to adhere to strict Hot Path constraints (such as the Radon Law of $CC=1$ and zero heap allocations). The `mfw` tooling (specifically `mfw-shacl` and `mfw-meaning`) serves as the Ahead-of-Time (AOT) orchestrator and gatekeeper.

Before `cmca_generated.rs` (a fixed-width, zero-allocation artifact) is emitted for the runtime, `mfw-shacl` applies rigorous offline constraint enforcement against the semantic ontology. The core constraints include:

## 1. Structured Refusals for Semantic Admission
All admission checks (e.g., property presence, type conformance) must produce typed, structured refusals upon failure. These checks cannot be implemented as language assertions that might be disabled by compiler optimization flags.

## 2. Property Conformance ("Missing is not zero")
If a required property is missing from the ontology, `mfw-shacl` issues a typed refusal. It is constitutionally forbidden from silently substituting default values like `0`, empty strings, or empty collections to bypass validation.

## 3. Strict Cycle Refusals
Any cycle within the consequence or derivation graph of ontology objects immediately triggers a typed refusal. The generator will never emit partial traversal results or use arbitrary default values to resolve cyclic dependencies.

## 4. Semantic Index Invariants
The tooling rigorously validates the semantic indices assigned to ontology objects. It checks three distinct conditions:
- **Injective**: Every index must be unique to a single semantic object.
- **Capacity-bounded**: Indices must fall strictly within the declared capacity of the target array/table.
- **Contiguous**: Where dense layouts are required, the assigned indices must not contain any gaps.
A failure in any of these conditions produces a specifically typed refusal.

## 5. Exact Decimal Arithmetic
To ensure absolute mathematical determinism, conversion of numeric literals into fixed-point numeric representations must be performed via exact decimal arithmetic. The use of binary floating-point logic is explicitly prohibited during this process to prevent representation errors and precision loss.

## 6. Mathematical Normalization (Canonical N-Quads)
The tooling converts dynamic `.ttl` files into Canonical N-Quads, enforcing an absolute, byte-stable representation (e.g., lexicographical sorting, strict UTF-8 with UNIX line endings, and no insignificant whitespace). This enforces cryptographic stability and allows for exact hashing (`rdf_digest` and `admission_digest`).

By enforcing these constraints entirely out-of-band, the `mfw-shacl` tooling ensures that only flattened, structurally sound, and cryptographically verified data passes over the boundary into `bcinr`'s deterministic hardware layer.
