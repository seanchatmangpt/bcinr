# AST Parsing Strategy for BCINR Scanners

In the BCINR deterministic systems substrate, maintaining absolute branchlessness ($CC=1$) and preventing evasion requires structural code verification. Traditional linting mechanisms based on regular expressions or text matching are fundamentally insufficient. 

To enforce these strict constitutional laws, the BCINR verification suite—comprising `bcinr-cheat-scanner` and `bcinr-contract-gate`—physically parses the Rust Abstract Syntax Tree (AST) using the `syn` crate.

## Why AST-Level Inspection is Superior to Regex

A purely text-based scanner (regex) can be trivially evaded through several techniques:
1. **Formatting Obfuscation:** Control flow keywords can be split across multiple lines, or embedded with comments (e.g., `if /* comment */ condition {`) to break regex matches.
2. **Macro Expansion:** Branching logic can be hidden inside `macro_rules!` definitions. A text scanner looking for `if` or `match` at the function level will miss branches injected during macro expansion.
3. **Implicit Branches:** Operators like `?` or methods like `.unwrap()` introduce hidden panics and branches without using explicit structural keywords like `if`.
4. **Structural Deception:** Dead paths or self-canceling operations (e.g., `A ^ A`) can be formatted to look complex to a line-counter, but mathematically do nothing.

By parsing the code into a true `syn::File` AST, the scanner inspects the semantic structure of the code exactly as the compiler sees it, completely neutralizing formatting tricks.

## Physical Parsing with `syn`

The BCINR scanners load source files and parse them using `syn::parse_file`, generating a full AST. They then define visitors that implement the `syn::visit::Visit` trait, allowing them to traverse specific semantic nodes (like functions, expressions, and macros) recursively.

```rust
if let Ok(syntax) = syn::parse_file(&src) {
    let mut visitor = CallGraphVisitor { ... };
    visitor.visit_file(&syntax);
}
```

## Enforcing $CC=1$ via `Expr` Detection

While `bcinr-cheat-scanner` handles anti-cheat policies and obfuscation, the strict $CC=1$ enforcement is handled by `bcinr-contract-gate`.

When `bcinr-contract-gate` visits a function, it spawns a `CalleeVisitor` that implements `visit_expr` to traverse every expression inside the function body. It detects explicit and implicit branching by matching directly against the `syn::Expr` enum variants.

Instead of searching for text, it strictly increments a complexity counter upon encountering semantic branch nodes:

```rust
impl<'ast> Visit<'ast> for CalleeVisitor {
    fn visit_expr(&mut self, i: &'ast Expr) {
        match i {
            // Explicit control flow branches
            Expr::If(_) | Expr::Match(_) | Expr::Loop(_) | Expr::While(_) | Expr::ForLoop(_) => {
                self.complexity += 1;
            }
            // Implicit branches (the `?` operator)
            Expr::Try(_) => {
                self.complexity += 1;
            }
            // Branch-bearing method calls (e.g., .unwrap(), .expect())
            Expr::MethodCall(mc) => {
                let m = mc.method.to_string();
                if m == "unwrap" || m == "expect" || m == "unwrap_or" || m == "unwrap_or_else" {
                    self.complexity += 1;
                }
            }
            _ => {}
        }
        visit::visit_expr(self, i);
    }
}
```

If the `complexity` counter exceeds 1, the gate immediately fails the build, reporting a violation of the Radon Law ($CC=1$).

## Catching Obfuscation and Scanner Evasion

The AST approach also enables `bcinr-cheat-scanner` to enforce the Anti-Cheat Manifesto by analyzing structural relationships:

* **Scanner Evasion (CHEAT-006):** The scanner overrides `visit_item_macro` to extract the raw token stream of `macro_rules!` blocks. It verifies that no control flow tokens (`if`, `match`) are being hidden inside macros to evade function-level checks.
* **Self-Canceling Operations (CHEAT-001):** By visiting `Expr::Binary`, the scanner isolates the left-hand and right-hand AST nodes (e.g., `A ^ A`). It stringifies them via `quote!`, strips whitespace, and structurally compares them. This catches dummy complexity that a regex would miss due to arbitrary spacing and nesting.

## Conclusion

By relying on `syn` for deep semantic traversal, the BCINR tooling ensures that no branch, panic path, or evasion tactic can survive in authoritative code. AST parsing guarantees that verification focuses on the structural reality of the code, not its textual representation.
