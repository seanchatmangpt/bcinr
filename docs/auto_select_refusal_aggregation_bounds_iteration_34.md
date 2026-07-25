# Auto Select Refusal Aggregation Operator: Mathematical Bounds

> **Owner:** `@hoare_oracle`
> **Phase:** Auto Select Implementation Loop (Iteration 34)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Axiomatic Mission & Scope

This specification defines the strict mathematical bounds for the **Auto Select Refusal Aggregation Operator** ($f_{refuse}$). As the logical step following Trace Integration (Iteration 33), this operator guarantees that the refusal codes from all independent stages of the Full MAPE-K Autonomic Loop (Adaptive Mutation, Proposal, Dispatch, Convergence, Receipt Integration, OCEL Emission, and Epoch Reclamation) are atomically and symmetrically reduced into a single `FullMapekRefusal` code. It strictly enforces the $CC=1$ Radon Law, preventing early returns or priority-based branching.

## 2. Hoare Contract

Let $R = \{r_{base}, r_{adapt}, r_{dispatch}, r_{conv}, r_{receipt}, r_{ocel}, r_{epoch}\}$ be the set of intermediate refusal codes.
Let $m_{update} \in \{0, 1\}$ be the boolean transaction admission mask, where $m_{update\_mask} = 0 - m_{update}$.

$$
\{ \forall r \in R, r \in \operatorname{TypedRefusal} \land m_{update} \in \{0, 1\} \}
\quad
f_{refuse}(R, m_{update})
\quad
\{ r_{final} = r_{base} \lor r_{adapt} \lor r_{epoch} \lor r_{conv} \lor (m_{update\_mask} \land (r_{dispatch} \lor r_{receipt} \lor r_{ocel})) \}
$$

## 3. Definitional Bounds & Invariants

To comply with the BCINR Constitution, the primitive strictly enforces the following mathematical properties:

### A. Valid Input Domain
* $R$: A set of bounded 8-bit unsigned integers representing localized typed refusal codes from each stage of the MAPE-K loop.
* $m_{update}$: The canonical boolean bitmask ($0$ or $1$) authorizing the transactional commit, derived from the `PolicyGuard`.

### B. Output Range
* $r_{final}$: A bounded 8-bit unsigned integer representing the aggregated refusal code. If $0$, the loop succeeded without refusal.

### C. Conservation Law
* **Zero-Allocation**: No memory is allocated. Operations are purely bitwise reductions in registers.
* **Instruction Work**: Execution complexity is strictly constant $\mathcal{O}(1)$. All inputs are unconditionally evaluated and reduced.

### D. Monotonicity Law
* The aggregated refusal code monotonically accumulates set bits from the intermediate refusals. If any stage produces a non-zero refusal, $r_{final}$ is strictly non-zero.

### E. Overflow Behavior
* **Wrapping Guarantee**: Not applicable, as reduction operates via bitwise OR (`|`) which naturally saturates without overflow.

### F. Invalid-Input Refusal
* The operator itself does not refuse input; it is the terminal aggregator of refusals.
* Downstream refusals (dispatch, receipt, ocel) are explicitly masked by $m_{update}$ so that if a proposal is rejected, subsequent stages do not contribute phantom refusals.

### G. Determinism (Radon Law $CC=1$)
* The control flow graph of $f_{refuse}$ must contain zero conditional branches.
* Refusals are accumulated using bitwise OR (`|`), avoiding `if $r_x \neq 0$ { return $r_x$; }`.
* Masking of downstream refusals uses bitwise AND (`&`) with the broadcasted $m_{update}$ mask.

### H. State-Mutation Boundary
Adheres to Rule 10 (No mutation before complete admission):
* Refusal aggregation determines the final `refusal_code` output but does not mutate internal substrate state.
* If $r_{final} \neq 0$, the previously applied state masking guarantees that the global state remains mathematically unmodified.

### I. Numeric Error Envelope
* **Absolute Error**: $0$. Operations are purely bitwise.
* **Relative Error**: $0$. No arithmetic scaling occurs.

## 4. Proof Obligations for Next Steps

* **`@turing_machine`**: Must formally verify that the generated assembly for the refusal aggregation block within `execute_full_mapek_loop` contains exactly zero loop backedges and zero conditional jumps (`jxx`).
* **`@armstrong_fault`**: Must introduce hostile mutants that attempt to return early upon encountering a refusal (e.g. `if r != 0 return r`), verifying that such deviations fail the $CC=1$ check and cause timing variations.
* **`@von_neumann_bypass`**: Must ensure the bitwise reduction is fully folded in the object code, avoiding sequential dependency stalls where possible.
