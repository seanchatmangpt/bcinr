Here is the documentation on how the `bcinr-cheat-scanner` parses the Rust AST to detect rule violations, based on my inspection of `tools/bcinr-cheat-scanner/src/main.rs`.

The scanner leverages the `syn` crate to parse the source code into an Abstract Syntax Tree (AST) and uses the `syn::visit::Visit` trait (via a custom `SynCheatVisitor` struct) to traverse expressions and items.

### CHEAT-001: Self-Canceling Operations
To detect self-canceling logic (e.g., `A ^ A` or `A.wrapping_add(B) ^ A`), the visitor specifically overrides the `visit_expr` method and looks for binary expressions (`Expr::Binary`):

1. **Operation Match:** It checks if the binary operator is either a Bitwise XOR (`BinOp::BitXor`) or Subtraction (`BinOp::Sub`).
2. **Stringification & Comparison:** It extracts the left and right sides of the expression, converts them to strings using the `quote!` macro, and strips all whitespace.
3. **Detection Patterns:**
   - **Direct Cancellation:** If the stringified left side exactly matches the right side (e.g., `A - A`), it flags a violation.
   - **Method Call (Left):** If the left side is a method call to `wrapping_add` or `wrapping_sub`, and the receiver is a "simple expression" (like a path, field, index, or literal), it checks if the stringified receiver matches the right side. This catches patterns like `(A.wrapping_add(B)) ^ A`.
   - **Method Call (Right):** Conversely, if the right side is a method call to `wrapping_add` or `wrapping_sub`, it checks if the stringified receiver matches the left side. This catches patterns like `A ^ (A.wrapping_sub(B))`.

### CHEAT-003: Magic Constants
The scanner implements detection for forbidden magic constants (specifically `0xDEADBEEF` and `0xCAFEBABE`) using a dual-layered approach (AST + Text):

1. **AST Layer:**
   - Inside `visit_expr`, it looks for literal expressions (`Expr::Lit`).
   - If the literal is an integer (`syn::Lit::Int`), it attempts to parse it into a base-10 `u64`.
   - It then checks if the numeric value exactly equals `3735928559` (the decimal value of `0xDEADBEEF`) or `3405691582` (the decimal value of `0xCAFEBABE`).
2. **Text Layer (Fallback):**
   - The scanner also runs a line-by-line text scan (`scan_file_text_rules`) that ignores code within `mod tests` or `#[cfg(test)]`.
   - It lowercases each line and strips underscores (e.g., turning `0xDEAD_BEEF` into `0xdeadbeef`), string-matching against `"0xdeadbeef"` and `"0xcafebabe"` to catch constants hiding in documentation, comments, or unparseable macros.

### Other Notable AST Parsing Implementations
- **CHEAT-002 (Circular Oracles):** The scanner collects the names and stringified bodies of all non-test functions (`visit_item_fn` and `visit_impl_item_fn`). It then compares them to see if any function ending in `_reference` or `_oracle` has the exact same stringified body as its production counterpart.
- **CHEAT-006 (Scanner Evasion):** It visits macro definitions (`visit_item_macro`). For `macro_rules`, it stringifies the definition and manually token-scans for `"if"` or `"match"` to prevent branching logic from being hidden inside macros.
- **CHEAT-008 (Benchmark Theater):** It looks for method calls (`Expr::MethodCall`) named `bench_function` or `iter`. If the stringified arguments contain the words `"branchless"` or `"allocate"` but *do not* contain `"black_box"`, it flags it as benchmark theater.
