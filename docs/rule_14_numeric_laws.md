# Rule 14: Numeric-Law Requirements in BCINR

BCINR's mandate as a deterministic computational substrate demands that authoritative arithmetic is mathematically rigorous, completely deterministic, and fundamentally branchless (adhering to the Radon Law $CC=1$). Rule 14 establishes the absolute laws governing numerical operations to prevent silent errors, timing side-channels, and non-deterministic behavior.

## Precision Constraints for Branchless Arithmetic

### 1. Strict Contracts for Saturation and Wrapping
In BCINR, arithmetic operations must explicitly define their behavior at the boundaries of their fixed-width domains. Standard arithmetic operations that may panic on overflow are strictly prohibited. Instead, every arithmetic primitive must adhere to an explicit mathematical contract specifying whether it:
- **Saturates**: Clamps to the maximum or minimum representable value.
- **Wraps**: Applies modular arithmetic.

This ensures that state transitions remain entirely deterministic and never trigger language-generated panic paths, unwrap failures, or data-dependent branches.

### 2. Prohibition of NaN and Infinity
Floating-point non-determinism, including architecture-dependent rounding, `NaN` (Not-a-Number), and `Infinity`, introduces variability that violates authoritative runtime laws. Since all logic must be expressed as bitwise polynomials, the presence of undefined or hardware-variable numeric states is outlawed. All authoritative arithmetic must be deterministic and free of floating-point variability, guaranteeing bit-for-bit identical results across all supported architectures.

### 3. Declared Error Envelopes for Approximations
When exact arithmetic is impossible and an approximation must be used, the approximation cannot be arbitrary. It must be mathematically bounded by a rigorously declared error envelope. 

Every approximation primitive requires full documentation and formal verification of:
- **Domain & Codomain**: The exact bounds of valid inputs and corresponding outputs.
- **Error Bounds**: Explicitly quantified maximum absolute error and maximum relative error.
- **Behavioral Proofs**: Mathematical proofs for its monotonicity, saturation behavior, and boundary handling.
- **Verification Matrix**: An independent reference implementation (oracle), hostile mutants, and a full object-code audit confirming its branchless nature.

Primitives such as reciprocal, logarithm, exponential, fixed-point multiplication/division replacement, absolute value, min/max, clamp, and normalization require special scrutiny to guarantee they never introduce implicit branching or silent failures.

### 4. Formal Derivation of Smoothing and Clamp Constants
"Magic numbers" or arbitrary constants (`CHEAT-003`) are constitutional violations in BCINR. Rule 14 mandates that no epsilon may be inserted silently, and every smoothing or clamp constant must be:
- **Named**: Clearly identified within the system.
- **Derived**: Its mathematical origin, purpose, and bounds must be explicitly derived.
- **Admitted**: It must pass through authoritative verification gates and be accepted by the mathematical law owner (`@hoare_oracle`).
- **Included in the Influence Digest**: Constants fundamentally impact decision boundaries and semantic mass. Including them in the influence digest ensures that any changes to algorithmic tuning parameters are cryptographically tracked, preventing silent alterations to the system's operational envelope. 

By treating all numeric constraints and parameters as formally admitted mathematical axioms, BCINR maintains an airtight guarantee over its deterministic, branchless execution.
