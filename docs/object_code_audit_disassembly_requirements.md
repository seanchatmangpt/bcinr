# Object-Code Audit Disassembly Requirements

## Why Source-Level `CC=1` is Necessary But Insufficient

Under Rule 20 of the BCINR Deterministic Substrate Constitution, achieving a source-level Cyclomatic Complexity (CC) of 1 is required but fundamentally insufficient for proving deterministic, branchless execution. The core philosophy of BCINR dictates that the authoritative runtime instruction shape must not depend on semantic input (Rule 3). 

Relying solely on source-level metrics is inadequate because the Rust compiler (rustc/LLVM) and the target architecture can introduce hidden control flow during the compilation and optimization phases. Specifically:

1. **Compiler-Inserted Branches:** The compiler may inject bounds-check panic paths, division-by-zero checks, or integer overflow checks that introduce branches not visible in the source code.
2. **Hidden Loop Backedges:** A fixed Rust source loop is only lawful if the final machine code is proven fully unrolled and free of loop backedges (Rule 13). Source analysis cannot guarantee that the optimizer successfully unrolled the loop.
3. **Implicit Fallbacks & Library Calls:** Abstractions or arithmetic operations might compile down to branching intrinsic calls or unexpected runtime library calls depending on the specific target architecture (e.g., architectures lacking hardware support for certain operations like PDEP/PEXT).
4. **Language Constructs:** Features like traits or macro expansions can hide branches that appear flat in the parent file. 

Therefore, claiming "The function contains no `if`" is prohibited. The only permitted claim is: *"The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target"* (Rule 7).

## Prohibited Machine Instructions and Symbols

Every supported release target requires an exact production-profile disassembly audit. The audit must inspect all authoritative root symbols and transitive helper symbols. The following must be proven definitively absent from the compiled object code:

*   **Conditional Jumps:** No input-dependent branch instructions (e.g., `je`, `jne` on x86).
*   **Loop Backedges:** No backwards jumps that would enable variable or unbounded execution durations.
*   **Panic and Bounds-Check Symbols:** No calls to panic handlers or bounds-check failures.
*   **Allocator Symbols:** No calls to memory allocation routines (e.g., `malloc`), enforcing the strict zero heap allocation boundary (Rule 3).
*   **Indirect Calls:** No dynamic dispatch or function pointers resolved at runtime.
*   **Floating-Point Instructions:** All logic must be handled via bitwise polynomials or branchless arithmetic; no floating-point operations are admitted.
*   **Division Instructions:** Hardware division is prohibited unless specifically audited, requiring branchless fixed-point division replacements.
*   **Unexpected Runtime Library Calls:** Any hidden fallback functions or compiler intrinsics injected into the hot path.

## Required Evidence Format

The audit result must list each inspected symbol individually. Any unclassified authoritative symbol blocks the merge. The permitted evidence format must look like this:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |
