# Auto Select Fieldwise Masked Commit Oracle (Iteration 16)

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` hot path transactional state mutation.

This document defines the mathematical bounds and Hoare contracts for the next logical step in the Auto Select pipeline (Iteration 16): the deterministic, branchless Fieldwise Masked Commit of the selected tool candidate's state transitions to the persistent `AutonomicState`.

---

## 1. Mathematical Law and Execution Domain

Following the Canonical Mass and CMCA Selection (Iteration 13) and the subsequent extraction of the optimal `AutoSelectResult`, the pipeline must now commit the selected action's envelope to the persistent state. According to the substrate constitution, speculative mutation is strictly prohibited. State must be updated via a bit-parallel fieldwise masked commit.

Let $x_t$ be the current immutable state vector.
Let $x_{\text{candidate}}$ be the proposed candidate state derived from applying the `AutoSelectResult`.
Let $m_{\text{admitted}} \in \{0, 2^{w}-1\}$ be the bit-expanded admission mask derived from `AutoSelectResult::is_ok` (where $w$ is the word size of the field).

The lawful transactional commit is defined strictly as:

$$ x_{t+1} = \operatorname{select}(m_{\text{admitted}}, x_{\text{candidate}}, x_t) $$
$$ x_{t+1} = (m_{\text{admitted}} \land x_{\text{candidate}}) \lor (\neg m_{\text{admitted}} \land x_t) $$

If the selection produced a refusal (`is_ok == 0`, $m_{\text{admitted}} = 0$), $x_{t+1}$ remains strictly bit-for-bit identical to $x_t$.

---

## 2. Hoare Contracts

### Masked State Commit (`commit_selected_action`)

**Mathematical Law:**
The persistent state transition must take the form of an exact bitwise selection mapping over fixed-width structured fields. A rejected operation must leave persistent state bit-for-bit unchanged without resorting to conditional jumps.

**Hoare Contract:**
* **Valid Input Domain:** 
  - `current_state`: The existing `AutonomicState` (fixed capacity).
  - `candidate_state`: The fully populated fixed-width `AutonomicState` modeling the successful action execution.
  - `admitted_mask`: A fully expanded mask ($\in \{0, 0xFFFFFFFF\}$ for 32-bit fields, etc.).
* **Output Range:** Returns the next immutable `AutonomicState` $x_{t+1}$.
* **Conservation Law:** If $m_{\text{admitted}} = 0$, then $\forall k, x_{t+1,k} = x_{t,k}$. The state mass and historical invariants are perfectly conserved upon refusal.
* **Monotonicity Law:** Under successful admission ($m_{\text{admitted}} \neq 0$), the accumulated execution metrics and epoch timestamps must advance monotonically.
* **Overflow Behavior:** Epoch counters and resource accumulations must use strictly saturating arithmetic (`saturating_add`) to prevent wrapping undefined behavior upon sustained high-frequency iterations.
* **Invalid-Input Refusal:** Bound parametrically by $m_{\text{admitted}}$. Rejected inputs implicitly cause the state transition to act as the identity function, emitting a typed refusal code propagated from the previous stage.
* **Determinism:** Execution strictly enforces $CC=1$. Masking is fieldwise and applied using purely scalar bitwise operations (SWAR). The forbidden pattern `if valid { candidate } else { current }` is algorithmically eradicated.
* **State-Mutation Boundary:** Exactly 0 heap allocations. The transition is a pure functional mapping from $(x_t, x_{\text{candidate}}, m_{\text{admitted}}) \rightarrow x_{t+1}$ using stack-allocated structs.
* **Numeric Error Envelope:** The commit function performs exact bit-level preservation and replacement. $E_{abs} = 0$.

---

## 3. Proof Obligations

To satisfy integration integrity before downstream merging:

1. **Topological Object-Code Audit (@turing_machine):**
   Audit the release assembly (`-C opt-level=3`) for `commit_selected_action`. The object code must consist entirely of `and`, `or`, `not`, and memory load/store instructions. Validate the total absence of conditional branch instructions (`jcc`), loop backedges, and dynamic allocator symbols.

2. **Refusal Conservation (@armstrong_fault):**
   Inject a hostile mutant that attempts speculative mutation prior to full admission:
   ```rust
   // MUTANT (Cheat Detected)
   state.mass[i] = candidate.mass[i];
   if !admitted { return Err(...); }
   ```
   The test matrix must demonstrate that such a mutant incorrectly leaks partial state changes, whereas the branchless baseline correctly preserves the entire state uncorrupted.

3. **Exhaustive Mapping Matrix (@hoare_oracle):**
   Produce an independent reference oracle that defines the identical transactional update logic using high-level conditional branches. Assert that for all permutations of $x_t$, $x_{\text{candidate}}$, and binary admission masks, the branchless Q-mapped `commit_selected_action` yields byte-for-byte identical output to the branching oracle.
