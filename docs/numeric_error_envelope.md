# Substrate Numeric Error Envelope

This document defines the strict mathematical bounds for relative and absolute error required for approximations in the `bcinr` fixed-point math engine, as governed by the `@hoare_oracle` and the absolute runtime laws of the deterministic substrate.

In the BCINR architecture, floating-point operations are explicitly banned in the authoritative hot path. Instead, mathematical operations are implemented via fixed-point SWAR (SIMD Within A Register) and branchless polynomials. Every approximation requires a declared numeric error envelope detailing its domain, codomain, absolute error, relative error, and boundary behavior. Silent epsilons are strictly prohibited.

## 1. Branchless Reciprocal Approximation (Newton-Raphson)
The fixed-point math engine replaces variable-latency hardware unsigned division (`udiv`) with a constant-time branchless reciprocal approximation using 3 Newton-Raphson iterations.
- **Domain**: Divisor $D \in [0, 2^{32}-1]$, Dividend $N \in [0, 2^{32}-1]$
- **Codomain**: Q16.16 saturating result.
- **Maximum Absolute Error**: The uncorrected reciprocal approximation error compared to the true real division is strictly bounded by 1 LSB in Q16.16:
  $$ \left| \frac{\text{result}.0}{65536} - \frac{N}{D} \right| < \frac{1}{65536} $$
- **Equivalence Bound**: After applying a branchless remainder-based correction, the result achieves $E_{\text{abs}} = 0$ (100% bit-identical matching with standard truncated division for all non-overflow/non-zero inputs).

## 2. Fixed-Point Logarithm ($\log_2$)
Log-domain approximations (`fixed_point_log2`) are used extensively for normalization and relative entropy ($\kappa_q$) estimations. 
- **Integer Part**: Computed exactly in constant time via hardware leading zero counts: `63 - clz(val)`.
- **Fractional Part**: Extracted from the highest mantissa bits. Because it drops the implicit leading one and utilizes a piecewise linear mapping ($\log_2(1+x) \approx x$), the approximation carries a known error envelope.
- **Maximum Absolute Error**: The theoretical maximum absolute error bound for the piecewise linear $\log_2$ approximation is:
  $$ \max |E_{\text{abs}}| = 1 - \frac{1}{\ln 2} + \frac{\ln(1/\ln 2)}{\ln 2} \approx 0.08607 $$

## 3. Eigenvalue Lower Bounds (Gram Distinguishability)
The CMCA-RDF Observatory must bound its vision before proposing learning mutations. When estimating the smallest positive eigenvalue of the Gram matrix ($\widehat\gamma_{\min}^{+}$), statistical and numerical errors must be conservatively separated and subtracted.
- **Activation Interval Bound**:
  $$ \underline\gamma_{\min}^{+} = \widehat\gamma_{\min}^{+} - \varepsilon_\Gamma \geq \epsilon_{\mathrm{gram}} $$
- **Constraint**: The numerical error envelope $\varepsilon_\Gamma$ must be subtracted directly from the estimate. Learner activation requires this strictly bounded lower limit to prevent false numerical confidence and degenerate scaling.

## 4. Real Conformance Metric Estimators (RCME)
To avoid variable-bound loops and heap allocations during process replay verification, the verifier computes Q16.16 estimators for Generalization ($G$) and Simplicity ($S$).
- **Maximum Absolute Error**: The fixed-point RCME calculations are guaranteed to deviate from the arbitrary-precision `f64` reference oracle by no more than:
  $$ \varepsilon_{\mathrm{RCME}} \le 1.5 \times 2^{-16} $$
- This bounds the deviation to at most 1.5 LSBs in the Q16.16 fixed-point space, ensuring monotonic, stable predicates.

## 5. Normalization and Sum-to-One Error Bounds
For log-domain normalization converting weights into probability distributions without explicit division, the approximation sequences (logarithm, exponential, and reciprocal) result in an accumulated sum-to-one error envelope.
- **Invariant**: The fixed-point sum must satisfy:
  $$ \left| \sum_i p_i - 1 \right| \leq \varepsilon_{\mathrm{sum}} $$
- **Mandate**: Any $\varepsilon_{\mathrm{sum}}$, smoothing factor, or clamp constant must be explicitly named, derived, mathematically admitted, and included in the influence digest. Epsilon values can never be inserted silently into the codebase.
