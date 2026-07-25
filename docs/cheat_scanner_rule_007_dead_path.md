I have investigated how `CHEAT-007` (Dead-path compliance) is detected in the `bcinr-cheat-scanner`.

The scanner detects `CHEAT-007` using a simple textual substring match on the contents of the scanned source files. Specifically, it flags a file if its source code simultaneously contains the exact strings `"if false {"` and `"dummy_branchless"`. 

This detects attempts to trick the system by placing branchless "compliant dummy" code inside a completely unreachable `if false` block, which might otherwise allow unlawful code in the active path to evade human scrutiny or pass certain naïve compliance tests.

Here is the exact detection logic from `tools/bcinr-cheat-scanner/src/main.rs`:

```rust
    // CHEAT-007: DEAD_PATH_COMPLIANCE
    if src.contains("if false {") && src.contains("dummy_branchless") {
        findings.push(format!(
            "CHEAT[CHEAT-007]: {} — dead-path compliance: compliant dummy placed in unreachable if-false block",
            path.display()
        ));
    }
```
