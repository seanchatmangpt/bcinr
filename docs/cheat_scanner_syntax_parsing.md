# `bcinr-cheat-scanner`: Rule 17 Syntax Tree Parsing

According to the **BCINR Deterministic Substrate Constitution** (specifically **Rule 17: Cheat-scanner requirements**), the overarching mandate for the `bcinr-cheat-scanner` is that it must parse the full syntax tree of the repository rather than relying on source-line regex or naive string matching.

This document details why this requirement exists and how it actively defeats obfuscation tactics.

## Why Full Syntax Tree Parsing Over Source Line Scanning?

A text-based, regex-driven scanner is fundamentally insufficient for enforcing the strict mathematically verifiable laws of BCINR (such as the Radon Law: $CC=1$). Text scanning can be trivially bypassed through:

1. **Formatting Obfuscation:** Developers can split `if` statements or prohibited operations across multiple lines, or insert block comments directly inside tokens to break regex matches.
2. **Whitespace Variations:** Adding artificial whitespace can defeat rigid text patterns.
3. **Dead Paths:** Hiding branchless, compliant code in unreachable paths (e.g., `if false { ... }`) while executing prohibited code.

By parsing the entire source code into a Rust Abstract Syntax Tree (AST) using the `syn` crate, the `bcinr-cheat-scanner` operates at the **structural and semantic layer**. Using `syn::visit::Visit`, the scanner mathematically evaluates expressions, ensuring that branches and forbidden patterns cannot be concealed by text formatting tricks.

## Detecting Prohibited Operations in Obfuscated Code

### 1. Macros (CHEAT-006: Scanner Evasion)
A common evasion tactic is hiding branching control flow (like `if` or `match`) inside `macro_rules!` blocks, meaning the branches don't appear in the hot path until expansion. 

To prevent this, the scanner overrides `visit_item_macro` during its AST traversal. When it encounters a macro definition, it intercepts and stringifies the macro's raw AST token stream (via `quote::quote!`). It then explicitly searches that pre-expansion token stream for hidden control-flow tokens. This guarantees that branches are flagged before they can successfully expand and bypass function-level complexity gates.

### 2. Generated Boilerplate (CHEAT-004 & CHEAT-006)
Rule 17 mandates that the scanner must audit generated Rust output just as strictly as human-written source. 

Evasion tactics that involve building strings during code generation that evaluate into prohibited Rust constructs once compiled are caught because the scanner parses the final, expanded AST of the generated files. Furthermore, because the scanner normalizes whitespace and comments before structural analysis, text-mangling techniques (like adding consecutive numbered dummy comments or padding strings to inflate file lengths) are structurally exposed and mathematically verified for compliance.

### 3. Equivalent Hex Spellings (CHEAT-003: Magic Constants)
According to the Anti-Cheat Manifesto, magic constants controlling production behavior (like `0xDEADBEEF`) are strictly forbidden. Formatting changes do not make a constant lawful.

A purely text-based scanner might catch `0xDEADBEEF` but miss `0xDEAD_BEEF`, `0xdeadbeef`, or the exact decimal equivalent. Because `bcinr-cheat-scanner` operates on the syntax tree, it strips numeric separators and evaluates the underlying parsed numeric value of literal expressions (`Expr::Lit`). This means the exact underlying value is evaluated mathematically, neutralizing any formatting or base-conversion tricks entirely.
