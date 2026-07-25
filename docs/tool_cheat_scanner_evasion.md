I have inspected `tools/bcinr-cheat-scanner/src/main.rs`. Here is exactly how the cheat scanner detects `CHEAT-006` and `CHEAT-002`:

### CHEAT-006: Scanner Evasion (Macro Indirection)
The scanner visits all macros defined in the AST (`syn::ItemMacro`). It detects macro evasion using this logic:
1. It specifically targets `macro_rules` definitions.
2. It converts the entire macro definition back into a string using the `quote` crate (`quote::quote!(#i).to_string()`).
3. It custom-tokenizes this string (splitting by non-alphanumeric characters, excluding `_`) and checks if the macro contains an exact token match for `if` or `match`.
4. If either token is found, it flags `CHEAT-006`, preventing developers from hiding branching or control flow inside macro expansions.

**Key Source Code Reference:**
```rust
fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
    // CHEAT-006: SCANNER_EVASION
    if let Some(ident) = &i.mac.path.get_ident() {
        if ident.to_string() == "macro_rules" {
            let mac_str = quote::quote!(#i).to_string();
            if has_token(&mac_str, "if") || has_token(&mac_str, "match") {
                self.findings.push(format!(
                    "CHEAT[CHEAT-006]: {} — macro hides control flow or branches (scanner evasion)",
                    self.path.display()
                ));
            }
        }
    }
    visit::visit_item_macro(self, i);
}
```

### CHEAT-002: Circular Oracle
The scanner compares function bodies to ensure independent oracles aren't just exact copies of the production implementations. It does this via:
1. During the AST pass, it collects all non-test function names and their bodies. The bodies are stringified via the `quote` crate, and all spaces are stripped (`body_str.replace(" ", "")`) for a normalized comparison.
2. It searches for any function name ending with `_reference` or `_oracle` (e.g., `calculate_oracle`).
3. It extracts the base name by trimming the suffix (e.g., `calculate`).
4. It then searches for another function in the same file with that exact base name.
5. If the normalized, space-stripped AST-string of the oracle/reference function matches the production implementation exactly, it flags a `CHEAT-002` circular oracle violation.

**Key Source Code Reference:**
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
