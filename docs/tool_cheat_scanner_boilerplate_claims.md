# How `bcinr-cheat-scanner` Detects CHEAT-005 (Boilerplate Verification Claims)

According to `tools/bcinr-cheat-scanner/src/main.rs`, the scanner detects **CHEAT-005** by searching for a specific pattern of repeated boilerplate comments intended to feign verification without actual substance.

### Detection Logic:

1.  **Line-by-Line Scanning:** The scanner iterates through every line of the source code.
2.  **Phrase Matching:** For each line, it checks if it contains both of the following strings:
    *   `"Hoare-logic Verification Line"`
    *   `"Branchless path is the unique solution to the state constraints of"`
3.  **Threshold Counting:** It maintains a counter (`hoare_count`) that increments each time a line matching both phrases is found.
4.  **Violation Trigger:** If the counter reaches **5**, it triggers a `CHEAT-005` violation, recording a finding formatted as:
    `CHEAT[CHEAT-005]: <file-path> — mock Hoare-logic verification claims detected`
5.  **Early Exit:** Once 5 instances are found in a single file, the scanner stops checking that file for CHEAT-005 and moves on.

### Source Code Snippet (from `tools/bcinr-cheat-scanner/src/main.rs`):
```rust
    // CHEAT-005: BOILERPLATE_VERIFICATION_CLAIMS
    let mut hoare_count = 0;
    for line in src.lines() {
        if line.contains("Hoare-logic Verification Line")
            && line.contains("Branchless path is the unique solution to the state constraints of")
        {
            hoare_count += 1;
            if hoare_count >= 5 {
                findings.push(format!(
                    "CHEAT[CHEAT-005]: {} — mock Hoare-logic verification claims detected",
                    path.display()
                ));
                break;
            }
        }
    }
```
