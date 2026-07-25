# Cheat-Scanner Implementation Theory: Enforcing Rules 16 and 17

## 1. The Mandate
The `bcinr` determinism mandate requires an authoritative runtime free from branches, heap allocation, and unverified complexity. Rules 16 (Anti-Cheat Manifesto) and 17 (Cheat-Scanner Requirements) of `AGENTS.md` explicitly demand a robust, unevadable auditing mechanism (`bcinr-cheat-scanner`) to detect prohibited patterns, such as self-canceling operations, magic constants, and structural evasions.

## 2. Why Regular Expressions (Regex) Are Insufficient
A naive approach to code scanning relies on regular expressions. However, regex is fundamentally ill-suited for enforcing strict structural and semantic rules in a language like Rust for several reasons:

* **Semantic Ignorance**: Regex operates on character streams, not programmatic logic. It cannot distinguish between a prohibited keyword `if` in executable code, an `if` inside a string literal (`"What if?"`), or an `if` in a doc comment.
* **Vulnerability to Formatting Changes**: A regex looking for `0xDEADBEEF` will fail to match `0xDEAD_BEEF` or `0xde_ad_be_ef`. Attempting to write a regex that catches all possible valid numerical separator placements quickly becomes unmaintainable.
* **Inability to Understand Precedence and Nesting**: Regex cannot reliably match nested structures like `a.wrapping_add(b) ^ a` (CHEAT-001) if arbitrary whitespace, parentheses, or complex variable names are used.
* **The Evasion Problem**: Because regex only looks at the surface text, it is trivial for an adversarial agent or developer to evade detection (CHEAT-006) by splitting operators across lines, inserting benign comments inside expressions, or obfuscating variable names.

## 3. The Necessity of Full Abstract Syntax Tree (AST) Parsing
To reliably detect rule violations, the scanner must understand the code exactly as the Rust compiler (rustc) does. Full AST parsing provides the foundational semantic layer required for rigorous enforcement:

* **Structural Pattern Matching**: By analyzing the AST, the scanner can look for specific expression topologies. For example, to detect CHEAT-001 (`a.wrapping_add(b) ^ a`), the scanner searches for a `BitXor` binary operation where one operand is structurally identical to the base of a `wrapping_add` method call on the other side, regardless of whitespace or parenthesis usage.
* **Contextual Awareness**: The AST cleanly separates executable logic from literals and comments. The scanner will never falsely flag a string containing prohibited text because it only analyzes `Expr` (Expression) and `Stmt` (Statement) nodes for logical violations.
* **Scope and Visibility Analysis**: Full parsing allows the scanner to inspect both public and private functions, traits, and modules, preventing developers from hiding prohibited logic in private helpers (CHEAT-006) or using dead-path compliance (CHEAT-007).

## 4. Macro Expansion Analysis
Rust's macro system is incredibly powerful, allowing arbitrary token streams to be transformed into code. This introduces a massive loophole for scanner evasion (CHEAT-006: "using macro indirection to hide a pattern").

* **The Blind Spot of Unexpanded Code**: If the scanner only inspects the source text, a macro invocation like `branchless_add!(x, y)` looks perfectly safe. However, that macro might expand into prohibited branching logic (e.g., `if x > 0 { ... } else { ... }`).
* **Enforcing the Substrate Rules on Generated Logic**: To guarantee `CC=1` and branchlessness, the cheat-scanner must hook into the compiler's expansion phase. By analyzing the *expanded output*, the scanner ensures that no `if`, `match`, or prohibited operations are smuggled into the authoritative call graph via macros or code generators (`build.rs`).

## 5. Token and Whitespace/Comment Normalization
While the AST handles structural logic, some rules apply to the fundamental tokens themselves (e.g., CHEAT-003: Magic Constants). Adversaries might attempt to bypass lexical checks by:
* Inserting block comments inside tokens (if the parser allows or by exploiting procedural macros).
* Using arbitrary numeric separators (`_`) or casing changes (`0xDeaD_bEEf`).
* Splitting logical units across multiple lines to break simplistic linters.

**The Solution**:
The scanner must perform a normalization pass on primitive tokens:
1. **Whitespace and Comment Stripping**: Before or during tokenization, all comments and non-semantic whitespace are discarded, neutralizing attempts to split operators or hide patterns.
2. **Numeric and Hex Normalization**: All numeric literals are stripped of `_` separators and converted to a canonical format (e.g., lowercase hexadecimal). This guarantees that `0xDEADBEEF`, `0xdead_beef`, and `0xDe_ad_BE_ef` all map to the same normalized value, making CHEAT-003 enforcement bulletproof.

## 6. Conclusion
Building a cheat-scanner capable of enforcing `AGENTS.md` Rules 16 and 17 requires abandoning text-based regex tools. A determinism-enforcing auditor must operate on the normalized Abstract Syntax Tree and fully expanded macro streams. Only by analyzing the code structurally and semantically can `bcinr` guarantee that no branching, allocation, or unverified complexity infects the authoritative runtime.
