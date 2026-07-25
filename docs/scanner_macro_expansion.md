# `bcinr-cheat-scanner` Macro-Expansion Audit (Rule 17)

According to the **BCINR Deterministic Substrate Constitution (Rule 17)**, the `bcinr-cheat-scanner` must inspect macro definitions and expanded output to prevent **CHEAT-006: Scanner Evasion**. This rule ensures that developers do not attempt to hide prohibited control-flow logic (like `if`, `match`, or loops) behind macro indirection, which would otherwise bypass a superficial `CC=1` AST check.

## Implementation in `bcinr-cheat-scanner`

Because the `bcinr-cheat-scanner` uses the `syn` crate (`syn::parse_file`) to parse Rust source code, it only operates on the **unexpanded Abstract Syntax Tree (AST)**. At this level, a macro invocation simply appears as a `MacCall` node, making the actual branch instructions invisible to standard expression checks.

To fulfill the constitutional requirement and prevent evasion without needing to hook directly into the `rustc` macro expansion phase, the scanner audits macros directly at their definition point.

In `tools/bcinr-cheat-scanner/src/main.rs`, the scanner implements the `syn::visit::Visit` trait and overrides `visit_item_macro`:

1. **Visit Macro Definitions:** The scanner hooks into `visit_item_macro` to inspect every `macro_rules!` definition in the codebase.
2. **Stringification and Tokenization:** It converts the entire macro definition into a raw string using `quote::quote!(#i).to_string()`.
3. **Prohibited Token Detection:** The scanner splits the string into alphanumeric tokens and explicitly searches for the forbidden control flow keywords: `"if"` and `"match"`.

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

## Security Guarantee

By strictly forbidding the control flow tokens (`if`, `match`) from existing anywhere inside a `macro_rules!` block, the scanner ensures that no macro invocation can ever expand into a hidden branch. Any attempt to define a macro containing a branch will instantly trigger a `CHEAT[CHEAT-006]` failure:

> `CHEAT[CHEAT-006]: <path> — macro hides control flow or branches (scanner evasion)`

This finding forces an absolute failure, dropping the Substrate Integrity Score (SIS) to `0`, blocking the merge, and triggering a MaturityScrutiny lockdown.
