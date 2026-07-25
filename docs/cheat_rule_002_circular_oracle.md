# CHEAT-002: Circular Oracle

## Overview

In the BCINR Deterministic Substrate Constitution, **CHEAT-002 (Circular oracle)** is explicitly prohibited under Rule 16 (Anti-cheat manifesto). It occurs when a reference implementation (the oracle) is copied from or trivially derived from the production implementation.

## Why it is Considered Circular

A "circular oracle" means the verification process is essentially checking the production code against itself. If a bug, incorrect assumption, or logic flaw exists in the production code, copying it to the oracle ensures that the exact same flaw will exist in the reference. As a result, tests will pass despite the implementation being mathematically or logically incorrect. 

When an oracle merely mirrors the implementation, equivalence testing reduces to a tautology: `f(x) == f(x)`. This provides zero independent mathematical assurance of the system's correctness. It creates a false sense of verification, completely undermining the project's requirement for rigorous, adversarial object-code and logic testing.

## Violation of the Independent Oracle Law (Rule 15)

Rule 15 mandates that an oracle must provide **independent reference semantics**. A circular oracle directly violates this law for several key reasons:

1. **Lack of Structural and Logical Distinction**: Rule 15 strictly requires the oracle to be structurally and logically distinct from the production implementation. It explicitly prohibits:
   - Line-by-line translation of production code.
   - Reuse of production normalization, lookup tables, or fixed-point helpers.
   - Identical control structures (even if evaluated with different types, like `f64`).
   - Importing and wrapping the authoritative function.

2. **Failure to Use Admitted Independent Forms**: Instead of mirroring the production logic, an independent oracle must be built from mathematical first principles using permitted forms. Valid independent oracles include:
   - Direct mathematical formula
   - Hoare specification
   - Abstract state machine
   - Symbolic proof
   - Arbitrary-precision implementation
   - SAT/SMT bit-vector model
   - Exhaustive reduced-domain enumerator

3. **Breach of Separation of Duties (Rule 5 & 27)**: The Mandatory Decomposition Protocol dictates that `@hoare_oracle` owns the mathematical law and independent reference semantics, while `@von_neumann_bypass` owns the authoritative branchless code. A circular oracle implies a fundamental failure in this separation of duties. It acts as a form of self-certification, which is strictly banned: "No implementation agent may author its own final oracle and self-certify equivalence."

By copying the production code, a circular oracle bypasses the required structural separation, masking defects that an independent, first-principles mathematical verification would expose.
