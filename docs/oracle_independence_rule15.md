# Rule 15: Independent Oracle Law

In the BCINR Deterministic Substrate, **Rule 15** establishes that an oracle (test reference) is not independent merely because it resides in a separate file (e.g., `tests/reference.rs`). True independence requires a strict structural and logical separation between the production implementation and the mathematical truth that verifies it.

## The Ban on Circular Oracles (CHEAT-002)
The project strictly prohibits **Circular Oracles**—reference implementations that are structurally identical to the production code. This ban is mechanically enforced by `bcinr-cheat-scanner` (comparing ASTs) and prevents the following:
- Line-by-line transliterations of production code.
- Identical control structures simply swapped to use higher-precision types (like `f64` instead of fixed-point arithmetic).
- Reuse of production normalizations, lookup tables, or fixed-point helpers.
- Importing the authoritative function and just wrapping it in a test.

Because the project mandates absolute branchlessness ($CC=1$) and bitwise polynomials for production, an oracle that mirrors the production code inherits its flaws and violates the **No Self-Certification** law (Rule 27).

## How Structural Independence is Ensured
Instead of writing equivalent test code, the project structurally ensures oracle independence by leveraging mathematical and symbolic verification. Permitted independent forms include:

### 1. SAT/SMT Bit-Vector Models
To satisfy **Rule 4 (Full-Domain Requirement)** across $2^{64}$ permutations without relying on impossible brute-force execution, the project uses **bit-blasting**. 
- The production branchless operations (SWAR, masked state selections) are translated into a network of boolean logic gates.
- An SMT solver (e.g., Z3, CVC5) structurally compares this bit-level model to the independent mathematical contract (`{P(x)} f(x) {Q(x, f(x))}`).
- By using Conflict-Driven Clause Learning (CDCL), the solver proves no counterexample exists anywhere in the $2^{64}$ domain.
- The solver outputs a **Bit-Vector Solver Certificate** (e.g., DRAT/LRAT), which acts as an externally checkable artifact to prove the code strictly adheres to the mathematical truth.

### 2. Direct Mathematical Formulas and Hoare Specifications
Rather than executing step-by-step logic, the oracle strictly defines preconditions, postconditions, and invariants. The output validates the conservation law and numeric error envelope without mimicking the step-by-step execution path of the implementation.

### 3. Abstract State Machines & Arbitrary-Precision Implementations
When a functional reference is needed, the oracle models state transitions in a completely abstract domain (Abstract State Machines) or using infinite/arbitrary precision, sidestepping the rigid fixed-width, bit-parallel requirements imposed on the production runtime.

## Role Segregation
Finally, structural independence is socially and systematically enforced by **Rule 5 (Mandatory Decomposition Protocol)**: 
The `@hoare_oracle` role exercises exclusive write ownership over the mathematical laws and oracles, while `@von_neumann_bypass` builds the branchless bounds. An implementation agent cannot act as its own final mathematical approver.
