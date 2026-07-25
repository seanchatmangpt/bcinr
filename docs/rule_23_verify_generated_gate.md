Here is the requested information about the `cargo make verify-generated` gate and its relationship to the Generated-code law based on the `AGENTS.md` file:

### `cargo make verify-generated` Gate

Under **Rule 23 (Required repository gates)**, `cargo make verify-generated` is one of the mandatory gates that must be executed to establish repository standing. When reporting the results of this gate, one must prove its jurisdiction includes the changed files and report the command, exit status, files/features/targets inspected, findings, and artifact digest. A successful execution (a "green command") with incomplete jurisdiction is not considered evidence.

### Relationship to the Generated-code law

This gate is the mechanical enforcement mechanism for **Rule 21 (Generated-code law)**. 

The Generated-code law requires that all generated code must be strictly **reproducible**. The `cargo make verify-generated` gate ensures this by enforcing the required lifecycle process:
1. Clean generation
2. Digest output
3. Regenerate
4. Verify byte-identical output

By enforcing byte-for-byte reproducibility, the gate ensures compliance with several key restrictions in Rule 21:
- **No Hand-editing:** Hand-editing generated output is strictly prohibited. The gate prevents this by verifying that regenerating the files overwrites or matches any manual modifications.
- **No Unexplained Drift:** Generated files with unexplained drift invalidate the repository standing. The verification process acts as the automated check against this drift.

Additionally, generated authoritative code is not exempt from the core substrate laws and must also bind to source graph and certificate digests, pass the cheat scanner, pass `CC=1`, and pass disassembly inspection.
