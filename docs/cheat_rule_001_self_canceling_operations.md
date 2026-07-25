# CHEAT-001: Self-Canceling Operations

## Introduction
Under **Rule 16 (Anti-cheat manifesto)** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), **CHEAT-001** explicitly prohibits the use of self-canceling operations in both production and verification code. A self-canceling operation is an expression that executes intermediate logic only to mathematically nullify itself, such as `a.wrapping_add(b) ^ a`, when included strictly to create apparent complexity.

## What are Self-Canceling Operations?
Self-canceling operations are expressions that evaluate in a way that resolves back to their original state, a trivial constant, or otherwise zeroes out their own computational work. 

Examples flagged by the syntax tree scanners include:
- `a.wrapping_add(b) ^ a` (when not part of a validated bitwise trick, but used to obscure logic)
- `(A) ^ (A)`
- `A - A`
- Any operation strictly structured to "look busy" while having zero functional impact on the returned result.

## Why are Operations Without a Contractual Contribution Prohibited?

1. **Mathematical Purity and the Hoare Contract (Rule 4):**
   Every primitive in the authoritative runtime must be backed by an exact mathematical contract ($P(x) \implies Q(x, f(x))$). An operation that does not contractually contribute to the postcondition ($Q$) breaks the one-to-one equivalence between the mathematical proof and the structural implementation.
   
2. **Fixed Bounded Execution Work (Rule 3):**
   The runtime laws mandate "fixed bounded execution work." Every CPU cycle and instruction in the hot path must be accounted for. Extraneous, non-contributing operations violate this law by executing arbitrary work that provides no semantic mass.

3. **Auditability and Transparent Mechanics (Rule 5 & 10):**
   The architecture strictly requires "Rich semantics upstream. Fixed deterministic mechanics downstream." Spurious logic adds structural noise that hampers the `@turing_machine` (Enforcer of Determinism) and `@von_neumann_bypass` (Architect of Arithmetic Logic) during object-code and MIR audits. Every machine instruction must directly map back to an admitted law.

## Why is Creating "Apparent Complexity" a Constitutional Violation?

1. **Complexity Theater (Fake Compliance):**
   Because the BCINR constitution strictly bans all branches, developers must express logic as fixed-width bitwise polynomials. A malicious or lazy agent might inject self-canceling operations to artificially inflate a function's complexity, making a simple or stubbed function appear as a sophisticated branchless polynomial to deceive reviewers or naive complexity scanners.

2. **Scanner Evasion (Rule 16):**
   Injecting apparent complexity is classified as cheating because it is an attempt to bypass the Substrate Integrity Score (SIS). It obscures the actual runtime logic and prevents the `bcinr-cheat-scanner` and independent oracles from properly evaluating the algorithmic domain.

3. **Violation of the Substrate Integrity Score (Rule 24):**
   A constitutional violation like CHEAT-001 indicates that the implementation lacks a genuine, independent mathematical oracle or is attempting to hide a failed branchless derivation. According to Rule 24, detecting fabricated verification or scanner evasion triggers an absolute failure (`SIS = 0`) and forces a `MaturityScrutiny` quarantine.

## Detection and Enforcement
- **Enforcement Tool:** `bcinr-cheat-scanner` (AST parsing and Text layers).
- **Scanner Signature:** Evaluates the Abstract Syntax Tree (AST) for binary expressions where operands are structurally identical and canceled out, or where operations lack a data-flow contribution to the function's return.
- **Penalty:** Immediate blocking of the merge. As with all absolute failures, it resets the SIS to 0, forcing a freeze on feature development until the structural defect is repaired and a new standing receipt is issued (Rule 25).
