# Auto Select Semantic-to-Measure Projection Oracle (Iteration 12)

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` semantic coordinate mapping phase.

This document defines the mathematical bounds and Hoare contracts for the next logical step in the Auto Select pipeline (Iteration 12): the deterministic, zero-allocation Semantic-to-Measure Projection mapping RDF constraints to fixed-width `ToolCandidate` coordinates.

---

## 1. Mathematical Law and Execution Domain

The Semantic-to-Measure Projection maps the evaluation of semantic structures (SHACL shapes, authority graphs, execution DAGs) to a fixed-width vector $x_i \in [0, 255]^7$.
Let $C_{req}$ be the semantic constraint matrix of the request and $C_{tool}$ be the capability matrix of tool $i$.
The projection function $f_{proj}: (C_{req}, C_{tool}) \rightarrow \text{ToolCandidate}$ is strictly bounded.

The output vector components are:
- $s_i \in \{0, 192, 255\}$ (Semantic Fit)
- $e_i \in \{0, 220, 255\}$ (Evidence Fit)
- $a_i \in \{0, 224, 255\}$ (Authority Fit)
- $t_i \in [0, 255]$ (Timing Fit)
- $d_i \in \{0, 255\}$ (Downstream Fit)
- $r_i \in [0, 255]$ (Reliability)
- $c_i \in [0, 255]$ (Cost Fit)

## 2. Hoare Contracts

### Semantic Projection Mapping (`project_semantic_coordinate`)

**Mathematical Law:**
$$ s_i = \operatorname{select}(m_{exact}, 255, \operatorname{select}(m_{lossless}, 192, 0)) $$

**Hoare Contract:**
* **Valid Input Domain:** Validated $C_{req}$ and $C_{tool}$ structural bitmasks (not arbitrary RDF subgraphs in the hot path). The slow rail handles RDF parsing and provides pre-validated fixed-width adjacency matrix masks.
* **Output Range:** Returns a populated `ToolCandidate` structure with all fit coefficients $\in [0, 255]$.
* **Conservation Law:** Information entropy of semantic capabilities is losslessly mapped into a clamped 8-bit metric space. Sum of structural capacities is bounded.
* **Monotonicity Law:** Adding additional independent capability tokens strictly monotonically increases or maintains the respective coordinate.
* **Overflow Behavior:** Wrapping cannot occur because mapping utilizes constant saturation clamping values (e.g. 192, 255).
* **Invalid-Input Refusal:** If an input shape matrix is invalid or indicates an authority contradiction, the projection sets $m_{admitted} = 0$, generating a structurally refused `ToolCandidate` (e.g., yielding 0 mass), emitting `TypedRefusal::UnsupportedDomain`.
* **Determinism:** Execution enforces $CC=1$. Mask-based selection `select_u8` entirely replaces semantic branching.
* **State-Mutation Boundary:** Fixed-size `ToolCandidate` struct construction on the stack. Exactly 0 heap allocations.
* **Numeric Error Envelope:** $E_{abs} = 0$. Pure fixed-width integer mapping.

---

## 3. Proof Obligations

To satisfy integration integrity before downstream merging:

1. **Topological Object-Code Audit (@turing_machine):**
   Audit assembly output of the projection mapping to verify the absence of loop backedges, implicit panics, and dynamic allocation. Zero conditional branching during semantic structural comparison.
   
2. **Refusal Conservation (@armstrong_fault):**
   Inject hostile mutants simulating a contradictory semantic graph (e.g., requires strict determinism but provides uncertified LLM). The integration must mathematically force $s_i = 0$ and yield `TypedRefusal::UnsupportedDomain` without `assert_ne!`.
   
3. **Exhaustive Mapping Matrix (@hoare_oracle):**
   Demonstrate a structural proof that every permutation of semantic fit categories evaluates exactly to the integer constants defined in the domain ($255, 192, 0$) using bitwise verification over the $2^{32}$ domain space of the capability masks.
