# CHEAT-001: Self-Canceling Operations

## What is CHEAT-001?
Under Rule 16 of the BCINR Constitution (the Anti-Cheat Manifesto), **CHEAT-001** strictly forbids the use of "Self-canceling operations." This rule applies to any instruction or arithmetic expression that is functionally inert and makes no contractual contribution to the final output.

Examples of such operations include:
- `A ^ A`
- `A - A`
- `a.wrapping_add(b) ^ a`

## Why are they prohibited?
In BCINR, the core mandate is to provide a bounded, branchless, allocation-free execution environment—a "deterministic computational substrate." Every authoritative primitive must be backed by a mathematical contract defined by the `@hoare_oracle` and possess a structurally lawful implementation.

Operations that do not contractually contribute to the output are prohibited because:
1. **The Deterministic Substrate Contract:** Every single instruction must map precisely to a proven mathematical contract (Rule 1). If an operation does not advance the state toward the mathematical postcondition, it is illegitimate.
2. **Purity of the Authoritative Call Graph:** BCINR mandates rigorous object-code audits (Rule 20) and fixed bounded execution work. Superfluous logic pollutes the binary, pollutes the call graph, and obscures the true nature of the implementation.

## How are they used to fake complexity?
Self-canceling operations are an algorithmic anti-pattern used to create **apparent complexity theater**.

Because BCINR mandates Cyclomatic Complexity $CC=1$ and zero data-dependent branches, developers might attempt to artificially inflate the structural appearance of a function by injecting dummy logic. By adding meaningless but valid-looking bit-parallel operations or arithmetic (like `a.wrapping_add(b) ^ a`), an author might attempt to:
- Make trivial, naive logic appear as advanced branchless arithmetic (e.g., SWAR or SIMD construction) to satisfy the `@turing_machine` role or bypass structural audits.
- Satisfy visual expectations of complexity or inflate the codebase without performing any necessary mathematical work.
- Mask incomplete or functionally deficient code under layers of convoluted but self-negating bitwise operations.

## Enforcement
To combat this, the `@turing_machine` role oversees the **`bcinr-cheat-scanner`**. The scanner does not just rely on regex text-matching; it traverses the full Abstract Syntax Tree (AST) using `syn::visit::Visit`. By analyzing `Expr::Binary` nodes, it isolates the left-hand and right-hand AST nodes (e.g., `A ^ A`), stringifies them, strips whitespace, and structurally compares them. 

If a self-canceling operation is detected, the scanner emits a `CHEAT[CHEAT-001]` finding containing the exact file and matched expression. Because BCINR does not permit warning-only violations, this immediately drops the Substrate Integrity Score (SIS) to 0 and hard-blocks the CI/CD merge gate.
