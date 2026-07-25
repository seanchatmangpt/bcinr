# Cheat Scanner Architecture (Rule 17) Documentation

Based on the investigation of the `bcinr-cheat-scanner` crate (`tools/bcinr-cheat-scanner/src/main.rs`), the scanner enforces anti-cheat rules (Rule 16 and 17) through a combination of AST parsing and normalized text inspection. Here is a breakdown of the architectural implementation:

## 1. Full Syntax Tree Parsing (`syn` and `visit`)
The scanner uses the `syn` crate to parse Rust source code into complete Abstract Syntax Trees (ASTs). The `SynCheatVisitor` struct implements the `syn::visit::Visit` trait to traverse specific node types across both public and private elements:
- `visit_expr`: Analyzes expressions, binary operators, and method calls.
- `visit_item_fn` / `visit_impl_item_fn`: Traverses function definitions (while distinguishing `#[cfg(test)]` paths).
- `visit_item_macro`: Inspects macro definitions.

Because it operates on the syntax tree rather than raw source lines, it can inspect logically structured patterns regardless of arbitrary formatting.

## 2. Whitespace Normalization
To prevent developers from evading detection using extra spaces, newlines, or unusual formatting, the scanner normalizes whitespace. This is achieved by taking an AST node, converting it back to a standard string using the `quote` crate, and then stripping out spaces:
```rust
let left_str = quote::quote!(#left).to_string().replace(" ", "");
let right_str = quote::quote!(#right).to_string().replace(" ", "");
```
This ensures structurally identical expressions (e.g., `A ^ A` vs `A   ^   A`) are reliably matched for rules like `CHEAT-001` (Self-Canceling Operations).

## 3. Stripping Numeric Separators and Normalizing Literals
For text-based scanning, the scanner strips underscores (numeric separators) and normalizes capitalization to defeat simple obfuscation:
```rust
let text_no_us = line.replace("_", "").to_lowercase();
```
For AST-level checks, it avoids fragile string matching entirely. Instead, it evaluates the literal's parsed base-10 value. This catches magic constants (CHEAT-003) regardless of whether they are written in decimal, hex, or using arbitrary underscores:
```rust
if let Expr::Lit(l) = i {
    if let syn::Lit::Int(li) = &l.lit {
        if let Ok(val) = li.base10_parse::<u64>() {
            if val == 3735928559 || val == 3405691582 {
                // Detects 0xDEADBEEF (3735928559) or 0xCAFEBABE (3405691582)
            }
        }
    }
}
```

## 4. Catching Scanner Evasion (CHEAT-006)
`CHEAT-006` explicitly forbids hiding prohibited logic (like branches) inside abstractions such as macros. The `SynCheatVisitor` intercepts macro definitions and scans their token streams for banned keywords:
```rust
fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
    if let Some(ident) = &i.mac.path.get_ident() {
        if ident.to_string() == "macro_rules" {
            let mac_str = quote::quote!(#i).to_string();
            if has_token(&mac_str, "if") || has_token(&mac_str, "match") {
                // Reports: CHEAT[CHEAT-006]: macro hides control flow or branches
            }
        }
    }
}
```
The `has_token` helper splits the macro text into alphanumeric blocks to ensure that the prohibited words ("if" or "match") are exact matches. This prevents evasion attempts that try to bypass `CC=1` and branchlessness rules by wrapping conditional flow in macro expansions.
