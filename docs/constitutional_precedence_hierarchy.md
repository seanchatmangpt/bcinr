# Constitutional Precedence Hierarchy

According to **Rule 2 (Constitutional precedence)** in `AGENTS.md`, the BCINR project enforces a strict, unyielding hierarchy of rules. When instructions, goals, or methodologies conflict, the following order of precedence strictly applies:

1. **Mathematical safety and typed refusal**
2. **`AGENTS.md`** (The Constitution)
3. **Repository contract gates**
4. **Crate-local architecture documents**
5. **Issue or task requirements**
6. **Agent preferences**
7. **Implementation convenience**

## The Strict Hierarchy Explained

This hierarchy guarantees that foundational substrate properties—such as determinism, branchlessness, and provability—are never compromised for the sake of downstream conveniences or specific feature requirements. 

- **No Weakening:** No agent or developer is permitted to weaken or bypass a higher-order rule to satisfy a lower-order objective. 
- **Absolute Enforcement:** Claims such as “faster,” “simpler,” “idiomatic,” or assurances that “the compiler will optimize it” explicitly **do not** override this constitution.

## Theoretical Examples of Precedence Conflicts

### Example 1: The "Faster" Implementation
**Scenario:** A developer submits a pull request optimizing a fixed-point division algorithm. They remove the bounds-checking mask to avoid a bitwise operation, claiming the new algorithm is significantly "faster" in micro-benchmarks. 
**Outcome:** The PR is unequivocally rejected. The claim of being "faster" falls under rank 7 (Implementation convenience) or rank 5 (Issue or task requirements). Removing the bounds-checking mask violates rank 1 (Mathematical safety and typed refusal). The higher-order rule strictly forbids sacrificing mathematical safety for execution speed.

### Example 2: The "Simpler" Idiomatic Rust Code
**Scenario:** An agent is tasked with writing a state transition function. To make the code "simpler" and more idiomatic, the agent uses an `if let` statement to handle an `Option`, arguing that it makes the codebase more readable for future maintainers.
**Outcome:** The code is blocked by the structural enforcer (`@turing_machine`). While idiomatic Rust might satisfy rank 6 (Agent preferences) or rank 7 (Implementation convenience), the `if let` introduces a control-flow branch. This violates rank 2 (`AGENTS.md` rule mandating `CC=1` and no data-dependent branches). The constitution explicitly nullifies the "simpler" defense.

### Example 3: The "Compiler Will Optimize It" Defense
**Scenario:** A developer writes a loop with variable bounds (`for item in variable_slice`) because it aligns perfectly with the crate-local architecture document (rank 4). When challenged on the branch, they claim "the compiler will unroll it and optimize it away in release mode."
**Outcome:** The implementation fails the object-code audit and is rejected. Crate-local architecture (rank 4) cannot override `AGENTS.md` (rank 2), which explicitly bans unbounded execution and requires structural branchlessness. Trusting the compiler is not an accepted mathematical proof within BCINR.
