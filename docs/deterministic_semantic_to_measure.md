# Deterministic Semantic-to-Measure Projection Pipeline

This document explains the end-to-end "deterministic semantic-to-measure projection" pipeline and how it directly maps to the **Rule 1 (Mission)** mathematical contract of bounded execution:

$$
\text{admitted input} \rightarrow \text{fixed instruction shape} \rightarrow \text{deterministic output}
$$

## The End-to-End Pipeline

Based on `docs/auto_select_semantic_projection.md`, the canonical pipeline is structured as follows:

`RDF semantics -> SHACL eligibility -> partial-order workflow -> numeric measure vector -> CMCA -> selected tool`

### 1. Canonical RDF Graph & SHACL Eligibility
Requests and tools are defined as semantic triples in an RDF graph. Tools possess specific `sh:NodeShape` requirements (e.g., `requiresDeterminism`). **SHACL determines legality** by filtering out incompatible tools before they reach the hot path. If a tool fails its shape, it is strictly eliminated before reaching CMCA.

### 2. Canonical Partial-Order Workflow
Evaluations occur concurrently across a set of defined activities `V = { A, C, S, U, T, E, M, W, Q, R }`:
- **A** (Admit request) $\rightarrow$ **C** (Compile shape)
- **C** $\rightarrow$ **S** (Semantic), **U** (Authority), **T** (Timing)
- **S**, **U**, **T** $\rightarrow$ **E** (Eligible set)
- **E** $\rightarrow$ **M** (Measures), **W** (Workflow)
- **M**, **W** $\rightarrow$ **Q** (CMCA selection)
- **Q** $\rightarrow$ **R** (Receipt)

### 3. Semantic-to-Measure Projection
The RDF compiler converts the semantics of each eligible tool into a bounded numeric coordinate vector `x_i`:
$x_i = (s_i, e_i, a_i, t_i, d_i, r_i, c_i)$

Each coordinate maps a semantic property to a fixed integer $x_{ik} \in [0, 255]$. For example:
- **Semantic fit**: Exact output (255) -> Lossless adapter (192) -> Incompatible (0)
- **Authority fit**: Exact token (255) -> Delegated (224) -> No authority (0)
- **Reliability**: Measured `successful / eligible executions * 255`

### 4. Constructing Canonical Mass
An unweighted geometric mean is calculated to prevent weak coordinates from being obscured by strong ones:
$m_i = 255 \times \prod (x_{ik} / 255)^{1/7}$

To preserve the branchless and floating-point-free laws of the substrate, this calculation does not happen at runtime. It avoids floating-point exponentiation in the hot path by using **precomputed lookup tables or generated mass values**.

### 5. CMCA Lens Selection
Selection is driven deterministically by applying a CMCA lens (e.g., exploitation-focused where $q=2$):
$\arg\max_i P_q(i) = (m_i^q) / (\sum m_j^q)$

For strict single-tool selection, this simplifies to $\arg\max_i (m_i^q)$.

---

## Mapping to the Mathematical Contract

The pipeline rigorously enforces the `admitted input -> fixed instruction shape -> deterministic output` constitutional law from `AGENTS.md`:

### 1. Admitted Input
**SHACL acts as the admission gate.** Only rigorously defined and semantically legal tools are permitted into the numeric projection. Tools failing their shape validation are completely rejected outside the hot path. This fulfills the requirement that unadmitted states and runtime discovery are strictly prohibited.

### 2. Fixed Instruction Shape
The AOT compiler translates semantic properties into **fixed-width C-ABI structures** (`ToolCandidate` and `AutoSelectInput8`). All coordinates become bounded `u8` integers. Floating-point mass is replaced with precomputed tables. As a result, the hot-path selection function executes the canonical invariants **entirely without branches** and across a fixed-size `candidates: [ToolCandidate; 8]` array.

### 3. Deterministic Output
The ultimate selection occurs through deterministic constant-time operations. The $\arg\max_i$ choice is evaluated without data-dependent control flow (e.g., via bitwise masks), guaranteeing that the output state transition is perfectly predictable and equivalent across all executions for the same admissible input mask.
