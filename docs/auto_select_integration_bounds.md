# Auto Select Integration Pipeline: Mathematical Bounds

> **Owner:** `@hoare_oracle`
> **Phase:** Auto Select Implementation Loop (Iteration 23)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Axiomatic Mission & Scope

This specification defines the strict mathematical bounds for the **Auto Select Pipeline Integration Operator** ($f_{integrate}$), bridging the gap between independent branchless components (`semantic_projection`, `canonical_mass`, `auto_select`, and `auto_select_bridge`) into a single, transactional, bounded execution primitive.

## 2. Hoare Contract

Let $C_{req}$ be the semantic constraint matrix, $\mathbf{C}_{tool}$ be the fixed-width capabilities of 8 candidates, and $x_t$ be the persistent POWL execution tape state.

$$
\{ C_{req} \in \text{SemanticConstraintMatrix} \land \mathbf{C}_{tool} \in \text{ToolCapabilityMatrix}^8 \land x_t \in \text{PowlRunState} \}
\quad f_{integrate}(C_{req}, \mathbf{C}_{tool}, x_t) \quad
\{ x_{t+1} \in \text{PowlRunState} \}
$$

## 3. Definitional Bounds & Invariants

To comply with the BCINR Constitution (Rule 4), the primitive strictly enforces the following mathematical properties:

### A. Valid Input Domain
* $C_{req}$: Bounded 32-bit matrices representing `required_mask` and `authoritative_mask`.
* $\mathbf{C}_{tool}$: Fixed size $N = 8$ array of `ToolCapabilityMatrix`.
* $x_t$: The persistent tape state, modeled as a 64-bit `active_mask`.

### B. Output Range
* $x_{t+1}$: A deterministic target state update.
* $\text{Refusal}$: A bounded `u8` typed refusal code mapping to `AutoSelectRefusal`.

### C. Conservation Law
A single pipeline execution emits at most one active execution token:
$$ \text{popcnt}(x_{t+1}.\text{active\_mask}) \le \text{popcnt}(x_t.\text{active\_mask}) + 1 $$

### D. Monotonicity Law
Within the boundary of a single transactional selection, the execution tape strictly accumulates tokens:
$$ x_{t+1}.\text{active\_mask} \ge x_t.\text{active\_mask} $$ *(Bitwise Domination)*

### E. Overflow Behavior
* Intermediate canonical mass values saturate at `u32::MAX` to prevent arithmetic folding.
* Shift operations for tape token selection ($1 \ll S_{out}$) are bitwise bounded via $S_{out} \land 63$, preventing architectural panics.

### F. Invalid-Input Refusal
Any invalid domain immediately projects into a failure mask ($M_{admit} = 0$), emitting a typed refusal mapping while ensuring state persistence:
* Semantics unmet $\rightarrow$ `UnsupportedDomain` or `SupportMismatch`
* Authority unmet $\rightarrow$ `ControlStateUnadmitted`
* Mass < `q_lens` $\rightarrow$ `ContractionMarginInsufficient`

### G. Determinism
* $CC=1$ across the entire transitive call graph.
* **Radon Law:** The instruction shape and timing remain constant regardless of the distribution of valid vs. invalid capabilities in $\mathbf{C}_{tool}$.

### H. State-Mutation Boundary
Adheres to Rule 10 (No mutation before complete admission):
$$ M_{admit} = V_{semantic} \land V_{authority} \land V_{mass\_margin} $$
$$ x_{t+1} = \operatorname{select}(M_{admit}, x_{candidate}, x_t) $$
State remains mathematically and bit-for-bit unchanged upon typed refusal.

### I. Numeric Error Envelope
* The semantic-to-measure projection relies strictly on fixed-width token integer mappings: $s_i \in \{0, 192, 220, 224, 255\}$.
* The absolute and relative error bounds for fixed-point token selection are mathematically zero ($E = 0$). All fractional components are resolved through exact geometric lookup tables or strict bounded shifts. No floating-point instructions or $NaN$/Infinity values exist in the pipeline.

## 4. Proof Obligations for Next Steps

* **`@turing_machine`**: Must verify that $f_{integrate}$ produces object code with 0 loop backedges and 0 conditional jumps.
* **`@armstrong_fault`**: Must introduce hostile mutants targeting the $M_{admit}$ transactional mask and verify they produce typed refusals instead of partial state drifts.
* **`@von_neumann_bypass`**: Must implement the $f_{integrate}$ pipeline composing the existing `project_semantic_coordinate` and `powl_bridge_select` primitives.
