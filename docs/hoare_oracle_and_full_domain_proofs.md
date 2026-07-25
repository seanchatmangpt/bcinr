# `@hoare_oracle`: Architect of Invariants and the Full-Domain Requirement

## Overview: The Role of `@hoare_oracle`
In the BCINR Deterministic Substrate, the `@hoare_oracle` acts as the **Axiomatic proof lead and specification owner**. This architect holds exclusive authority over the mathematical and logical boundaries of the system, including preconditions, postconditions, invariants, algebraic laws, admissible domains, refusal conditions, and independent reference semantics.

If a property cannot be stated precisely, it is not yet law. The `@hoare_oracle` ensures that every authoritative primitive has a mathematically sound and verifiable contract before it is implemented.

---

## The Hoare Contract: $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$
For every primitive, the `@hoare_oracle` must produce a formal Hoare contract. In this classical logic framework:
- **$P(x)$ (Precondition)**: The exact state or domain assumptions that must be true before the function executes (e.g., the valid input domain or state requirements).
- **$f(x)$ (The Primitive)**: The branchless, allocation-free function being executed.
- **$Q(x, f(x))$ (Postcondition)**: The invariants and outputs that are guaranteed to be true after the execution completes.

To pass the constitutional standard of BCINR, the contract is required to explicitly define:
1. **Valid Input Domain & Output Range**: Strict bounds on input and output data.
2. **Conservation & Monotonicity Laws**: Proof that invariants (like total mass or energy) are conserved and that properties like order preservation hold.
3. **Overflow & Numeric Error Boundaries**: Explicit saturation/wrapping behaviors and maximum absolute/relative error envelopes.
4. **Refusal & State-Mutation Boundaries**: Mathematical conditions under which invalid inputs are rejected (via typed refusals) leaving state entirely unmodified.
5. **Determinism**: The guarantee that the identical input $P(x)$ structurally maps to an exact, invariant $Q(x, f(x))$.

---

## The Full-Domain Requirement ($2^{64}$)
BCINR fundamentally rejects random testing or partial test coverage as proof of universal standing. For a 64-bit input, the state space contains $2^{64}$ (over 18 quintillion) possible values. The constitution demands that the contract covers this **entire domain**.

However, brute-force enumeration of $2^{64}$ inputs is computationally infeasible. The `@hoare_oracle` must therefore satisfy the full-domain requirement using one of the following rigorous techniques:

### 1. Formal Proofs
Instead of executing code, a formal proof relies on mathematical induction and symbolic logic to prove that the code conforms to its specification. This is often achieved using interactive theorem provers (like Coq, Lean, or Isabelle/HOL). If the symbolic mathematics prove that $P(x) \implies Q(x, f(x))$ logically holds for an arbitrary $x$ of type `u64`, the property is universally established.

### 2. Exhaustive Proof Over a Finite Partition
While enumerating $2^{64}$ values is impossible, the domain can often be mathematically partitioned into a finite set of equivalence classes (e.g., negative values, zeros, subnormals, positive values, and specific boundary conditions). By proving that the code's behavior is structurally uniform within each class, and exhaustively testing or proving the property for the representative cases covering every partition, the entire domain is effectively covered.

### 3. SAT/SMT Bit-Vector Solver Certificates
Satisfiability Modulo Theories (SMT) solvers (like Z3, CVC4) can reason about bit-vectors natively. The branchless code and its Hoare contract are translated into a boolean formula representing the question: *"Is there any input $x$ in the $2^{64}$ domain where $P(x)$ is true but $Q(x, f(x))$ is false?"* 
The solver searches for a counterexample algebraically rather than by enumeration. If the solver returns `UNSAT` (unsatisfiable), it provides a mathematical certificate that no such input exists, implicitly verifying all $2^{64}$ cases.

### 4. Equivalent Bounded Theorem Artifact
An external artifact—such as an abstract state machine or arbitrary-precision model—can mathematically constrain the operation space. If a mathematically proven theorem bounds the system’s behavior (e.g., proving that a specific fixed-point SWAR operation can never exceed a specific output limit), that mathematical artifact can serve as proof over the full domain.
