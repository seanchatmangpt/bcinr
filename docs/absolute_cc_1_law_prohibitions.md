# Absolute `CC=1` Law (Rule 8) Prohibitions

In accordance with Rule 8 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the following Rust constructs are explicitly prohibited in authoritative code when they produce control-flow branches:

- `if`
- `if let`
- `else`
- `match`
- `while`
- `loop`
- `break`
- `continue`
- `early return`
- `?`
- `unwrap`
- `unwrap_or`
- `unwrap_or_else`
- `expect`
- `checked arithmetic with branch-bearing handling`
- `Option-based control flow`
- `Result-based control flow`
- `iterator short-circuiting`
- `variable-bound iteration`
- `bounds-check panic paths`

## Why Hidden Branches Violate the Law

Branches hidden in private wrappers, macros, trait implementations, or dependencies still violate the `CC=1` law because the branchless requirement applies to the **entire transitive call graph**, not just the public entry points. 

According to the constitution (Rules 3, 7, and 8):
- **Transitive Application:** The absolute runtime laws apply transitively across all reachable code. A branchless public function calling a branching private helper, trait method, or dependency is a violation.
- **Scanner Enforcement:** The compliance scanner inspects the parsed syntax tree (AST) and expanded macro outputs rather than just source lines. Object-code audits further inspect the final release assembly, seeing through any syntactic abstractions.
- **Complexity Standing:** Encapsulating a branch inside a private function or a dependency does not reduce the cyclomatic complexity of the overall authoritative execution path. If a branch is reachable from the authoritative call graph, the resulting machine code will contain input-dependent conditional jumps, violating the core mandate of deterministic, fixed-instruction-shape execution.
