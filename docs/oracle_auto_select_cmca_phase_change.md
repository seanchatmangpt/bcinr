# MFW Auto-Select Oracle and Proof Obligations: CMCA Stochastic Phase Change Integration

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` authoritative hot path CMCA lens adjustment (`/Users/sac/bcinr/mfw-auto-select/src/phase_change.rs`)
**Iteration:** 10

This document defines the strict mathematical laws, Hoare contracts, valid domains, and proof obligations for integrating the CMCA Phase Change (lens exponent adjustment) into the `mfw-auto-select` pipeline. This step logically follows the Topological Scheduler (Iteration 9) and Receipt Integration, enabling dynamic adaptation between exploration and exploitation in accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`).

---

## 1. Mathematical Law and Execution Domain

The objective of the CMCA Phase Change is to dynamically adjust the geometric lens exponent $q_{lens}$ based on the error bound derived from the execution receipt, without using branching, loops, or floating-point arithmetic.

Let $q_t \in [1, 4]$ be the current CMCA lens exponent.
Let $E_{receipt} \in [0, 255]$ be the bounded error metric extracted from the most recent POWL execution receipt.
Let $T_{threshold} \in [0, 255]$ be the fixed error threshold for phase transitions.

The logical transition rule is:
* If $E_{receipt} \le T_{threshold}$, the system is stable; increase exploitation (increment $q$, clamped to 4).
* If $E_{receipt} > T_{threshold}$, the system is unstable; increase exploration (decrement $q$, clamped to 1).

To implement this branchlessly:
Let $M_{exploit} = \operatorname{mask\_lte}(E_{receipt}, T_{threshold})$ where $M_{exploit} \in \{0, \sim0\}$.
$$ q_{exploit} = \operatorname{min\_u8}(q_t + 1, 4) $$
$$ q_{explore} = \operatorname{max\_u8}(q_t - 1, 1) $$
$$ q_{t+1} = \operatorname{select}(M_{exploit}, q_{exploit}, q_{explore}) $$

---

## 2. Hoare Contract

**Primitive:** `calculate_next_q_lens(q_current: u8, e_receipt: u8, t_threshold: u8) -> u8`

**Contract:**
$$ \{ q_{current} \in [1, 4] \land E_{receipt} \in [0, 255] \land T_{threshold} \in [0, 255] \} \quad \operatorname{calculate\_next\_q\_lens} \quad \{ q_{next} \in [1, 4] \} $$

* **Valid Input Domain:** $q_{current} \in [1, 4]$. $E_{receipt}$ and $T_{threshold}$ are fully defined over $u8$.
* **Output Range:** $q_{next} \in [1, 4]$.
* **Conservation Law:** $q_{next} = q_{current}$ if and only if $q_{current} = 4 \land E_{receipt} \le T_{threshold}$ OR $q_{current} = 1 \land E_{receipt} > T_{threshold}$.
* **Monotonicity Law:** $q_{next}$ is monotonically non-increasing with respect to $E_{receipt}$.
* **Overflow Behavior:** $q_{current} + 1$ and $q_{current} - 1$ are perfectly safe as $q_{current} \in [1, 4]$. Handled via bounded integer arithmetic.
* **Invalid-Input Refusal:** None. The function is total over the typed inputs, assuming preconditions are met via prior masking.
* **Determinism:** Branchless $CC=1$. Uses bitwise selection (`(mask & a) | (!mask & b)`).
* **State-Mutation Boundary:** Pure mathematical function, 0 heap allocations, purely immutable inputs.
* **Numeric Error Envelope:** $E_{abs} = 0$. Exact integer logic.

---

## 3. Proof Obligations

To maintain the Substrate Integrity Score (SIS = 100) and full PhD-Verified status, the following proofs must be supplied:

1. **Topological Loop Freedom (`@turing_machine`):**
   Must mechanically verify the compiled object code contains 0 conditional jump instructions (`je`, `jg`, etc.) and no allocator paths.
2. **Exhaustive Selection Matrix (`@hoare_oracle`):**
   Since the input domain is infinitesimally small ($4 \times 256 \times 256 = 262,144$ states), the function must be exhaustively verified against a branching oracle over the entire domain $2^{18}$.
3. **Refusal & State Adherence (`@armstrong_fault`):**
   Hostile mutants (e.g. inverted mask `mask_gt` instead of `mask_lte`, omitted clamp, or inverted select args) must be proven to trigger invariant assertion failures in the test matrix.
