Based on the inspection of `tools/bcinr-cheat-scanner/src/main.rs`, the cheat scanner detects `CHEAT-003` (Magic constants) using a two-pronged approach:

### 1. Abstract Syntax Tree (AST) Scan
The scanner visits every expression in the AST of Rust files. It specifically looks for integer literals (`Expr::Lit` -> `syn::Lit::Int`). If it finds an integer, it parses the value into a `u64` base-10 integer. It will flag a violation if the value exactly matches:
- `3735928559` (which is `0xDEADBEEF` in hex)
- `3405691582` (which is `0xCAFEBABE` in hex)

```rust
if let Expr::Lit(l) = i {
    if let syn::Lit::Int(li) = &l.lit {
        if let Ok(val) = li.base10_parse::<u64>() {
            if val == 3735928559 || val == 3405691582 {
                // flags CHEAT-003
            }
        }
    }
}
```

### 2. File Text / Doc Comment Scan
In addition to the AST, the scanner iterates through the raw text of the file line-by-line to catch magic constants hidden in documentation comments, regular comments, or string literals. 

- This text check is bypassed entirely for test and benchmark files (paths containing `/tests/` or `/benches/`).
- Within non-test files, it tracks braces (`{` and `}`) to deliberately skip lines inside `#[cfg(test)]` or `mod tests` blocks.
- For all other lines, it converts the text to lowercase, removes all underscores (to handle formatting like `0xDEAD_BEEF`), and checks if the resulting string contains:
  - `"0xdeadbeef"`
  - `"0xcafebabe"`

```rust
let text_no_us = line.replace("_", "").to_lowercase();
if text_no_us.contains("0xdeadbeef") || text_no_us.contains("0xcafebabe") {
    // flags CHEAT-003
}
```

This design ensures that the forbidden magic constants cannot be hidden by varying the formatting, case, or numeric separators, nor can they be placed inside comments/macros without triggering the rule.
