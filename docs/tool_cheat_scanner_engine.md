# Analysis of `bcinr-cheat-scanner`

## Overview
`bcinr-cheat-scanner` is a standalone Rust binary crate located at `/Users/sac/bcinr/tools/bcinr-cheat-scanner/`. It acts as a static analysis tool designed to enforce the "Anti-cheat manifesto" rules defined in the project's constitutional guidelines (e.g., `AGENTS.md`).

## Execution Context
- **Invocation:** It is executed as a standalone executable (likely via a `cargo make` task like `cargo make scan-cheats`).
- **Target Directories:** It is hardcoded to scan `.rs` files within the `crates/bcinr-logic` and `crates/bcinr-cmca` directories. It also invokes `cargo metadata` to inspect transitive dependencies (like `proptest` and `criterion`) for reachable branch code.
- **Output:** It collects violations in a vector and prints them to `stderr`. If any findings are present, it exits with a non-zero status code (`process::exit(1)`), blocking the CI/build pipeline.

## Implementation Details

### AST Parsing Strategy
The scanner relies on the `syn` crate to parse Rust source code into an Abstract Syntax Tree (AST) and uses the `syn::visit::Visit` trait to traverse it:
- **`SynCheatVisitor`:** A custom visitor struct that overrides methods like `visit_expr`, `visit_item_fn`, `visit_item_macro`, and `visit_item_mod`.
- **Test/Bench Exclusion:** The visitor intentionally skips traversing into modules, implementations, or functions annotated with `#[test]` or `#[bench]` attributes, ensuring production code is evaluated strictly while allowing leniency in test fixtures.
- **AST Checks:**
  - **Expressions (`visit_expr`):** Detects `CHEAT-001` (self-canceling binary operations like `A ^ A` or `A.wrapping_add(B) ^ A`), `CHEAT-003` (magic constants like `0xDEADBEEF`), and `CHEAT-008` (benchmark theater where branchless algorithms aren't consumed via `black_box`).
  - **Macros (`visit_item_macro`):** Detects `CHEAT-006` (scanner evasion) by ensuring `macro_rules!` do not hide `if` or `match` control flow statements.
  - **Functions (`visit_item_fn`):** Captures the stringified bodies of functions to perform cross-function structural checks, such as `CHEAT-002` (circular oracles where a `_reference` function body is exactly identical to the production function body) and `CHEAT-020` (mutation before admission checks).

### Text-based Scanning
Beyond AST parsing, it performs raw text-based scans on the source code (`scan_file_text_rules`) to enforce rules that are difficult or unnecessary to parse via AST:
- Reads the file line-by-line, tracking scope depth (e.g., identifying if inside a test module `mod tests { ... }`).
- Detects textual violations such as `CHEAT-004` (artificial line padding/inflation), `CHEAT-005` (mock/boilerplate Hoare-logic verification claims), and `CHEAT-007` (dead-path compliance where a compliant dummy block is placed in an `if false { ... }` block).
- Checks test modules for `CHEAT-009` (mutant theater) by ensuring that when `assert_ne!` is used, the test also contains evidence of a typed refusal or named law verification.

### Emitting Findings (`CHEAT[rule-id]`)
When a violation is detected, the scanner formats a string matching the exact error pattern required by the constitution and pushes it to a `findings` vector. 
Example formatting:
- `"CHEAT[CHEAT-001]: {path} — self-canceling expression detected: {left} ^/sub {right}"`
- `"CHEAT[CHEAT-003]: {path} — magic constant found in file text/doc comments"`

All accumulated findings are logged at the end of the scan, preventing any code that evades these rules from being merged.
