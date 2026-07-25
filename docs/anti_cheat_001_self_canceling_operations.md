# CHEAT-001: Self-Canceling Operations

## Overview
In the BCINR codebase, the **CHEAT-001** rule strictly prohibits "Self-canceling operations." As defined by Rule 16 of the BCINR Anti-Cheat Manifesto, this applies to expressions or arithmetic calculations that functionally cancel themselves out or make no contractual contribution to the output.

An explicit example of this behavior is the expression:
`a.wrapping_add(b) ^ a`

## Why is it Explicitly Banned?

Code like `a.wrapping_add(b) ^ a` is banned because it introduces **artificial complexity**. In a project governed by strict deterministic and branchless constraints, developers might be tempted to "game" the system.

1. **Apparent Complexity Inflation:** Developers might insert dummy bit-parallel operations to make trivial logic appear as advanced branchless arithmetic (such as SWAR or SIMD construction) to satisfy the visual expectations of the `@turing_machine` structural auditor.
2. **The "Contractual Contribution" Law:** The BCINR constitution dictates that *"Any operation without a contractual contribution to the output is prohibited."* Every single instruction must be mathematically necessary to satisfy the Hoare contract defined by the `@hoare_oracle`. If an operation does not advance the state towards the mathematical postcondition, it is illegitimate.
3. **Execution Bounding and Purity:** BCINR enforces absolute runtime laws, including "fixed bounded execution work." Self-canceling operations bloat the hot path, pollute the binary with dead logic, and violate the principle of rigorous object-code purity.

## How the Structural Audit Detects and Rejects It

The BCINR structural audit enforces this rule mechanically using the `bcinr-cheat-scanner` tool. To avoid being tricked by whitespace or formatting (which a simple regex might miss), the scanner operates on the Abstract Syntax Tree (AST) of the code.

Here is exactly how the scanner detects these non-contributing operations:

1. **AST Traversal:** The scanner uses the `syn` crate to parse Rust source code into an AST and implements a custom visitor (`SynCheatVisitor`) to traverse every expression in the codebase.
2. **Binary Operator Inspection:** The visitor specifically intercepts `Expr::Binary` expressions looking for the `BitXor` (`^`) and `Sub` (`-`) operators.
3. **Stringified Comparison:** It converts the left-hand and right-hand sides of the binary operation into strings (stripping all whitespace) and compares them. If they match identically (e.g., `A ^ A` or `A - A`), it immediately flags a `CHEAT-001` violation.
4. **Targeted Method Call Detection:** To catch the exact pattern `a.wrapping_add(b) ^ a`, the scanner inspects the left and right operands for method calls to `wrapping_add` or `wrapping_sub`. If it finds one, it extracts the receiver (the `a` in `a.wrapping_add(b)`) and compares its stringified form against the opposite operand. If they match, it triggers the detection rule.
5. **Merge Blocking:** When the scanner detects these patterns, it emits a `CHEAT[CHEAT-001]` finding containing the exact file and matched expression. Because there are no "warning-only" violations in BCINR, this finding automatically drops the Substrate Integrity Score (SIS) to 0 and hard-blocks the CI/CD merge gate.
