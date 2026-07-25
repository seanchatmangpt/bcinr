# Rule 14: Numeric-Law Requirements

According to Rule 14 in `AGENTS.md` from the `bcinr` deterministic substrate constitution, there are strict rules governing authoritative arithmetic, approximations, and clamps.

## Constraints on Authoritative Arithmetic
Authoritative arithmetic operations must strictly adhere to the following constraints:
* **Fixed-width**: Arithmetic must operate on types with a fixed size.
* **Deterministic**: Operations must yield the same predictable results regardless of the environment.
* **Saturating or Wrapping**: Behavior must explicitly follow a predefined contract indicating whether the arithmetic wraps or saturates on overflow/underflow.
* **Free of NaN and Infinity**: Floating point undefined or infinite values are not permitted (further enforced by the general ban on floating-point operations).
* **Free of Architecture-Dependent Rounding**: Rounding behaviors must be explicit and deterministic, not relying on CPU or platform-specific implementations.
* **Bounded by a Declared Error Envelope**: Any potential deviation or error must fall within a clearly stated bound.

## Requirements for Approximations
Every approximation must explicitly declare and document the following requirements:
* **Domain**: Valid input space.
* **Codomain**: Expected output space.
* **Maximum Absolute Error**: The absolute bound of approximation error.
* **Maximum Relative Error**: The relative bound of approximation error.
* **Monotonicity Result**: Proof or contract of monotonic behavior.
* **Saturation Behavior**: How the function behaves at limits.
* **Boundary Behavior**: The function's handling of edge cases.
* **Independent Reference**: A mathematically independent reference implementation for comparison.
* **Mutants**: Hostile tests altering behavior to verify the contract is enforced.
* **Object-Code Audit**: Evidence that the disassembly complies with the required structural constraints.

## Primitives Requiring Special Scrutiny
The following operations are prone to rule violations and require heightened scrutiny:
* Reciprocal, Logarithm, Exponential
* Fixed-point multiplication
* Fixed-point division replacement
* Absolute value
* Min/max, clamp
* Normalization
* Eigenvalue lower bounds
* KL accumulation
* Digest comparison

## Requirements for Clamps and Smoothing Constants
* **No Silent Epsilon**: No epsilon (small value) may be inserted silently into logic or arithmetic.
* **Constants**: Every constant used for smoothing or clamping must be:
  * **Named**: Clearly identified in code.
  * **Derived**: Having a clear mathematical or logical derivation.
  * **Admitted**: Formally accepted into the state/logic.
  * **Included in the Influence Digest**: Hashed or tracked to ensure auditing transparency and immutability.
