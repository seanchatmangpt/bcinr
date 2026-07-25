# The Anti-Panic Law and `@turing_machine` Assembly Audit

According to the constitution (`AGENTS.md`) and the object-code auditing protocols of the `bcinr` (BranchlessCInRust) project, source-level assertions like "contains no `if`" or "uses no panicking calls" are considered **necessary but insufficient**. The Rust compiler and LLVM might silently inject implicit branches, overflow checks, or array bounds checks. To strictly ban all panic pathways (enforcing the Anti-Panic Law), the `@turing_machine` requires a mechanical object-code audit on the final assembly.

Here is how the `@turing_machine` physically scans the final assembly for `core::panicking::*` and `unwrap_failed` symbols:

### 1. Harness Execution to Force LLVM Codegen
Because modern compilers aggressively eliminate dead code or inline functions in standard `.rlib` outputs, the `@turing_machine` employs dedicated **linked-executable harnesses** (e.g., `bcinr-cmca-audit-harness`). 
- A harness binary explicitly calls the specific hot-path root function with fixed inputs.
- It sinks the result (e.g., via a checksum or printing) so LLVM is forced to retain the true code shape rather than stripping it.
- This creates a final linked executable containing the exact `rustc`/LLVM codegen shape as it will run in production.

### 2. Disassembly Generation
The `@turing_machine` generates raw textual assembly using platform-specific disassembly tools on the fully compiled release artifacts (built via `cargo build --release` across required feature matrix targets):
- macOS/Darwin: `otool -tv`
- Linux: `objdump -d`
- Alternatively, `cargo asm` can be used.

### 3. Exhaustive Call-Graph Walk
The audit enumerates the *whole authoritative call graph* for every root symbol, walking downwards through:
`root function → direct callees → transitive callees → compiler intrinsics → linked runtime symbols`

This includes inspecting private functions, trait monomorphizations, macro expansions, integer arithmetic, and standard library behaviors.

### 4. Physical Symbol Scan
Within the raw textual dump, the `@turing_machine` mechanically scans for prohibited references and symbols across the entire instruction graph, specifically targeting:
- `core::panicking::*` (including bounds check failures like `panic_bounds_check`)
- `unwrap_failed`
- `Option::unwrap`, `Result::unwrap`, `expect`

Any JMP, CALL, or branch to these symbols inside the generated assembly triggers an absolute violation.

### 5. The Per-Symbol Classification Matrix
The final verification requires an explicit, per-symbol classification table documenting the complete absence of panic pathways:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |

For a symbol to receive `ALIVE` or `BRANCHLESS_ALIVE` standing, the **Panic path** column MUST be `No`. Even a single panic path hidden deep in an implicit slice indexing operation halts the feature from merging, mathematically guaranteeing zero panic pathways in the deterministic substrate.
