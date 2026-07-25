I have reviewed `AGENTS.md`, specifically Rule 19 on the **Hostile mutation protocol** and Rule 4 which outlines "truncation of a bounded table" as a meaningful law for mutants under the `@armstrong_fault` role.

### What is "Truncation of a Bounded Table" as a Mutant?

In the BCINR deterministic substrate, sequential control flow and semantic decisions must be transformed into fixed lookup tables, bit-parallel masks, and arithmetic selection to preserve branchlessness (the $CC=1$ rule). 

"Truncation of a bounded table" refers to an intentional, adversarial modification (a "mutant") where the length or capacity of one of these fixed-size lookup tables is artificially reduced. For example, if a branchless algorithm relies on a complete 256-element table for a mathematical mapping, the mutant would truncate this table (e.g., dropping the upper half or changing its defined bounded length).

### Why Testing Against It Is Required

According to the project's constitutional laws, a test suite is defective if it cannot kill a syntactically plausible mutant. Testing against table truncation is mandatory for several reasons:

1. **Validation of Branchless Clamping/Masking:** Because runtime bounds-check panics and branches (`if`, `match`) are strictly prohibited, indices into a table must be clamped or masked using pure arithmetic. A truncated table mutant verifies that these arithmetic bounds-checks are correctly implemented.
2. **Enforcement of Typed Refusals:** Rule 18 mandates that unsupported inputs must produce a bounded typed refusal (e.g., `UnsupportedDomain` or `NumericRangeExceeded`) rather than panicking, falling back, or returning a default value. Truncating the table ensures that out-of-bounds queries correctly and consistently produce the expected refusal path.
3. **Guarantee of Full-Domain Oracle Coverage:** The `@hoare_oracle` role requires that properties cover the entire mathematical domain. If a table can be truncated and the test suite still passes (a "surviving mutant"), it proves the tests do not actually cover the full domain, causing the project standing to fail (`MUTATION_GATE_FAILED`).
4. **Protection Against Silent Corruption:** In a system where state mutation is committed via masked selection, a flawed lookup could silently propagate invalid data into the persistent state. This mutant ensures that any violation of the table's bounds prevents the operation from mutating the state before complete admission.
