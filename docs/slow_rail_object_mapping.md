# Slow Rail Object Mapping and Index Sorting in `bcinr`

In the BranchlessCInRust (BCINR) architecture, the Authoritative Hot Path is strictly governed by the **Radon Law ($CC=1$)**, `#![no_std]`, and absolute zero heap allocation. Unbounded Semantic Web data (like RDF graphs) intrinsically violate these runtime constraints. To safely ingest semantic metadata, the **Slow Rail** utilizes Ahead-of-Time (AOT) codegen tooling (`mfw-codegen` / `generator.py`) to flatten dynamic properties into deterministic, zero-indexed $O(1)$ arrays via explicit mapping properties such as `cmca:measureIndex`.

## 1. The Need for Deterministic Array Indexing

Traditional graph databases and dictionaries process properties dynamically, but they do not inherently guarantee a fixed runtime sequence. If the Hot Path cannot allocate queues or loop dynamically, all entities must be definitively pinned to physical memory offsets.

The Slow Rail guarantees strict ordering by requiring explicit indices on key structural elements in the RDF layer:
- **`cmca:measureIndex`** pins independent metric constructs (`cmca:MeasureHead`) to fixed array bounds ($K$).
- **`cmca:lensIndex`** orders distortions or weightings (`cmca:Lens`) to fixed sequence bounds ($Q$).

For example, a Turtle (`.ttl`) definition enforces sequence at the source:
```turtle
cmca:MeasureRetrieval a cmca:MeasureHead .
cmca:MeasureRetrieval cmca:measureIndex "2"^^xsd:integer .
```

## 2. AOT Sorting and Fixed Sequence Bounds

During the `generate` phase, `generator.py` extracts these RDF primitives. Instead of yielding a dynamic hash map, the generator extracts `cmca:measureIndex` properties and sorts them explicitly:

```python
# Extract and deterministically sort Measure Heads
measure_heads = [mh for mh, cls in classes.items() if cls == 'cmca:MeasureHead']
mh_indices = {mh: properties.get(mh, {}).get('cmca:measureIndex', 0) for mh in measure_heads}
sorted_mh = sorted(measure_heads, key=lambda m: (int(mh_indices[m]), m))

# Establish fixed bound K
K = len(sorted_mh)
if K > K_MAX:
    raise ValueError("CMCA_MEASURE_COUNT_EXCEEDED")
```
By resolving the sequence bound *offline*, `K` and `Q` become strictly bounded `pub const` values in the output IR, mapping exactly to standard 0-indexed C-ABI array layouts.

## 3. Resolving the LAMBDA Matrix and Relationships

Cross-dimensional graphs—such as a `cmca:LambdaCoefficient` specifying a distortion value for a specific measure/lens combination—are resolved identically. The codegen tool takes the declared `measureIndex` and `lensIndex`, determines their position within the established zero-indexed bounds, and yields a static $K \times Q$ 2D matrix (`[[NonNegativeFixed; Q]; K]`).

Additionally, for execution and topological rules (`cmca:dependsOn`), the Slow Rail runs **Kahn's Topological Sort** offline. The dependencies are transformed from dynamic pointers into $O(1)$ **SWAR Bitmasks** (`u64`). 
- **`pred_mask`:** A bit is set if the dependency mapped to index `j` must precede index `i`.
- **`succ_mask`:** A bit is set for downstream consequences.

## 4. Emitting the Hot Path $CC=1$ IR

The resulting data layout is serialized directly into static Rust IR (`cmca_generated.rs`), producing perfectly C-ABI aligned structures. 

```rust
pub const K: usize = 4;
pub const Q: usize = 4;
pub const MEASURE_RETRIEVAL: usize = 2; // Derived directly from cmca:measureIndex

pub static LAMBDA: [[NonNegativeFixed; Q]; K] = [
    // Pre-calculated Q16.16 Fixed Point Vectors
    [NonNegativeFixed::from_bits(26214), /* ... */], 
    // ...
];
```

To eliminate any chance of data-dependent looping over these arrays, the generator emits purely macro-unrolled execution logic (`unroll_k_static!`, `unroll_q_static!`). Consequently, the Hot Path operates completely branchlessly in `CC=1`.

## 5. Substrate Integrity Sealing

To comply with Substrate Constitution Rule 21 ("Generated-code law"), the tooling binds this precise fixed-index generation to the source materials cryptographically. The exact hash of the RDF input and the generator itself are permanently embedded (`RDF_INPUT_DIGEST`) into the `cmca_generated.rs`. This enforces the **`Gamma_CMCA` Contract**, assuring that runtime state indices have securely and deterministically trickled down from validated Semantic models.
