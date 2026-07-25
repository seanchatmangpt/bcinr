# Mandatory Decomposition Protocol (Rule 5)

Rule 5 of the BCINR Deterministic Substrate Constitution dictates that every nontrivial implementation task must be immediately decomposed into four strictly independent workstreams. Each is governed by a distinct persona to ensure mathematical, structural, adversarial, and implementation integrity. 

**Crucially, independence is mandatory, and self-certification is strictly prohibited.** An implementation agent cannot author its own oracle; a structural auditor cannot silently repair code; and a mutation agent cannot derive expectations from the implementation.

## The Four Workstreams & Project Organization

### 1. Mathematical Law (`@hoare_oracle`)
- **Role:** Axiomatic proof lead and specification owner.
- **Outputs:** Contracts, proof obligations, and independent reference semantics (the oracle).
- **Project Organization:**
  - **Artifacts:** Documented in project-level or crate-level artifacts like `CONTRACT.md`, `HOARE_TRIPLES.md`, and `ORACLE_INDEPENDENCE.md`.
  - **Source Code:** Manifests via Hoare-logic proof annotations directly in the `.rs` files (e.g., `/// # Hoare contract` or `// Hoare-logic Verification Line X: [Proof statement]`).
  - **Testing:** Implemented as completely independent mathematical models in the test suite (e.g., reduced-domain enumerators or symbolic proofs) rather than line-by-line copies of the implementation.

### 2. Structural Enforcement (`@turing_machine`)
- **Role:** Structural auditor and merge gatekeeper enforcing determinism, absolute `CC=1` (Cyclomatic Complexity), and zero heap allocation.
- **Outputs:** Source and object-code audit plans.
- **Project Organization:**
  - **Artifacts:** Documented in logs like `OBJECT_CODE_AUDIT.md` (e.g., `crates/bcinr-cmca/OBJECT_CODE_AUDIT.md`), which record specific disassembly checks against conditional jumps, loop backedges, and allocator symbols.
  - **Source Code:** Inline structural assertions proving branchlessness, manifesting as explicit `// Hoare-logic Verification Line X: Radon Law verified.` comments.
  - **Infrastructure:** Enforced via `bcinr-cheat-scanner` which parses the full syntax tree and expanded macros to catch hidden branches and scanner evasion.

### 3. Hostile Verification (`@armstrong_fault`)
- **Role:** Adversarial test architect and mutation owner.
- **Outputs:** Hostile mutants and refusal expectations.
- **Project Organization:**
  - **Artifacts:** Tracked via the mutant ledger in `MUTANT_KILL_MATRIX.md` (e.g., `crates/bcinr-cmca/MUTANT_KILL_MATRIX.md`), which logs the mutant id, exact mutation, and expected typed refusal.
  - **Source Code:** Deliberate hostile faults are injected into the production `src/` modules using conditional feature flags (e.g., `#[cfg(feature = "mutant_1")]` in files like `allocator.rs` or `fixed.rs`).
  - **Testing:** Dedicated tests in files like `tests/hostile_mutants.rs` explicitly target these flags (e.g., `kill_mutant_7_...`) to verify that corrupted logic yields exact, bounded typed refusals rather than just assertion failures.

### 4. Implementation (`@von_neumann_bypass`)
- **Role:** Architect of Arithmetic Logic and authoritative implementation owner.
- **Outputs:** Branchless bounded code (`#![no_std]`).
- **Project Organization:**
  - **Source Code:** Resides in the `src/` directories of the authoritative crates (e.g., `crates/bcinr-logic/src/`).
  - **Implementation Style:** Manifests through the total absence of standard control flow (`if`, `match`, variable-loops). Driven exclusively by deterministic, constant-time operations using masks, fixed-width arithmetic, SWAR, and bitwise logic blocks.

## The Decomposition Workflow

1. **Specification Phase:** `@hoare_oracle` establishes the mathematical boundary (`CONTRACT.md`), defining exactly what the domain, limits, and behavior must be without concern for implementation constraints.
2. **Adversarial Setup Phase:** `@armstrong_fault` reads the contract and writes the expectation matrix (`MUTANT_KILL_MATRIX.md`), preparing specific logical mutations (via `#[cfg(feature = "...")]`) and exactly which typed refusal each should trigger.
3. **Execution Phase:** `@von_neumann_bypass` writes the hot-path arithmetic logic in `src/`, replacing standard operations with bit-parallel mathematics to strictly adhere to the contract.
4. **Audit Phase:** `@turing_machine` disassembles the built binaries and scans the expanded source, proving that the exact target architecture object code contains zero branches, producing the final `OBJECT_CODE_AUDIT.md`.
5. **Convergence:** A feature is only admitted when all four independent streams clear with exact alignment. A surviving mutant or a hidden branch in the audit immediately pauses the pipeline and triggers Maturity Scrutiny.
