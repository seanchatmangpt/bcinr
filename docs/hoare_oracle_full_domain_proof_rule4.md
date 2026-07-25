# `@hoare_oracle` Full-Domain Proof Standard

In the BCINR Deterministic Substrate, the **`@hoare_oracle`** acts as the axiomatic proof lead and specification owner. Random testing or partial test coverage is fundamentally rejected as proof of universal standing. For a 64-bit input space (over 18 quintillion values), the constitution requires that mathematical contracts cover the **entire $2^{64}$ domain**. 

Because brute-force enumeration of $2^{64}$ inputs is computationally infeasible, the `@hoare_oracle` must satisfy the full-domain requirement using one of the following four rigorous techniques:

### 1. Formal Proofs
Formal proofs rely on mathematical induction and symbolic logic rather than executing code. Using interactive theorem provers (such as **Coq**, **Lean**, or **Isabelle/HOL**), symbolic mathematics are used to prove that a specific property logically holds for an arbitrary $x$ of type `u64`, thereby establishing it universally across the domain.

### 2. Exhaustive Proof Over a Finite Partition
The infinite or computationally infeasible domain is mathematically partitioned into a finite set of equivalence classes (e.g., negative values, zeros, subnormals, positive values, and boundary conditions). The code's behavior must be proven structurally uniform within each class. By exhaustively testing or proving the property for representative cases that cover every partition, the entire domain is effectively covered.

### 3. SAT/SMT Bit-Vector Solver Certificates
Satisfiability Modulo Theories (SMT) solvers (like **Z3** or **CVC4**) are used to reason natively about bit-vectors. The branchless code and its Hoare contract are translated into a boolean formula representing the question: *"Is there any input $x$ in the $2^{64}$ domain where precondition $P(x)$ is true but postcondition $Q(x, f(x))$ is false?"* The solver searches for a counterexample algebraically. If it returns `UNSAT` (unsatisfiable), it provides a mathematical certificate that no such input exists, verifying all $2^{64}$ cases.

### 4. Equivalent Bounded Theorem Artifact
An external artifact—such as an abstract state machine or an arbitrary-precision model—can be used to mathematically constrain the operation space. If a proven mathematical theorem bounds the system’s behavior (e.g., proving a specific fixed-point SWAR operation can never exceed a specific output limit), that artifact serves as proof over the full domain.

---

## The Hoare Contract Baseline

Before executing the above proofs, every primitive must be defined by a formal Hoare contract: $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$. 

To pass the constitutional standard of BCINR, the contract must explicitly state:
1. **Valid Input Domain & Output Range:** Strict bounds on input and output data.
2. **Conservation & Monotonicity Laws:** Proof that invariants (like total mass or energy) are conserved and order preservation holds.
3. **Overflow & Numeric Error Boundaries:** Explicit saturation/wrapping behaviors and maximum absolute/relative error envelopes.
4. **Refusal & State-Mutation Boundaries:** Mathematical conditions under which invalid inputs are rejected (via typed refusals) leaving state bit-for-bit unmodified.
5. **Determinism:** The guarantee that the identical input $P(x)$ structurally maps to an exact, invariant $Q(x, f(x))$.
