# Auto Select Terminal Convergence: Mathematical Bounds

> **Owner:** `@hoare_oracle`
> **Phase:** Auto Select Implementation Loop (Iteration 41)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Axiomatic Mission & Scope

This specification defines the strict mathematical bounds for the **Auto Select Terminal Convergence Operator** ($f_{converge}$), completing the final stage of the Auto Select pipeline integration. This primitive safely maps the accumulated execution tape mask and intermediate autonomic selections into the persistent substrate state without branching, allocating, or violating the `ReceiptSound` law.

## 2. Hoare Contract

Let $M_{tape}$ be the bitwise execution tape mask, $R_{aggr}$ be the accumulated typed refusal aggregation state, and $x_{persistent}$ be the authoritative control state.

$$
\{ M_{tape} \in [0, 2^{64}-1] \land R_{aggr} \in \text{RefusalAggregationState} \land x_{persistent} \in \text{PersistentControlState} \}
\quad f_{converge}(M_{tape}, R_{aggr}, x_{persistent}) \quad
\{ x_{persistent+1} \in \text{PersistentControlState} \}
$$

## 3. Definitional Bounds & Invariants

To comply with the BCINR Constitution (Rule 4), the primitive strictly enforces the following mathematical properties:

### A. Valid Input Domain
* $M_{tape}$: Bounded 64-bit integer representing the complete execution tape mask.
* $R_{aggr}$: Fixed-size struct representing the zero-allocation refusal trace map.
* $x_{persistent}$: The authoritative 256-byte control state struct representing the current terminal.

### B. Output Range
* $x_{persistent+1}$: A deterministic target state update.
* $\text{Refusal}$: A bounded `u8` typed refusal code mapping to `TerminalConvergenceRefusal`.

### C. Conservation Law
A single terminal convergence execution must conserve the semantic mass of the execution tokens. No tokens may be spontaneously generated or destroyed during binding:
$$ \text{mass}(x_{persistent+1}) = \text{mass}(x_{persistent}) + \operatorname{select}(M_{admit}, \text{mass}(M_{tape}), 0) $$

### D. Monotonicity Law
The epoch clock strictly increases, ensuring forward temporal progression upon a successful terminal binding:
$$ x_{persistent+1}.\text{epoch\_clock} \ge x_{persistent}.\text{epoch\_clock} $$

### E. Overflow Behavior
* Terminal semantic mass accumulations saturate at `u64::MAX`.
* All offset calculations for artifact substrate binding wrap via `wrapping_add` but are logically constrained by the arena index capacity (masked to $< 2^{16}$).

### F. Invalid-Input Refusal
Any invalid domain immediately projects into a failure mask ($M_{admit} = 0$), emitting a typed refusal while ensuring state persistence:
* Empty tape mask $\rightarrow$ `NoLeaves` or `ControlStateUnadmitted`
* Unresolved critical refusal in $R_{aggr}$ $\rightarrow$ `BranchlessContractFailed`
* Epoch misalignment $\rightarrow$ `ContractViolation`

### G. Determinism
* $CC=1$ across the entire transitive call graph.
* **Radon Law:** The instruction shape and execution timing remain constant regardless of the bits active in $M_{tape}$ or the specific refusal path encoded in $R_{aggr}$.

### H. State-Mutation Boundary
Adheres to Rule 10 (No mutation before complete admission):
$$ M_{admit} = V_{tape} \land V_{refusal} \land V_{epoch} $$
$$ x_{persistent+1} = \operatorname{select}(M_{admit}, x_{candidate}, x_{persistent}) $$
State remains mathematically and bit-for-bit unchanged upon typed refusal.

### I. Numeric Error Envelope
* Terminal mass distribution ratios use strictly bounded fixed-point integer mathematics ($Q16.16$).
* The maximum absolute error ($E$) in mass projection is bounded to $\le 2^{-16}$ (1 bit of fractional precision).
* No floating-point instructions or variable precision floats exist in the mapping.

## 4. Proof Obligations for Next Steps

* **`@turing_machine`**: Must verify that $f_{converge}$ produces object code with 0 loop backedges, 0 conditional jumps, and completely zero allocations.
* **`@armstrong_fault`**: Must introduce hostile mutants targeting the terminal $M_{admit}$ mask selection and verify they produce typed refusals instead of silent drift or persistent state corruption.
* **`@von_neumann_bypass`**: Must implement the branchless $f_{converge}$ logic integrating the terminal tape metrics into the persistent state selection mechanism.
