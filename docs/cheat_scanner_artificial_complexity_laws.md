Based on the `BCINR` architecture and Rule 17 (Cheat-scanner requirements), here is the documentation regarding `CHEAT-001` and the Cheat-Scanner's Artificial Complexity laws.

### CHEAT-001: Self-Canceling Operations in Practice

In practice, a **self-canceling operation** is any expression that mathematically negates itself, providing no meaningful change to the underlying state. They are typically added strictly to create "apparent complexity" in the source code without advancing the algorithm.

**Examples:**
- `a.wrapping_add(b) ^ a`
- `(x) ^ (x)`
- `A - A`

### Why the Scanner Catches `a.wrapping_add(b) ^ a`

The `bcinr-cheat-scanner` enforces Rule 17 by parsing the Abstract Syntax Tree (AST) directly using the `syn` crate, rather than relying on simple regex or text matching. 

When the scanner encounters a binary expression with a bitwise XOR (`^`) or subtraction (`-`), it performs the following structural checks:
1. It stringifies and compares the Left-Hand Side (LHS) and Right-Hand Side (RHS). If they are structurally identical (e.g., `A ^ A`), it flags a violation.
2. It specifically checks for **method calls** on either side of the operator. For `a.wrapping_add(b) ^ a`, it sees that the LHS is a method call to `wrapping_add` (or `wrapping_sub`).
3. It extracts the **receiver** of the method call (`a` in this case) and compares it against the RHS. If the stringified receiver perfectly matches the RHS, the scanner recognizes that the root variable is mutually XOR'ed or subtracted against its own derived operation. It then flags this as `CHEAT[CHEAT-001]`.

---

### Cheat-Scanner Artificial Complexity Laws

Under the **BCINR Deterministic Substrate Constitution**, all authoritative code is strictly gated against metric theater, artificial complexity, and scanner evasion. Every instruction must have a mathematically proven contribution to the output contract. 

The core anti-cheat laws designed to prevent artificial complexity and inflation include:

#### CHEAT-001: Self-Canceling Operations
* **Law**: Any operation without a contractual contribution to the output is prohibited. 
* **Enforcement (AST Level)**: Detects operations that cancel themselves out (e.g., `A ^ A`, `A - A`, or mutual negation across `wrapping_add`/`wrapping_sub`), preventing the illusion of algorithmic complexity.

#### CHEAT-003: Magic Constants
* **Law**: Unexplained literals controlling behavior are forbidden, even if visually formatted to look innocuous.
* **Enforcement (Text & AST Level)**: Rejects hardcoded magic hex constants (e.g., `0xDEADBEEF`, `0xCAFEBABE`). All constants must be named, derived, or certified configuration values included in the influence digest.

#### CHEAT-004: Artificial File Inflation
* **Law**: Adding padding, generated boilerplate, or dead code to satisfy line-count or artifact-count expectations is prohibited.
* **Enforcement (Text Level)**: Scans for sentinel padding strings, redundant commented blocks, or consecutive numbered lines used solely for artificially inflating the file length.

#### CHEAT-005: Boilerplate Verification Claims
* **Law**: Repeated documentation claims asserting verification without linking to a reproducible proof or axiomatic receipt are forbidden.
* **Enforcement (Text Level)**: Identifies consecutive mock verification comments (e.g., repeating "Hoare-logic Verification Line") without providing actual symbolic proofs or bit-vector certificates.

#### CHEAT-006: Scanner Evasion
* **Law**: Splitting operators across lines, inserting comments inside tokens, or using macro indirection to hide a prohibited pattern is forbidden.
* **Enforcement (AST & Text Level)**: Inspects expanded macro outputs (`macro_rules!`) and syntax trees to catch branches (`if`, `match`) and complexity intentionally obfuscated behind abstractions.

#### CHEAT-007: Dead-path Compliance
* **Law**: You cannot bypass compliance by writing lawful code that is never executed while the "real" path remains unlawful.
* **Enforcement (AST Level)**: Detects compliant "dummy" implementations placed inside unreachable `if false { ... }` blocks used to fool naive cyclomatic complexity or branchless scanners.

> **Constitutional Enforcement:** The detection of any of these patterns immediately changes the project standing to `CHEAT_DETECTED`, sets the Substrate Integrity Score (SIS) to 0, and blocks feature work until structural remediation is proven.
