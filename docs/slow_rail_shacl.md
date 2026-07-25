# The Dichotomy Between the Slow Rail and the Hot Path in BCINR

The `bcinr` (BranchlessCInRust) project enforces a strict, civilizational-scale architectural dichotomy between the **Slow Rail** and the **Hot Path** (Authoritative Runtime). This separation ensures that the core computational substrate remains a deterministic, bounded, allocation-free execution environment, while still allowing for complex semantic configurations and RDF/ontology-driven behaviors.

## The Hot Path (Authoritative Runtime)

The Hot Path is the authoritative execution environment governed by absolute runtime laws. Its primary directive is the **Radon Law ($CC=1$)**: no public primitive shall contain a single conditional branch (`if`, `match`, or data-dependent `loop`). Logic is executed purely as bitwise polynomials.

Key characteristics of the Hot Path include:
- **Zero Allocation:** Execution is `#![no_std]` with zero heap allocations. Memory is strictly managed via structures like `BumpArena` and `LockFreeSlab`.
- **Branchless Execution:** All semantic decisions are transformed into masks, arithmetic selection (e.g., `const_select_u32`), and fixed-width state transitions.
- **Fixed-Point Arithmetic:** Floating-point operations are strictly prohibited. The runtime relies entirely on branchless, saturating Q16.16 fixed-point math (`fixed.rs`).
- **No Runtime Theorem Discovery:** The Hot Path never parses text, traverses variable graphs, or performs dynamic discovery at runtime. It operates exclusively on fixed-size, pre-calculated matrices and arrays.

## The Slow Rail

The Slow Rail is the asynchronous, off-path environment where complex, branching, and allocating logic is safely isolated. It is responsible for tasks that inherently violate the Radon Law, such as symbolic mathematics, test oracle generation, artifact serialization, and complex data validation. 

Crucially, the Slow Rail is entirely responsible for handling all **RDF parsing** and **SHACL validation** before the hot path ever interacts with the data.

### RDF Parsing and SHACL Validation

In the Cross-Measure Cognitive Allocation (CMCA) specialization (`CMCA-RDF`), the system relies on rich semantic schemas (ontologies) written in Turtle/RDF format. Since parsing text and dynamically validating graph structures fundamentally require branching and memory allocation, they are strictly relegated to the Slow Rail (often housed in a separate repository like `mfw`).

1. **RDF Parsing:** The Slow Rail ingests the ontology definitions (e.g., `cmca-rdf.ttl`). This involves mapping classes (`cmca:SemanticObject`, `cmca:MeasureHead`, `cmca:Lens`) and their properties into an internal structured representation.
2. **SHACL Validation & Admission:** Before any state is permitted forward, the Slow Rail executes a rigorous semantic admission process. It validates the RDF graph against SHACL/ShEx shapes to ensure required property presence, index injectivity, type conformance, and capacity bounds. 
3. **Structured Refusals:** If validation fails (e.g., due to missing properties or dependency cycles), the Slow Rail issues a structured, typed refusal. It never defaults to zero, an empty string, or attempts to guess missing values.
4. **Off-Path Consequence Resolution:** Any dynamic graph traversals—such as recursively propagating downstream consequence mass across dependent objects (`cmca:dependsOn`)—are fully resolved and flattened during this phase.

## The Artifact Boundary (`Gamma_CMCA`)

The two domains never intersect at runtime. The dependency graph of the `bcinr-cmca` runtime crate explicitly forbids the inclusion of any RDF parser, graph store, or SHACL engine. 

Instead, the domains communicate strictly through a one-way, deterministic artifact boundary known as **`Gamma_CMCA`**. 

Once the Slow Rail successfully parses the RDF and validates it via SHACL, a generator (e.g., `generator.py`) translates the validated semantic state into static Rust code (such as `src/generated/case_studies.rs`). This generated artifact contains:
- A designated `schema_version`.
- Pre-computed, flattened, and capacity-bounded state tables (e.g., the 10 fixed factors of a `SemanticObject`).
- Strict content-identity digests covering the admitted RDF graph, the semantic validation pass, the generation tool, and the fixed-point numeric profiles.

The Hot Path subsequently consumes this `Gamma_CMCA` artifact as opaque, compiled constants, verifying its digests at build or test time. This strict boundary guarantees that the authoritative runtime benefits from rich RDF semantics without ever sacrificing its deterministic, branchless, and allocation-free guarantees.
