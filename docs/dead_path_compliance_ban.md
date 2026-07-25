# Dead-Path Compliance Ban (CHEAT-007)

## Overview
In the BCINR deterministic systems substrate, the **Anti-Cheat Manifesto** (Rule 16 of the constitution) defines **CHEAT-007: Dead-Path Compliance** as the act of adding mathematically lawful, branchless code that is never actually executed, while leaving the true, active execution path unlawful (e.g., containing hidden branches or allocations). 

This rule is designed to prevent developers or agents from trying to satisfy structural compliance checkers by feeding them "dummy" compliant code (for instance, inside an `if false { ... }` block or an unused function) to mask the fact that the underlying hot-path still contains illegal branching logic.

## Prevention Mechanisms

The project utilizes a multi-layered approach to strictly enforce this ban and ensure that only truly compliant code is executed:

### 1. AST Analysis (`bcinr-cheat-scanner`)
During the `cargo make scan-cheats` pipeline step, the `bcinr-cheat-scanner` tool parses the Rust source code into an Abstract Syntax Tree (AST) using the `syn` crate. 
- The scanner actively searches the AST for unused functions, unreachable blocks, or trivially dead paths (such as `if false { ... }`) that contain branchless stubs. 
- If the scanner detects compliant code hidden in an unreachable path while the active path contains control flow statements, it immediately fails the build.

### 2. MIR and Call-Graph Gates
Compliance is additionally verified at the Mid-level Intermediate Representation (MIR) layer. 
- The pipeline audits compile-time MIR output specifically looking for unreachable blocks and unused paths.
- This layer guarantees that the logic statically proven to be branchless is genuinely a part of the active execution call-graph rather than dead, unlinked code.

### 3. Object-Code Disassembly Audits
Because LLVM optimization passes can rewrite logic, source-level or MIR assertions alone are insufficient. The project relies on rigorous object-code audits (`cargo make audit-object-code`):
- The final target release binary is disassembled to inspect all authoritative root symbols and transitive helper symbols.
- If the real execution path secretly relies on conditional branches (e.g., `b.eq`, `je`) or loop backedges, the disassembly audit will explicitly expose them, neutralizing any source-level dead-path deception.

### 4. Hostile Mutation Protocol
Enforced by the `@armstrong_fault` role, the project mandates rigorous mutant testing to ensure test adequacy. 
- Every critical law must be mutated. If compliant code is placed in a dead path, mutating its logic will not cause any tests or invariants to fail (because it is never executed).
- The survival of these mutants immediately flags the dead path, enforcing that the true branchless logic is actively executing and load-bearing.
