# Rule 23: Required Repository Gates

According to Rule 23 of `AGENTS.md`, the repository has a strict set of required gates that must be executed to verify the deterministic substrate's integrity.

## Minimum Required `cargo make` Commands

At a minimum, the following admitted equivalents of these commands must be executed:

- `cargo make scan-cheats`
- `cargo make contract-gate`
- `cargo make ci`
- `cargo make test-mutants`
- `cargo make audit-object-code`
- `cargo make verify-generated`

## Proving Task Jurisdiction

Rule 23 explicitly states: *"Before reporting results, prove each task’s jurisdiction includes the changed files."* 

Furthermore, *"A green command with incomplete jurisdiction is not evidence."*

### What this means:
Proving task jurisdiction means you must definitively demonstrate that the automated gates (scanners, tests, and auditors) actually evaluated the specific code, files, crates, or targets that were modified. Simply running a command and receiving a successful (green) exit status is meaningless if the newly introduced or modified code was implicitly or explicitly excluded from the tool's scope. 

This prevents an anti-pattern known in the constitution as **CHEAT-010 — Gate-jurisdiction theater**, which is defined as *"Reporting a passing scanner that does not inspect the relevant crate, file, generated output, feature set, or target."*

To prove jurisdiction and establish that a "green" result is valid evidence, every report must explicitly confirm exactly what was evaluated. A valid report for any of these required gates must explicitly state:

- **command**: The exact command executed.
- **exit status**: The result/success of the command.
- **files inspected**: Explicitly listing the files that were part of the tool's run (must include the changed files).
- **features inspected**: The feature flags tested.
- **targets inspected**: The compilation targets evaluated.
- **findings**: Any results or violations discovered.
- **artifact digest**: The cryptographic digest of the outputs.
