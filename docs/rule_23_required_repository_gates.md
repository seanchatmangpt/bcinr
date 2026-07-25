# Rule 23: Required Repository Gates

At minimum, the repository requires executing the admitted equivalents of these 6 gates:

```bash
cargo make scan-cheats
cargo make contract-gate
cargo make ci
cargo make test-mutants
cargo make audit-object-code
cargo make verify-generated
```

## Proving Jurisdiction
Before reporting results, you must prove that each task’s jurisdiction includes the changed files. A green command (successful exit status) with incomplete jurisdiction is not valid evidence.

## Final Evidence Report Requirements
The final evidence report must explicitly state the following details:

```text
command
exit status
files inspected
features inspected
targets inspected
findings
artifact digest
```
