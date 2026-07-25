# Enforcement of the Zero Heap Allocation Law

Under the BCINR Deterministic Substrate Constitution (`AGENTS.md`), relying solely on the `#![no_std]` crate-level attribute is considered necessary but **insufficient** to guarantee zero heap allocations. The `@turing_machine` enforcer requires explicit proof that the *compiled release artifact* does not trigger the OS memory manager. This is achieved through a rigorous **Object-Code Audit** pipeline.

## 1. Exact Release Profile Compilation
The audit begins by compiling the project using the exact production release profile (`cargo build --release`) across all supported feature flags and target architectures. This ensures the analyzed code perfectly matches the artifact that will run in production.

## 2. Anti-Dead-Code Evasion Harnesses
To prevent the compiler from optimizing away unused allocation paths (a form of "dead-path compliance" or scanner evasion), the audit relies on linked executable harnesses. These test harnesses explicitly call the authoritative roots (e.g., `allocate(...)`) with fixed inputs, fold the entire resulting state into a checksum, and sink it (e.g., via `println!`). This forces the compiler to emit the full transitive call graph into the binary.

## 3. Disassembly Extraction
Once the release artifact or linked executable is built, the pipeline extracts the raw machine instructions. For example, via `cargo make audit-object-code`:
- On macOS (`Darwin`), it uses `otool -tv`.
- On Linux, it uses `objdump -d`.

## 4. Per-Symbol Classification
The `@turing_machine` (via the `object-code-audit` skill) traverses the entire authoritative call graph—from direct callees down to compiler intrinsics. It explicitly scans the disassembly for:
- **Rust Allocator Symbols**: `__rust_alloc`, `alloc::alloc::*`
- **Libc Allocator Symbols**: `malloc`, `free`, `calloc` (if any FFI boundaries exist)

## 5. Tabulation and Standing
Every symbol is strictly tabulated in an `OBJECT_CODE_AUDIT.md` matrix mapping to its structural compliance metrics:

| Symbol | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
|---|---:|---:|---:|---|---|---|
| `allocate` | 1 | 0 | 0 | No | No | `BRANCHLESS_ALIVE` |

If a single symbol in the hot-path transitive graph resolves to an allocator reference, the crate standing drops to `UNKNOWN` or fails outright, the Substrate Integrity Score (SIS) falls to 0, and the merge is hard-blocked. No exceptions or waivers are granted for hot-path authoritative allocations.
