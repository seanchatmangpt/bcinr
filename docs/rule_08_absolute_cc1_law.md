# Rule 8: Absolute `CC=1` Law

In the BCINR substrate, Rule 8 enforces a strict cyclomatic complexity of 1 (`CC=1`) across the entire authoritative call graph. This guarantees that code is perfectly linear and deterministically bounded, meaning no execution path can diverge based on the input data.

## Prohibited Constructs

The following constructs are strictly prohibited in authoritative code because they inherently produce conditional control-flow branches:

* **Explicit Control Flow:** `if`, `if let`, `else`, `match`, `while`, `loop`, `break`, `continue`, `early return`.
* **Option/Result Handling:** `?` (try operator), `unwrap`, `unwrap_or`, `unwrap_or_else`, `expect`, Option/Result-based control flow.
* **Iteration:** Variable-bound iteration, iterator short-circuiting (e.g., `.take_while()`).
* **Hidden Panics:** Checked arithmetic with branch-bearing handling, bounds-check panic paths.

## Why They Create Hidden Branches

These constructs are banned because they translate into data-dependent conditional jumps at the machine code level:
* **`unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`:** Under the hood, these evaluate whether a value is `Some`/`Ok` or `None`/`Err`. This implicit check introduces a branch to either unwrap the value or execute a panic/fallback path.
* **`?` Operator:** The try operator expands into a hidden `match` statement that either yields the success value or executes an early return with the error, introducing an immediate branch.
* **Iterator Short-circuiting / Variable-bound Iteration:** Operations that terminate loops early based on runtime data (like `.any()`, `.find()`, or `.take_while()`) require conditional checks on every iteration. They cause the instruction shape and execution time to vary depending on the input.
* **Bounds-check Panic Paths:** Indexing operations (e.g., `arr[i]`) implicitly inject branches to verify if the index is within bounds, jumping to a panic handler if it isn't.

## How the Scanner Inspects the Syntax Tree

To rigorously enforce this rule, the repository uses custom Rust syntax tree scanners (e.g., `bcinr-contract-gate` and `bcinr-cheat-scanner`). Rather than just doing text-based regex matching, they parse the true structure of the code to prevent evasion:

1. **AST Parsing:** The scanner parses the Rust source code into an Abstract Syntax Tree (AST) using the `syn` crate.
2. **Traversal:** It implements a visitor pattern (`syn::visit::Visit`) to walk through every expression in the AST.
3. **Complexity Tracking:** It tracks a `complexity` counter for each function, starting at 1. 
4. **Pattern Detection:** 
   * When it encounters explicit branch expressions like `Expr::If`, `Expr::Match`, `Expr::Loop`, `Expr::While`, or `Expr::ForLoop`, it increments the complexity.
   * When it encounters the `?` operator (`Expr::Try`), it increments the complexity.
   * It intercepts method calls (`Expr::MethodCall`) and inspects the method identifier. If the identifier is `"unwrap"`, `"expect"`, `"unwrap_or"`, or `"unwrap_or_else"`, it increments the complexity.
5. **Enforcement:** If a function's final complexity exceeds 1, the scanner flags a violation ("Branch detected!") and blocks the merge. It also builds a reachability graph to recursively check the transitive call graph, ensuring that branches aren't hidden inside private helpers, trait implementations, macros, or reachable dependencies.
