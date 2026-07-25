# Hoare Contract Schema (Rule 4)

The `@hoare_oracle` mandates that every authoritative primitive must be backed by a mathematically sound, executable specification. 

**Format:** `{P(x)} f(x) {Q(x,f(x))}`

Where:
- **$P(x)$ (Precondition)**: The exact state/domain assumptions before execution.
- **$f(x)$ (The Primitive)**: The branchless, allocation-free function.
- **$Q(x, f(x))$ (Postcondition)**: The invariants and outputs guaranteed after execution.

To satisfy the constitutional requirements, every Hoare Contract explicitly specifies seven core properties:
1. **Valid Input Domain & Output Range** (Admissible domains)
2. **Conservation & Monotonicity Laws** (where applicable)
3. **Overflow Behavior**
4. **Invalid-input Refusal**
5. **Determinism**
6. **State-mutation Boundary**
7. **Numeric Error Envelope**

---

# Structural Definitions for Numeric Approximations

Under Rule 14 (Numeric-law requirements) and Rule 4, implementations for fixed-width algorithms (like fixed-point multiplication and logarithms) define their mathematical bounds strictly:

### 1. Admissible Domains
- **Domain (Precondition $P(x)$) & Codomain**: Explicitly bounded input/output spaces (e.g., `{ val, aux ∈ U64 }` mapping to a Qx.fb fixed-point value).
- **Full-Domain Proof Requirement ($2^{64}$)**: Mere random testing is explicitly rejected. Admissible domains must be certified across the entire 64-bit state space via formal interactive theorem provers, exhaustive partition proofs, SAT/SMT bit-vector solvers, or equivalent theorem artifacts.
- **Boundary Behavior**: Edge cases must be resolved deterministically without branches. For instance, in the fixed-point logarithm, the boundary condition `val == 0` is mathematically mapped to `0` via purely bitwise shift-and-mask polynomials (avoiding `if val == 0`).

### 2. Monotonicity Law
- **Order Preservation**: The postcondition $Q$ strictly includes the monotonicity law, serving as proof or enforcement that the approximation preserves order when mapping from domain to codomain.
- **Directional Consistency**: This establishes structural verification that the numeric approximation will not result in erratic behavior (e.g., producing a smaller output for a larger input where mathematically prohibited). 

### 3. Numeric Error Envelope
Instead of floating-point arithmetic or unbounded loops, operations use fixed-point SWAR limits (e.g., Q16.16).
- **Maximum Absolute and Relative Error**: Bounded bounds on deviation from true mathematical results. For example, in the substrate's $\log_2$ piecewise linear approximation, the maximum absolute error is mathematically bounded to $\max |E_{\text{abs}}| \approx 0.08607$.
- **No Silent Epsilons**: Slipping in "magic" epsilons to avoid division by zero or numerical instability is constitutionally banned. Any constant must be named, logically derived, and bound by the independent oracle's theorem.
- **Branchless Enforcement (Radon Law $CC=1$)**: Error envelope violations are evaluated mathematically to a `CanonicalMask`. This mask bitwise-selects the `APPROX_ENVELOPE` fault and joins it to the ongoing state via a bitwise `OR` semilattice accumulation, securely intercepting the mutation boundary and ultimately signaling a `StabilityRefusal::EnvelopeViolated` without using control flow.
