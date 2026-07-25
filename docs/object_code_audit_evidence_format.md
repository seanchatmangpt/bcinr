# Object-Code Audit Evidence Format (Rule 20)

According to Rule 20 of the `AGENTS.md` constitution, every supported release target requires an exact production-profile disassembly audit. This ensures that the generated machine code strictly adheres to the deterministic substrate's requirements.

## Permitted Evidence Format

The audit result must list each symbol individually using a mandatory table format. The permitted evidence format is:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |

**Note:** Any unclassified authoritative symbol will immediately block merge.

### Field Definitions:
- **Symbol**: The name of the authoritative root symbol or transitive helper symbol being audited.
- **CC**: Cyclomatic Complexity. For authoritative code, this must be exactly `1`.
- **Conditional jumps**: The number of conditional jump instructions (must be `0`).
- **Loop backedges**: The number of loop backedge instructions (must be `0`).
- **Panic path**: Whether any panic or bounds-check symbols are reachable (must be `No`).
- **Allocator**: Whether any dynamic memory allocator symbols are reachable (must be `No`).
- **Standing**: The current status/standing of the audited symbol (e.g., `ALIVE`).

## Why Source-Level `CC=1` is Necessary but Insufficient

Having a cyclomatic complexity of 1 (`CC=1`) at the source AST level is a strict prerequisite, ensuring no `if`, `match`, or explicit data-dependent loops are written. However, this is **insufficient** on its own because the compiler can introduce hidden branching and complexity during code generation:

1. **Compiler-Injected Branches**: The Rust compiler implicitly inserts conditional jumps for operations such as array bounds checking, arithmetic overflow checking (in debug or specialized profiles), and implicit `Drop` mechanics.
2. **Hidden Panic Paths**: Seemingly linear code might include paths that branch to panic handlers (e.g., failed unwraps, division by zero, or out-of-bounds indexing).
3. **Macro and Generic Expansions**: Macros and generic monomorphizations can expand into hidden control flow that isn't immediately visible in the pre-expanded source text.
4. **Library Calls & Intrinsics**: Function calls to seemingly simple standard library routines or compiler intrinsics may contain hidden branches, allocations, or loop backedges.
5. **Optimization Artifacts**: The compiler's optimizer might restructure the binary layout in ways that create unexpected conditional logic or indirect calls.

Therefore, an exact production-profile disassembly audit is the only way to mathematically prove that the final, executed object code is entirely free of branches, allocations, and panics, preserving the fixed deterministic mechanics of the system.
