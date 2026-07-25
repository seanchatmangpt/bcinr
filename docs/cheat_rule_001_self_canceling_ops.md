# CHEAT-001: Self-Canceling Operations

**Constitutional Clause:** Rule 16 (Anti-cheat manifesto), *AGENTS.md*

## What Constitutes CHEAT-001?
CHEAT-001 strictly prohibits the use of self-canceling expressions in production and verification code. A self-canceling operation is any expression that mathematically negates itself or provides no meaningful change to the underlying state, included solely to create apparent complexity.

**Examples of Violations:**
```rust
// An operation where terms cancel out:
a.wrapping_add(b) ^ a

// Direct self-cancellation:
(x) ^ (x)
```

## Why is it Prohibited?
Under the **BCINR Deterministic Substrate Constitution**, all operations must have a strict, mathematical justification. The prohibition is grounded in the following principles:

1. **No Contractual Contribution:** The constitution mandates that *any operation without a contractual contribution to the output is prohibited.* In the BCINR architecture, logic is expressed as exact bitwise polynomials. Every instruction in the hot-path must structurally correspond to an algebraic law defined in the mathematical contract (authored by `@hoare_oracle`).
2. **Artificial Complexity:** These operations are typically introduced to artificially inflate code complexity, pad out execution, or create the illusion of sophisticated logic without actually advancing the state.
3. **Scanner Evasion & Theater:** Adding self-canceling operations is considered a form of metric theater or scanner evasion (akin to other cheats like `CHEAT-004` or `CHEAT-008`). It obscures the true authoritative call graph, which undermines the rigorous `CC=1` (Cyclomatic Complexity = 1) enforcement and deterministic object-code audits. 

## Detection Mechanism
The `bcinr-cheat-scanner` enforces this rule mechanically at the **AST (Abstract Syntax Tree)** and **Text** layers. 

During the `cargo make scan-cheats` phase, the scanner parses the AST to detect binary expressions where the Left-Hand Side (LHS) and Right-Hand Side (RHS) operands are structurally identical or are arranged in a way that is mathematically canceled out by the operator. Any detected violation immediately blocks the merge, marks a constitutional violation, and forces the Substrate Integrity Score (SIS) to 0.
