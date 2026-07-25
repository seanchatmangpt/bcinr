# `bcinr-contract-gate` Analysis

The `bcinr-contract-gate` tool validates branchless contract compliance (`CC=1`) and checks for potential panic paths by analyzing the **Abstract Syntax Tree (AST)** using the `syn` crate. It **does not** analyze the compiled object code.

## How it validates branchless contract compliance (`CC=1`)
1. **AST Parsing**: The tool recursively scans Rust source files (defaulting to `crates/bcinr-logic` and `crates/bcinr-cmca`) and parses their contents into an AST using `syn::parse_file`.
2. **Call Graph & Reachability**: It builds a call graph starting from defined `AUTHORITATIVE_ROOTS` (namely `allocate` and `evaluate_calibration`). It checks functions that are transitively reachable from these roots (or matched by specific file/function names like `temp_`).
3. **Complexity Calculation**: It implements the `syn::visit::Visit` trait via `CalleeVisitor` to traverse the expressions in function bodies. The base complexity for a function starts at 1, and is incremented for every branching or looping AST node:
   - `Expr::If` (`if` expressions)
   - `Expr::Match` (`match` expressions)
   - `Expr::Loop` (`loop` expressions)
   - `Expr::While` (`while` loops)
   - `Expr::ForLoop` (`for` loops)
   - `Expr::Try` (`?` operator)
4. **Enforcement**: If any checked function accumulates a complexity greater than 1, the tool records a failure: `FAIL: <name> in <path> has Cyclomatic Complexity <X> (Branch detected!)`.

## How it checks for panic paths and forbidden operations
- **Panic-inducing Methods**: During the AST traversal, if the visitor encounters an `Expr::MethodCall` for methods known to branch or panic (`unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`), it increments the complexity score, thus failing the `CC=1` requirement.
- **Forbidden Operators (Bluff Detection)**: It tracks basic binary operators (`+`, `-`, `*`, `/`). If these operators are found in functions expected to use bitwise logic (e.g., function names containing `add_bitwise` or `sub_bitwise`), it raises a bluff detection failure.
- **Contract Enforcement**: The tool checks function attributes (`ItemFn`, `ImplItemFn`) and file-level documentation for required strings (`"Branchless Contract"`, `"BRANCHLESS CONTRACT"`, or `"u64_contract!"`). Any public primitive missing these contracts is flagged with a `MISSING_U64_CONTRACT` error.

In summary, `bcinr-contract-gate` statically analyzes the Rust source code syntax to enforce the $CC=1$ rule and prevent certain panic paths, but relies on other tools for actual object code / disassembly audits.
