# The Monotonicity Law in BCINR Approximations

## 1. Mathematical Definition

In the `bcinr` deterministic substrate, the **Monotonicity Law** is a strict mathematical proof obligation that dictates the expected directional evolution of numeric fixed-point approximations across a function's codomain.

For any two admitted inputs $x_1, x_2$ within a valid domain, the implementation must preserve order mathematically:
- **Monotonically Increasing Functions:** $x_1 \le x_2 \implies f(x_1) \le f(x_2)$
- **Monotonically Decreasing Functions:** $x_1 \le x_2 \implies f(x_1) \ge f(x_2)$

## 2. Requirements under Hoare Contracts (Rule 4) and Numeric Laws (Rule 14)

- **Rule 4 (Hoare Contracts):** Every primitive operating on a domain of inputs must explicitly define a Hoare contract: $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$. The postcondition $Q$ must include the Monotonicity Law to ensure directional relationships between inputs are consistent and verifiable, strictly preventing erratic numerical behaviors.
- **Rule 14 (Numeric approximations):** Authoritative arithmetic must be fixed-width, deterministic, and bounded by a declared error envelope. The approximation must rigorously define boundary behaviors without using non-finite values or architecture-dependent rounding. The Monotonicity Law ensures these numerical bounds act as a mathematical guarantee, not merely a best-effort approximation.

## 3. Why Approximations Must Preserve Order

Fixed-point numeric approximations are highly vulnerable to silent integer overflow, wrapping artifacts, and saturation pitfalls. If an approximation fails to preserve order, it creates a "directional reversal" — where crossing a maximum threshold unexpectedly wraps the value to the opposite extreme of the codomain.

By preserving order, approximations ensure that mathematical consistency remains intact even at the extreme bounds of the domain (e.g., Q16.16). This prevents out-of-bounds operations from implicitly violating the underlying mathematical contract.

## 4. How Breaking Monotonicity Destroys Autonomic Stability

The MAPE-K Autonomic Loop in BCINR (Observe, Infer, Propose, Accept, Execute) relies on constant-time, bounded execution. If monotonicity is broken, autonomic stability is destroyed in several ways:
- **Non-linear Threshold Crossings:** Erratic directional reversals cause the autonomic state (e.g., `RlState`) to react oppositely to extreme conditions. For example, an extremely high resource usage might overflow and incorrectly register as near-zero usage.
- **Pricing and Security Bypasses:** As seen in a historical "saturation-negation vulnerability" in the resource allocator, a non-monotonic exponentiation decay evaluated $e^{-\text{huge}}$ as $\approx 1.0$ instead of the expected $0.0$, reversing the highest pricing penalties and yielding massive unearned discounts.
- **Verification Failure:** Breaking monotonicity breaks the guarantee of deterministic state transitions. Without strict monotonicity, hostile verification (`@armstrong_fault`) and exact error envelopes become mathematically impossible to bound, completely destroying the deterministic integrity of the "hard substrate."
