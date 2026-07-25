# BCINR Rule 16 (Anti-Cheat Manifesto) Analysis

The `bcinr-cheat-scanner` enforces the deterministic constitution of the BCINR project by detecting code constructs explicitly forbidden by Rule 16 (Anti-cheat manifesto). It uses a combination of line-by-line text analysis and deep Abstract Syntax Tree (AST) traversal using the `syn` crate.

Here is a detailed breakdown of how the scanner blocks `CHEAT-001` through `CHEAT-010`, including its scanner evasion detection mechanisms:

## `CHEAT-001` — Self-canceling operations
* **The Cheat**: Introducing redundant mathematical operations that logically cancel themselves out to artificially inflate apparent code complexity (e.g., `A ^ A`, `A - A`, or `A.wrapping_add(B) ^ A`).
* **How it is blocked**: The AST visitor (`visit_expr`) specifically inspects binary expressions with subtraction or bitwise XOR operators. It formats and compares the left and right sides of the expression. If the structural stringification of both sides matches exactly, or if it catches known transitive cancellations via method calls like `.wrapping_add()` combined with `^`, it triggers a violation.

## `CHEAT-002` — Circular oracle
* **The Cheat**: Asserting correctness against an independent "oracle" reference that is actually a copy of the production implementation.
* **How it is blocked**: The scanner collects all stringified function bodies in the AST. It searches for functions ending in `_reference` or `_oracle`, extracts their base names, and checks if their code body is structurally identical to the corresponding production function body.

## `CHEAT-003` — Magic constants
* **The Cheat**: Using unexplained hardcoded literal values (e.g., `0xDEADBEEF`, `0xCAFEBABE`) to dictate logic.
* **How it is blocked**: The scanner runs an AST check looking for exact parsed literal values (e.g., `3735928559`). It also runs a text-based scan over non-test files to reject specific hexadecimal strings, forcing developers to use named, derived, or certified configuration constants instead.

## `CHEAT-004` — Artificial file inflation
* **The Cheat**: Padding files with dead code, repeated comments, or generated boilerplate to satisfy line-count or artifact-count maturity metrics.
* **How it is blocked**: The text scanner flags the file if it explicitly detects the string `"PADDING ENSURING FILE LENGTH REQUIREMENT"`, or if it detects 5 or more consecutive lines matching a numbered padding block format (`// N. Line N`).

## `CHEAT-005` — Boilerplate verification claims
* **The Cheat**: Adding repetitive comments asserting verification (e.g. "Hoare-logic Verification") without supplying an actual linked proof or receipt.
* **How it is blocked**: It tracks mock verification comments (e.g., lines containing "Hoare-logic Verification Line"). If a file has 5 or more of these boilerplate claims, it forces the developer to provide real axiomatic proofs or remove them.

## `CHEAT-006` — Scanner evasion (Evasion Detection Mechanism)
* **The Cheat**: Hiding prohibited operations, control-flow branches (`if`/`match`), or cheating patterns through formatting obfuscation (splitting operators across lines, inserting comments inside tokens) or macro indirection.
* **How it is blocked**: 
  1. **Formatting/Token Obfuscation**: Because the scanner natively parses the syntax tree rather than using regex, whitespace splitting and inline comments are entirely ignored by the `syn` parser. 
  2. **Macro Evasion**: To prevent hiding branches inside generated source, the scanner implements a `visit_item_macro` hook. When inspecting a `macro_rules!` definition, it stringifies the macro expansion block and detects if forbidden standard branch tokens (`if`, `match`) have been injected inside it.

## `CHEAT-007` — Dead-path compliance
* **The Cheat**: Embedding a fully compliant, zero-allocation, branchless code block within a dead path (e.g., an unreachable `if false { ... }` block) to placate basic scanners, while executing branching logic in the active path.
* **How it is blocked**: The scanner detects common dead-path compliance artifacts (such as a dummy compliant function nested inside an `if false {` conditional check) and rejects the merge, demanding the hot path itself be compliant.

## `CHEAT-008` — Benchmark theater
* **The Cheat**: Benchmarking a stub or constant-folded code block. If the compiler deduces the output of the benchmark is never used, it simply optimizes the benchmark into a no-op, fabricating unrealistic performance characteristics.
* **How it is blocked**: The AST scanner validates method calls on `criterion` benchmark harnesses (`bench_function`, `iter`). If a benchmark calls core algorithmic functions but fails to pipe the return value into `core::hint::black_box`, the scanner detects that LLVM optimization isn't defeated and flags it as benchmark theater.

## `CHEAT-009` — Mutant theater
* **The Cheat**: Creating a hostile mutant test, but only verifying it using a weak `assert_ne!` to prove the baseline differs from the mutant without validating *why* it failed.
* **How it is blocked**: The text scanner detects test files referencing "mutant". If it finds a raw `assert_ne!` assertion without accompanying proof of a typed refusal—such as `Err(StabilityRefusal::`, `.is_refused()`, or a strictly formatted `// Named law:` comment binding to a violated mathematical postcondition—the test is blocked for lacking strict failure assertions.

## `CHEAT-010` — Gate-jurisdiction theater
* **The Cheat**: Having a passing scanner that passes only because it intentionally omits the relevant target directory or target crate from its search path.
* **How it is blocked**: The scanner audits its own source code (`tools/bcinr-cheat-scanner/src/main.rs`). It verifies that its search roots list hardcodes the main runtime targets (`crates/bcinr-logic` and `crates/bcinr-cmca`). If the scanner does not inspect the primary crates, it throws a violation.
