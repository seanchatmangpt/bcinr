# Turing Machine Object-Code Auditing for CC=1

The `@turing_machine` (the "Enforcer of Determinism") explicitly uses disassembly and object-code auditing to mechanically verify that the Rust compiler (`rustc` and LLVM) does not silently insert `JCC` (Jump Conditional Code) branches, loop backedges, or panic paths into the `CC=1` hot path. This is a critical requirement of the "Radon Law" ($CC=1$) and the absolute runtime laws defined in the project constitution (`AGENTS.md`).

## 1. Why Source-Level Auditing is Insufficient

According to `AGENTS.md` (Section 20), source-level `CC=1` (no `if`, `match`, or data-dependent `loop` structures) is *necessary but insufficient*. The compiler might implicitly insert conditional branches that violate branchless execution, such as:
- Array/slice bounds checks (`panic_bounds_check`).
- Integer overflow checks (in debug or improperly configured release profiles).
- Implicit loop backedges for iterators.
- Dynamic dispatch or indirect calls.
- Allocator code or implicit unwinding paths.

To ensure pure mathematical determinism and branchless execution, the `@turing_machine` mandates inspecting the final, exact production-profile disassembly.

## 2. Mechanical Object-Code Auditing Workflow

The project uses a structured, mechanical approach to object-code auditing:

### The `audit-object-code` Task
The repository includes a dedicated `cargo make audit-object-code` task in `Makefile.toml`. This task performs the following steps:
1. Builds the target authoritative crate in release profile (`cargo build --release -p <crate>`).
2. Locates the compiled artifact (`.rlib`, `.a`, `.dylib`, or `.so`).
3. Executes a platform-specific disassembler (`otool -tv` on Darwin/macOS, or `objdump -d` on Linux) to produce a raw textual dump of the assembly.

### Harnesses for Accurate Disassembly
Because modern compilers aggressively inline and eliminate dead code, auditing `.rlib` files directly can be unreliable. As documented in `crates/bcinr-cmca/OBJECT_CODE_AUDIT.md`, the `@turing_machine` employs dedicated **linked-executable harnesses** (e.g., `bcinr-cmca-audit-harness`). 
- A harness binary calls the specific hot-path root function (like `bcinr_cmca::allocator::allocate`) using fixed inputs.
- It sinks the result (e.g., folding it into a checksum and printing it) to prevent the optimizer from stripping the code.
- Disassembly is then performed on this final linked executable to obtain the true `rustc`/LLVM codegen shape.

## 3. The Per-Symbol Classification Matrix

Once the raw disassembly is generated, the `@turing_machine` requires it to be parsed and classified into a strict per-symbol table. The audit must inspect all authoritative root symbols and all transitive helper symbols.

The required evidence format is:
| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |

For a symbol to pass the audit and maintain `ALIVE` standing:
- **Conditional jumps**: Must be `0`. The disassembler output is checked for conditional jump instructions (like `je`, `jne`, `ja`, etc. on x86, or equivalent on other architectures).
- **Loop backedges**: Must be `0`. The execution flow must be completely unrolled and straight-line.
- **Panic path**: `No`. Code must be free of calls to `core::panicking::*`.
- **Allocator**: `No`. Memory operations must use fixed-size stack or static arena allocations.

## 4. Enforcement and Merge Gates

The `@turing_machine` serves as the merge gatekeeper for these audits. 
- It actively searches for findings like `CHEAT[rule-id]`, which includes attempting to hide behavior behind macro expansions, private helpers, or generating prohibited source code.
- The authoritative call graph classification extends to *everything*: macro-generated branches, trait monomorphizations, compiler intrinsics, and fixed-point helpers. 
- If the object-code inspection fails or cannot be completed (e.g., if a build fails as noted in `OBJECT_CODE_AUDIT.md`), the project standing is blocked or marked `UNKNOWN`, and the feature is not merged. 

In summary, the `@turing_machine` ensures that the "branchless" claim is not merely syntactic sugar in Rust source, but a mechanically verified guarantee in the final machine code executing on the processor.
