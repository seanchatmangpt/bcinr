# CMCA Slow Rail Generation Pipeline

In the `bcinr` architecture, the **Slow Rail** is responsible for all allocation-heavy, branching, and string-parsing logic associated with reading ontological rules. Its job is to deterministically flatten this complex semantic graph into statically bounded, fixed-width structures that the **Hot Path** can consume without branching or allocating memory.

## 1. How the Slow Rail Generates Artifacts

The core of the generation pipeline is a Python script (e.g., `tools/cmca-generator/generator.py`), operating on RDF files (e.g., `cmca-rdf.ttl`, `generalization.ttl`). 

- **Highly Constrained Parsing**: The script reads the Turtle (`.ttl`) files and strictly validates shapes, enforcing limits and explicitly rejecting non-deterministic or unbounded features like blank nodes, multiline literals, and recursive relative IRIs.
- **Ahead-of-Time (AOT) Graph Resolution**: Any dynamic logic that would require runtime traversal (such as calculating transitive `cmca:downstreamConsequence` weights based on `cmca:dependsOn`) is calculated and flattened ahead of time.
- **Fixed-Point Conversion**: Floating-point values defined in the RDF (like `cmca:eta` or factor weights) are statically converted into `Q16.16` fixed-point representations by the generator script (multiplying by 65536 and rounding).
- **Deterministic Emission**: The script guarantees byte-for-byte reproducibility across runs. It deterministically sorts and maps ontology terms to static zero-based index integers to ensure that array definitions are strictly ordered. It emits three core artifacts: `cmca_generated.rs`, `cmca_generation_manifest.json`, and `cmca_generation_receipt.json`.

## 2. What the Generated Artifacts Contain

The primary artifact `cmca_generated.rs` acts as a static Intermediate Representation (IR). It contains:

- **Mathematical Constants & Bounds**: Absolute capacity limits derived from the graph, such as `N` (Object Count), `F` (Factor Count), `K` (Measure Heads), and `Q` (Lenses).
- **Digests & Provenance Data**: Constant bindings like `RDF_INPUT_DIGEST` and `GENERATOR_SOURCE_DIGEST` to ensure that the code generation was derived from an exact, verified input state.
- **Static Registries**: 
  - `OBJECT_REGISTRY` mapping each semantic entity into a `PackedSemanticState` struct of flat factor arrays.
  - `LAMBDA` containing a dense coefficient matrix (`K x Q`) flattened for branchless ingestion.
  - Constant `NonNegativeFixed` bindings replacing runtime string identifiers with integer indices (e.g., `MEASURE_CACHE = 0`).

## 3. Bridging to the Authoritative Hot Path

The emitted Rust file forms a one-way deterministic boundary known as **`Gamma_CMCA`**.

- **Branchless Evaluation**: Because the Slow Rail pre-computed dependencies and scaled numerics, the Hot Path never traverses a graph or checks validity bounds. It simply imports `cmca_generated.rs` and processes its fixed-size arrays (`OBJECT_REGISTRY`) using loop unrolling and bitwise masks (`CC=1` enforcement).
- **Zero-Allocation Memory Profile**: The hot path receives a guaranteed flat memory layout bounded by `[PackedSemanticState; N]`, allowing calculations to execute securely on the stack.
- **Cryptographic Enforceability**: The Hot Path explicitly checks the bundled digests from the generated code, confirming the semantic integrity of the AOT pipeline.
- **Audited Compliance**: Despite being generated code, the output is not exempt from core laws. It is verified by the `bcinr-cheat-scanner` to ensure no branches are accidentally introduced in macro expansions, and it undergoes rigorous disassembly inspection for loop backedges and conditional jumps.
