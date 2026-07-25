# AutonomicPolicyGuard Integration Oracle

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` authoritative hot path

This document defines the strict mathematical bounds, Hoare contracts, and proof obligations for integrating the `AutonomicPolicyGuard` with the `mfw-auto-select` loop without introducing conditional execution, adhering to the BCINR Deterministic Substrate Constitution.

---

## 1. Mathematical Law and Execution Domain

The `AutonomicPolicyGuard` evaluates global invariant bounds and semantic safety. To integrate with the `mfw-auto-select` pipeline while preserving $CC=1$, the policy guard must project its decision into a strictly branchless execution mask $G_{mask}$.

Let $G_{mask}$ be the full-width policy acceptance mask, where $G_{mask} \in \{0, \sim0\}$.
Let $A \in [0, 255]$ be the base admission mask for the 8 tool candidates.

The integrated, policy-guarded admission mask is defined as:
$$ A_{guarded} = A \land (G_{mask} \pmod{256}) $$

When the PolicyGuard rejects the operation ($G_{mask} = 0$), $A_{guarded}$ evaluates to $0$. The downstream `mfw-auto-select` polynomial will deterministically assign a score of $0$ to all candidates. The root `select` function must then construct a typed refusal (e.g., `ControlStateUnadmitted`) using purely arithmetic structural mapping, bypassing all early-return semantics.

---

## 2. Hoare Contract for Integration

### PolicyGuard Admission Filter (`apply_policy_guard`)

**Mathematical Law:**
$$ G_{mask} = 0 \text{ (modulo } 2^{64}\text{)} - ( \text{PolicyValid} \text{ as } u64 ) $$
$$ A_{guarded} = A \land G_{mask\_8} $$

**Hoare Contract:**
* **Valid Input Domain:** Valid semantic state telemetry and fixed `mfw-auto-select` candidate admission mask $A$.
* **Output Range:** $A_{guarded} \in [0, 255]$.
* **Conservation Law:** If `PolicyValid = 0`, then $A_{guarded} = 0$ precisely, ensuring no candidate can achieve an evaluated score $>0$.
* **Monotonicity Law:** A more restrictive policy threshold reduces or preserves the bitwise weight of $A_{guarded}$.
* **Overflow Behavior:** $0u64 \text{ wrapping\_sub } (0|1)$ securely fills all bits. The downward projection to `u8` is truncation-safe.
* **Invalid-Input Refusal:** If $G_{mask} = 0$, the zeroed candidate scores must cascade into a $0$-value `any_found` indicator, forcing the selection outcome to yield a typed refusal `ControlStateUnadmitted` via bitwise selection.
* **Determinism:** Execution utilizes $CC=1$ logic. Uses only `&`, `|`, `!`, `^`, and wrapping integer arithmetic. Zero control-flow branches.
* **State-Mutation Boundary:** 0 heap allocations, purely algebraic on the stack. No mutation occurs if $G_{mask} = 0$.
* **Numeric Error Envelope:** Exact integer mathematics. $E_{abs} = 0$.

---

## 3. Proof Obligations

To maintain the Substrate Integrity Score (SIS) of 100, the following obligations must be met during integration:

1. **Mask Orthogonality (@turing_machine):**
   The source audit must verify that `G_mask` is applied to the admission mask $A$ exclusively via bitwise `&`. Any presence of `if G_mask == 0 { return Err(...); }` is a constitutional violation and must be flagged by the cheat scanner.
2. **Refusal Mapping (@armstrong_fault):**
   Hostile verification must prove that when `PolicyValid` is forced to false, the system exactly returns the `ControlStateUnadmitted` typed refusal. The transition from a valid `AutoSelectOutcome` to the refusal state must be achieved entirely through mask-based state selection.
3. **Determinism Preservation (@turing_machine):**
   The final `.s` object-code disassembly for the integration target must exhibit zero new conditional jumps (`jxx` / `b.xx`). The integration of the policy guard must compile to straight-line bitwise operations.
