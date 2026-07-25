# Auto Select Epoch Reclamation Operator: Mathematical Bounds

> **Owner:** `@hoare_oracle`
> **Phase:** Auto Select Implementation Loop (Iteration 29)
> **Jurisdiction:** BCINR Deterministic Substrate

## 1. Axiomatic Mission & Scope

This specification defines the strict mathematical bounds for the **Auto Select Epoch Reclamation Operator** ($f_{reclaim}$). As the logical step following Substrate Convergence (Iteration 28), this operator safely reclaims memory back into the global `LockFreeSlab` or `BumpArena` using epoch-based synchronization. It asserts that all memory management is completely allocation-free (Rule 3) and strictly branchless ($CC=1$), ensuring deterministic reclamation without stopping the world.

## 2. Hoare Contract

Let $E_g \in [0, 2^{64}-1]$ be the global epoch, $\vec{E}_{local} \in [0, 2^{64}-1]^N$ be the vector of active participant epochs for $N$ concurrent stages, and $\vec{E}_{retire} \in [0, 2^{64}-1]^B$ be the vector of retirement epochs for $B$ blocks in the slab.

$$
\{ \forall i \in [0, N-1], E_{local}[i] \ominus E_g < 2^{63} \}
\quad
f_{reclaim}(E_g, \vec{E}_{local}, \vec{E}_{retire})
\quad
\{ \vec{R} \in \{0, 1\}^B \}
$$

## 3. Definitional Bounds & Invariants

To comply with the BCINR Constitution (Rule 4, Rule 13), the primitive strictly enforces the following properties:

### A. Valid Input Domain
* $E_g$: The global epoch provided by the `AutonomicSubstrate`.
* $\vec{E}_{local}$: Fixed-size vector of local epochs for pipeline participants. Circular distance bounds must hold.
* $\vec{E}_{retire}$: Fixed-size vector of block retirement epochs.

### B. Output Range
* $\vec{R}$: A fixed-width bitmask or array of size $B$, where $R_j = 1$ strictly indicates that block $j$ is authorized for reclamation.

### C. Conservation Law
The total number of active, retired, and free blocks is exactly equal to $B$.
$$ \Sigma(\text{active}) + \Sigma(\text{retired}) + \Sigma(\text{free}) = B $$

### D. Monotonicity Law
The global epoch $E_g$ is monotonically non-decreasing over time, up to modular wrapping. The global safe epoch $E_{safe} = \min_{i=0}^{N-1} E_{local}[i]$ must be monotonically computed without conditional branches.

### E. Overflow Behavior
All epoch comparisons utilize wrapping subtraction modulo $2^{64}$. The evaluation $a \ominus b < 2^{63}$ maps cleanly into integer overflow semantics, transparently handling epoch wrapping safely and branchlessly.

### F. Invalid-Input Refusal
Any participant epoch exceeding the global epoch $E_g$ forces an immediate failure projection mask, halting reclamation and leaving the availability mask bit-for-bit unchanged. This functionally serves as an `EpochDesync` or `ControlStateUnadmitted` refusal.

### G. Determinism
* $CC=1$ across the entire transitive call graph.
* **Radon Law:** The instruction shape and cycle latency remain perfectly constant. Reductions over $N$ and mapping over $B$ are structurally unrolled loops with zero backedges.

### H. State-Mutation Boundary
Adheres to Rule 10. The global free-list or allocation mask is mutated using a bitwise `OR` combined with the derived $\vec{R}$ mask. No block is freed speculatively.

### I. Numeric Error Envelope
Absolute zero error. All logic maps exactly to constant-time boolean arrays and modular arithmetic.

## 4. Proof Obligations for Next Steps

* **`@turing_machine`**: Must verify that $f_{reclaim}$ produces release object code with 0 loop backedges and 0 conditional jumps (`jxx`). The reduction of the safe epoch and the broadcast of the reclamation mask must be structurally unrolled via `const` generics.
* **`@armstrong_fault`**: Must introduce hostile mutants targeting the epoch distance calculation (e.g., `< 1<<62` instead of `< 1<<63`) and assert failure against an arbitrary-width integer independent oracle.
* **`@von_neumann_bypass`**: Must implement the branchless $E_{safe}$ reduction kernel (`select_u64` minimums) and the parallel SIMD/SWAR mask evaluation mapping retired blocks back to the free list.
