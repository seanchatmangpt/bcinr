# Bounded Numeric Error Envelope in BCINR

The `bcinr` (BranchlessCInRust) project enforces strict mathematical bounds for its branchless fixed-point math engine via the `@hoare_oracle` specifications.

## 1. Hoare Mathematical Contract Specifications

The Hoare Oracle contract demands that every approximation have a strictly defined **Numeric Error Envelope**. This executable specification formally sets the boundaries for absolute and relative errors.

* **Explicit Boundaries**: The contract must specify the exact mathematical domain, output codomain, and the maximum absolute and relative error boundaries for every approximation.
* **No Silent Epsilons**: Any error margin ($\varepsilon$), smoothing factor, or clamping constant must be explicitly named, derived, mathematically admitted, and tracked in the influence digest.
* **Specific Error Envelopes**:
    * **`log2`**: The fixed-point logarithm relies on leading zero counts and a piecewise linear mapping ($\log_2(1+x) \approx x$) for the fractional part. Its theoretical maximum absolute error bound is explicitly specified as:
      $$ \max |E_{\text{abs}}| = 1 - \frac{1}{\ln 2} + \frac{\ln(1/\ln 2)}{\ln 2} \approx 0.08607 $$
    * **Reciprocal (Division)**: The branchless reciprocal approximation using 3 Newton-Raphson iterations specifies an uncorrected maximum absolute error bound of 1 LSB in Q16.16 space:
      $$ \left| \frac{\text{result}.0}{65536} - \frac{N}{D} \right| < \frac{1}{65536} $$

## 2. Q16.16 Engine Enforcement & `NumericRangeExceeded`

To comply with the absolute $CC=1$ rule (no branching, panics, or early returns), the Q16.16 engine employs **SWAR (SIMD Within A Register) Canonical Masking** to assert these boundaries safely.

* **Branchless Saturation**: When an operation (e.g., `wrapping_mul` followed by `>> 16` downshifting) evaluates out of Q16.16 bounds, bitwise math is used to calculate a `CanonicalMask` (evaluating to either all 1s or all 0s). This mask is then used to branchlessly `select` and clamp the result to the saturated bounds (e.g., `i32::MAX` or `i32::MIN`).
* **Sticky Error Accumulation**: The fixed-point structs (`SignedFixed`, `NonNegativeFixed`) couple the raw bit value with an `err` field. Using a `branchless_err_acc` accumulator, the engine uses the canonical overflow mask to select the `StabilityRefusal::NumericRangeExceeded` error code and bitwise union it into the struct's error state.
* **Propagation**: Because of the sticky accumulator, the bounded refusal propagates deterministically through any subsequent operations without triggering a panic or conditional exception, ensuring that the invalid-input refusal is correctly registered and returned while the hot path remains unbroken.
