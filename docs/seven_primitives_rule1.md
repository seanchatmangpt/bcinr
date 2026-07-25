# The Seven Primitives of Completeness (Rule 1)

According to the **BCINR Deterministic Substrate Constitution**, a feature is never considered complete based merely on the fact that it "appears correct in tests." Testing only validates semantic correctness for a finite set of inputs, but does not guarantee the required fixed execution path. 

To preserve the fundamental mandate of `admitted input -> fixed instruction shape -> deterministic output`, every authoritative primitive must possess the following seven elements. These elements are rigorously enforced through a **Mandatory Decomposition Protocol**, where specific agent roles cross-examine the feature without self-certification.

### 1. Mathematical Contract
- **Requirement:** A precise Hoare contract defining valid input domains, output ranges, conservation and monotonicity laws, overflow behaviors, and deterministic state-mutation boundaries.
- **Enforcement:** Owned and validated by `@hoare_oracle` (Axiomatic proof lead). A feature cannot proceed without explicit mathematical bounds and defined typed refusals.

### 2. Structurally Lawful Implementation
- **Requirement:** Code that adheres to absolute runtime laws: $CC=1$, branchless, 0 heap allocations, and bit-parallel mechanics over byte-sequential control flow.
- **Enforcement:** Authored by `@von_neumann_bypass` (Architect of Arithmetic Logic). Logic must be constructed using bitwise polynomials, masks, and fixed-size API selections without data-dependent branches.

### 3. Independent Oracle or Proof
- **Requirement:** A mathematically separate specification (formal proof, bit-vector solver certificate, or abstract state machine) used as a baseline for correctness. It cannot be a line-by-line mirror of the implementation.
- **Enforcement:** Owned by `@hoare_oracle`. Strict prohibition against self-certification guarantees that the implementation agent cannot author its own oracle to prevent shared flawed assumptions and circular logic.

### 4. Hostile Mutants
- **Requirement:** Syntactically plausible, adversarial mutations of the implementation (e.g., dropped factors, bypassed refusals, index skew) designed to verify safety bounds.
- **Enforcement:** Authored by `@armstrong_fault` (Master of Failure Law). The test suite must actively kill these mutants by triggering the expected typed refusal or identifying the precise violated postcondition.

### 5. Source-Level Verification
- **Requirement:** Automated parsing and auditing of the syntax tree (including macros, generated Rust, and private wrappers) to confirm $CC=1$ and ensure no hidden branches or prohibited operations exist.
- **Enforcement:** Scanned by `@turing_machine` (Enforcer of Determinism) using tools like `bcinr-cheat-scanner`. Any violation immediately blocks the merge.

### 6. Object-Code Verification
- **Requirement:** Exact production-profile disassembly audits proving the compiled machine code has no conditional jumps, loop backedges, floating-point instructions, panic paths, or dynamic dispatch.
- **Enforcement:** Audited by `@turing_machine`. Source-level branchlessness is necessary but insufficient; the ultimate structural proof must lie in the physical object code for all supported targets.

### 7. Reproducible Evidence
- **Requirement:** A mechanical artifact (e.g., a standing receipt, verified artifact digest) proving that all verifications passed across every feature configuration and supported architecture.
- **Enforcement:** Gatekept by `@turing_machine`. The constitution strictly declares that agent agreement is not evidence; verifiable, reproducible logs and structural digests are required for final clearance.
