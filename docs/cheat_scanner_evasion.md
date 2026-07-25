# bcinr-cheat-scanner: Enforcing CHEAT-006 (Scanner Evasion)

The `bcinr-cheat-scanner` enforces the deterministic substrate's Anti-Cheat Manifesto. A critical aspect of this is **Rule 16 (CHEAT-006: Scanner evasion)**, which prevents developers from hiding prohibited constructs through code obfuscation. 

Rather than relying purely on easily tricked string matching, the scanner leverages the `syn` crate to parse the full Rust Abstract Syntax Tree (AST). 

Here is how the scanner uses AST traversal to thwart specific evasion techniques:

### 1. Catching Prohibited Control Flow in Macros
A developer might attempt to hide prohibited `if` or `match` statements by defining them inside a macro that later expands into the hot path. The scanner prevents this by implementing `visit_item_macro` in its AST visitor. When it encounters a `macro_rules!` definition, it extracts its AST token stream and specifically tokenizes the contents to search for hidden `if` and `match` keywords, catching the evasion at the macro definition level before expansion.

### 2. Inspecting Trait Implementations
Code hiding inside traits could bypass a naive scanner that only searches for standalone function declarations. By implementing full `syn::visit::Visit` traversal, the scanner structurally analyzes `ItemImpl` (trait and struct implementation blocks) and `ImplItemFn` (methods inside those blocks). Logic tucked away in trait implementations is captured and evaluated as strictly as top-level functions for violations like self-canceling expressions (CHEAT-001) or circular reference oracles (CHEAT-002).

### 3. Defeating Spaced-out Hex Spellings (AST Literal Parsing)
A common evasion technique against string-based scanners is altering the format of banned constants (e.g., turning `0xDEADBEEF` into `0xDEAD_BEEF`, `0xdeadbeef`, or its decimal equivalent `3735928559`). The scanner thwarts this by examining the AST directly. When it visits an expression, it checks for integer literals (`syn::Lit::Int`) and natively parses the underlying numeric value using `base10_parse::<u64>()`. Because it evaluates the mathematical *value* rather than the textual *string*, formatting tricks and base changes are mathematically neutralized.

### Conclusion
By analyzing the structural syntax tree instead of raw strings, the `bcinr-cheat-scanner` reliably exposes branching in macros, deeply nested trait logic, and syntactically varied magic numbers, firmly upholding the strict zero-branching laws of the substrate.
