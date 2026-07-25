# CHEAT-006: Scanner Evasion in BCINR

According to the **BCINR Deterministic Substrate Constitution** (`AGENTS.md`), the overarching mission of the project is to provide a deterministic computational substrate strictly governed by bounded, branchless, and allocation-free execution laws. 

`CHEAT-006` explicitly forbids **Scanner Evasion**—the use of obfuscation or architectural indirection to conceal unlawful structures from the `bcinr-cheat-scanner` (the structural auditor overseen by `@turing_machine`).

## Prohibited Patterns

The Anti-Cheat Manifesto (Rule 16) specifically outlaws the following evasion tactics:

1. **Splitting Operators Across Lines**
   Breaking expressions or operators into fragments across multiple lines in an attempt to bypass simple text-based regular expression matching.

2. **Inserting Comments Inside Tokens**
   Injecting block comments (e.g., `/* ... */`) within or between tokens to disrupt text parsing and hide prohibited constructs.

3. **Using Macro Indirection to Hide a Pattern**
   Wrapping forbidden control flow (`if`, `match`, `while`) or prohibited operations inside `macro_rules!` definitions or macro invocations, hoping the scanner only inspects the raw unexpanded source code.

4. **Moving Prohibited Code into Private Helpers**
   Relocating branching logic out of public APIs into private helper functions under the false assumption that only public-facing root functions are audited for $CC=1$.

5. **Moving Code into Generated Output**
   Placing non-compliant behavior into files produced by build scripts to bypass standard static analysis. (Rule 21 strictly states that generated authoritative code is not exempt and must pass all gates).

6. **Hiding Behavior Behind Traits**
   Concealing conditional logic or dynamic behavior inside trait implementations, generic monomorphizations, or dynamic dispatch to mask the complexity of the control flow graph.

7. **String Construction that Produces Prohibited Source**
   Building or concatenating strings during code generation or macros that eventually compile to prohibited Rust source after the static scanning phase.

## Why This Constitutes a Violation

`CHEAT-006` is a severe violation of the constitution, undermining several absolute laws:

* **Whole-Call-Graph Branchlessness (Rule 7):** Branchlessness and determinism apply to the *transitive* call graph. Branches hidden in private wrappers, macro expansions, or traits still compile into input-dependent conditional jumps in the object code. The true instructional shape of the program remains unlawful.
* **Subversion of Structural Enforcement (Rule 17):** The integrity of BCINR relies on the rigorous structural audit performed by `@turing_machine`. Any attempt to trick the auditor—rather than refactoring the code into branchless arithmetic selection (`@von_neumann_bypass`)—constitutes an adversarial attack on the project's verification matrix.
* **Superficial vs. Mathematical Compliance:** The substrate requires actual mathematical proofs and bit-parallel mechanics over byte-sequential control flow. Hiding branches via formatting tricks achieves only superficial compliance while maintaining the timing and structural vulnerabilities BCINR was built to eradicate.

Because of this, `bcinr-cheat-scanner` is mandated to parse the full syntax tree (AST), inspect macro expansions, normalize whitespace, and scan all generated Rust and private functions. Any confirmed `CHEAT-006` finding is an absolute failure that forces the Substrate Integrity Score (SIS) to `0`, blocks the merge, and triggers a full `MaturityScrutiny` lockdown (Rules 24 & 25).
