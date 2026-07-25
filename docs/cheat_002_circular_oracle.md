# CHEAT-002: Circular Oracle

In the BCINR Deterministic Substrate, **CHEAT-002** defines the anti-pattern of the "Circular Oracle." Under Rule 16 (Anti-Cheat Manifesto), a circular oracle is explicitly defined as:

> *A reference implementation copied from the production implementation.*

## What Specifically Constitutes a "Circular Oracle"?

An oracle (or test reference) is not considered independent merely because it resides in a separate test directory such as `tests/reference.rs`. According to the **Independent Oracle Law** (Rule 15), a circular oracle occurs when the reference exhibits any of the following characteristics:

* **Line-by-line translation** of the production code.
* **Identical control structures**, even if swapped to use higher-precision types (e.g., `f64` instead of fixed-point).
* **Reuse of production components**, such as lookup tables, normalizations, or fixed-point helpers.
* **Directly importing and wrapping** the authoritative production function.

Instead, a genuinely independent oracle must be structurally and logically distinct. Valid forms include direct mathematical formulas, Hoare specifications, abstract state machines, arbitrary-precision implementations, or SAT/SMT bit-vector models. 

## Why is it Explicitly Flagged as Cheating?

Providing a test reference that is copied from or structurally identical to the production code is flagged as cheating for several foundational reasons:

1. **Defeating Verification ("Self-Canceling Operations"):** The fundamental mandate of BCINR is to enforce an axiomatic calculus for branchless execution ($CC=1$). The oracle exists to prove the production code adheres strictly to a mathematical contract. A circular oracle inherits all the logical flaws, incorrect assumptions, or masking errors of the production code. It merely proves that "the code does what the code does," rendering the test suite useless.
2. **Violation of Role Segregation:** Under the mandatory decomposition protocol (Rule 5) and the **No self-certification** law (Rule 27), the mathematical specification must be owned by the `@hoare_oracle` role, while the implementation is owned by `@von_neumann_bypass`. An implementation agent cannot act as its own final mathematical approver. 
3. **Substrate Integrity Score:** Any detection of a circular oracle is treated as an absolute failure, instantly dropping the Substrate Integrity Score (SIS) to 0. The cheat scanner enforces this mechanically by comparing the Abstract Syntax Trees (ASTs) of the reference and implementation, meaning that formatting changes cannot bypass the governance.

By forbidding Circular Oracles, BCINR ensures an uncompromised structural firewall between algorithmic truth and deterministic execution.
