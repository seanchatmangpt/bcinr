# bcinr-cheat-scanner Source AST Parsing

In the BCINR deterministic systems substrate, maintaining the Radon Law (Cyclomatic Complexity $CC=1$) requires an adversarial approach to code verification. Traditional linting via regex or text-matching is fundamentally flawed because it can be trivially bypassed by code formatting, macros, or dead paths. 

To ensure absolute adherence to $CC=1$ and the Anti-Cheat Manifesto, `bcinr-cheat-scanner` and `bcinr-contract-gate` use the `syn` crate to parse the full Rust Abstract Syntax Tree (AST), ensuring branches and forbidden patterns cannot be concealed.

## 1. Why Text Regex is Insufficient

A purely text-based scanner can be evaded through several trivial techniques:
* **Formatting Obfuscation:** Splitting an `if` condition across multiple lines or embedding comments to break regex matches.
* **Macro Expansion:** Hiding conditional logic (`if`, `match`) inside `macro_rules!` blocks that expand into branches during compilation, evading function-level text scans.
* **Whitespace Variations:** Creating dummy complexity (like self-canceling variables) with varying whitespace that defeats strict regex patterns.
* **Implicit Branches:** Using methods like `.unwrap()` or the `?` operator which introduce hidden panics and branches without using structural `if` keywords.

## 2. Full AST Traversal with `syn`

`bcinr-cheat-scanner` parses the entire source code into a `syn::File` syntax tree and implements the `syn::visit::Visit` trait via `SynCheatVisitor`. This allows the scanner to inspect the structural semantics of the code regardless of text formatting.

### Catching Hidden Branches in Macros (CHEAT-006: SCANNER_EVASION)
To prevent developers from evading $CC=1$ checks by moving branches into macros, the scanner overrides `visit_item_macro`. When it encounters a `macro_rules!` definition, it extracts and stringifies the macro's raw AST token stream via `quote::quote!`. It then specifically searches the token stream for control-flow tokens (`if`, `match`). This ensures that macros designed to expand into branching code are flagged as scanner evasion before they can bypass function-level complexity gates.

### Defeating Structural Obfuscation (CHEAT-001: SELF_CANCELING_OPERATIONS)
Self-canceling operations (e.g., `a.wrapping_add(b) ^ a`) used to artificially inflate code complexity cannot be reliably caught by regex due to arbitrary spacing, nesting, and variable names. By hooking into `visit_expr` and matching on `Expr::Binary`, the AST parser:
1. Isolates the left-hand and right-hand sub-expressions.
2. Stringifies them independently via `quote!`.
3. Strips all whitespace and formatting.
4. Structurally compares them.

This algorithm mathematically proves that terms like `A` and `A` (or even `A.wrapping_add(B)` and `A.wrapping_add(B)`) cancel each other out, catching artificial complexity injections regardless of how the code is formatted.

### Benchmark Theater Prevention (CHEAT-008: BENCHMARK_THEATER)
To ensure constant-folding doesn't invalidate benchmarks, the scanner inspects all `Expr::MethodCall` nodes in the AST. If it detects a call to `bench_function` or `iter` involving authoritative logic without a subsequent AST node consuming the output via `black_box`, it flags the code.

## 3. Synergy with `bcinr-contract-gate`

While `bcinr-cheat-scanner` hunts for obfuscation, evasion, and fake complexity, the actual $CC=1$ strict enforcement is handled by `bcinr-contract-gate`, which also leverages `syn::visit`. 

Instead of searching for the text `"if"`, `bcinr-contract-gate` implements a `CalleeVisitor` that increments complexity directly based on semantic AST nodes:
* `Expr::If`, `Expr::Match`, `Expr::Loop`, `Expr::While`, `Expr::ForLoop`
* `Expr::Try` (the `?` operator)
* `Expr::MethodCall` (specifically rejecting branch-bearing methods like `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`)

## Conclusion
By parsing the true Rust AST, the `bcinr-cheat-scanner` ecosystem ensures that rules like `CHEAT-006: SCANNER_EVASION` are strictly enforced. Branches hidden behind macros, formatting tricks, or structural aliases are structurally exposed. This robust AST analysis, paired with downstream machine-code audits, maintains the physical integrity of the BCINR branchless substrate.
