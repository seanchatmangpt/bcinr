Based on the inspection of `tools/bcinr-cheat-scanner/src/main.rs`, the cheat scanner detects `CHEAT-004` (Artificial file inflation) using a text-based scan in the `scan_file_text_rules` function.

It flags a file for `CHEAT-004` if it matches either of the following two conditions:

1. **Explicit Sentinel String**: The source code contains the exact string `"PADDING ENSURING FILE LENGTH REQUIREMENT"`.
2. **Consecutive Numbered Padding Comments**: The file contains 5 or more consecutive single-line comments (`//`) that contain the substring `". Line"`. The scanner tracks this by incrementing a counter (`consecutive_padding`) for each matching comment line and resetting the counter to 0 whenever a line does not match.

Here is the exact Rust implementation handling this detection:

```rust
// CHEAT-004: ARTIFICIAL_FILE_INFLATION
if src.contains("PADDING ENSURING FILE LENGTH REQUIREMENT") {
    findings.push(format!(
        "CHEAT[CHEAT-004]: {} — artificial file-length inflation detected",
        path.display()
    ));
}
let mut consecutive_padding = 0;
for line in src.lines() {
    if line.trim().starts_with("//") {
        let after_slashes = line.trim()[2..].trim();
        if after_slashes.contains(". Line") {
            consecutive_padding += 1;
            if consecutive_padding >= 5 {
                findings.push(format!(
                    "CHEAT[CHEAT-004]: {} — numbered padding block detected",
                    path.display()
                ));
                break;
            }
        }
    } else {
        consecutive_padding = 0;
    }
}
```
