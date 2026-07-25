# CHEAT-007: Dead-Path Compliance in BCINR

According to the **Anti-Cheat Manifesto** (Rule 16 of the BCINR constitution in `AGENTS.md`), **CHEAT-007 (Dead-Path Compliance)** is strictly defined as:
> "Adding lawful code that is never executed while the real path remains unlawful."

### What constitutes dead-path compliance?
Dead-path compliance occurs when a developer or agent inserts compliant, structurally valid code (e.g., branchless logic) into an unreachable execution path. The purpose is to create compliance theater—deceiving static analysis tools and source-level scanners into seeing compliance, while the actual, active execution path (the "hot-path") continues to use prohibited, branching control flow. 

### How developers attempt to evade the scanner
A common method of evasion is to wrap mathematically lawful, branchless code inside an unreachable condition, such as an `if false { ... }` block, or placing it in an unused function or dead path. Because the code is syntactically valid and satisfies the structural checks of the AST-level scanner, the intent is to trick the automated gates into verifying the file as "compliant" while keeping an unlawful, non-deterministic branching version active in the underlying real logic. This fundamentally breaks the deterministic contract of the BCINR substrate (the Radon Law, $CC=1$).

### How is this caught?
To rigorously catch and prevent this deception, `bcinr` relies on a multi-layered enforcement pipeline:

1. **AST Analysis (`bcinr-cheat-scanner`)**: During the `cargo make scan-cheats` step, the scanner parses the Rust source code into an Abstract Syntax Tree (AST). It actively looks for unreachable blocks, unused functions, or trivially dead paths containing branchless stubs. If detected while the active path contains control flow statements, it fails the build.
2. **MIR and Call-Graph Gates**: Compliance is verified at the Mid-level Intermediate Representation (MIR) layer. The pipeline audits compile-time MIR output for unreachable blocks and unused paths to ensure the branchless logic is genuinely a part of the active execution call-graph.
3. **Object-Code Disassembly Audits**: Because LLVM passes can alter logic, rigorous object-code audits (`cargo make audit-object-code`) are conducted. The final release target is disassembled to inspect all authoritative and transitive helper symbols for conditional jumps or loop backedges. Any hidden conditional branches will be exposed here.
4. **Hostile Mutation Protocol**: Enforced by the `@armstrong_fault` role, every critical law must be mutated. If compliant code is placed in a dead path, mutating its logic will not cause tests or invariants to fail. The survival of these mutants immediately flags the dead path, forcing the Substrate Integrity Score (SIS) to 0 and halting all feature work until the deception is removed.
