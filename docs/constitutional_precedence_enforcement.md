# Constitutional Precedence and Enforcement (Rule 2)

In the BCINR repository, **Constitutional Precedence** (Rule 2 of `AGENTS.md`) establishes a strict, non-negotiable hierarchy for resolving conflicting instructions during development and merge requests.

## The Hierarchy of Precedence

When instructions conflict, the following order of authority must be applied:
1. Mathematical safety and typed refusal
2. `AGENTS.md` (The Constitution)
3. Repository contract gates
4. Crate-local architecture documents
5. Issue or task requirements
6. Agent preferences
7. Implementation convenience

**Core Rule:** *No agent may weaken a higher-order rule to satisfy a lower-order objective.*

## Handling Claims of "Faster," "Simpler," or "Idiomatic"

When an agent or contributor attempts to justify a violation of mathematical safety (Rank 1) or repository contract gates (Rank 3) by claiming the code is "faster," "simpler," "idiomatic," or that "the compiler will optimize it":

1. **Immediate Rejection by Precedence:** The constitution explicitly invalidates these justifications. Such claims fall under "Implementation convenience" (Rank 7) or "Agent preferences" (Rank 6), which are at the absolute bottom of the precedence hierarchy.
2. **Merge Blocking (Zero Warning Policy):** The overarching enforcement policy of the repository dictates that **violations block merge**. There are no warning-only violations. A merge request prioritizing performance or idiomatic Rust over mathematical safety is fundamentally unconstitutional and cannot be merged.
3. **Gatekeeper Intervention:** The `@turing_machine` (Enforcer of Determinism) acts as the structural auditor and merge gatekeeper. It enforces the constitutional laws using automated gating (e.g., `bcinr-cheat-scanner`, disassembly audits) and will reject the merge if higher-order invariants are compromised for optimization.
4. **Substrate Integrity Score (SIS) Collapse:** Compromising mathematical safety or deterministic mechanics for speed often involves introducing hidden branches, bounds checks, or allocations. These constitute absolute failures that instantly force the repository's `SIS` to `0`. This not only blocks the merge but freezes feature development and triggers the mandatory `MaturityScrutiny` protocol.
