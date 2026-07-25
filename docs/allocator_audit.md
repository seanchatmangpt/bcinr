# Allocator Symbol Detection Mechanics (Rule 20)

In the `bcinr` repository, enforcing the zero-allocation law (Rule 20) requires more than just relying on the `#![no_std]` crate-level attribute. The `@turing_machine` enforcer mandates explicit proof that the compiled release artifact is completely allocation-free on the hot path. 

This verification is performed through a rigid **Object-Code Disassembly Audit Pipeline**:

## 1. Release Profile Compilation
The audit begins by compiling the target authoritative crate using the exact production release profile (`cargo build --release`). This is done across all supported feature flags and target architectures to ensure the analyzed code is identical to what will run in production.

## 2. Anti-Dead-Code Evasion Harnesses
To prevent modern compilers (like LLVM) from optimizing away unused allocation paths (a form of "dead-path compliance" or scanner evasion), the pipeline utilizes **linked-executable harnesses**:
- The harness calls the authoritative hot-path root functions with fixed inputs.
- It sinks the output (e.g., via `println!` or folding into a checksum) to force the compiler to preserve the full transitive execution path in the final binary.
- The resulting final linked binary is what gets analyzed, allowing observation of the true codegen shape.

## 3. Raw Disassembly Extraction
The raw machine instructions are extracted from the compiled artifact using platform-specific disassemblers via the `cargo make audit-object-code` task:
- **macOS (`Darwin`)**: `otool -tv <artifact>`
- **Linux**: `objdump -d <artifact>`

This task generates a reproducible raw text dump of the assembly (e.g., to `target/audit/bcinr-cmca-object-audit.txt`).

## 4. Per-Symbol Classification
Because the exact classification is not fully automated by CI scripts, a manual or agent-driven (`@turing_machine` using the `object-code-audit` skill) review is required. 

The auditor enumerates the complete authoritative call graph (from direct callees down to compiler intrinsics) and scans the raw disassembly dump specifically for:
- **Rust Allocator Symbols**: `__rust_alloc`, `alloc::alloc::*`
- **Libc Allocator Symbols**: `malloc`, `free`, `calloc` (if any FFI boundaries exist)

## 5. Tabulation and Hard-Blocking
The findings for every symbol are tabulated in a rigid `OBJECT_CODE_AUDIT.md` matrix required for verification gates:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |

**Enforcement:**
If a single symbol in the hot-path transitive graph resolves to an allocator reference:
- The crate's standing drops to `UNKNOWN` or fails outright.
- The Substrate Integrity Score (SIS) drops to 0.
- The pull request/merge is **hard-blocked**. 
- No exceptions or waivers are granted for hot-path authoritative allocations.
