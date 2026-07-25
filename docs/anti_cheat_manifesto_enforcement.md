# BCINR Anti-Cheat Manifesto (Rule 16) - Enforcement Mechanisms

Based on the `bcinr-cheat-scanner` source code (`tools/bcinr-cheat-scanner/src/main.rs`), here is the documentation of how `CHEAT-001` through `CHEAT-010` are enforced. The scanner utilizes a combination of text-based scanning and Abstract Syntax Tree (AST) inspection via the `syn` crate.

## CHEAT-001: Self-canceling operations
**Detection Mechanism:** AST Parsing (`syn::visit::Visit`)
The scanner visits binary expressions (`Expr::Binary`). For XOR (`^`) and subtraction (`-`) operators, it stringifies both the left and right sides (removing whitespace) and checks for exact equivalence (e.g., `A ^ A` or `A - A`). It also checks for specific method calls like `A.wrapping_add(B) ^ A` and `A ^ A.wrapping_add(B)` by verifying if the receiver of `wrapping_add`/`wrapping_sub` is identical to the other operand.

## CHEAT-002: Circular oracle
**Detection Mechanism:** AST Parsing (Function Body Comparison)
The scanner collects all function names and their stringified bodies (ignoring whitespace). It looks for any function ending with `_reference` or `_oracle`. It strips this suffix to find the base implementation function. If the base implementation function exists and its body is exactly identical to the oracle function's body, it flags it as a circular oracle.

## CHEAT-003: Magic constants
**Detection Mechanism:** Text & AST Parsing
- **AST:** Checks integer literals (`Expr::Lit`) in non-test files for specific banned decimal values `3735928559` (`0xDEADBEEF`) and `3405691582` (`0xCAFEBABE`).
- **Text:** Scans lines in non-test blocks for case-insensitive matches of `0xdeadbeef` or `0xcafebabe` (ignoring underscores).

## CHEAT-004: Artificial file inflation
**Detection Mechanism:** Text Scanning
The scanner reads the file line-by-line and looks for:
1. The exact string: `PADDING ENSURING FILE LENGTH REQUIREMENT`.
2. Blocks of 5 or more consecutive comment lines starting with `//` that contain the phrase `. Line` (indicating artificial line numbering for padding).

## CHEAT-005: Boilerplate verification claims
**Detection Mechanism:** Text Scanning
The scanner counts occurrences of lines containing both `"Hoare-logic Verification Line"` and `"Branchless path is the unique solution to the state constraints of"`. If a file contains 5 or more of these mock verification claims, it triggers the violation.

## CHEAT-006: Scanner evasion
**Detection Mechanism:** AST Parsing (Macro Inspection)
When the visitor encounters a `macro_rules!` definition, it stringifies the macro's body and tokenizes it. If it finds the tokens `if` or `match` hidden inside the macro, it flags it as an attempt to hide control flow branches from standard source analysis.

## CHEAT-007: Dead path compliance
**Detection Mechanism:** Text Scanning
The scanner searches the source text for the simultaneous presence of `if false {` and the identifier `dummy_branchless`. This detects compliant dummy code placed inside statically unreachable blocks while the active path may remain unlawful.

## CHEAT-008: Benchmark theater
**Detection Mechanism:** AST Parsing
The scanner examines method calls named `bench_function` or `iter`. It stringifies the arguments; if the benchmark arguments contain the words `branchless` or `allocate` but lack `black_box`, it flags the code. This ensures that benchmark outputs are consumed by `core::hint::black_box` to prevent compiler optimization dead-code elimination.

## CHEAT-009: Mutant theater
**Detection Mechanism:** Text Scanning (Test files only)
If a test file contains the word `mutant` and uses the `assert_ne!` macro, the scanner demands explicit evidence of a typed refusal or a named law. It requires the file to contain at least one of the following strings:
- `Err(StabilityRefusal::`
- `Err(ObservatoryFlag::`
- `.is_refused()`
- `.numeric_faults()`
- `.faults()`
- `.refusals()`
- `// Named law:`
Without one of these, an `assert_ne!` on a mutant is considered too weak ("theater").

## CHEAT-010: Gate-jurisdiction theater
**Detection Mechanism:** Self-Inspection (Text Scanning)
The scanner reads its own source code (`tools/bcinr-cheat-scanner/src/main.rs`). It verifies that its own code contains the target directory strings `"crates/bcinr-logic"` and `"crates/bcinr-cmca"`. This prevents developers from maliciously modifying the scanner to simply omit production crates from its search paths.
