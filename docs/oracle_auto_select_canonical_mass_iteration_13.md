# Auto Select Canonical Mass and CMCA Selection Oracle (Iteration 13)

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` hot path canonical mass generation and deterministic selection.

This document defines the mathematical bounds and Hoare contracts for the next logical step in the Auto Select pipeline (Iteration 13): the deterministic, zero-allocation derivation of Canonical Mass ($m_i$) and the branchless CMCA Lens selection ($\arg\max$) over bounded `ToolCandidate` sets.

---

## 1. Mathematical Law and Execution Domain

Following the Semantic-to-Measure Projection (Iteration 12), Iteration 13 calculates the Canonical Mass ($m_i$) and evaluates the CMCA Lens for a set of eligible tool candidates.

Let $x_i = (s_i, e_i, a_i, t_i, d_i, r_i, c_i)$ be the fixed-width numeric coordinate vector for candidate $i$, where $x_{ik} \in [0, 255]$.

The canonical mass is mathematically defined as an unweighted geometric mean over the 7 coordinates:
$$ m_i = 255 \times \prod_{k=1}^{7} \left(\frac{x_{ik}}{255}\right)^{1/7} $$

In the hot path, fractional exponentiation and floating-point logic are strictly prohibited. The operation is transformed into a fixed-point log-space accumulation:
$$ \log_2(m_i) = \log_2(255) + \frac{1}{7} \sum_{k=1}^{7} \left( \log_2(x_{ik}) - \log_2(255) \right) $$
implemented via fixed-point Q16.16 lookup tables and branchless addition. The final output is $m_i \in [0, 255]$.

The CMCA Lens selection (for exploitation, $q=2$) resolves to a strict maximum search over the admissible mask:
$$ \operatorname{selected\_index} = \arg\max_{i \in \text{admitted}} (m_i^q) $$

---

## 2. Hoare Contracts

### Canonical Mass & Selection (`select_optimal_candidate`)

**Mathematical Law:**
The selection must take a form equivalent to a bit-parallel reduction:
$$ \operatorname{selected\_index} = \operatorname{reduce\_max}( \operatorname{select}(m_{admitted}, m_i, 0) ) $$

**Hoare Contract:**
* **Valid Input Domain:** An array of `ToolCandidate` (fixed capacity $N = 8$) and an `admitted_mask` $\in [0, 2^8 - 1]$.
* **Output Range:** Returns a populated index $\in [0, 7]$, or a typed refusal.
* **Conservation Law:** An unadmitted candidate ($m_{admitted}[i] = 0$) contributes a mass of strictly $0$ to the reduction. The selected candidate has the maximal $m_i^q$ score among all admitted bits.
* **Monotonicity Law:** Increasing any coordinate $x_{ik}$ of an admitted candidate monotonically increases or maintains its total $m_i$, thereby monotonically increasing its probability of selection.
* **Overflow Behavior:** $m_i$ calculations saturate at 255. Intermediate calculations for $m_i^q$ use safe zero-extended bounds (e.g., `u16`) to prevent overflow before final bitwise reduction.
* **Invalid-Input Refusal:** If `admitted_mask == 0` (no candidates are legal under SHACL), the projection sets $m_{admitted} = 0$, generating a structurally refused evaluation emitting `TypedRefusal::NoLeaves` or `TypedRefusal::ControlStateUnadmitted`.
* **Determinism:** Execution rigorously enforces $CC=1$. Loop backedges and data-dependent conditional jumps (`if`, `match`) are entirely replaced by SWAR and bit-parallel masking (`select_u8`, `mask_gt`).
* **State-Mutation Boundary:** Fixed-size calculation over stack values. Exactly 0 heap allocations.
* **Numeric Error Envelope:** Fixed-point mapping yields $E_{abs} \le 1$ unit of least precision (ULP) in the final `u8` mass compared to the arbitrary-precision mathematical reference.

---

## 3. Proof Obligations

To satisfy integration integrity before downstream merging:

1. **Topological Object-Code Audit (@turing_machine):**
   Audit the assembly output of the fixed-point mass derivation and `argmax` reduction mapping. Verify the absolute absence of loop backedges, implicit panics, and dynamic allocation. Zero conditional branching during the array maximum reduction (must use `cmov` or SIMD selection).

2. **Refusal Conservation (@armstrong_fault):**
   Inject hostile mutants simulating a corrupted `admitted_mask` (e.g., allowing a candidate with $m_{admitted} = 0$ but high mass to be selected). The test matrix must mathematically force the pipeline to yield the correct uncorrupted index or `TypedRefusal::ControlStateUnadmitted` without using `assert_ne!`.

3. **Exhaustive Mapping Matrix (@hoare_oracle):**
   Demonstrate a structural proof that the fixed-point log geometric mean preserves strict monotonicity across the domain $2^{16}$ per coordinate, and the argmax reduction accurately selects the true maximum among $N=8$ candidates under all $2^8$ permutation masks.
