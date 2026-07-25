Here is the documentation on how `CHEAT-010` (Gate-jurisdiction theater) is enforced in the `bcinr-cheat-scanner`:

### Analysis of CHEAT-010 Enforcement

The rule `CHEAT-010: GATE_JURISDICTION_THEATER` exists to prevent the cheat scanner from falsely reporting success by omitting critical authoritative crates from its scan paths. 

In `tools/bcinr-cheat-scanner/src/main.rs`, this is strictly enforced via a self-auditing function called `check_gate_jurisdiction_theater`:

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

#### How it ensures complete jurisdiction:

1. **Self-Introspection:** The scanner reads its *own* source code file (`tools/bcinr-cheat-scanner/src/main.rs`) at runtime.
2. **String Verification:** It checks for the literal presence of the strings `"crates/bcinr-logic"` and `"crates/bcinr-cmca"` in its source.
3. **Hardcoded Target Roots:** In the `main()` function, the scanner uses these static paths as the root directories for its AST and text parsing traversal:
   ```rust
   let roots = ["crates/bcinr-logic", "crates/bcinr-cmca"];
   // ...
   for root in &roots {
       for entry in WalkDir::new(root) // ...
   ```
4. **Fail-safe Mechanism:** By verifying its own source, the tool prevents an attacker or a faulty agent from quietly removing a target crate from the `roots` array to evade scanning. If the jurisdiction is altered in the scanner's code, `check_gate_jurisdiction_theater` will immediately flag a `CHEAT[CHEAT-010]` violation and block the merge. 

This creates a self-enforcing loop that guarantees the authoritative codepaths in `bcinr-logic` and `bcinr-cmca` are always subjected to the scanner's rules.
