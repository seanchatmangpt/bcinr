# ContractionMarginInsufficient

### Definition
`ContractionMarginInsufficient` is a **bounded typed refusal** code used in the BCINR Deterministic Substrate. It indicates that a proposed stability transition matrix fails to mathematically prove bounded convergence. In the context of the branchless runtime (Rule 18 in `AGENTS.md`), invalid states cannot panic or silently fallback; they must deterministically yield a typed refusal. 

This refusal maps to both the `StabilityDerivationRefusal` enum (for witness derivation) and the `StabilityRefusal` enum (legacy-compatible runtime control loop errors).

### Mathematical Condition
Mathematically, the system requires that for a given gain/transition matrix ($G$), a positive bounding/witness vector ($d$), and a required contraction margin ($\delta$), the matrix must strictly contract the bounding vector by at least the margin.

The required condition is:
$$ \widehat{G} d \le (1 - \delta)d $$

If the arithmetic calculation yields the inverse:
$$ (G d)_i > (1 - \delta) d_i $$
for **any** dimension $i$, it means the proposed matrix fails to contract the vector by the required margin. The system immediately rejects the state transition by emitting the `ContractionMarginInsufficient` typed refusal.

### Branchless ($CC=1$) Execution
To comply with the strict branchless ($CC=1$) rules on the authoritative hot path, this condition is calculated and enforced without any conditional jumps (`if`, short-circuiting, loop backedges):
1. **Static Unrolling:** The vector and matrix multiplications are unrolled using static macros (e.g., `unroll_5_static!`) preventing iteration bounds-checking and branches.
2. **Fixed-Width Arithmetic:** The calculation is evaluated exactly in 128-bit integer arithmetic on scaled fixed-point values to avoid overflow or floating-point non-determinism.
3. **Boolean Accumulation & Masking:** The condition results in a boolean flag `gd_ok`. Instead of branching, the inverse `!gd_ok` is bitwise OR'd with other anomaly flags to form an error mask. This mask is arithmetically applied (via a `masked` method) to union the `PROPOSAL_REJECTED` bit into a `RefusalSet`, which maps internally to `StabilityRefusal::ContractionMarginInsufficient`.
