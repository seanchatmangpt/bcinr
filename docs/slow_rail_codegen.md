# Slow Rail Orchestration of Authoritative Rust Code Generation

In the `bcinr` project, the **Slow Rail** is an asynchronous, off-path environment that safely handles tasks inherently violating the deterministic, allocation-free, and branchless constraints of the **Hot Path** (Authoritative Runtime). A primary responsibility of the Slow Rail is parsing complex semantic schemas (RDF/ontologies) and transforming them into rigid, branchless, static Rust intermediate representation (IR).

Here is how the Slow Rail orchestrates this pipeline, ensuring reproducibility, binding to source graphs, and passing the cheat scanner:

## 1. Highly Constrained Semantic Parsing and Validation
The pipeline begins by ingesting ontologies (e.g., `cmca-rdf.ttl`) using a custom generator script (e.g., `crates/bcinr-cmca/generator.py`). 
- **Bounded Parsing:** The parser explicitly rejects constructs that could lead to unbounded complexity or non-deterministic mapping, such as blank nodes, multiline literals, or language tags.
- **SHACL Shape Validation:** It mathematically guarantees structural compliance before generation. For instance, `cmca:Lens` must possess a numeric `cmca:lensExponent`. If validation fails, the Slow Rail issues a structured typed refusal rather than guessing or defaulting values.

## 2. Deterministic Flattening and Resolution
Because the Hot Path requires constant-time execution without branching or traversing variable-size graphs, the Slow Rail performs all dynamic graph traversals **Ahead-of-Time (AOT)**.
- **Index-Sorting:** Ontological entities are deterministically mapped to array offsets based on explicitly defined properties (e.g., `cmca:measureIndex`). This establishes rigid sequence bounds like `K` (Measure Heads) and `Q` (Lenses).
- **Consequence Resolution:** Dependency chains (`cmca:dependsOn`) are recursively explored and flattened, pre-calculating and summing properties like downstream consequence mass into fixed tables.

## 3. Artifact Generation and Source Graph Binding (Gamma_CMCA)
The validated semantic state is translated into pre-computed, fixed-width Rust code (e.g., `src/generated/case_studies.rs`), communicating across a one-way deterministic artifact boundary known as **`Gamma_CMCA`**.
- **Static IR Compilation:** Multi-dimensional relationships are serialized into pure static constants and fixed-point arrays (e.g., the `LAMBDA` Matrix and `OBJECT_REGISTRY` representing `PackedSemanticState`). It also creates macros (`unroll_n_static!`) for zero-overhead loop unrolling.
- **Source Graph Binding:** To ensure absolute traceability, the generated Rust file embeds strict **content-identity digests**. This includes digests covering the admitted RDF input graph (`RDF_INPUT_DIGEST`), the semantic validation pass, the fixed-point numeric profiles, and the generator source script itself (`GENERATOR_SOURCE_DIGEST`). The Hot Path verifies these digests at build or test time.

## 4. Cheat Scanner and Auditing Gates
According to the `bcinr` constitution (Rule 21), generated code is not exempt from the core architectural laws.
- **Reproducibility Checks:** The generation process itself is verified via a strict `clean generation → digest output → regenerate → verify byte-identical output` workflow. Any unexplained drift invalidates the repository's standing.
- **Cheat Scanner (`bcinr-cheat-scanner`):** The generated code must pass rigorous syntax tree (AST) and text analysis. The scanner explicitly parses generated Rust output to enforce rules `CHEAT-001` through `CHEAT-031`, ensuring no hidden branching logic, scanner evasion, or magic constants exist in the expanded structures.
- **Object-Code Audits:** The generated module must still achieve `CC=1` (Cyclomatic Complexity of 1) at the source level, and its resulting assembly must pass physical release object-code inspection for the absolute absence of conditional jumps, loop backedges, and allocator calls.
