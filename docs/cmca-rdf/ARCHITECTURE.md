# CMCA-RDF Architecture & Projection Specification

This document details the architecture and projection rules for the **Chatman Multifractal Consequence Allocation (CMCA)** model over RDF-aligned semantic state within the deterministic substrate of `bcinr`. ("Cross-Measure Cognitive Allocation" was this document's earlier, unreconciled expansion of the acronym — see `../CMCA_EXPLANATION.md` for the canonical name and the full list of superseded backronyms.)

---

## 1. Architectural Topology

CMCA-RDF functions as an interchangeable decision surface, mapping semantic meaning to resource allocation without introducing runtime branching (complying with Radon Law $CC=1$).

```text
       [ Ontology: cmca-rdf.ttl ]
                   │
                   ▼ (Project/Compile time)
        [ generator.py ] 
                   │
                   ▼ (Generates static registries)
   [ src/generated/case_studies.rs ] ──┐
   [ src/generated/generalization.rs ] ├─► [ CMCA Allocator Kernel ] (Decoupled & generic)
                   ▲                   │
                   │                   ▼
           [ Fixed-Point Math ] ◄──────┘
            (fixed.rs, Q16.16)
```

By parameterizing the allocator kernel (accepting references like `&[PackedSemanticState]`), we decouple the logic of allocation completely from specific generated namespaces.

---

## 2. Fixed-Point Arithmetic (`fixed.rs`)

To guarantee constant-time execution and support `#![no_std]`, floating-point operations are prohibited. All arithmetic is calculated via a branchless, saturating Q16.16 fixed-point type wrapping `u32`.

### Branchless Primitives
- **Selection (`const_select_u32`)**:
  ```rust
  const fn const_select_u32(condition: u32, a: u32, b: u32) -> u32
  ```
- **Comparison (`const_lt_u32`)**:
  ```rust
  const fn const_lt_u32(a: u32, b: u32) -> u32
  ```
- **Saturating Arithmetic**:
  - `saturating_add`: Flips to `u32::MAX` on overflow.
  - `saturating_sub`: Clamps to `0` on underflow.
  - `saturating_mul`: Multiplies in `u64` space and shifts right by 16 bits, saturating to `u32::MAX` on overflow.
  - `saturating_div`: Scales the numerator by $2^{16}$ in `u64` space, checks for division-by-zero, and divides safely.

---

## 3. Ontology Definition & RDF Schema

We define the vocabulary and schemas in Turtle format with standard classes and properties. 

### Schema Classes
- `cmca:SemanticObject`: An entity in the system representing an allocation candidate (e.g. proof obligation, cache artifact, workflow activity).
- `cmca:MeasureHead`: An independent valuation function/heuristic.
- `cmca:Lens`: A concentration level ($q$-exponent) defining allocation intensity.
- `cmca:LambdaCoefficient`: A weight parameter linking a specific `MeasureHead` and `Lens`.

### Factor Layout
Each `SemanticObject` maps to a packed state table of $F = 10$ factors:
1. `recomputationCost` (F0)
2. `verificationCost` (F1)
3. `standing` (F2)
4. `validity` (F3)
5. `accessFrequency` (F4)
6. `searchDemand` (F5)
7. `retrievalDemand` (F6)
8. `schedulingDemand` (F7)
9. `businessValue` (F8)
10. `downstreamConsequence` (F9) - *Computed transitively off-path*

---

## 4. Off-Path Downstream Consequence Mass

To avoid dynamic graph traversals or data-dependent loops on the hot path, downstream consequence is computed off-path at generation time.

For each object $v$, the consequence mass is computed recursively:
$$m(v) = \text{Value}(v) + \sum_{v \to u} w(v, u) \cdot m(u)$$

where $v \to u$ indicates that $v$ depends on $u$ (`cmca:dependsOn`). The generator runs a topological closure process to propagate values from downstream sinks back to upstream sources.

---

## 5. Verification Cases

### Case Study 1: Cache Choice
- **`Artifact_A`**: High recomputation, low verification, standing true, validity true.
- **`Artifact_B`**: Low recomputation, high verification, standing true, validity true.
- Identical or near-equal access frequencies.

### Case Study 2: Single Object, Multiple Decisions
- **`Obj_Single`**: Cache demand is high, search is low, retrieval is hot/high, scheduling is zero.

### Case Study 3: Consequence Chain
- Linear dependency path: `Obj_Obligation` $\to$ `Obj_Activity` $\to$ `Obj_Deployment` $\to$ `Obj_Outcome` $\to$ `Obj_Value` (carrying businessValue = 1000).
- Consequence mass propagates backwards, giving all nodes in the chain a mass of 1000.

### Case Study 4: Generalization
- Second configuration (`ontology/generalization.ttl`) with modified global coefficients (`eta = 0.3`, different lambda weights), proving that the schema compiles and generalizes without changing any core logic or generator code.
