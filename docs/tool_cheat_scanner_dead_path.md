Here is the documentation on how the cheat scanner detects `CHEAT-007` and `CHEAT-010`:

### `CHEAT-007` (Dead-path compliance)
The cheat scanner checks the source code of each file for the simultaneous presence of two specific substrings:
- `"if false {"`
- `"dummy_branchless"`

If both of these strings are found in a file, it triggers the detection and logs the following finding:
```text
CHEAT[CHEAT-007]: <file_path> — dead-path compliance: compliant dummy placed in unreachable if-false block
```
*(Reference: `tools/bcinr-cheat-scanner/src/main.rs`, lines 508-514)*

### `CHEAT-010` (Gate-jurisdiction theater)
The scanner has a dedicated function `check_gate_jurisdiction_theater` that checks the source code of the scanner itself (`tools/bcinr-cheat-scanner/src/main.rs`). It verifies that the scanner hasn't been modified to ignore critical crates.

Specifically, it reads the scanner's own source file and checks if either of the following substrings is missing:
- `"crates/bcinr-logic"`
- `"crates/bcinr-cmca"`

If either string is missing, it means the scanner is ignoring one of the foundational crates, and it logs:
```text
CHEAT[CHEAT-010]: tools/bcinr-cheat-scanner/src/main.rs — scanner ignores logic or cmca crates
```
*(Reference: `tools/bcinr-cheat-scanner/src/main.rs`, lines 583-593)*
