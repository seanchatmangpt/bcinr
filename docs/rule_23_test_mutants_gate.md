# `cargo make test-mutants` Gate Enforcement and the Hostile Verification Workstream

Based on the `AGENTS.md` Constitution, here is an overview of the `cargo make test-mutants` gate and its relationship to the Hostile verification workstream.

## What the Gate Enforces (Rule 23 & 19)
The `cargo make test-mutants` gate is one of the repository's required gates. It enforces the execution and verification of hostile mutants against the project's source code:
*   **Adversarial execution**: Runs the test suite against explicitly designed, syntactically plausible hostile mutants injected into the build path.
*   **Typed refusals**: Ensures the test suite detects corruption by verifying a *specific contract violation* or *typed refusal* occurs, rather than just asserting simple inequality (`assert_ne!`).
*   **Zero survival**: Any surviving mutant causes an absolute failure, changing the project's standing to `MUTATION_GATE_FAILED` and blocking all feature work.
*   **Jurisdiction reporting**: Requires proof that the tested mutants cover the modified files, along with details like inspected features, targets, findings, and artifact digests.

## Relationship to the Hostile Verification Workstream (Rule 4 & 5)
The Hostile verification workstream is one of the four mandatory independent workstreams for any nontrivial feature.
*   **Owner**: `@armstrong_fault` (Master of Failure Law).
*   **Responsibility**: This workstream is solely responsible for designing the counterfactual mutants, negative-domain tests, and refusal expectations that the `test-mutants` gate executes.
*   **Protocol**: Under Rule 19, the workstream must identify at least three load-bearing laws per implementation file, produce a mutant for each, verify they are killed by the test suite (via typed refusals), and record the "kill evidence" in a mutant ledger.
*   **Independence**: The workstream operates independently of the implementation owner (`@von_neumann_bypass`), preventing the implementer from deriving expected results or self-certifying against their own logic.
