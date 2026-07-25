Based on the `AGENTS.md` and `GEMINI.md` rules of the BCINR Deterministic Substrate, here is an explanation of **Static Domination**, **Fixed Witness** verification, and the mathematics in Rule 12.

### The Problem: Theorem Discovery is Unbounded
In system stability, determining properties like convergence, contraction, or stability usually requires iterative algorithms (e.g., eigenvalue search, spectral-radius estimation, power iteration, or Lyapunov searches). These algorithms depend on data to determine when to terminate (e.g., `while error > epsilon`), which fundamentally requires branching and variable loop bounds.

Under BCINR's core constitutional laws, the authoritative runtime (the "hot path") must be strictly branchless ($CC=1$) and execute in bounded, deterministic time. Therefore, the runtime is strictly prohibited from running any iterative algorithms to *discover* stability properties. 

### The Solution: Static Domination and Fixed Witnesses
Instead of discovering stability, the runtime merely *verifies* a mathematical certificate (the "Fixed Witness") that was provided to it. 

The heavy lifting of discovering the witness—calculating the exact transition bounds, contraction margins ($\delta$), and weighting vectors ($d$)—is delegated to the **"slow rail"**. The slow rail is allowed to branch, allocate, and run unbounded loops to derive these values out-of-band.

Once derived, the slow rail passes these properties to the hot path as packed values. 

### The Formula: $\widehat G \leq G_{\mathrm{certified}}$
When the authoritative runtime receives a candidate state or transition matrix, it performs a **Static Domination** check using the formula:

$$ \widehat G \leq G_{\mathrm{certified}} $$

- **$G_{\mathrm{certified}}$**: A pre-calculated bound that has already been mathematically proven to be stable.
- **$\widehat G$**: The current dynamic state or candidate matrix in the runtime.

By enforcing that $\widehat G$ is dominated by (less than or equal to) $G_{\mathrm{certified}}$, the runtime guarantees that $\widehat G$ is also stable, without having to calculate $\widehat G$'s actual eigenvalues.

This stability is anchored by the second formula:
$$ G_{\mathrm{certified}}d \leq (1-\delta)d $$
This is a contraction mapping proof. It proves that multiplying by $G_{\mathrm{certified}}$ shrinks the state by at least a margin of $\delta$, using the fixed weighting vector $d$ (the fixed witness).

### Why the Hot Path Only Compares Packed Values
By restricting the hot path to only comparing packed values:
1. **$O(1)$ Execution:** Verifying the inequality $\widehat G \leq G_{\mathrm{certified}}$ is reduced to parallel, constant-time arithmetic (like SIMD bounds checking).
2. **Branchless Compliance ($CC=1$):** There is no need for `while` loops or conditional `if` statements to check for algorithm convergence. The system simply masks out invalid states using bitwise selection if the inequality fails.
3. **Deterministic Mechanics:** Every execution of the hot path takes the exact same number of CPU cycles, preserving the system's absolute immunity to timing side-channels.
