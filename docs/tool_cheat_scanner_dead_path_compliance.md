### CHEAT-007 (Dead-path compliance) Detection

In `tools/bcinr-cheat-scanner/src/main.rs`, the detection logic for `CHEAT-007` is implemented as a simple string-matching check against the raw file source code, rather than parsing the Abstract Syntax Tree (AST). 

#### 1. Rule Definition
The rule is defined around line 96 with the following metadata:
```rust
        CheatRule {
            id: "CHEAT-007".to_string(),
            title: "DEAD_PATH_COMPLIANCE".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-007)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string()],
            authoritative_only: true,
            detection_contract: "Detects dead or unreachable code blocks displaying compliance while active path is not.".to_string(),
            required_fixture_ids: vec!["fixture_dead_path".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Remove dead paths and make the active hot path fully compliant.".to_string(),
        },
```
*Note that despite the rule declaring its layer as `"AST"`, the actual implementation operates purely on text.*

#### 2. Detection Implementation
The actual detection occurs inside the `scan_file_text_rules` function (around line 508):
```rust
    // CHEAT-007: DEAD_PATH_COMPLIANCE
    if src.contains("if false {") && src.contains("dummy_branchless") {
        findings.push(format!(
            "CHEAT[CHEAT-007]: {} — dead-path compliance: compliant dummy placed in unreachable if-false block",
            path.display()
        ));
    }
```

**How it works:**
The scanner checks if the raw source string (`src`) simultaneously contains both the literal string `"if false {"` and the literal string `"dummy_branchless"`. If both substrings are found anywhere in the file's text, it triggers a `CHEAT[CHEAT-007]` violation, assuming that a dummy compliant implementation has been hidden in an unreachable code block.
