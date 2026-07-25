# Rule 23: Required Repository Gates

Based on `AGENTS.md` and the repository documentation, Rule 23 establishes the absolute minimum mechanical verification threshold that any change must pass before being considered valid. The authoritative runtime's integrity is guaranteed entirely by an unbroken matrix of these gating checks.

At a minimum, the repository requires executing the admitted equivalents of the following six gates:

1. `cargo make scan-cheats`
2. `cargo make contract-gate`
3. `cargo make ci`
4. `cargo make test-mutants`
5. `cargo make audit-object-code`
6. `cargo make verify-generated`

Before reporting results, you must prove each task’s jurisdiction includes the changed files. A green command with incomplete jurisdiction is not evidence. The final evidence report must state:
- `command`
- `exit status`
- `files inspected`
- `features inspected`
- `targets inspected`
- `findings`
- `artifact digest`

### The `contract-gate` Task

**`cargo make contract-gate`** validates branchless mathematical contract compliance. 

It acts as the Enforcer (`@turing_machine`) and is responsible for enforcing the following in the pipeline:
- **Radon Law Compliance:** Ensures every authoritative function strictly adheres to Cyclomatic Complexity $CC=1$.
- **Branchless Logic:** Enforces that logic must be expressed as bitwise polynomials (mask-based execution).
- **No Hidden Control Flow:** Verifies the absolute absence of hidden branches, data-dependent loops, panic paths, or unwinding.

By enforcing these constraints, the `contract-gate` guarantees that the implementation perfectly adheres to the deterministic, branchless, and allocation-free computational substrate mission of the project.
