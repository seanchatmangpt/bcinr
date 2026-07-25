Here is the documentation detailing how the `bcinr-cheat-scanner` detects `CHEAT-004` and `CHEAT-005`, based on `tools/bcinr-cheat-scanner/src/main.rs`:

### CHEAT-004: Artificial File Inflation
The cheat scanner uses a text-based scan (`scan_file_text_rules`) to detect artificial padding in two ways:
1. **Explicit Sentinel Phrase**: It checks if the source file contains the exact string `"PADDING ENSURING FILE LENGTH REQUIREMENT"`. If found anywhere, it triggers a violation.
2. **Consecutive Numbered Comments**: It iterates through the file line-by-line looking for consecutive comments that contain `". Line"`. Specifically:
   - A line must start with `//` (ignoring leading whitespace).
   - The text after the `//` must contain the string `". Line"`.
   - If it detects **5 or more consecutive lines** matching this pattern, it triggers a "numbered padding block detected" violation. The counter resets if any line breaks the sequence.

### CHEAT-005: Boilerplate Verification Claims
The scanner also uses a text-based scan to detect repetitive, boilerplate verification claims:
- It iterates through the file line-by-line and looks for lines that contain **both** of the following exact strings:
  1. `"Hoare-logic Verification Line"`
  2. `"Branchless path is the unique solution to the state constraints of"`
- It counts the total number of such lines in the file (they do not need to be consecutive).
- If the file contains **5 or more occurrences** of these lines, it triggers a "mock Hoare-logic verification claims detected" violation.
