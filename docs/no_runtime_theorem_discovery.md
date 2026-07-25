# No Runtime Theorem Discovery (Rule 12)

In the `bcinr` deterministic computational substrate, Rule 12 strictly dictates that **the authoritative runtime may verify a supplied witness, but it may never discover one.**

## Prohibited Runtime Operations

To maintain bounded, branchless, and allocation-free execution, `bcinr` strictly prohibits complex, unbounded theorem discovery operations on the authoritative hot path. These include:

*   Spectral-radius estimation
*   Power iteration
*   Jacobian derivation
*   Optimization over weighting vectors
*   Lyapunov search
*   Adaptive threshold discovery
*   Automatic q-range expansion
*   Dynamic graph analysis

### Why are they prohibited?

The fundamental mission of `bcinr` is to preserve a purely deterministic execution model: `admitted input → fixed instruction shape → deterministic output`. 

Dynamic graph traversals, algorithm searches, and iterative discoveries inherently require data-dependent loops, conditional branching, variable execution work, and unpredictable instruction paths. This fundamentally violates the absolute runtime laws of the framework (such as `CC=1`, no data-dependent loop termination, and zero allocations). If runtime theorem discovery were allowed, the execution could no longer guarantee constant-time, bounded operation, breaking the core premise of the deterministic substrate.

## Enforcing Stability through Static Domination

To resolve the need for complex theorem properties without violating runtime laws, `bcinr` physically separates **discovery** from **verification**.

### The Slow Rail (Discovery)

The slow rail operates outside the authoritative runtime. It is permitted to branch, allocate memory, and execute unbounded logic. The slow rail is responsible for doing the heavy lifting of deriving the actual proofs and stability parameters, which include:

$$G,\ d,\ \delta,\ R_{\mathrm{noise}},\ R_{\mathrm{switch}}$$

### The Authoritative Hot Path (Verification)

The hot path must never discover a witness; it may only verify a fixed witness provided by the slow rail. It does this by performing constant-time, packed-value comparisons. 

For stability, the runtime verifies that the observed state $\widehat{G}$ is statically dominated by the provided certified graph $G_{\mathrm{certified}}$, and that the certified graph meets the contraction requirement using the witness $d$:

1.  $$\widehat G \leq G_{\mathrm{certified}}$$
2.  $$G_{\mathrm{certified}}d \leq (1-\delta)d$$

By verifying these properties via simple packed value comparisons rather than complex iterative search algorithms, the hot path successfully enforces mathematical stability bounds while remaining fully branchless, deterministic, and bounded.
