# Rule 16: Anti-Cheat Theater Prevention in BCINR

The BCINR Deterministic Substrate Constitution strictly prevents "compliance theater"—situations where validation appears to pass without enforcing actual deterministic/mathematical guarantees. Here are the specific mechanisms guarding against three critical cheating strategies:

## 1. CHEAT-007: Dead-Path Compliance
**Definition**: Adding lawful, branchless code into unreachable or unused paths while the active hot-path continues to use prohibited branching control flow. This is done to deceive simple AST scanners.

**How the Gates Prevent It:**
*   **AST Analysis (`bcinr-cheat-scanner`)**: During `cargo make scan-cheats`, the scanner parses the AST and actively flags unreachable blocks (e.g., `if false { ... }`), unused functions, or trivially dead paths containing branchless stubs.
*   **MIR and Call-Graph Gates**: Audits compile-time MIR output for unreachable blocks, ensuring the verified branchless logic is genuinely part of the active call-graph.
*   **Object-Code Disassembly Audits**: LLVM optimizations can rewrite logic, so `cargo make audit-object-code` disassembles the final release target. It inspects all authoritative and transitive helper symbols for conditional jumps or loop backedges that the source-level scanner might have missed.
*   **Hostile Mutation Protocol**: The `@armstrong_fault` agent must mutate every critical law. If compliant code is placed in a dead path, its mutation won't trigger any test failures. The survival of these mutants instantly exposes the dead path, forcing the Substrate Integrity Score (SIS) to 0.

## 2. CHEAT-009: Mutant Theater
**Definition**: Creating fake, trivial, or easily caught mutants merely to satisfy the "three mutants per file" quota, rather than genuinely challenging the substrate's structural or mathematical boundaries.

**How the Gates Prevent It:**
*   **Strict Plausibility Requirements**: Mutants must successfully compile, bypass basic compiler checks, and execute through the real hot path. They must alter a load-bearing law (e.g., inverting signs, dropping factors, omitting normalization). Trivial changes to comments or formatting are rejected.
*   **The Typed Refusal Mandate (No `assert_ne!`)**: Tests cannot merely check if outputs diverge (`assert_ne!(baseline, mutant)`). A valid kill requires asserting an **exact typed refusal** (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`) or identifying a precise postcondition violation dictated by the `@hoare_oracle`. Divergence alone proves nothing about the substrate's mathematical guardrails.
*   **Scanner Verification**: The AST scanner validates that mutant test cases explicitly assert these typed refusals rather than relying on weak equality checks.

## 3. CHEAT-010: Gate-Jurisdiction Theater
**Definition**: Reporting a passing scanner or audit (like `scan-cheats` or `audit-object-code`) where the execution scope intentionally or accidentally omits the relevant authoritative crates, newly modified files, generated code, or specific feature targets.

**How the Gates Prevent It:**
*   **Required Proof of Jurisdiction (Rule 23)**: A green test command is not evidence on its own. The final report must explicitly state the command, exit status, files inspected, features tested, targets inspected, and include the artifact digest. 
*   **Configuration Audits**: The scanner specifically verifies that configurations target the authoritative roots, such as `crates/bcinr-logic` and `crates/bcinr-cmca`.
*   **Automatic `SIS = 0` Penalty (Rule 24 & 25)**: Gate-jurisdiction omission is an absolute constitutional failure. Regardless of other passing tests, it instantly forces `SIS = 0` and triggers the `MaturityScrutiny` protocol. This freezes all feature development, quarantines the affected code, and mandates a complete root-cause repair and rerun of the entire matrix. This strict penalty ensures the `@turing_machine` (Enforcer of Determinism) is never bypassed.
