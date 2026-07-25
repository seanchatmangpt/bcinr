# MFW Auto-Select Oracle and Proof Obligations

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` authoritative hot path (`/Users/sac/mfw/mfw-auto-select/src/lib.rs`)

This document defines the strict mathematical laws, Hoare contracts, valid domains, and proof obligations for the CMCA `mfw-auto-select` tool selection implementation in accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`).

---

## 1. Mathematical Law and Execution Domain

The objective of the `mfw-auto-select` hot path is to project a semantic state into a strict, predictable measure (mass), and perform a selection bounding a given query $q$.

Let $C = \{c_0, c_1, \dots, c_7\}$ be a fixed set of 8 tool candidates.
Let $A$ be the admission mask, representing exogenous system admissibility (e.g. SHACL eligibility).
Let $S_{mask}$ be the active cognition state mask.
Let $q_{lens} \in \{1, 2, 3, 4\}$ be the geometric exponent.

The auto-select system must compute:
$$ i^* = \text{argmax}_{i \in [0, 7]} \left( \text{admissible}(c_i, A, S_{mask}) \cdot \text{mass}(c_i)^{q_{lens}} \right) $$

Where admissibility is the bitwise conjunction of systemic eligibility and local cognition constraints. 

---

## 2. Hoare Contracts

For every primitive in the hot path, a strict Hoare contract $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$ is enforced.

### 2.1 Canonical Mass Calculation (`calculate_canonical_mass`)

**Mathematical Law:** 
Computes the unweighted geometric mean of 7 semantic parameters.
$$ M = \lfloor \left( \prod_{k=1}^7 fit_k \right)^{1/7} \rfloor $$

**Hoare Contract:**
* **Valid Input Domain:** $fit_k \in [0, 255]$ for all $k \in \{1, \dots, 7\}$.
* **Output Range:** $M \in [0, 255]$.
* **Conservation Law:** $M = 0 \iff \exists k . fit_k = 0$.
* **Monotonicity Law:** $\forall x, y$: if $\forall k. x_k \ge y_k$, then $M(x) \ge M(y)$.
* **Overflow Behavior:** The internal product maximum is $255^7 \approx 7.03 \times 10^{16}$, which fits entirely within a 64-bit integer ($1.84 \times 10^{19}$). No overflow occurs.
* **Invalid-Input Refusal:** None, the function is total over the $u8$ domain.
* **Determinism:** Branchless (CC=1) static 8-bit unrolled binary search.
* **State-Mutation Boundary:** 0 heap allocations, pure stack evaluation.
* **Numeric Error Envelope:** Maximum absolute error is strictly $E_{abs} < 1.0$ due to floor truncation.

### 2.2 Candidate Evaluation (`evaluate_candidate`)

**Mathematical Law:**
Computes the candidate score conditionally mapped to $0$ if inadmissable, and retains the running maximum in a strictly stable sort structure (preserving earlier indices on tie).

**Hoare Contract:**
* **Valid Input Domain:** Valid $c_i$, $q_{lens} \in [1, 4]$, current best score $S_{best} \le 255^4$.
* **Output Range:** Yields `(next_best_score, next_best_id, next_any_found)` conforming to the updated bounds.
* **Conservation Law:** Exactly preserves `current_best_score` and `current_best_id` if the active candidate is either inadmissable or yields a score $\le S_{best}$.
* **Monotonicity Law:** The accumulated score $S_{best}$ is strictly non-decreasing over sequential calls.
* **Overflow Behavior:** Score polynomials $m^4 \le 255^4 = 4,228,250,625 \le 2^{32}-1$. Safe from 32-bit unsigned overflow.
* **Invalid-Input Refusal:** Handled at the pipeline root; internal evaluation assumes inputs are sanitized.
* **Determinism:** Execution uses arithmetic bitmask selection `(mask & active) | (!mask & current)`. Contains no branching constructs (CC=1).
* **State-Mutation Boundary:** Fixed inputs, pure value return.
* **Numeric Error Envelope:** Exact integer mathematics. $E_{abs} = 0$.

### 2.3 Selection Root (`select`)

**Mathematical Law:**
Strict unrolled evaluation of the 8 bounded candidates, returning the maximal outcome or typed refusal.

**Hoare Contract:**
* **Valid Input Domain:** Fully defined over `AutoSelectInput8`.
* **Output Range:** `Result<AutoSelectOutcome, AutoSelectRefusal>`.
* **Conservation Law:** Emits exact selected candidate matching the maximal score subject to static topological index stability.
* **Monotonicity Law:** N/A for static inputs.
* **Overflow Behavior:** Fixed array access [0..7]; statically bounded logic.
* **Invalid-Input Refusal:** 
  - `UnsupportedDomain` if $q_{lens} \notin [1, 4]$.
  - `NumericRangeExceeded` if any $\text{tool\_id} \ge 8$.
  - `ContractViolation` if cognition rule `add_mask` and `del_mask` intersect.
* **Determinism:** CC=1 loop-free static evaluation pipeline.
* **State-Mutation Boundary:** Immutable reference in, structurally unpadded bounded struct out. 0 allocations.
* **Numeric Error Envelope:** Exact integer return.

### 2.4 SHACL Eligibility Translator (`translate_shacl_eligibility`)

**Mathematical Law:**
$ E_{mask} = \sum_{i=0}^7 (conformance_i \ll i) $

**Hoare Contract:**
* **Valid Input Domain:** $conformance_i \in \{0, 1\}$ for $i \in [0, 7]$.
* **Output Range:** $E_{mask} \in [0, 255]$.
* **Conservation Law:** Bit $i$ in $E_{mask}$ exactly equals $conformance_i$.
* **Monotonicity Law:** N/A.
* **Overflow Behavior:** Shifts strictly constrained $\le 7$ into an 8-bit integer.
* **Invalid-Input Refusal:** (Caller must guarantee domain).
* **Determinism:** Branchless shift-OR sequence.
* **State-Mutation Boundary:** Pure stack mapping.
* **Numeric Error Envelope:** 0 error.

---

## 3. Proof Obligations

To ensure rigorous integrity of the `mfw-auto-select` implementation, the following independent proof obligations must be certified by the respective agents.

1. **Topological Loop Freedom (@turing_machine):**
   Must formally verify that the generated assembly (e.g. `objdump`) for `select` contains exactly zero loop backedges, branch instructions (`je`, `jne`, `jl`, etc. based on dynamic evaluation), and no heap allocator calls.
2. **Exhaustive Selection Matrix (@hoare_oracle):**
   Using property testing, prove that for any combination of `q_lens`, `masses`, `masks`, and `admissibility`, the branchless `select` logic yields identical outputs to the branching iteration oracle `oracle_select` (equivalence bounded by $2^{64}$).
3. **Refusal Adherence (@armstrong_fault):**
   Must assert all three typed refusals trigger under exact specified domain breaches (e.g. $q_{lens} = 0$, overlapping cognition masks), maintaining exact structural immutability upon rejection.
4. **Mutant Survivability (@armstrong_fault):**
   All 3 structural mutants (`mutant_1`, `mutant_2`, `mutant_3`) plus implicit logic faults (mask inversions, improper LT mappings) must unequivocally fail testing. If a mutant survives, SIS score falls to 0.
