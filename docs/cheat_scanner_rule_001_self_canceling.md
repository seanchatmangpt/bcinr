# How the AST Scanner Detects CHEAT-001 (Self-Canceling Operations)

The `bcinr-cheat-scanner` uses the `syn` crate to parse the Rust Abstract Syntax Tree (AST) and implements a visitor pattern (`syn::visit::Visit`) via `SynCheatVisitor`. 

Here is exactly how it detects `CHEAT-001` patterns in `tools/bcinr-cheat-scanner/src/main.rs`:

### 1. Hooking into Binary Expressions
The scanner intercepts binary expressions by overriding the `visit_expr` method and looking for `Expr::Binary`. 

It specifically targets two operators: Bitwise XOR (`^`) and Subtraction (`-`):
```rust
if let Expr::Binary(b) = i {
    if matches!(b.op, BinOp::BitXor(_) | BinOp::Sub(_)) {
        // ...
    }
}
```

### 2. Identifying Direct Self-Cancellation (`A ^ A` or `A - A`)
The scanner stringifies the left and right operands using the `quote!` macro and removes all whitespace. It then performs a direct string comparison:

```rust
let left = &b.left;
let right = &b.right;
let left_str = quote::quote!(#left).to_string().replace(" ", "");
let right_str = quote::quote!(#right).to_string().replace(" ", "");

if left_str == right_str {
    // Detects: A ^ A or A - A
}
```

### 3. Detecting Wrapped Obfuscation (`A.wrapping_add(B) ^ A` and `A ^ A.wrapping_add(B)`)
To catch attempts to hide the cancellation within a method call, the scanner inspects both sides of the binary operation to see if one is a method call to `wrapping_add` or `wrapping_sub`. 

It validates that the receiver (the `A` in `A.wrapping_add(B)`) matches the opposite operand. It also enforces that the receiver is a "simple expression" to avoid false positives.

**Left-side method call check (`(A.wrapping_add(B)) ^ A`):**
```rust
if let Expr::MethodCall(mc) = &*b.left {
    if (mc.method == "wrapping_add" || mc.method == "wrapping_sub")
        && is_simple_expr(&mc.receiver)
    {
        let receiver = &mc.receiver;
        let rec_str = quote::quote!(#receiver).to_string().replace(" ", "");
        if rec_str == right_str {
            // Detects: (A.wrapping_add(B)) ^ A
        }
    }
}
```

**Right-side method call check (`A ^ (A.wrapping_add(B))`):**
```rust
if let Expr::MethodCall(mc) = &*b.right {
    if (mc.method == "wrapping_add" || mc.method == "wrapping_sub")
        && is_simple_expr(&mc.receiver)
    {
        let receiver = &mc.receiver;
        let rec_str = quote::quote!(#receiver).to_string().replace(" ", "");
        if rec_str == left_str {
            // Detects: A ^ (A.wrapping_add(B))
        }
    }
}
```

### Definition of `is_simple_expr`
The `is_simple_expr` helper ensures the AST traversal only matches when the receiver of the `wrapping_add`/`wrapping_sub` is a straightforward value (like a variable, struct field, array index, literal, or reference), rather than a complex nested expression:

```rust
fn is_simple_expr(e: &Expr) -> bool {
    match e {
        Expr::Path(_) => true,
        Expr::Field(f) => is_simple_expr(&f.base),
        Expr::Index(idx) => is_simple_expr(&idx.expr) && is_simple_expr(&idx.index),
        Expr::Lit(_) => true,
        Expr::Reference(r) => is_simple_expr(&r.expr),
        _ => false,
    }
}
```
