# Numeric Error Envelopes in BCINR

## Overview
In the BCINR deterministic computational substrate, floating-point operations are explicitly banned in the authoritative hot path (as per Rule 14). Mathematical operations are instead implemented via fixed-point SWAR (SIMD Within A Register) and branchless polynomials. To maintain bit-for-bit reproducibility and branchless stability, every mathematical approximation (such as reciprocal, log, or exponential) must define an exact, mathematically rigorous **Numeric Error Envelope**. 

## The Hoare Oracle Contract Specifications
According to Rule 14 and the `@hoare_oracle`, every approximation requires an executable specification that formally sets boundaries for its behavior. This mathematical contract must specify:
- Exact mathematical domain and codomain
- Maximum absolute error ($E_{\text{abs}}$)
- Maximum relative error
- Monotonicity result
- Saturation and boundary behaviors

**Crucial Mandate:** No "silent epsilons" are permitted. Any error margin ($\varepsilon$), smoothing factor, or clamping constant must be explicitly named, derived, mathematically admitted, and tracked in the influence digest.

## Why Exact Envelopes and Monotonicity are Required
1. **Axiomatic Provability:** A documented error envelope acts as a formal proof obligation, explicitly bounding numerical noise and ensuring the approximation adheres to strict mathematical contracts. "If a property cannot be stated precisely, it is not yet law."
2. **Deterministic State Transitions:** Strict monotonicity guarantees that mathematical order and state logic are preserved without introducing non-linear artifacts or oscillations that could violate determinism.
3. **Hostile Verification (`@armstrong_fault`):** Exact error envelopes and monotonicity define precisely when a mutation illegally violates the acceptable limits of an approximation. This allows hostile mutants to trigger typed refusals or oracle mismatches correctly.
4. **Substrate Integrity:** Undocumented bounds or silent errors break the guarantee of bounded execution work and deterministic fixed-width state transitions, destroying the "hard substrate" requirement.

## Tracking and Proving Complex Primitives
The fixed-point math engine tracks and enforces these envelopes through strict mathematical and structural guarantees for complex primitives:

### 1. Branchless Reciprocal Approximation (Newton-Raphson)
Replaces variable-latency hardware unsigned division (`udiv`) with a constant-time branchless reciprocal approximation using 3 Newton-Raphson iterations.
- **Domain**: Divisor $D \in [0, 2^{32}-1]$, Dividend $N \in [0, 2^{32}-1]$
- **Codomain**: Q16.16 saturating result
- **Maximum Absolute Error**: The uncorrected approximation error compared to true real division is strictly bounded by 1 LSB in Q16.16:
  $$ \left| \frac{\text{result}.0}{65536} - \frac{N}{D} \right| < \frac{1}{65536} $$
- **Equivalence Bound**: After applying a branchless remainder-based correction, the result achieves $E_{\text{abs}} = 0$ (100% bit-identical matching with standard truncated division for all non-overflow/non-zero inputs).

### 2. Fixed-Point Logarithm ($\log_2$)
Log-domain approximations are used extensively for normalization and relative entropy ($\kappa_q$) estimations.
- **Integer Part**: Computed exactly in constant time via hardware leading zero counts (`63 - clz(val)`).
- **Fractional Part**: Extracted from the highest mantissa bits utilizing a piecewise linear mapping ($\log_2(1+x) \approx x$).
- **Maximum Absolute Error**: The theoretical maximum absolute error bound for the piecewise linear approximation is explicitly proven and specified:
  $$ \max |E_{\text{abs}}| = 1 - \frac{1}{\ln 2} + \frac{\ln(1/\ln 2)}{\ln 2} \approx 0.08607 $$

### 3. Exponential and Sum-to-One Normalization
For log-domain normalization converting weights into probability distributions without explicit division, the approximation sequences (logarithm, exponential, and reciprocal) result in an accumulated sum-to-one error envelope.
- **Invariant**: The fixed-point sum must satisfy $\left| \sum_i p_i - 1 \right| \leq \varepsilon_{\mathrm{sum}}$

### 4. Eigenvalue Lower Bounds (Gram Distinguishability)
When estimating the smallest positive eigenvalue of the Gram matrix ($\widehat\gamma_{\min}^{+}$), the numerical error envelope ($\varepsilon_\Gamma$) must be conservatively separated and subtracted:
- **Activation Interval Bound**: $\underline\gamma_{\min}^{+} = \widehat\gamma_{\min}^{+} - \varepsilon_\Gamma \geq \epsilon_{\mathrm{gram}}$
- **Constraint**: This bounded lower limit must be subtracted directly from the estimate to prevent false numerical confidence and degenerate scaling before learner activation.

## Q16.16 Engine Enforcement & `NumericRangeExceeded`
To comply with the absolute $CC=1$ rule (no branching, panics, or early returns), the engine enforces these envelopes safely using **SWAR Canonical Masking**:
- **Branchless Saturation**: If an operation evaluates out of Q16.16 bounds, bitwise math calculates a canonical mask (all 1s or 0s) to branchlessly `select` and clamp the result to the saturated bounds.
- **Sticky Error Accumulation**: Fixed-point structs couple their raw bit value with an `err` field. A `branchless_err_acc` accumulator uses canonical overflow masks to select the `StabilityRefusal::NumericRangeExceeded` error code and bitwise union it into the struct's error state.
- **Deterministic Propagation**: The bounded refusal propagates deterministically through any subsequent operations without triggering a panic or conditional exception, ensuring that the invalid-input refusal is safely returned while the hot path remains unbroken.
