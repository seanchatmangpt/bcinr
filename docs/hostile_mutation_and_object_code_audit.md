# The Intersection of Hostile Mutation (Rule 19) and Object-Code Audit (Rule 20)

In the BCINR deterministic substrate, the intersection of **Hostile Mutation (Rule 19)** and **Object-Code Audit (Rule 20)** forms a critical defense against illusionary verification. 

Rule 19 mandates that mutants must be injected "through the real build path" and tested for specific typed refusals or oracle mismatches. Rule 20 dictates that "source-level CC=1 is necessary but insufficient," requiring an exact production-profile disassembly audit. The synthesis of these two rules addresses a profound vulnerability in software verification: compiler optimization.

## The Threat of Compiler Optimization
Modern compilers (like LLVM, which powers Rust) apply aggressive optimizations such as Dead Code Elimination (DCE), constant folding, auto-vectorization, and branch elision. When a hostile fault is introduced at the source level, the compiler may optimize the fault away entirely if it deduces that the mutated state is unreachable, structurally impossible, or mathematically constant under release conditions.

If a mutant is tested only in a debug build or evaluated purely at the source level, the verification suite may falsely report a "killed" mutant. In reality, the compiler might have excised the corrupted logic, meaning the test is passing because the fault no longer exists in the executable—not because the authoritative runtime correctly detected and rejected the hostile input.

## Eradicating "Mutant Theater" (CHEAT-009)
Rule 16 explicitly bans "CHEAT-009 — Mutant theater," which includes creating mutants that are trivially different or do not reflect real-world mechanical failures. By forcing mutants through the **real build path** (the exact release profile used in production) and subjecting them to the **object-code audit**, BCINR ensures that:
1. The hostile mutation actually survives into the machine code.
2. The runtime's mathematical laws and masks actively neutralize or explicitly refuse the fault via branchless execution.

Without object-code verification, a source-level mutation might cause the compiler to introduce a hidden branch (violating the absolute $CC=1$ law) to handle an otherwise impossible state. If the test passes but the resulting binary contains a conditional jump, the substrate's timing-side-channel immunity is compromised.

## Guaranteeing Deterministic Refusal
BCINR's absolute runtime laws demand that operations resolve via fixed bounded execution work without panicking or unwinding. Rule 19 requires that neutralizing a mutant means producing a bounded **Typed Refusal** (e.g., `DigestMismatch`, `ContractViolation`) or failing an **Oracle Mismatch**. 

By demanding evidence at the object-code level, the substrate proves that the typed refusal is generated strictly through bitwise polynomials and masked state selection (Rule 9: Mask-based execution law), not through an optimization artifact or an implicitly generated branching panic path. 

## Conclusion
The requirement to prove mutants at the object-code level ensures that the hostile fault is structurally present and deterministically handled by the machine code itself, rather than by a source-level abstraction. It enforces the governing principle of the constitution: **Rich semantics upstream. Fixed deterministic mechanics downstream.** Source claims are never a substitute for disassembly evidence.
