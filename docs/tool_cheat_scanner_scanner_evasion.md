# Detection of CHEAT-006 in `bcinr-cheat-scanner`

The `CHEAT-006` (Scanner evasion) rule is implemented in `tools/bcinr-cheat-scanner/src/main.rs`. Although the rule's description mentions detecting obfuscated operators, macro-nested control flow, or formatting evasion, the actual implementation focuses specifically on **detecting branches hidden within macro definitions**.

## Implementation Details

The detection uses an Abstract Syntax Tree (AST) visit via the `syn` crate.

1. **Macro Declaration Visit**: The `SynCheatVisitor` implements `visit_item_macro` to inspect all `syn::ItemMacro` nodes.
2. **`macro_rules` Check**: The scanner checks if the macro is being defined using `macro_rules`.
3. **Stringification**: The entire macro item is converted back to a string using `quote::quote!(#i).to_string()`.
4. **Tokenization and Matching**: The stringified macro is passed to a helper function `has_token()`, which splits the string by any character that is not alphanumeric or an underscore (i.e. `!c.is_alphanumeric() && c != '_’`).
5. **Forbidden Tokens**: It then checks if any of the resulting tokens exactly match `"if"` or `"match"`.

If either `"if"` or `"match"` is found inside the `macro_rules` definition, it flags the violation and reports:
`"CHEAT[CHEAT-006]: <path> — macro hides control flow or branches (scanner evasion)"`

## Code Snippet

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

fn has_token(text: &str, token: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|t| t == token)
}
```

Notably, while the description mentions split tokens, the current code in `main.rs` does not contain explicit logic to catch operators split across multiple lines or comments within tokens. It relies solely on `quote::quote!` normalizing the AST back to a string, and then tokenizing it to look for control flow keywords in macros.
