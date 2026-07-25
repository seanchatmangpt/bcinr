# CHEAT-003 (Magic Constants) Scanner Implementation

The `bcinr-cheat-scanner` detects `CHEAT-003` (Magic Constants like `0xDEADBEEF` and `0xCAFEBABE`) using a two-layered approach in [`/Users/sac/bcinr/tools/bcinr-cheat-scanner/src/main.rs`](file:///Users/sac/bcinr/tools/bcinr-cheat-scanner/src/main.rs): 

## 1. AST-Based Verification (Syntax Tree)
The scanner uses the `syn` crate to parse the source code into an Abstract Syntax Tree (AST) and visits every expression looking for integer literals:

```rust
// CHEAT-003: MAGIC_CONSTANTS (AST check for numeric literals)
if let Expr::Lit(l) = i {
    if let syn::Lit::Int(li) = &l.lit {
        if let Ok(val) = li.base10_parse::<u64>() {
            if val == 3735928559 || val == 3405691582 {
                self.findings.push(format!(
                    "CHEAT[CHEAT-003]: {} — magic constant literal detected: 0x{:X}",
                    self.path.display(),
                    val
                ));
            }
        }
    }
}
```
**How it normalizes tokens:**
The `syn::Lit::Int::base10_parse::<u64>()` method converts the parsed integer literal directly into its numeric value (a `u64`). This bypasses textual variations completely—whether the constant is written as `0xDEADBEEF`, `0xdead_beef`, or its decimal equivalent `3735928559`, the AST parser normalizes them all into the exact same numeric value for comparison.

## 2. Text-Based Verification (Doc Comments & Text Scan)
Because the AST parser won't catch magic constants written inside comments or documentation, the scanner implements a fallback text line scanner:

```rust
// CHEAT-003: MAGIC_CONSTANTS (Doc comment or text scan)
if !is_test {
    let mut in_test_module = false;
    let mut test_depth = 0;
    // ... test module skipping logic ...
    
    let text_no_us = line.replace("_", "").to_lowercase();
    if text_no_us.contains("0xdeadbeef") || text_no_us.contains("0xcafebabe") {
        findings.push(format!(
            "CHEAT[CHEAT-003]: {} — magic constant found in file text/doc comments",
            path.display()
        ));
        break;
    }
}
```
**How it normalizes text:**
For each line outside of test files (`/tests/`, `/benches/`) or inline `mod tests { ... }` blocks, it normalizes the text by:
1. Stripping all underscores `_` using `replace("_", "")` (e.g., flattening `0xDEAD_BEEF` into `0xDEADBEEF`).
2. Converting the entire line to lowercase via `to_lowercase()`.
3. Checking if the resulting normalized string contains `0xdeadbeef` or `0xcafebabe`.
