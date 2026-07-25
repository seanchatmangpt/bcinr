# Numeric-Law Requirements (Rule 14) and Error Envelopes

The `bcinr` project enforces strict mathematical bounds for its branchless fixed-point math engine via the `@hoare_oracle` specifications. Rule 14 dictates that authoritative arithmetic must be bounded by a formally declared **Numeric Error Envelope**.

## How the Mathematical Bound is Defined

The **Hoare Mathematical Contract Specifications** set the boundaries for approximations:
- **Explicit Boundaries**: The contract must specify the exact mathematical domain, output codomain, and the maximum absolute and relative error boundaries for every approximation.
- **No Silent Epsilons**: Any error margin ($\varepsilon$), smoothing factor, or clamping constant must be explicitly named, derived, mathematically admitted, and tracked in the influence digest. 
- **Executable Specification**: The error envelope acts as an executable specification where no deviations are tolerated.

## Numeric-Law Envelope Bounds for Approximations

### Fixed-Point Logarithm (`log2`)
The fixed-point logarithm relies on leading zero counts and a piecewise linear mapping ($\log_2(1+x) \approx x$) for the fractional part. 
- **Maximum Absolute Error Bound**: The theoretical maximum absolute error bound is explicitly specified as:
  $$ \max |E_{\text{abs}}| = 1 - \frac{1}{\ln 2} + \frac{\ln(1/\ln 2)}{\ln 2} \approx 0.08607 $$

### Division / Reciprocal
The branchless division relies on a reciprocal approximation using 3 Newton-Raphson iterations. 
- **Maximum Absolute Error Bound**: It specifies an uncorrected maximum absolute error bound of 1 Least Significant Bit (LSB) in Q16.16 space:
  $$ \left| \frac{\text{result}.0}{65536} - \frac{N}{D} \right| < \frac{1}{65536} $$

## How Bounds are Verified

The bounds are enforced and verified through a combination of independent architectural checks and fixed-point execution engine mechanisms:

1. **Independent Oracle Verification (`@hoare_oracle`)**: The production branchless implementation is validated against a structurally and logically independent reference to avoid "circular oracle" violations.
2. **Hostile Mutation Testing (`@armstrong_fault`)**: The implementation is subjected to negative mutant testing to prove the integrity of the approximation bounds. The independent oracle must kill adversarial mutations (like off-by-one errors or masking errors).
3. **Q16.16 Engine Enforcement**: 
   - **SWAR (SIMD Within A Register) Canonical Masking**: When an operation evaluates out of bounds, bitwise math calculates a mask (all 1s or all 0s) to branchlessly `select` and clamp the result to the saturated boundaries. This complies with the absolute $CC=1$ rule.
   - **Sticky Error Accumulation**: A `branchless_err_acc` accumulator captures bounded refusals (like `StabilityRefusal::NumericRangeExceeded`) and bitwise unions them into the struct's error state. This ensures errors propagate deterministically without triggering panics or branching.
