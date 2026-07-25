# Full-Domain Requirement for @hoare_oracle

In the `bcinr` (BranchlessCInRust) deterministic substrate, the **Oracle of Invariants** (`@hoare_oracle`) governs the mathematical integrity of the runtime. A core constitutional law of the codebase is the **Full-Domain Requirement**. 

Since brute-forcing a $2^{64}$ state space is computationally unfeasible, and because "random testing alone never establishes universal standing," `@hoare_oracle` mandates rigorous mathematical proof techniques. A property that cannot be stated precisely is "not yet law." 

To achieve full-domain standing, the oracle relies on independent reference semantics, formal Hoare logic contracts ($\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$), and structural verification anchored in the codebase via **PhD Gates**. Here is how `@hoare_oracle` mathematically proves full-domain standing without $2^{64}$ brute-force enumeration:

## 1. Exhaustive Proofs Over Finite Partitions
Instead of testing every single $2^{64}$ integer, the input domain is mathematically partitioned into a finite set of equivalence classes (cases) whose union covers the entire domain. 
- **Reduced-Domain Enumerators:** A correctness verifier can be run over a scaled-down but functionally and structurally equivalent domain.
- Because the implementation is branchless, operations within each partition exhibit uniform behavior regarding constraints, saturation, and overflow. Proving correctness for the finite partition mathematically guarantees universal correctness across the full $2^{64}$ domain.

## 2. Bit-Vector Solvers (SAT/SMT)
`@hoare_oracle` leverages bounded model checking using SAT/SMT (Satisfiability Modulo Theories) solvers to produce exact bit-vector certificates.
- The branchless implementation and its mathematical contract (preconditions, postconditions, bounds, error envelopes) are translated into symbolic bit-vector formulas.
- A solver mechanically evaluates the structural proof by searching for any input that violates the postcondition. 
- If the solver proves that the negated contract is *unsatisfiable*, it serves as a mathematical proof that no input in the $2^{64}$ domain can break the invariants, establishing true full-domain standing.

## 3. Symbolic Proofs & Artifacts
The oracle establishes an axiomatic reference using symbolic logic rather than standard execution. Acceptable symbolic artifacts include:
- **Symbolic Proofs:** Formal verification of the algorithm in a proof assistant, documented as a theorem artifact in the project thesis.
- **Abstract State Machines:** A formal state transition definition where deterministic mechanics and invariant preservation (e.g., semantic mass conservation, strict monotonicity) are proven at the specification level.
- **Arbitrary-Precision Implementations:** Using arbitrary-precision arithmetic (`BigInt`/`BigRational`) or closed-form, direct mathematical formulas to serve as a structurally distinct oracle. This prevents **circular oracles** (where a test simply mirrors the production control flow).

## 4. Codebase Integration: PhD Gates and Counterfactual Mutants
The mathematical proofs derived from the above techniques are embedded directly into the hot path as **PhD Gates** (e.g., `// Hoare-logic Verification Line N: Radon Law verified.`). These serve as verification anchors linked to the formal thesis.
- **Executable Verification:** The independent oracle (the mathematical truth) is aggressively tested against the implementation using property testing (`proptest`).
- **Hostile Mutation (`@armstrong_fault`):** To prove the proof has "teeth", intentional adversarial mutants (corruptions of the laws) are injected into the code. The oracle must strictly catch the discrepancy, and the implementation must trigger a precise, bounded **Typed Refusal** (e.g., `NumericRangeExceeded`, `ContractViolation`), proving that the full-domain mathematically rejects invalid states.
