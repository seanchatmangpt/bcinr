# BCINR Cheat-Scanner Evasion Counter-Measures

The `bcinr-cheat-scanner` (located at `tools/bcinr-cheat-scanner/src/main.rs`) enforces constitutional rules against obfuscation and cheat attempts by analyzing both the raw text and the parsed Abstract Syntax Tree (AST) of the Rust source code. 

Here is how the scanner implements specific evasion counter-measures:

## 1. AST Normalization
To prevent developers from hiding prohibited patterns (such as `CHEAT-001` self-canceling operations like `A ^ A`) using formatting tricks like line breaks or extra spaces, the scanner relies on the `syn` and `quote` crates:
- **Structural Parsing:** Source files are parsed into an AST using `syn::parse_file()`. This fundamentally ignores purely lexical formatting like indentation or whitespace.
- **Token Stringification & Normalization:** When the scanner needs to compare complex AST nodes (e.g., verifying if the left side of a binary operator is identical to the right side), it uses `quote::quote!(#expr).to_string()` to convert the node back into a canonical token string.
- **Whitespace Stripping:** It further normalizes the string by unconditionally removing all spaces (`.replace(" ", "")`). As a result, `a . wrapping_add ( b )` and `a.wrapping_add(b)` evaluate to the exact same normalized string.

## 2. Stripping Numeric Separators & Equivalent Hex Spellings (`CHEAT-003`)
Rule `CHEAT-003` prohibits magic constants (e.g., `0xDEADBEEF`, `0xCAFEBABE`). Developers might attempt to evade simple text scanners by using alternative radixes (decimal instead of hex) or Rust's numeric separators (`_`, e.g., `0xDEAD_BEEF`). The scanner defeats this using a two-layered defense strategy:

### Layer A: AST Mathematical Parsing
When inspecting expressions, the scanner intercepts all literal numbers (`Expr::Lit` -> `syn::Lit::Int`). 
- It uses the `syn` crate's built-in `.base10_parse::<u64>()` method, which automatically resolves Rust's numeric literal syntax.
- This natively strips all underscores (`_`) and converts hexadecimal (`0x`), octal (`0o`), binary (`0b`), and decimal literals into a single mathematical `u64` value.
- It then explicitly checks if the evaluated value matches the prohibited constants (e.g., `3735928559` for `0xDEADBEEF`, or `3405691582` for `0xCAFEBABE`). 
- **Result:** `0xDEADBEEF`, `0xDEAD_BEEF`, `3735928559`, and `37_35_92_85_59` are all instantly caught as the exact same integer value.

### Layer B: Text-Level Fallback Normalization
To prevent magic constants from being hidden inside documentation comments, macros, or strings where they wouldn't parse as an integer literal expression, the scanner employs a fallback textual scan:
- It iterates over the file line-by-line.
- It strips all underscores manually using `let text_no_us = line.replace("_", "");`.
- It converts the string to lowercase: `.to_lowercase()`.
- It then searches for the normalized substrings `"0xdeadbeef"` and `"0xcafebabe"`.
