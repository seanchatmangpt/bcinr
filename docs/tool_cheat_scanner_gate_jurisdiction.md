# CHEAT-010 Detection (Gate-Jurisdiction Theater)

The `bcinr-cheat-scanner` detects `CHEAT-010` (Gate-jurisdiction theater) by performing a self-audit on its own source code to ensure that it has not been modified to skip essential directories.

Specifically, in `tools/bcinr-cheat-scanner/src/main.rs`, the scanner implements a function `check_gate_jurisdiction_theater` that does the following:

1. It reads its own source file (`tools/bcinr-cheat-scanner/src/main.rs`) into a string.
2. It checks if the source code contains the strings `"crates/bcinr-logic"` and `"crates/bcinr-cmca"`.
3. If either of these strings is missing, it triggers the violation.

### Source Code Reference

```rust
fn check_gate_jurisdiction_theater(findings: &mut Vec<String>) {
    // CHEAT-010: GATE_JURISDICTION_THEATER
    // Check if bcinr-cheat-scanner search roots omit either crates/bcinr-logic or crates/bcinr-cmca.
    let scanner_src = match fs::read_to_string("tools/bcinr-cheat-scanner/src/main.rs") {
        Ok(s) => s,
        Err(_) => return,
    };
    if !scanner_src.contains("crates/bcinr-logic") || !scanner_src.contains("crates/bcinr-cmca") {
        findings.push("CHEAT[CHEAT-010]: tools/bcinr-cheat-scanner/src/main.rs — scanner ignores logic or cmca crates".to_string());
    }
}
```

By ensuring its own source code explicitly mentions these specific crates, the scanner prevents a developer from secretly removing `bcinr-logic` or `bcinr-cmca` from the search paths (which would result in an incomplete gate jurisdiction).
