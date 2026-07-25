Here are the findings regarding the `ContractionMarginInsufficient` typed refusal from the `AGENTS.md` and related documentation:

### What it is
The `ContractionMarginInsufficient` refusal is triggered when a proposed stability transition matrix fails to mathematically prove bounded convergence in the deterministic substrate. 

### How it relates to numerical bounds and stability
1. **Mathematical Bounding (Static Domination):** 
   The runtime hot-path performs an element-wise verification in fixed-point arithmetic using the inequality:
   $G d \leq (1 - \delta) d$
   Where $G$ is the state transition matrix, $d$ is a strictly positive witness vector, and $\delta$ is the contraction margin. If the arithmetic determines that $(G d)_i > (1 - \delta) d_i$ for any dimension $i$, it means the proposed matrix fails to contract the bounding vector by the required margin. The transition is immediately rejected with `ContractionMarginInsufficient`.

2. **No Runtime Theorem Discovery (Rule 12):**
   To maintain stability, the authoritative runtime (hot path) never searches for a valid stability witness, spectral radius, or bounds. The bounds ($G$, $d$, $\delta$) are calculated on the "slow rail." The hot path only performs a deterministic $O(1)$ verification of this proof. If the bounds are insufficient, it outright rejects them rather than attempting to discover valid parameters.

3. **Prevention of Divergence (Rule 11):**
   By mathematically rejecting dynamics that are not rigorously proven to converge, adaptive mutations are halted and the system falls back to a frozen learning mode. This strict numeric boundary ensures the substrate never diverges, never oscillates past its bounds, and never enters unbounded loops.

4. **Immutable State Protection (Rule 10):**
   Because this numerical bound verification occurs over fixed-size stack values *before* any state mutation is allowed, failing the check guarantees the persistent state remains bit-for-bit unchanged, keeping mathematically unproven states from corrupting the system.
