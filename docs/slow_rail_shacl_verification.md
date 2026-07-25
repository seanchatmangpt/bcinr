# Slow Rail `mfw-shacl` Constraint Verification

In the `bcinr` architecture, the semantic parsing and validation of RDF/Turtle (`.ttl`) ontologies are strictly quarantined to the **Slow Rail**. This ensures that the dynamic, allocation-heavy nature of Semantic Web data does not violate the constitutional laws of the Authoritative Hot Path (such as the Radon Law of $CC=1$, `#![no_std]`, and zero heap allocations).

The `mfw-shacl` tooling acts as an Ahead-of-Time (AOT) gatekeeper in this offline pipeline. It rigorously validates the canonicalized `.ttl` data against authoritative SHACL shape graphs, ensuring structurally invalid ontology never reaches the `cmca_generated.rs` artifact.

The constraint verification process incorporates the following critical enforcement mechanisms:

## 1. Mathematical Normalization & Cryptographic Binding
Before SHACL validation occurs, the dynamic `.ttl` files are mathematically normalized into an absolute, byte-stable representation known as Canonical N-Quads. 
- `mfw-shacl` hashes this canonicalized graph to generate an `rdf_digest`.
- Once SHACL structural validation is successful, an `admission_digest` is produced.
These cryptographic digests bind the validated graph, preventing tampering and sealing the environment before handoff to the hot path generator (`mfw-codegen`).

## 2. Property Conformance ("Missing is not zero")
`mfw-shacl` rigorously checks that required properties and types conform exactly to the authoritative SHACL shape graphs.
- **No Silent Defaults:** If a required property is missing from the ontology, the tooling issues a typed, structured refusal. It is constitutionally forbidden from silently substituting default values like `0`, empty strings, or empty collections to bypass validation.
- **Typed Refusals:** Admission checks must produce bounded, typed refusals (e.g., `ContractViolation`, `ControlStateUnadmitted`) upon failure, rather than relying on language-level assertions or panics that might be stripped by compiler optimization flags.

## 3. Strict Cycle Refusals and Topological Flattening
Ontology dependencies are analyzed offline. `mfw-shacl` attempts topological traversals (such as Kahn's Topological Sort) to flatten the dependencies.
- **Zero Tolerance for Cycles:** Any cycle within the consequence or derivation graph of ontology objects immediately triggers a typed refusal. 
- **No Partial Evaluation:** The pipeline will never emit partial traversal results or utilize arbitrary default values to resolve cyclic dependencies. The graph must be perfectly acyclic to proceed.

## 4. Semantic Index & Arithmetic Invariants
During the pipeline evaluation, `mfw-shacl` enforces absolute structural integrity for the generation phase:
- **Index Invariants:** Semantic indices assigned to ontology objects must be **Injective** (unique), **Capacity-bounded** (strictly within target array limits), and **Contiguous** (no gaps for dense layouts).
- **Exact Decimal Arithmetic:** When validating bounds on numeric ontology literals, exact decimal arithmetic must be used. Binary floating-point logic is explicitly banned to prevent representation errors and precision loss.

By enforcing these constraints out-of-band on the Slow Rail, `mfw-shacl` guarantees that only fully flattened, structurally sound, cycle-free, and cryptographically verified data passes over the `Gamma_CMCA` boundary to `mfw-codegen`, which then emits the final zero-allocation `cmca_generated.rs` artifact.
