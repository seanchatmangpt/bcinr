Here is exactly how `bcinr-cheat-scanner` inspects for `CHEAT-006` (Scanner evasion) in `tools/bcinr-cheat-scanner/src/main.rs`:

The scanner uses the `syn` crate for AST traversal and inspects macro definitions (`macro_rules!`) which might be used to hide branches or control flow.

1. **AST Traversal of Macros:** The visitor implements the `visit_item_macro` function to examine every macro definition in the AST.
2. **Stringification (String Construction):** It takes the macro's AST node and constructs a normalized string representation using the `quote` crate: `let mac_str = quote::quote!(#i).to_string();`. This converts the syntax tree back into text, eliminating superficial whitespace formatting or splitting that could evade raw text regexes.
3. **Token Analysis:** It uses a custom `has_token` helper function that splits the stringified macro text by any non-alphanumeric character (excluding underscores) to reliably match exact tokens.
4. **Keyword Detection:** If it detects `"if"` or `"match"` tokens inside the stringified macro, it flags a `CHEAT-006` violation: `macro hides control flow or branches (scanner evasion)`.

### Code Implementation from `main.rs`

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

```rust
fn has_token(text: &str, token: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|t| t == token)
}
```

`AGENTS.md` explicitly calls out _“string construction that produces prohibited source after generation”_ and _"using macro indirection to hide a pattern"_ as forms of `CHEAT-006` evasion. The scanner mitigates this by directly evaluating the generated/stringified macro bodies to prevent hidden branches.
