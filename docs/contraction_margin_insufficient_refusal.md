# ContractionMarginInsufficient

## Mathematical Meaning

The `ContractionMarginInsufficient` refusal occurs when a proposed stability transition matrix fails to mathematically prove bounded convergence. 

In the deterministic substrate, a proposed transition is evaluated using a state transition matrix $G$, a strictly positive witness vector $d$, and a contraction margin $\delta$ (`margin_delta`). The runtime hot-path verifies static domination element-wise in fixed-point arithmetic:

$$ G d \leq (1 - \delta) d $$

Mathematically, this ensures that for every dimension $i$ in the system's state space, the proposed dynamics $G$ strictly contract the bounding vector $d$ by at least the margin $\delta$. If the arithmetic yields $(G d)_i > (1 - \delta) d_i$ for any dimension, the proposed matrix fails to demonstrate sufficient contraction. The system immediately rejects the transition with the `ContractionMarginInsufficient` typed refusal.

## Protection of Substrate Stability Guarantees

Rejecting a state due to this refusal strictly enforces the substrate's laws and guarantees:

1. **Rule 12 (No Runtime Theorem Discovery):** The authoritative runtime must not search for a stable witness, run power iterations, or calculate spectral radii. The "slow rail" computes and proposes the matrix $G$, witness $d$, and margin $\delta$. The hot path merely performs a deterministic $O(1)$ fixed-point verification of this proof. If it fails, the runtime aggressively rejects it rather than attempting to discover a valid bound.
2. **Rule 10 (No Mutation Before Complete Admission):** Because the static domination verification is performed over fixed-size stack values before any state mutation occurs, failing the contraction margin check triggers an immediate typed refusal. The persistent state remains bit-for-bit unchanged, preventing mathematically unproven dynamics from corrupting the admitted state.
3. **Rule 11 (ReceiptSound Law) and Divergence Prevention:** By rejecting dynamic components that are not rigorously proven to converge (contract), adaptive mutations are halted, and the system falls back to a frozen learning mode. This guarantees the deterministic substrate never diverges, never oscillates beyond bounded margins, and never enters unbounded loops.
