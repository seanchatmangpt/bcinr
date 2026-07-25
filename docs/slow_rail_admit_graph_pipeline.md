# Slow Rail `admit_graph` Pipeline in BCINR

In the `bcinr` architecture, the Authoritative Runtime (Hot Path) operates under absolute deterministic laws (the Radon Law $CC=1$, `#![no_std]`, zero heap allocation). Processing dynamic Semantic Web data (RDF/Turtle) is fundamentally incompatible with these laws as it requires variable-length strings, dynamic pointer chasing, and heap allocations. To bridge this gap, all semantic evaluation is quarantined to the asynchronous, offline **Slow Rail**.

The first, foundational phase of this Slow Rail pipeline is the **`admit_graph`** process. This phase is responsible for ingesting, mathematically normalizing, and cryptographically binding the unbounded semantic ontology before it is flattened and passed to the topological generator.

## 1. Ingestion of the Semantic Ontology

The pipeline begins with the raw semantic data, typically authored as RDF/Turtle (`.ttl`) files (e.g., `cmca-rdf.ttl`). The Slow Rail tooling (often guided by `mfw` or `mfw-meaning`) parses these complex, unbounded graph structures. 

Because the resulting structures will eventually dictate the memory-aligned logic of the Hot Path, the ontology is rigorously analyzed to guarantee bounded capacity constraints and to ensure all indices remain injective and contiguous.

## 2. Mathematical Normalization (Canonical N-Quads)

The semantic data must be transformed into an absolute, byte-stable representation so that its cryptographic identity cannot be altered by insignificant formatting differences. During the `admit_graph` phase, the RDF graph is serialized into **Canonical N-Quads**. 

This canonicalization strictly dictates:
- **Lexicographical Sorting:** All quads are sorted lexicographically by their full quad string (subject, predicate, object, graph) in text form, rather than term-by-term.
- **Encoding & Line Endings:** The payload is strictly UTF-8 encoded with Unix (`LF` / `\n`) line endings. Any `\r\n` carriage returns must be normalized prior to processing.
- **Whitespace Rules:** Insignificant whitespace is prohibited. There must be no trailing whitespace on any line and no trailing blank line at the end of the file.

## 3. Cryptographic Binding (`rdf_digest`)

Once normalized, the canonical N-Quads are hashed using the **BLAKE3** algorithm. This yields the `rdf_digest` (formatted as a lowercase hex string prefixed with `blake3:`).

This cryptographic seal ensures that the identical semantic input will produce the identical digest across any machine, time, or environment. The `rdf_digest` forms the foundational root of the Slow Rail’s hash-chained `cmca_generation_receipt.json`, effectively locking the exact semantic state in place. 

## 4. Semantic Validation (SHACL Pre-Checks)

Concurrent with or immediately succeeding ingestion (`validate` step), the normalized graph is subjected to structural enforcement via **SHACL (Shapes Constraint Language) / ShEx**:
- **Property Conformance:** Ensures all required properties are strictly present and type-compliant (e.g., forbidding missing values defaulting to `0` or `""`).
- **Exact Decimal Arithmetic:** Enforces exact fixed-point conversion of numeric literals according to a designated rounding-mode profile, prohibiting binary floating-point representations.
- **Cycle Detection:** Identifies any dependency cycles, which immediately trigger typed structured refusals instead of silent deadlocks.

The outcome of this pass (which shapes were checked, their content digests, and pass/fail states per shape) is normalized using RFC 8785-style **Canonical JSON** (JCS) and hashed with BLAKE3 to produce the **`admission_digest`**.

## 5. Handoff to the Topological Generator

With the ontology successfully admitted, normalized, and sealed by both the `rdf_digest` and the `admission_digest`, the pipeline progresses to the `generate` phase. 

The deterministic hashes are recorded in the **`cmca_generation_manifest.json`** identity record, while the sequential generation events are securely hash-chained in the **`cmca_generation_receipt.json`**. The validated graph is now handed off to the topological generator, which executes Kahn’s Topological Sort to flatten the execution dependencies into fixed-width hardware bitmasks (e.g., `pred_mask`, `succ_mask`). These hardware bitmasks are then emitted as C-ABI aligned, zero-allocation Rust structs within `cmca_generated.rs`, safely crossing the `Gamma_CMCA` boundary into the Hot Path.
