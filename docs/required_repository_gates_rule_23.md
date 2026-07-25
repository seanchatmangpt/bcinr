# Rule 23: Required Repository Gates

According to Rule 23 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), any changes must pass a strict set of verification gates. 

## Required `cargo make` Commands

At a minimum, the following verification commands (or their admitted equivalents) must be executed:

- `cargo make scan-cheats`
- `cargo make contract-gate`
- `cargo make ci`
- `cargo make test-mutants`
- `cargo make audit-object-code`
- `cargo make verify-generated`

## The Jurisdiction Requirement

Rule 23 strictly states: **"A green command with incomplete jurisdiction is not evidence."**

When reporting the results of the above gates, it is not enough to simply state that the command passed (exit status 0). The report must explicitly prove that the task's **jurisdiction** included the files that were actually changed. 

### Why is this required?

1. **Prevention of "Gate-Jurisdiction Theater" (CHEAT-010):** A common loophole in CI/CD pipelines is a test or scanner returning a green status simply because it was not configured to look at the newly added or modified code. Rule 16 explicitly prohibits reporting a passing scanner that fails to inspect the relevant crate, file, generated output, feature set, or target.
2. **Absolute Verification:** The BCINR project relies on absolute determinism and mathematical proofs (e.g., CC=1, zero allocation, object-code audits). If a file evades these tests—whether through misconfiguration, macro hiding, or omitted paths—its compliance is unknown, and the substrate's integrity is compromised.
3. **Traceability:** By mandating that the test report lists the exact "files inspected," "features inspected," and "targets inspected," the constitution ensures that reviewers and the Enforcer (`@turing_machine`) have verifiable proof that the changed code was rigorously evaluated by the required gates before merge.
