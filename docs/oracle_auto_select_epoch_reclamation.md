# `@hoare_oracle` — Auto Select Epoch-Based Reclamation Integration

**Jurisdiction:** `mfw-auto-select` token processing pipeline, `BumpArena`, and `LockFreeSlab` integration

## 1. Mission

This document defines the strict mathematical bounds, Hoare contracts, and proof obligations for integrating branchless, epoch-based memory reclamation (EBR) into the `mfw-auto-select` token processing pipeline. In accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the implementation must guarantee $CC=1$, zero heap allocations, and deterministic execution time.

## 2. Hoare Contract

Let $B$ be the fixed, maximum capacity of blocks in the `LockFreeSlab`.
Let $N$ be the fixed number of concurrent pipeline stages (participants).
Let $E_g \in [0, 2^{64}-1]$ be the global epoch.
Let $\vec{E}_{local} \in [0, 2^{64}-1]^N$ be the vector of participant epochs.
Let $\vec{E}_{retire} \in [0, 2^{64}-1]^B$ be the vector of retirement epochs for blocks in the slab.

**Contract:**
$$
\{ \forall i \in [0, N-1], E_{local}[i] \ominus E_g < 2^{63} \}
\quad
\operatorname{reclaim\_epoch\_blocks}(E_g, \vec{E}_{local}, \vec{E}_{retire})
\quad
\{ \vec{R} \in \{0, 1\}^B \}
$$

### Contract Invariants

*   **Valid Input Domain:** $\vec{E}_{local}$ and $E_g$ provided by the runtime. Circular distance $E_{local}[i] \ominus E_g \le 0$ must hold, meaning no local epoch can exceed the global epoch.
*   **Output Range:** A fixed-width bitmask or unrolled mask array $\vec{R}$ of size $B$, where $R_j = 1$ indicates block $j$ is reclaimed.
*   **Conservation Law:** The sum of active blocks, retired blocks, and free blocks is exactly equal to $B$ at all times:
    $$ \Sigma(\text{active}) + \Sigma(\text{retired}) + \Sigma(\text{free}) = B $$
*   **Monotonicity Law:** The global epoch $E_g$ is monotonically non-decreasing over time, up to modular wrapping.
*   **Overflow Behavior:** All epoch comparisons must use wrapping subtraction modulo $2^{64}$ (i.e., `a.wrapping_sub(b) < (1 << 63)`) to transparently handle integer overflow without branches.
*   **Invalid-Input Refusal:** If a participant reports an epoch strictly greater than $E_g$ (accounting for wrapping), the function returns a `ControlStateUnadmitted` or `EpochDesync` typed refusal using fixed structural mapping.
*   **Determinism:** The reclamation mask $\vec{R}$ is a pure mathematical projection of the inputs. There are no data-dependent loops over $B$ or $N$.
*   **State-Mutation Boundary:** The global free-list or allocation mask is mutated using a bitwise $OR$ or equivalent bitwise SWAR operation. Masked selection is used exclusively.
*   **Numeric Error Envelope:** $0$ error. Exact modular arithmetic applies.

## 3. Mathematical Bounds & Selection

### The Minimum Safe Epoch

The pipeline must compute the minimum observed epoch across all participants without branches. For two epochs $a$ and $b$:
$$
\operatorname{min\_epoch}(a, b) = \operatorname{select}(a \ominus b < 2^{63}, a, b)
$$
The global safe epoch $E_{safe}$ is computed via a fixed-size tree reduction of $\vec{E}_{local}$:
$$
E_{safe} = \min_{i=0}^{N-1} E_{local}[i]
$$

### The Reclamation Mask

For each block $j \in [0, B-1]$, it is safe to reclaim if it was retired strictly before $E_{safe}$:
$$
M_{reclaim}[j] = (E_{retire}[j] \ominus E_{safe} < 2^{63}) \land (E_{retire}[j] \neq E_{active\_sentinel})
$$
The above is evaluated as a bitwise parallel mask evaluation (e.g., SIMD or SWAR).

## 4. Proof Obligations

To ensure the epoch-based reclamation adheres to the Radom Law ($CC=1$), the following proofs must be supplied:

1.  **Object-Code Verification (`@turing_machine`)**: The reduction over $N$ and map over $B$ must compile to straight-line assembly with zero loop backedges or conditional jumps. $N$ and $B$ must be bounded via `const` generics, enforcing compile-time unrolling.
2.  **Hostile Mutants (`@armstrong_fault`)**:
    *   Mutate the epoch distance check `a.wrapping_sub(b) < 1<<63` to `< 1<<62`. The oracle must reject.
    *   Inject an invalid participant epoch $E_{local}[k] > E_g$. The implementation must yield a typed refusal rather than panicking or corrupting $E_{safe}$.
3.  **No Allocation Guarantee**: The memory reclamation must exclusively operate over indices of the `LockFreeSlab`. Reclaimed indices are mapped to the availability bitmap via `availability_mask |= M_reclaim`. No `alloc` or `free` calls may be linked.
4.  **Independent Oracle (`@hoare_oracle`)**: A reference implementation utilizing unconstrained arbitrary-width integers (e.g., Python or purely functional Rust test double) must demonstrate the exact same set of reclaimed indices for all fuzz permutations of $E_{retire}$ and $E_{local}$.
