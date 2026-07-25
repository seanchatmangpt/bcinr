# BCINR Rule 7: Whole-Call-Graph Branchlessness Audit Strategy

According to the BCINR constitution (`AGENTS.md`), Rule 7 mandates that branchlessness applies to the *entire* transitive call graph, not merely the public entry point. Source-level checks (e.g., "contains no `if`") are considered necessary but insufficient. The `@turing_machine` (structural auditor) ensures the $CC=1$ (Radon Law) constraint through a rigorous **Object-Code Audit** rather than trusting source code.

## 1. Object-Code Traversal Strategy

To prove compliance, the project strictly evaluates the **release profile disassembly**. The traversal strategy follows these steps:

1. **Harness Execution for Codegen:** To prevent LLVM from aggressively eliminating dead code or inlining standard `.rlib` outputs, dedicated linked-executable harnesses (e.g., `bcinr-cmca-audit-harness`) are used. These force LLVM to retain the exact machine-code shape as it will execute in production.
2. **Disassembly Generation:** The system builds the release artifact across the required feature matrix (`cargo build --release -p <crate>`) and generates raw textual assembly using platform-specific tools (`otool -tv` for macOS, `objdump -d` for Linux, or `cargo asm`).
3. **Exhaustive Call-Graph Enumeration:** The auditor maps the transitive closure of the call graph:
   `Root function → Direct callees → Transitive callees → Compiler intrinsics → Linked runtime symbols`
4. **Per-Symbol Classification Table:** Every identified symbol in the final binary is placed into an audit matrix and must verify:
   - $CC=1$
   - 0 Conditional jumps (`je`, `jne`, etc.)
   - 0 Loop backedges
   - 0 Panic paths
   - 0 Allocator calls (`__rust_alloc`)
   Any symbol left unclassified or failing these checks immediately blocks the merge.

## 2. Auditing `panic` Handlers and `unwrap`s

Because panics and `unwrap`s often hide deep inside dependencies or are implicitly inserted by the compiler (e.g., array bounds checks, overflow checks), they are audited at the assembly level:

- **Mechanical Symbol Scan:** Within the raw textual disassembly, the auditor scans the instruction graph for prohibited references.
- **Target Symbols:** It specifically looks for `core::panicking::*`, `panic_bounds_check`, `unwrap_failed`, `Option::unwrap`, `Result::unwrap`, and `expect`.
- **Enforcement:** Any `JMP`, `CALL`, or branch to these panicking symbols in the generated assembly triggers an absolute violation. The "Panic path" column in the per-symbol classification must be `No` for a feature to attain `ALIVE` or `BRANCHLESS_ALIVE` standing.

## 3. Auditing Traits and Generic Monomorphizations

Rust's abstractions, such as traits and generics, are notorious for concealing dynamic dispatch or type-specific branching logic. BCINR audits them by tracking their instantiated machine code:

- **Monomorphized Symbol Tracking:** Standing is never granted to an abstract generic function definition. The auditor tracks every concrete, monomorphized symbol (e.g., `process::<ComplexType>`) generated in the release build and inspects each unique machine-code call graph individually for conditional jumps introduced by the substituted type.
- **Blocking Dynamic Dispatch:** Using `&dyn Trait` introduces runtime vtable lookups, which rely on unpredictable control flow. The object-code audit strictly flags any symbol containing an **indirect call** instruction (e.g., `call *%rax`) as an immediate violation of the branchless contract.

By enforcing this deep, transitive object-code audit, BCINR guarantees that no hidden conditional logic, dynamic dispatch, or panic paths can bypass the substrate's deterministic constraints, even in deep transitive dependencies.
