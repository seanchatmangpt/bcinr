# Object-Code Disassembly Audit Pipeline (Rule 20)

In the `bcinr` repository, Rule 20 ("Object-code audit") strictly enforces that all authoritative machine code is 100% branchless ($CC=1$) and allocation-free. However, **the extraction and classification of `CC`, conditional jumps, loop backedges, and panic symbols is not currently automated by CI scripts.** Instead, the CI automates the generation of a reproducible raw assembly dump, which is then manually audited (or parsed by an AI agent acting as `@turing_machine`).

Here is the exact step-by-step disassembly pipeline:

## 1. The `cargo make audit-object-code` Task
The automation for this process lives in `Makefile.toml` under `[tasks.audit-object-code]`. It executes the following steps:

1. **Release Build:** Compiles the target authoritative crate (e.g., `bcinr-cmca`) in the release profile using `cargo build --release -p bcinr-cmca`.
2. **Artifact Discovery:** Finds the resulting library or executable artifact (`libbcinr_cmca*` such as `.rlib`, `.a`, `.dylib`, or `.so`) in `target/release`.
3. **Raw Disassembly:** Runs a platform-specific disassembler to extract the raw machine instructions.
   - **macOS (`Darwin`):** Uses `otool -tv <artifact>`.
   - **Linux:** Uses `objdump -d <artifact>`.
4. **Export:** Writes the raw textual assembly dump into a centralized file at `target/audit/bcinr-cmca-object-audit.txt`.

## 2. Dedicated Linked-Executable Harnesses
Because modern compilers (LLVM) heavily inline code and eliminate dead code, auditing an `.rlib` directly can be unreliable. The pipeline utilizes **linked-executable harnesses** (e.g., `bcinr-cmca-audit-harness`):
- The harness calls the authoritative hot-path root function with fixed inputs.
- It sinks the output (e.g., printing it or folding it into a checksum) to force the compiler to preserve the full execution path.
- The disassembly is performed on this final linked binary to observe the true codegen shape.

## 3. The Lack of Automated Parsing Scripts
The repository explicitly acknowledges the absence of Python/Bash scripts for parsing the assembly. The `audit-object-code` task prints this explicit disclaimer:

> `NOTE: this task produces only the reproducible raw dump. Per-symbol classification (conditional jumps, loop backedges, panic paths, allocator calls) per L3-3 / AGENTS.md §7/§13/§20 is a follow-on manual/tool step, not yet automated here.`

## 4. Manual / Agent-Driven Classification Workflow
After the raw dump is generated, the `@turing_machine` role (either a human reviewer or AI agent) must inspect the `otool`/`objdump` output manually to ensure compliance:

1. **Conditional jumps:** Scans for branching instructions (e.g., `je`, `jne`, `ja` on x86/arm equivalents).
2. **Loop backedges:** Ensures the flow is strictly unrolled and straight-line.
3. **Panic paths:** Checks for the presence of `core::panicking::*` or `Option::unwrap`.
4. **Allocators:** Checks for `__rust_alloc` or `alloc::*`.

The findings are recorded in a rigid per-symbol matrix in `OBJECT_CODE_AUDIT.md`, which is required to pass the verification gates:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |
