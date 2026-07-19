# Semantic RDF to Rust IR Projection Pipeline

## 1. Introduction and Overview

The projection pipeline transforms formal semantic ontologies expressed in RDF Turtle (`.ttl`) format into deterministic, strictly bounded, allocation-free Rust Intermediate Representation (IR). This bridges the expressive flexibility of the semantic web with the rigorous algebraic and mechanical constraints required by the BranchlessCInRust (BCINR) Cognitive Mass Conservation Architecture (CMCA) substrate.

This report details the execution mechanics of the generator script located at:
`crates/bcinr-cmca/generator.py`

Which operates on ontologies such as:
- `crates/bcinr-cmca/ontology/cmca-rdf.ttl`
- `crates/bcinr-cmca/ontology/generalization.ttl`

The result is a Rust source file typically placed at:
`crates/bcinr-cmca/src/generated/case_studies.rs`

## 2. Parsing the RDF Turtle

The pipeline eschews massive external graph parsing libraries in favor of a specialized, highly constrained Turtle parser (`parse_ttl`). This approach immediately enforces rigorous constraints by rejecting ambiguous or overly complex Turtle constructs.

### 2.1 Rejection of Unbounded Constructs

The pipeline deliberately refuses to parse any structures that could lead to unbounded parsing complexity or non-deterministic mapping, including:
- Multiline literals (`"""` or `'''`)
- Language tags (`@en`)
- Blank nodes (`_:`)
- Nested collections or named graphs

```python
# crates/bcinr-cmca/generator.py
# Reject unsupported constructs
if '"""' in clean_content or "'''" in clean_content:
    raise ValueError("Unsupported Turtle construct: multiline literals")
if '@en' in clean_content or '"@' in clean_content:
    raise ValueError("Unsupported Turtle construct: language tags")
if '_:' in clean_content:
    raise ValueError("Unsupported Turtle construct: blank nodes")
```

### 2.2 Extraction of Classes and Properties

The parsing separates the dataset into `classes` (derived from the `a` predicate) and `properties`. Typed literals (like `xsd:decimal`) are parsed natively into floats or integers.

```python
# crates/bcinr-cmca/generator.py
if pred == 'a':
    classes[subj] = val
else:
    if subj not in properties:
        properties[subj] = {}
    if pred == 'cmca:dependsOn':
        if 'cmca:dependsOn' not in properties[subj]:
            properties[subj]['cmca:dependsOn'] = []
        properties[subj]['cmca:dependsOn'].append(val)
    else:
        properties[subj][pred] = val
```

## 3. Shape Validation

After parsing, the generator mathematically guarantees shape requirements for critical architectural entities. `cmca:Lens` must possess a `cmca:lensExponent`, and `cmca:SemanticObject` must have numerics for properties like `cmca:businessValue`.

```python
def validate_shapes(classes, properties):
    for subj, cls in classes.items():
        if cls == 'cmca:SemanticObject':
            props = properties.get(subj, {})
            if 'cmca:businessValue' in props:
                assert isinstance(props['cmca:businessValue'], (int, float)), "businessValue must be numeric"
        elif cls == 'cmca:Lens':
            props = properties.get(subj, {})
            assert 'cmca:lensExponent' in props, "Lens must have exponent"
            assert isinstance(props['cmca:lensExponent'], (int, float)), "lensExponent must be numeric"
```

## 4. Deterministic Index-Sorting

In a system executing deterministically branchless operations, order matters profoundly. Dictionaries or graph databases do not inherently preserve a fixed sequence. The pipeline must force a stable layout by mapping semantic indices directly to array offsets.

### 4.1 Measure Heads (K)

`cmca:MeasureHead` instances (representing independent metrics like Cache, Search, Retrieval, Scheduling) are assigned numerical indices explicitly in the TTL (`cmca:measureIndex`).

```turtle
# ontology/cmca-rdf.ttl
cmca:MeasureRetrieval a cmca:MeasureHead .
cmca:MeasureRetrieval cmca:measureIndex "2"^^xsd:integer .
```

The generator extracts these elements, maps them to their declared index, and sorts them to form the sequence of size `K`.

```python
# crates/bcinr-cmca/generator.py
# 6. Measure Heads (K)
measure_heads = [mh for mh, cls in classes.items() if cls == 'cmca:MeasureHead']
mh_indices = {mh: properties.get(mh, {}).get('cmca:measureIndex', 0) for mh in measure_heads}
sorted_mh = sorted(measure_heads, key=lambda m: int(mh_indices[m]))
K = len(sorted_mh)
if K > K_MAX:
    raise ValueError("CMCA_MEASURE_COUNT_EXCEEDED")
```

### 4.2 Lenses (Q)

Similarly, `cmca:Lens` objects (representing non-linear distortions or weightings over measures) are strictly ordered using `cmca:lensIndex` up to size `Q`.

```python
# 7. Lenses (Q)
lenses = [lens for lens, cls in classes.items() if cls == 'cmca:Lens']
lens_indices = {lens: properties.get(lens, {}).get('cmca:lensIndex', 0) for lens in lenses}
lens_exponents = {lens: properties.get(lens, {}).get('cmca:lensExponent', 0.0) for lens in lenses}
sorted_lenses = sorted(lenses, key=lambda l: int(lens_indices[l]))
Q = len(sorted_lenses)
```

## 5. LAMBDA Matrix Generation

The LAMBDA Matrix is the K × Q core matrix characterizing how each lens distorts each measure. It maps `cmca:LambdaCoefficient` instances into a rigid, branchless two-dimensional Rust array.

The script scans for coefficients matching specific measures and lenses. If a URI is omitted, it falls back deterministically to the defined `lensIndex` or `measureIndex`.

```python
# 8. Lambda Matrix
lambda_coeffs = {}
for coeff_uri, cls in classes.items():
    if cls == 'cmca:LambdaCoefficient':
        props = properties.get(coeff_uri, {})
        m_uri = props.get('cmca:measure')
        l_uri = props.get('cmca:lens')
        # Dynamic fallback by index mapping...
        if m_uri in sorted_mh and l_uri in sorted_lenses:
            m_idx = sorted_mh.index(m_uri)
            l_idx = sorted_lenses.index(l_uri)
            lambda_coeffs[(m_idx, l_idx)] = val
```

This is then serialized into `[NonNegativeFixed; Q]` fixed-point vectors.

```rust
pub static LAMBDA: [[NonNegativeFixed; Q]; K] = [
    [NonNegativeFixed::from_bits(26214), NonNegativeFixed::from_bits(19661), NonNegativeFixed::from_bits(13107), NonNegativeFixed::from_bits(6554)], // MeasureCache
    [NonNegativeFixed::from_bits(6554), NonNegativeFixed::from_bits(13107), NonNegativeFixed::from_bits(19661), NonNegativeFixed::from_bits(26214)], // MeasureSearch
    // ...
];
```

## 6. Semantic Objects and Factor Mapping

`cmca:SemanticObject` instances define the state (`PackedSemanticState`). The script:
1. Discover all objects.
2. Identifies dependency chains (`cmca:dependsOn`).
3. Recursively resolves `downstreamConsequence` values dynamically.
4. Auto-discovers property keys across all semantic objects, generating deterministic indices for `F` metrics (e.g., `FACTOR_SEARCH_DEMAND`).

```python
def get_consequence_mass(obj, path=None):
    if path is None:
        path = set()
    if obj in memo:
        return memo[obj]
    if obj in path:
        return 0.0 # break cycles
    path.add(obj)
    val = business_values.get(obj, 0.0)
    for dep in dependencies.get(obj, []):
        val += get_consequence_mass(dep, path)
    path.remove(obj)
    memo[obj] = val
    return val
```

This ensures objects are statically compiled with all context correctly aggregated.

## 7. Compilation to Static Rust IR

Finally, the python pipeline interpolates all mapped variables into pure, precomputed Rust static variables and constants, bypassing runtime initialization or heap allocation overhead:

```rust
// Generated file: src/generated/case_studies.rs
pub const N: usize = 7;
pub const F: usize = 9;
pub const K: usize = 4;
pub const Q: usize = 4;

pub const GENERATOR_VERSION: &str = "v1.1.0";
pub const RDF_INPUT_DIGEST: &str = "...";
pub const GENERATOR_SOURCE_DIGEST: &str = "...";

pub const FACTOR_ACCESS_FREQUENCY: usize = 0;
// ...

pub static OBJECT_REGISTRY: [PackedSemanticState; N] = [
    // Artifact_A (cmca:Artifact_A)
    PackedSemanticState {
        id: 0,
        factors: [
            NonNegativeFixed::from_bits(32768), // accessFrequency: 0.50000
            NonNegativeFixed::from_bits(655360), // businessValue: 10.00000
            NonNegativeFixed::from_bits(58982), // recomputationCost: 0.90000
            // ...
        ],
    },
    // ...
];
```

It also generates critical macros (`unroll_n_static!`, `unroll_q_static!`, `unroll_k_static!`) which permit zero-overhead loop unrolling through standard Rust code generation mechanisms.

## Conclusion

The pipeline transforms unbound, graph-theoretic metadata representing abstract CMCA primitives (Lenses, MeasureHeads, Constraints) into strict deterministic fixed-point state variables mapped directly onto algebraic constructs. By indexing natively and recursively processing dependencies ahead-of-time (AOT), the Rust binary preserves branchless constraints at runtime while retaining complete traceability to a mathematically rigorous Semantic Web ontology.
