# bcinr-cheat-scanner AST Traversal Analysis (Rule 8: Absolute CC=1 Law)

Based on an in-depth analysis of `tools/bcinr-cheat-scanner/src/main.rs`, the expected AST traversal to rigorously enforce the **"Absolute CC=1 Law" (Rule 8)** is **not implemented**. 

Despite the `AGENTS.md` Constitution requiring the Enforcer (`@turing_machine`) to verify that the scanner inspects the parsed syntax tree for `if`, `match`, `loop`, `while`, and other branch-bearing nodes, the `bcinr-cheat-scanner` is currently incomplete or engaging in "Scanner Theater."

## How the AST Traversal (`SynCheatVisitor`) is Actually Implemented

The scanner utilizes the `syn::visit::Visit` trait, but its `visit_expr` method only enforces a specific subset of the Anti-Cheat Manifesto (Rule 16), ignoring `Expr::If`, `Expr::Match`, and `Expr::Loop`. 

Here is what the AST traversal actually checks:

1. **`CHEAT-001` (Self-Canceling Operations):** 
   Matches `Expr::Binary` to detect redundant, self-canceling logic like `A ^ A` or `A.wrapping_add(B) ^ A`.
2. **`CHEAT-003` (Magic Constants):** 
   Matches `Expr::Lit` to forbid unexplained hex literals like `0xDEADBEEF`.
3. **`CHEAT-008` (Benchmark Theater):** 
   Matches `Expr::MethodCall` to ensure branchless output values are consumed via `black_box`.

## The "Scanner Evasion" Text-Match (CHEAT-006)

The closest the scanner comes to checking for branches is inside its macro visitor (`visit_item_macro`). However, rather than utilizing the AST syntax tree to enforce CC=1, it simply casts the macro back to a string and checks for text tokens:

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
}
```

## Conclusion

The `bcinr-cheat-scanner` currently **does not** enforce the "Absolute CC=1 Law" (Rule 8) through its AST traversal. The tool:
- Never matches against `syn::ExprIf`, `syn::ExprMatch`, `syn::ExprLoop`, or `syn::ExprWhile`.
- Relies heavily on text-based substring searches (`scan_file_text_rules`) for other cheats like file inflation, dead paths, and mutant theater.
- Fails the constitutional mandate of Rule 8: *"The scanner must inspect the parsed syntax tree rather than only source lines... The following are prohibited: `if`, `if let`, `else`, `match`, `while`, `loop`..."*

This indicates a critical compliance gap in the `@turing_machine` enforcement layer, essentially rendering the scanner itself a violation of the substrate's integrity principles.
