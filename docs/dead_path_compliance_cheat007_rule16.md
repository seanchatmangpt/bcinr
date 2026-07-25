# Research Report: Enforcement of CHEAT-007 (Dead-Path Compliance)

In the BCINR deterministic systems substrate, **Rule 16 (Anti-cheat manifesto)** defines **CHEAT-007 (Dead-path compliance)** as the adversarial act of *"adding lawful code that is never executed while the real path remains unlawful."* This rule prevents developers from tricking structural compliance checkers by feeding them mathematically compliant "dummy" code to mask underlying non-compliant logic in the active hot path.

The system enforces this ban and ensures that only genuinely compliant, load-bearing code is executed through a multi-layered verification strategy:

## 1. Hostile Mutation Protocol (Mutant Reachability)
Under the `@armstrong_fault` role, the project mandates rigorous mutant testing to ensure the adequacy of the test suite.
- Every load-bearing law and mathematical rule must be subjected to hostile mutations (e.g., dropping factors, flipping signs, breaking masks).
- If compliant code is placed in a "dead path," mutating its logic will not cause any tests to fail, resulting in a **surviving mutant**.
- A surviving mutant instantly alters the project standing to `MUTATION_GATE_FAILED`, blocking all feature work and exposing the code as non-load-bearing.

## 2. Textual and AST Analysis (`bcinr-cheat-scanner`)
During the `cargo make scan-cheats` step, the custom `bcinr-cheat-scanner` actively analyzes both the syntax tree and source text.
- It scans the source for trivially dead paths specifically constructed for compliance theater. For instance, the scanner looks for exact structural patterns like placing branchless dummy variables inside unreachable `if false { ... }` blocks (e.g., detecting `if false {` and `dummy_branchless`).
- Any detection triggers a `CHEAT[CHEAT-007]` violation, breaking the build.

## 3. MIR and Transitive Call-Graph Gates
Compliance rules (such as $CC=1$ cyclomatic complexity) apply transitively across the *complete authoritative call graph*. 
- The build pipeline analyzes the Mid-level Intermediate Representation (MIR) to ensure that the proven branchless logic is an active, linked participant in the real execution flow.
- A legally compliant block of dead code cannot excuse the presence of hidden branches, `unwrap()`, or allocations in the active call-graph. 

## 4. Object-Code Disassembly Audits
Source-level and MIR checks are considered insufficient on their own because compiler optimizations can rewrite logic.
- During `cargo make audit-object-code`, the final target release binary is disassembled.
- The `Enforcer of Determinism` (`@turing_machine`) role strictly audits the resulting machine code for *all* authoritative root and transitive helper symbols. 
- If the true execution path conceals conditional jump instructions, loop backedges, or panic paths, the disassembly audit will expose them and the code will fail, overriding any source-level dead-path deception.

By intertwining textual/AST anti-cheat scanners, strict whole-call-graph requirements, machine-code audits, and hostile mutant verification, the substrate mathematically guarantees that any code granted compliance standing is genuinely executed in the hot path.
