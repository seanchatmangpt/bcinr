# Rule 15: Independent Oracle Law and Full-Domain Proofs

In the BCINR determinism framework, **Rule 15 (Independent oracle law)** dictates that the authoritative branchless implementation (`CC=1`) must be checked against an independent mathematical reference. This oracle must be structurally and algorithmically distinct, explicitly forbidding circular references, line-by-line translations, or identical control structures. 

Because **Rule 4 (Full-domain requirement)** demands that contracts hold for the entire $2^{64}$ domain, relying on randomized testing alone is prohibited. Since brute-force execution of $2^{64}$ permutations is computationally impossible, BCINR employs advanced formal verification and bit-vector modeling techniques to prove standing.

## Oracle Proof Techniques 

### 1. SAT/SMT Bit-Vector Solver Certificates (e.g., Z3)
Instead of brute-forcing billions of inputs sequentially, SMT solvers (like **Z3** or **CVC5**) explore the mathematical topology of the logic using **Symbolic Execution**:
- **Bit-Blasting:** A 64-bit input is modeled as an array of 64 independent boolean variables. Branchless code is mapped directly into an equivalent boolean gate circuit.
- **The Hoare Query:** The solver is tasked with finding any valid assignment where the precondition $P(x)$ holds but the postcondition $Q(x, f(x))$ fails (i.e., $P(x) \land \neg Q(x, f(x))$).
- **CDCL Navigation:** Using Conflict-Driven Clause Learning (CDCL), solvers logically eliminate entire classes of numbers mathematically rather than testing individually.
- **The Solver Certificate (The Bounded Artifact):** If the solver determines the formula is unsatisfiable (`UNSAT`), it generates a cryptographically verifiable solver certificate (like `DRAT` or `LRAT`). A linear-time proof checker verifies this artifact, structurally guaranteeing that the branchless code functionally matches the oracle across all $18.4$ quintillion possibilities.

### 2. Differential Property Testing (`proptest`)
While SAT solvers guarantee total correctness, BCINR pairs this with massive randomized property testing (`proptest`) acting against the independent `f64` oracles. 
- Found extensively in `bcinr-logic/src/algorithms/` and `bcinr-cmca/tests/differential.rs`.
- Proptest aggressively searches numeric edge boundaries (e.g., zero, `u32::MAX`, subnormals) that might expose discrepancies in fixed-point allocations, SWAR clamps, and numeric bitwise operations. It acts as a heuristic precursor to rigid bit-vector proofs, finding counterexamples dynamically.

### 3. Exhaustive Proofs Over Finite Partitions
For domains too complex to bit-blast completely, the `@hoare_oracle` partitions the $2^{64}$ state space into bounded, finite equivalence classes (e.g., boundaries of fixed-width overflows or signedness cliffs). By proving uniform structural mapping in each partition, full-domain proof standing is synthesized mechanically.

### 4. Bounded Pathological Test Models
To complement structural equivalence, the branchless execution paths themselves are mathematically smoke-tested under maximum hostile input conditions. Modules like `tests/jtbd_bounded_under_pathological_input.rs` construct explicitly pathological multi-agent fault combinations (e.g. dwell violations, certificate mismatches, extreme resource prices). The oracle validates that deterministic branchless latency, `CC=1` laws, and `no_alloc` barriers remain unbroken regardless of adversarial pressure.
