# BCINR Panic Path Auditing & Enforcement

In BCINR, the absolute runtime law **"no panic paths" (Rule 3)** is strictly enforced across two boundaries to ensure deterministic, branchless execution. Because source-level claims are deemed "necessary but insufficient", the enforcement utilizes both an advanced source-level AST scanner and a physical object-code assembly audit.

## 1. Source-Level Enforcement (AST Parsing)

At the source level, panic paths and hidden branches are statically banned. The codebase employs `bcinr-cheat-scanner` and `bcinr-contract-gate` (acting on behalf of the `@turing_machine` agent) to systematically reject panic-inducing constructs before they ever reach the compiler.

- **Full AST Traversal (`syn` crate):** Instead of relying on regex (which can be defeated by formatting or dead code), the scanner uses the `syn` crate to parse the true Abstract Syntax Tree (AST) via `SynCheatVisitor` and `CalleeVisitor`.
- **Targeting Panic Nodes:** The `CalleeVisitor` increments complexity or rejects builds if it encounters semantic nodes that imply panic or branch paths. This specifically targets:
  - `Expr::MethodCall` for methods like `unwrap`, `expect`, `unwrap_or_else`.
  - `Expr::Try` (the `?` operator).
  - Checked arithmetic with branch-bearing handling.
- **Macro and Obfuscation Inspection:** To prevent `CHEAT-006: SCANNER_EVASION`, the scanner intercepts `macro_rules!` definitions, stringifies their expanded token stream (`quote::quote!`), and inspects them to guarantee developers cannot hide `unwrap` or branches inside macro expansions.
- **Comprehensive Jurisdiction:** The scan spans all authoritative code, private functions, test references, and even generated Rust source files.

## 2. Machine-Code Enforcement (Assembly Audit)

The project formally acknowledges that standard Rust constructs (`core::hint::black_box`, bitwise logic, arrays) do not guarantee LLVM won't silently inject bounds checks, overflow panics, or loop vectors. To achieve `BRANCHLESS_ALIVE` standing, a mechanical assembly audit must prove zero panic paths exist in the final binary.

- **Forcing Codegen with Harnesses:** A dedicated linked-executable harness (e.g., `bcinr-cmca-audit-harness`) calls the authoritative function with fixed inputs and sinks the result. This prevents LLVM from aggressively dead-code stripping the logic during `cargo build --release`.
- **Disassembly Generation:** Raw assembly is generated for the exact supported architecture matrix using platform tools like `otool -tv` (macOS), `objdump -d` (Linux), `cargo asm`, or through python scripts like `check_panic.py` (which runs `rustc --crate-type=lib --emit=asm -O`).
- **Exhaustive Call-Graph Walk:** The audit traces the entire instruction graph downwards: `root function → direct callees → transitive callees → compiler intrinsics → linked runtime symbols`.
- **Physical Symbol Scan:** The assembly text is mechanically scanned for any `JMP`, `CALL`, or references to:
  - `core::panicking::*` (including `panic_bounds_check`)
  - `unwrap_failed`
  - `Option::unwrap`, `Result::unwrap`, `expect`
- **Per-Symbol Auditing:** Finally, a verification matrix is generated (`OBJECT_CODE_AUDIT.md`). Every symbol must explicitly log `"No"` under the "Panic path" column. If even a single hidden panic pathway exists (e.g., deeply nested array index bounds checking), the merge is blocked and the Substrate Integrity Score (SIS) falls to 0.
