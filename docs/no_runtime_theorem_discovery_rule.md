# Rule 12: No Runtime Theorem Discovery

In the BCINR substrate, Rule 12 ("No runtime theorem discovery") enforces a strict separation between discovering mathematical invariants (which is complex and computationally unbounded) and verifying them (which is deterministic and bounded).

## Why Dynamic Mathematical Searches are Banned at Runtime

Dynamic mathematical searches—such as power iteration, spectral-radius estimation, Jacobian derivation, Lyapunov search, and optimization over weighting vectors—are strictly prohibited in the authoritative runtime (the hot path) for several fundamental reasons:

1. **Unbounded Execution Work**: Iterative numerical algorithms inherently rely on convergence criteria. This means the number of loop iterations depends on the input data, violating the mandate for **fixed bounded execution work** and the explicit ban on **data-dependent loop termination** (Rule 3).
2. **Branching and Control Flow**: Algorithms like adaptive threshold discovery and dynamic graph analysis require conditional branches to make decisions based on intermediate state. This directly violates the absolute **$CC=1$ requirement** (the Radon Law) and the strict prohibition on control flow operations (Rule 8).
3. **Resource and Data Constraints**: Theorem discovery frequently demands dynamic memory allocation, unbounded intermediate state, and floating-point arithmetic. The authoritative runtime is explicitly `#![no_std]`, zero-allocation, and forbidden from using floating-point operations.

In short, the authoritative hot path must remain a purely deterministic, branchless instruction pipeline. It cannot be tasked with *finding* a mathematical solution, because doing so would cause the executed instruction shape to depend on the semantic input. 

## Verifying Pre-Computed Witnesses via the Slow Rail

To maintain strict mathematical guarantees (such as system stability) without executing unbounded algorithms in the hot path, BCINR utilizes a **Witness-Verifier** architecture divided between the non-authoritative "slow rail" and the authoritative runtime.

### 1. The Slow Rail (Discovery)
The slow rail is completely isolated from the hot path. It is permitted to branch, allocate heap memory, and execute unbounded, iterative computations (such as eigenvalue searches, symbolic mathematics, and SHACL validation). 
The slow rail derives the mathematical proof of stability and extracts bounding parameters, such as:
$$G,\ d,\ \delta,\ R_{\mathrm{noise}},\ R_{\mathrm{switch}}$$
Once the algorithm converges, the slow rail packages these parameters into a fixed-width "witness."

### 2. The Hot Path (Verification)
The authoritative runtime receives this pre-computed witness and merely verifies it. Verification is structurally lawful because it only requires constant-time, fixed-width arithmetic that compiles directly to branchless machine code.

For example, to guarantee stability, the hot path does not attempt to compute the spectral radius of the state transition graph. Instead, it evaluates the fixed witness using packed values to check two static domination conditions:
1. $\widehat G \leq G_{\mathrm{certified}}$ (verifying the current dynamic system $\widehat G$ is bounded by the certified matrix)
2. $G_{\mathrm{certified}}d \leq (1-\delta)d$ (verifying the contraction mapping over the vector $d$ by a margin of $\delta$)

By comparing these packed values using fieldwise bitwise masks (as mandated by Rule 9), the hot path proves the invariant in $O(1)$ constant time without any loop backedges, conditional jumps, or iterative refinement. If the mathematical verification fails, the hot path deterministically yields a typed refusal (e.g., `ContractionMarginInsufficient`) and leaves persistent state bit-for-bit unchanged.
