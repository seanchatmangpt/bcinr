Here is the documentation on how the hot path verifies static domination and eigenvalue lower bounds branchlessly, in accordance with Rule 12 (No runtime theorem discovery).

```markdown
# Branchless Verification of Rule 12 in `bcinr-cmca`

Rule 12 of the BCINR Deterministic Substrate Constitution states that **no runtime theorem discovery** is permitted. The authoritative runtime may only verify a supplied witness, never search for one. This strict boundary separating the "slow rail" (which derives $G$, $d$, $\delta$) from the "hot path" (which compares packed values) is rigorously enforced in `bcinr-cmca`.

## 1. Verifying Static Domination ($G d \le (1 - \delta) d$)

The verification of static domination operates over a compile-time fixed dimension (`DIM = 2`) to avoid variable iteration (satisfying Rule 13). It happens in two authoritative hops:

### A. Candidate Derivation (`stability.rs`)
The function `derive_stability_candidate` accepts the comparison matrix `g`, the positive witness vector `d`, and the margin `margin_delta` (all computed upstream by the slow rail).
Instead of discovering these values, it evaluates the inequality precisely in fixed-point arithmetic (`Q16.16`), upscaling to `i128` during multiplication to avoid overflow:
1. Calculates $G d$ exactly.
2. Calculates $(1 - \delta) d$.
3. Returns a typed refusal `StabilityDerivationRefusal::ContractionMarginInsufficient` if the bound is exceeded.

Only if the verification holds does it instantiate a sealed `StabilityCandidate` which structurally binds the elements and incorporates them into a `candidate_digest`.

### B. Independent Recomputation (`certification.rs`)
When `seal_certificate` is called to mint a `CertificateReceipt`, it refuses to trust a previously asserted boolean. It invokes `witness_holds(&candidate)` to **independently recompute** the static domination witness directly from the candidate's inner `g` and `d` fields. If this check fails, it issues `CertificationRefusal::WitnessMarginInsufficient`. This guarantees that the hot path verifies the algebraic law natively without relying on derived state.

## 2. Verifying Eigenvalue Lower Bounds

Eigenvalue lower bounds are another metric strictly prohibited from being calculated via runtime search (e.g., power iteration or spectral radius estimation). 

In `observatory.rs`, the minimum positive Gram eigenvalue is verified purely as a branchless comparative mask:
1. The lower bound eigenvalue is supplied via the `MeasurementArtifact` (`artifact.gram_lower_bound`) rather than discovered.
2. The telemetry engine (`evaluate_calibration`) executes a branchless bitwise comparison against the stability limit (`epsilon_gram`):
   ```rust
   let gamma_under_off = const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits());
   ```
3. The result of this check (`gamma_under_off`) is combined with condition number checks using bitwise `&` to evaluate `is_gram_degenerate` and other safety flags. No branches (`if/else`) or dynamic searches are used.

By stripping all power iterations and optimization searches out of the execution layer, `bcinr-cmca` successfully implements eigenvalue constraints and static domination mathematically, completely structurally deterministic, and free of $CC > 1$ branches.
```
