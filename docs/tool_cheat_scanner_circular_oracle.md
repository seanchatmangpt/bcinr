Here is the documentation on how `CHEAT-002` (Circular Oracle) is detected in the `bcinr-cheat-scanner`:

### How `CHEAT-002` (Circular Oracle) is Detected

The `bcinr-cheat-scanner` detects circular oracles using an Abstract Syntax Tree (AST) analysis of the Rust source files. The process works in three main steps:

1. **AST Parsing & Extraction:**
   The scanner uses the `syn` crate to parse each Rust file into an AST. A visitor (`SynCheatVisitor`) walks the tree to identify all functions (both standalone `ItemFn` and implementation methods `ImplItemFn`).

2. **Body Normalization:**
   For each function found, the visitor extracts the function block, converts it to a string representation using the `quote` crate, and removes all whitespace (`quote::quote!(#block).to_string().replace(" ", "")`). It stores a tuple of `(function_name, stringified_normalized_body)`.

3. **Pairwise Comparison:**
   The `check_circular_oracles` function processes this list of normalized functions:
   - It searches for any function whose name ends with the suffix `_reference` or `_oracle`.
   - It strips this suffix to determine the expected name of the production implementation (e.g., `compute_oracle` becomes `compute`).
   - It then checks if a function with that base name exists in the same file.
   - If the base function is found **and** its normalized stringified body is exactly identical to the oracle's normalized body, it raises a `CHEAT[CHEAT-002]` violation.

#### Code Reference from `tools/bcinr-cheat-scanner/src/main.rs`

```rust
fn check_circular_oracles(functions: &[(String, String)], path: &Path, findings: &mut Vec<String>) {
    // CHEAT-002: CIRCULAR_ORACLE
    for (name, body) in functions {
        if name.ends_with("_reference") || name.ends_with("_oracle") {
            let base_name = name
                .trim_end_matches("_reference")
                .trim_end_matches("_oracle");
            for (p_name, p_body) in functions {
                if p_name == base_name && body == p_body {
                    findings.push(format!(
                        "CHEAT[CHEAT-002]: {} — circular oracle: {} identical to implementation {}",
                        path.display(),
                        name,
                        p_name
                    ));
                }
            }
        }
    }
}
```

Because the bodies are compared after being parsed by `syn` and stripped of whitespace, formatting differences (like indentation or line breaks) will not bypass the detection. The oracle must be structurally and logically distinct (e.g. using a different mathematical model or approach) to avoid this violation.
