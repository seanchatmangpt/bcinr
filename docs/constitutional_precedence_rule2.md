# Rule 2: Constitutional Precedence

In the BCINR repository, **Rule 2 (Constitutional Precedence)** establishes a strict, non-negotiable hierarchy for resolving conflicting instructions during development. The core governing principle is **"Rich semantics upstream. Fixed deterministic mechanics downstream."**

When instructions or methodologies conflict, the following order of precedence is strictly applied (highest to lowest priority):

1. **Mathematical safety and typed refusal**
2. **`AGENTS.md`** (The Constitution)
3. **Repository contract gates**
4. **Crate-local architecture documents**
5. **Issue or task requirements**
6. **Agent preferences**
7. **Implementation convenience**

**Core Rule:** *No agent may weaken a higher-order rule to satisfy a lower-order objective.*

## Enforcement of Mathematical Safety over Convenience, Speed, and Idiomatic Code

The constitution explicitly invalidates justifications such as **"faster," "simpler," "idiomatic,"** or **"the compiler will optimize it"** when they conflict with higher-order laws. These appeals invariably fall under **Rank 6 (Agent preferences)** or **Rank 7 (Implementation convenience)**. 

If an optimization or an idiomatic Rust pattern introduces a hidden branch, an allocation, or deviates from the mathematical specification by even a single bit, it fundamentally violates **Rank 1 (Mathematical safety and typed refusal)**. 

The codebase strictly enforces this hierarchy through the following mechanisms:

- **Immediate Rejection by Precedence:** A merge request prioritizing performance or idiomatic code over mathematical safety is considered fundamentally unconstitutional. 
- **Zero Warning Policy (Merge Blocking):** There are no warning-only violations. Any violation of a higher-order rule blocks the merge entirely.
- **Automated Gatekeeper Enforcement:** The structural enforcer (`@turing_machine`) uses tools like `bcinr-cheat-scanner` and exact production-profile disassembly audits to verify structural branchlessness (`CC=1`). If an implementation hides branches in abstractions for the sake of simplicity, the automated structural audits will reject it.
- **Substrate Integrity Score (SIS) Collapse:** Compromising mathematical safety for the sake of speed (e.g., bypassing a bounds-checking mask) forces the repository's Substrate Integrity Score (SIS) to `0`. This triggers a mandatory `MaturityScrutiny` protocol, which freezes all feature development until the structural defect is repaired and all dependent artifacts are regenerated.

### Concrete Examples

1. **The "Faster" Implementation:** If a developer removes a bounds-checking mask from a fixed-point division algorithm to make it "faster" in micro-benchmarks, it is rejected. "Faster" ranks 7th, while mathematical safety (rank 1) requires the mask.
2. **The "Simpler" Idiomatic Code:** If an agent uses an `if let` statement to handle an `Option` to make the code "simpler" and more idiomatic, it is blocked. `if let` introduces a control-flow branch, violating the `AGENTS.md` (rank 2) mandate for $CC=1$, rendering the idiomatic preference (rank 6) irrelevant.
3. **The "Compiler Will Optimize It" Defense:** Writing a loop with variable bounds under the assumption that the compiler will unroll it in release mode is rejected. Relying on the compiler is not a mathematical proof. Mathematical safety (rank 1) and `AGENTS.md` (rank 2) demand structural branchlessness and explicit, bounded execution, overriding any crate-local architecture (rank 4).
