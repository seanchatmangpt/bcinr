# BCINR Rule 1: 7 Mandatory Requirements Checklist Tracking Mechanism

According to Rule 1 (Mission) of the BCINR Deterministic Substrate Constitution, a primitive is never considered complete simply because it "appears correct in tests." The authoritative runtime must preserve the strict physical invariant: `admitted input → fixed instruction shape → deterministic output`. 

To enforce this, the 7 mandatory requirements are tracked per primitive across two distinct layers: **Source-Level Scaffolding** (in-file tracking) and **Cryptographic-Style Artifact Ledgers** (repository-level checklists).

## 1. Source-Level Scaffolding (In-File Tracking)
The first four requirements are structurally embedded directly within the primitive's source code file (e.g., `crates/bcinr-logic/src/algorithms/abs_diff_i64.rs` or `hyperloglog_add_u64_registers.rs`).

1. **A mathematical contract:** 
   Tracked via strict documentation headers (e.g., `/// # CONTRACT` or `/// # Branchless Contract`) and internal scaffolding blocks (e.g., `// AXIOMATIC PROOF: Hoare-logic Analysis`). This establishes the Hoare contract (`{P(x)} f(x) {Q(x,f(x))}`), pre/postconditions, conservation laws, and invariant typed-refusal boundaries.
2. **A structurally lawful implementation:** 
   The core branchless implementation itself. It must utilize bitwise polynomials, SWAR, or mask-based state selection, enforcing zero dynamic control flow, zero allocations, and typically marked with `#![forbid(unsafe_code)]`.
3. **An independent oracle or proof:** 
   Tracked inside the file's `tests` module under standardized block headers like `// POSITIVE ORACLE: Reference implementation`. This provides an independent, logically distinct (often branching or wider-typed) reference version used exclusively as a baseline for correctness.
4. **Hostile mutants:** 
   Tracked alongside the oracle under blocks like `// NEGATIVE MUTANTS: Intentionally flawed versions` or `// MUTANT COUNTERFACTUALS`. These are syntactically plausible, adversarially flawed versions of the primitive (e.g., operator-swap bluffs, bit-skips) designed to test if the test suite actually catches contract violations.

## 2. Artifact Ledgers (Repository-Level Checklists)
The remaining three requirements are tracked through strict mechanical markdown ledgers. Per the **Mandatory Decomposition Protocol** and **Rule 29**, a primitive only achieves "PhD-Verified" status (a perfect Substrate Integrity Score of 100/100) when independent agents sign off on these ledgers:

5. **Source-level verification (`SOURCE_AUDIT.md`):**
   Tracked and owned by the `@turing_machine` role. This artifact serves as the checklist confirming that the `bcinr-cheat-scanner` has successfully passed the transitive call graph. It provides evidence of strict `CC=1` (cyclomatic complexity) enforcement, zero heap allocations, no panic paths, and the absence of scanner evasion or dead-path compliance.
6. **Object-code verification (`OBJECT_CODE_AUDIT.md`):**
   Tracked by `@turing_machine`. Source-level `CC=1` is necessary but insufficient. This matrix tracks exact production-profile disassembly audits. It contains a table explicitly verifying each symbol's compiled machine code state:
   `| Symbol | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |`
7. **Reproducible evidence (`MUTANT_KILL_MATRIX.md`):**
   Tracked and owned by `@armstrong_fault`. Standard test coverage is insufficient; adversarial boundaries are cryptographically tracked in a mutant ledger. For every primitive, this checklist must explicitly track the survival or death of at least three mutants, recording:
   - `mutant id` & `source file`
   - `changed law` & `exact mutation`
   - `expected detection` & `actual detection` (proving that a specific Typed Refusal was triggered, not just an `assert_ne!`)
   - `test name`
   - `receipt digest` (cryptographic proof of execution)
   - `standing` (must result in `ALIVE`; any surviving mutant triggers `MUTATION_GATE_FAILED`)

By segregating the tracking between tightly coupled source scaffolding and isolated, role-owned markdown ledgers, BCINR mathematically guarantees that no single agent can self-certify a primitive, fulfilling Rule 1's requirement for reproducible evidence.
