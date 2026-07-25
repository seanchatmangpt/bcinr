# Hoare Contract: The Monotonicity Law

## 1. Overview
In the `bcinr` (BranchlessCInRust) deterministic substrate, the **Monotonicity Law** is a strict mathematical proof obligation mandated by the **Oracle of Invariants** (`@hoare_oracle`). It dictates the expected directional evolution of states and numeric fixed-point approximations across a function's codomain. 

Every primitive operating on a domain of inputs must explicitly define a Hoare contract:
$$ \{P(x)\} \quad f(x) \quad \{Q(x,f(x))\} $$
Where applicable, $Q$ must include the Monotonicity Law, ensuring that directional relationships between inputs are preserved or inverted consistently in the output, strictly preventing erratic numerical behaviors.

## 2. Mathematical Definition
For numerical fixed-point implementations, the Monotonicity Law formalizes the relationship between ordered inputs and their resulting outputs. 

For any two admitted inputs $x_1, x_2$ within the valid domain:
*   **Monotonically Increasing Functions:** 
    $$x_1 \le x_2 \implies f(x_1) \le f(x_2)$$
*   **Monotonically Decreasing Functions** (e.g., `exp_minus` safe decay exponentiation): 
    $$x_1 \le x_2 \implies f(x_1) \ge f(x_2)$$

## 3. Preventing Erratic Directional Reversals
Fixed-point numeric approximations are highly susceptible to silent integer overflow, wrapping artifacts, and saturation pitfalls. If left mathematically unbound, these arithmetic faults cause "directional reversals" — where crossing a maximum threshold unexpectedly wraps the value to the opposite extreme of the codomain.

### Example: The Saturation-Negation Vulnerability
Prior to the `exp_minus` innovation in the resource allocator, a severe pricing bypass occurred due to a directional reversal:
1. An extremely high pricing cost overflowed and correctly saturated to `u32::MAX`.
2. To negate the value, a `0u32.wrapping_sub(u32::MAX)` was performed.
3. This wrapped the value to `1` (a near-zero penalty).
4. The exponentiation evaluated this as $e^{-\text{tiny}} \approx 1.0$, instead of the mathematically expected $e^{-\text{huge}} \approx 0.0$.

Instead of monotonic decay, the highest pricing penalties reversed direction, yielding massive discounts. The Monotonicity Law explicitly forbids this: mathematical consistency must remain intact even at the extreme bounds of the Q16.16 (or relevant) domain.

## 4. Branchless Enforcement & Compliance
To enforce the Monotonicity Law while strictly adhering to the `CC=1` (Cyclomatic Complexity of 1) and zero-allocation mandates of the substrate, implementations must prevent directional reversals without using conditional control flow (`if`, `match`, or panics). 

Compliant implementations achieve this via **bit-parallel logic**:
*   **Constant-Time Guarding:** Using bitwise masks and constant-time shifts to branchlessly clamp out-of-bounds values (e.g., forcing a value to `0` when an underflow threshold is reached).
*   **Direct Domain Approximation:** Computing operations (like minimax polynomials) entirely on an unsigned domain to avoid complex sign-dependent shift logic and wrapping arithmetic.

## 5. Verification Strategy
`@hoare_oracle` establishes the Monotonicity Law, but it must be independently verified to achieve PhD-Verified (100/100 SIS) standing:
1.  **Independent Oracle Testing:** The implementation is differential-tested against an independent, structurally distinct oracle (often a double-precision floating-point "slow rail" function). Differential tests verify perfect monotonicity across millions of inputs.
2.  **Hostile Mutation (`@armstrong_fault`):** Syntactically plausible mutants are injected to intentionally break monotonicity (e.g., altering a clamp threshold or inverting a polynomial sign). The test suite must demonstrate that these mutants cause a verifiable, typed refusal (e.g., `ContractViolation`) or explicit oracle mismatch.
