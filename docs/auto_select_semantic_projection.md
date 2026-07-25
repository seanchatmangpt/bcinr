# Canonical Auto Select: Deterministic Semantic-to-Measure Projection

The missing bridge in deterministic workflow allocation is the semantic-to-measure projection. RDF should not contain arbitrary "scores" with no provenance. It should contain semantic facts, which a compiler then converts into the bounded numeric coordinates CMCA requires.

The canonical pipeline is:
`RDF semantics -> SHACL eligibility -> partial-order workflow -> numeric measure vector -> CMCA -> selected tool`

## 1. Canonical RDF Graph \u0026 SHACL Eligibility

Requests and tools are defined as semantic triples (e.g., `RetrievalGoal`, `AdmittedObservation`, `ExactGraphRetrieval`).
Tools have specific `sh:NodeShape` requirements (e.g., `requiresDeterminism`, `hasObservation`).

**SHACL determines legality. CMCA determines preference among legal options.**
If a tool (like a Vector Tool) fails its shape (because the request requires determinism), it is strictly eliminated before reaching CMCA.

## 2. Canonical Partial-Order Workflow

Evaluations occur concurrently.
The activities are `V = { A, C, S, U, T, E, M, W, Q, R }`
- A: admit request -> C: compile shape
- C -> S (semantic), U (authority), T (timing)
- S, U, T -> E (eligible set)
- E -> M (measures), W (workflow)
- M, W -> Q (CMCA selection)
- Q -> R (receipt)

## 3. How Semantics Become Numbers

The RDF compiler produces one coordinate vector per eligible tool:
`x_i = (s_i, e_i, a_i, t_i, d_i, r_i, c_i)`
Each coordinate is a fixed integer `x_{ik} \in [0, 255]`.

### Coordinate Mappings
- **Semantic fit**: Exact output (255) -> Lossless adapter (192) -> Incompatible (0)
- **Evidence fit**: Primary record (255) -> Verified secondary (220) -> No evidence (0)
- **Authority fit**: Exact token (255) -> Delegated (224) -> No authority (0 - usually a hard refusal)
- **Timing fit**: Clamped integer based on `p99 latency` vs `deadline`.
- **Downstream fit**: Output satisfies next shape (255) -> Cannot feed next process (0)
- **Reliability**: Measured `successful / eligible executions * 255`.
- **Cost efficiency**: Resource cost clamped inverse.

## 4. Constructing Canonical Mass

An unweighted geometric mean is used so a weak coordinate is not hidden by a strong one:
`m_i = 255 * \prod (x_{ik} / 255)^{1/7}`
In Rust, this avoids floating-point exponentiation in the hot path by using precomputed lookup tables or generated mass values.

## 5. CMCA Lens

For an exploitation-focused ($q=2$) lens, CMCA chooses:
`\arg\max_i P_q(i) = (m_i^q) / (\sum m_j^q)`

For strict single-tool selection, this simplifies to `\arg\max_i (m_i^q)`.

## 6. Compiled Runtime Representation (The Hot Path)

The AOT compiler generates fixed-width C-ABI structures:

```rust
#[repr(C)]
pub struct ToolCandidate {
    pub tool_id: u8,
    pub semantic_fit: u8,
    pub evidence_fit: u8,
    pub authority_fit: u8,
    pub timing_fit: u8,
    pub downstream_fit: u8,
    pub reliability: u8,
    pub cost_fit: u8,
    pub mass: u8,
}

#[repr(C)]
pub struct AutoSelectInput8 {
    pub request_id: u32,
    pub eligible_mask: u8,
    pub ready_mask: u8,
    pub required_authority: u16,
    pub q_lens: u8,
    pub candidates: [ToolCandidate; 8],
}
```

The hot path selection function evaluates the canonical invariants entirely without branches to select the optimal tool from the admissible mask.
