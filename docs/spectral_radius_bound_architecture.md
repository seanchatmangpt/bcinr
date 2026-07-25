# Rule 12: No Runtime Theorem Discovery & The Spectral Radius Bound Architecture

Based on the BCINR Deterministic Substrate Constitution (`AGENTS.md`) and core architecture docs, here is an overview of Rule 12, why spectral-radius estimation is prohibited, and how the static domination check enforces stability.

## Why Spectral-Radius Estimation is Rejected
Rule 12 strictly forbids the "authoritative runtime" (the hot path) from discovering mathematical theorems or stability bounds dynamically. 

To determine if a system is stable (contractive), one would typically evaluate the **spectral radius** ($\rho$) of the state transition Jacobian/gain matrix to ensure $\rho(G) < 1$. However:
- Computing the spectral radius requires unbounded, iterative mathematical algorithms like **power iteration**, eigenvalue search, or Lyapunov search.
- Iterative searches rely on data-dependent loop termination conditions (e.g., `while error > epsilon`), which fundamentally require conditional branching and variable loop bounds.
- This directly violates the substrate's **Radon Law ($CC=1$)**, absolute bounded execution work, and zero-allocation constraints.

Because the authoritative code must execute branchlessly and in $O(1)$ bounded time, dynamic spectral-radius estimation is entirely banned.

## The Spectral Radius Bound Architecture
To preserve closed-loop stability without violating the branchless execution mandate, BCINR splits theorem *discovery* from theorem *verification* across two distinct domains:

1. **The "Slow Rail" (Theorem Discovery)**
   The non-authoritative slow rail is permitted to branch, allocate, and run iterative loops out-of-band. It uses power iteration/eigenvalue decomposition to calculate the spectral radius of the gain matrix ($G_a$). Once it mathematically proves that $\rho(G_a) < 1$, the slow rail searches for a strictly positive weighting vector $d$ (the fixed witness) and a contraction margin $\delta$. These parameters are serialized into a fixed static mathematical certificate.
   
2. **The "Hot Path" (Theorem Verification)**
   The authoritative branchless hot path is explicitly forbidden from generating or finding the stability parameters. Instead, it only receives the fixed-point parameters ($G_{\mathrm{certified}}$, $d$, $\delta$) derived by the slow rail and verifies the proof deterministically. 

## How the Static Domination Check Works Instead
Instead of dynamically computing the spectral radius or calculating eigenvalues on the fly, the hot path simply evaluates the fixed witness by performing an $O(1)$ **Static Domination** check using packed values. 

When the runtime evaluates a candidate state or transition matrix ($\widehat G$), it enforces static bounds by branchlessly validating two structural constraints:

1. **Matrix Domination:** 
   $$ \widehat G \leq G_{\mathrm{certified}} $$
   The current dynamic matrix $\widehat G$ must be numerically dominated by (less than or equal to) the pre-certified gain matrix. This guarantees that $\widehat G$ is also safely contractive without ever having to discover its actual eigenvalues.

2. **Contraction Verification:** 
   $$ G_{\mathrm{certified}}d \leq (1-\delta)d $$
   The system verifies the contraction mapping proof: it proves that applying the certified matrix $G_{\mathrm{certified}}$ to the fixed witness vector $d$ shrinks the vector by at least the margin $\delta$.

**Branchless Execution:**
The hot path evaluates these inequalities strictly through $O(1)$ parallel fixed-point arithmetic (similar to SIMD bounds checking). Instead of `if` conditions to check convergence or handle rejections, bitwise masking is used. If the candidate state fails the static domination check, the branchless selection mask zeros it out and falls back to the previous persistent state while immediately returning a typed refusal (`StabilityRefusal::ContractionMarginInsufficient`). This ensures the exact same instruction shape and cycle count are preserved unconditionally.
