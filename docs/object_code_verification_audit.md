# Object-Code Verification (`audit-object-code`) Implementation in BCINR

Based on the repository analysis, the Object-Code Verification (Rule 20, `audit-object-code`) is a strictly enforced process in the `bcinr` project, but **it does not currently use automated tools or scripts to parse the disassembler output**. Instead, it generates a reproducible raw dump which is then manually audited (typically by the `@turing-machine` agent role) to classify symbols and detect branches or allocations.

Here are the details of how the process is currently implemented:

## 1. The `cargo make audit-object-code` Task
The core automation resides in `Makefile.toml` under `[tasks.audit-object-code]`. Its responsibilities are restricted to extracting the raw machine instructions from the final release artifact:

1. **Release Build**: It builds the target crate in release mode (`cargo build --release -p bcinr-cmca`).
2. **Artifact Discovery**: It locates the compiled library or executable in `target/release` (e.g., `libbcinr_cmca*` `.rlib`, `.a`, `.dylib`, or `.so`).
3. **Disassembly**: It runs a platform-specific disassembler to create a raw textual dump of the assembly.
   - On macOS (`Darwin`), it uses `otool -tv`.
   - On Linux, it uses `objdump -d`.
4. **Raw Dump Export**: The output is written to a centralized file: `target/audit/bcinr-cmca-object-audit.txt`.

## 2. Lack of Automated Parsing Scripts
The repository explicitly notes the absence of an automated parser. The `Makefile.toml` task prints the following disclaimer:

> `"NOTE: this task produces only the reproducible raw dump. Per-symbol classification (conditional jumps, loop backedges, panic paths, allocator calls) per L3-3 / AGENTS.md §7/§13/§20 is a follow-on manual/tool step, not yet automated here."`

Earlier specifications (like `V26_7_17_HOOK_SPEC.md`) referenced a potential script `scripts/release/audit-object-code.sh`, but the documentation noted it "does not exist today," and a file system check confirms it was never created.

## 3. The Manual/Agent-Driven Audit Process
Because there is no script to parse the output, the verification relies on the `.claude/skills/object-code-audit/SKILL.md` process, which tasks the AI agent (acting as `@turing-machine`) or a human reviewer to perform the analysis manually:

1. **Enumerate the Call Graph**: Identify the authoritative root symbol and trace all direct/transitive callees, intrinsics, macro expansions, and runtime symbols.
2. **Inspect Symbols**: Read the generated `objdump` / `otool` output and look for:
   - Conditional jump instructions (data-dependent branches).
   - Loop backedges.
   - Panic/bounds-check symbols (`core::panicking::*`, `Option::unwrap`).
   - Allocator symbols (`__rust_alloc`, `alloc::*`).
   - Indirect calls (vtable dispatch).
3. **Generate the Audit Table**: Document the findings per symbol in `OBJECT_CODE_AUDIT.md` using the exact format required by Rule 20:

| Symbol | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
|---|---:|---:|---:|---|---|---|

## Conclusion
The `bcinr` project automates the *generation* of object code via `cargo make audit-object-code`, but it relies on manual, agent-driven inspection to parse the `otool`/`objdump` output. Any claim of branchlessness (`BRANCHLESS_ALIVE`) requires this manual tabular proof; source-level checks (like `CC=1` scanners) are considered necessary but insufficient without the raw machine-code evidence.
