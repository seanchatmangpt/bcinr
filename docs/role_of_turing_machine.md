# The Role of `@turing_machine` (Enforcer of Determinism)

Based on the `AGENTS.md` constitution, `@turing_machine` serves as the **Structural auditor and merge gatekeeper**. Their primary mission is to guarantee that the authoritative runtime remains purely deterministic, bounded, and completely branchless.

## Exclusive Authority

The Enforcer of Determinism holds exclusive authority over structural integrity gates, specifically:

- **Cyclomatic-Complexity Enforcement**: They verify that every single authoritative function adheres strictly to a Cyclomatic Complexity (CC) of 1. This means absolutely zero data-dependent branches, `if` statements, `match` blocks, or dynamic loop terminations anywhere in the authoritative path.
- **Cheat-Scanner Policy**: They define and oversee the rules for the `bcinr-cheat-scanner`, ensuring that no prohibited constructs (e.g., scanner evasion, hidden operators, dead-path compliance) bypass structural checks.
- **Object-Code Audits**: Source-level `CC=1` claims are necessary but insufficient ("Source claims do not substitute for disassembly evidence"). `@turing_machine` audits the exact production-profile disassembly to verify that no conditional jumps, loop backedges, panic paths, or allocator symbols exist in the final machine code.

Their exclusive authority also extends to authoritative-call-graph classification, source audits, panic-path audits, allocation audits, and gate-jurisdiction audits.

## Scanning Generated Rust and Macro Expansions

`@turing_machine` is required to scan all macro expansions, private functions, and generated Rust code because the whole-call-graph branchlessness rule applies transitively. 
- **Hidden Branches**: Branches hidden inside macro expansions or trait implementations explicitly count as violations. 
- **Scanner Evasion**: Developers might attempt to hide prohibited patterns using macro indirection or string construction that produces prohibited source after generation (classified as CHEAT-006).
- **Generated Code Accountability**: Generated code is executed by the runtime and is not exempt from the absolute runtime laws. It must pass all structural gates, pass `CC=1`, and contain no hidden branches or unbounded execution.

## Core Standard for Instruction Shape

The fundamental standard enforced by `@turing_machine` is encapsulated in this axiomatic law:

> **"The authoritative instruction shape must not depend on semantic input."**

This mandates that execution must follow a fixed deterministic structure, translating sequential semantic decisions into masks and bit-parallel arithmetic, without altering the actual sequence of machine instructions based on the data being processed.
