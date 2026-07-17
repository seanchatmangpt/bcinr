# Axiomatic & Formal Requirements (v26.7.15)
## Author: @hoare_oracle (The Oracle of Invariants)

This document formalizes the rigorous mathematical constraints, pre-conditions, post-conditions, and invariants for `bcinr`, `praxis`, and `mfact` for the v26.7.15 moonshot update. All logic detailed here is Law and subject to exact branchless execution.

### 1. V2 Tape Bridge (CompiledPowlV2 to Scheduler)

The V2 compiler natively outputs a `CompiledPowlV2` containing a cache-aligned `v2::PowlTape` (64-byte `Powl64Op`). The scheduler must bridge to this without legacy 32-byte degradation.

*   **Pre-condition**: `CompiledPowlV2` yields an exact mapping where `PowlNodeId` corresponds sequentially to tape slot indices. The active `tape` structure is loaded into memory without dynamic allocations.
*   **Invariant (The Radon Law $CC=1$)**: The `PriorityPetriEngine` must execute the `v2::PowlTape` and `v2::PowlTapeLarge` completely branchlessly. State transitions, specifically entry and exit guard evaluations, are represented as purely bitwise polynomials against the currently active token vector.
*   **Post-condition**: After a `tick`, the system state (token markings) must exactly match the sum of independent effects of the $fired$ mask minus consumed entry bounds, generating an immutable execution boundary.

### 2. Explicit Receipt Ready-Masks & Stateless Verification

The $fired \subseteq ready$ invariant has been hidden by the opaque `scheduler_decision_digest`. We establish explicit state evidence on the receipt.

*   **Pre-condition**: Given an `ExecutionReceipt`, the explicit `ready: EventSet` and `fired: EventSet` fields are present.
*   **Invariant 1 (Stateless Subset Law)**: For any valid tick execution, it is universally true that $fired \subseteq ready$.
*   **Invariant 2 (Guard Satisfaction Law)**: For any valid tick execution, $guards.admits(fired) = true$. The $fired$ operations must not contain any combinatorial nonfaces defined by the underlying capacity or numeric-fluent limits.
*   **Post-condition**: `verify_execution_receipt` achieves fully stateless $O(1)$ block validation. It asserts $fired \subseteq ready$ directly and verifies guard admittance mathematically without unrolling the simulation or performing a stateful replay.

### 3. 'Crown Theorem' and 'Portfolio Completeness' in `mfact`

The transition from pairwise independence approximations to generalized higher-order structural complexes in the `mfact` and `praxis` toolchains must be absolute.

*   **Portfolio Completeness**: The PDDL semantic analyzer strictly derives precedence from verifiable dependence witnesses (e.g., DeleteInterference or CausalSupport).
    *   **Axiom**: $(i, j) \in precedes \iff witness(i, j) \neq \emptyset$.
    *   **Corollary**: Vector index order ($i < j$) contributes exactly $\emptyset$ to the causality graph.
*   **The Crown Theorem (Numeric Concurrency Completeness)**: The `ExecutableConcurrencyComplex` is complete over the domain.
    *   **Axiom**: A nonface $N$ of size $K$ exists in the complex if and only if the joint execution of $N$ exceeds numeric/capacity invariants, despite all pairs in $N$ being classically independent.
    *   **Constraint**: If numeric constraints cannot be fully evaluated, `PddlConcurrencyAnalyzer` must rigorously return `Unsupported` rather than hallucinate a structurally deficient pairwise approximation.
