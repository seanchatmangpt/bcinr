Here are the details from `AGENTS.md` regarding "No runtime theorem discovery" under Rule 12:

Under **Rule 12: No runtime theorem discovery** in `AGENTS.md`, the core distinction lies in where computational effort is expended and the deterministic nature of the execution.

### Verifying a Witness (Allowed)
The "authoritative runtime" (the hot path) is strictly limited to **verifying** a provided, fixed witness or static domination. It must perform operations with guaranteed execution bounds and no active searching. 
- It achieves this by comparing packed values to check that specific mathematical constraints are met, such as verifying $\widehat G\leq G_{\mathrm{certified}}$ and $G_{\mathrm{certified}}d \leq (1-\delta)d$.
- Since verifying a proof is mathematically direct, deterministic, and avoids adaptive algorithms, it adheres to the branchless and bounded-execution requirements of the substrate.

### Discovering a Witness (Prohibited)
The authoritative runtime is strictly barred from **discovering** a witness or deducing complex system parameters dynamically. 
- Generating or discovering these proofs is a branching, adaptive, and often unbounded process.
- All discovery and derivation of these parameters (such as $G, d, \delta, R_{\mathrm{noise}}, R_{\mathrm{switch}}$) must be entirely relegated to the non-authoritative **"slow rail"**, which is isolated from the authoritative branchless hot path. 

### Specifically Prohibited at Runtime
The following active discovery, estimation, and dynamic derivation operations are explicitly forbidden in the authoritative runtime:
* Spectral-radius estimation
* Power iteration
* Jacobian derivation
* Optimization over weighting vectors
* Lyapunov search
* Adaptive threshold discovery
* Automatic q-range expansion
* Dynamic graph analysis
