# Object Code Verification Pipeline

The Object Code Verification Pipeline is the ultimate enforcement mechanism in the BCINR Deterministic Substrate, governed by **Rule 20** and enforced by the **`@turing_machine`** agent. It ensures that the strict, branchless, deterministic laws of the constitution are genuinely reflected in the final executable artifact.

## The Role of `@turing_machine`

The `@turing_machine` acts as the Enforcer of Determinism, serving as the structural auditor and merge gatekeeper. It holds exclusive authority over:
* Cyclomatic complexity (`CC=1`) enforcement.
* Authoritative-call-graph classification.
* Source and object-code audits.
* Panic-path and allocation audits.

The core mandate for the `@turing_machine` is to verify that **the authoritative instruction shape never depends on semantic input**.

## Why Disassembly is the Ultimate Source of Truth

The constitution explicitly states: *"Source claims do not substitute for disassembly evidence."* 

While enforcing `CC=1` (branchlessness) at the Rust source level is necessary, it is entirely insufficient to guarantee deterministic execution. The Rust compiler and LLVM optimizer can dramatically alter the structural execution of code:
* **Hidden Branches:** A seemingly simple array access might implicitly inject a bounds-check branch.
* **Implicit Panics:** Mathematical operations might compile into overflow-check branches or panic paths.
* **Compiler Optimizations:** The compiler might transform a complex bitwise polynomial into a conditional jump, or introduce loop backedges when compiling down `Iterator` abstractions that were expected to unroll.
* **Trait Dispatch:** Generic code might unexpectedly resolve into indirect dynamic dispatch (e.g., via vtables) if not perfectly monomorphized.

Thus, inspecting the parsed abstract syntax tree (AST) can only prove the *intent* of branchlessness. The exact production-profile disassembly provides the *physical evidence* of execution. 

## Mechanics of Symbol-by-Symbol Audits

Every supported release target requires an exact production-profile disassembly audit. The object-code audit pipeline systematically deconstructs the compiled binary and inspects:
1. **Scope:** All authoritative root symbols and their transitive helper symbols.
2. **Exclusions:** It confirms that zero panic, bounds-check, or allocator symbols are reachable in the authoritative path.
3. **Instruction Set Verification:** It validates that no unexpected runtime library calls, floating-point instructions, or division instructions exist (unless explicitly admitted).

Any unclassified authoritative symbol blocks the merge process.

## Identifying Violations in Machine Code

The pipeline identifies violations by applying structural checks directly against machine-code opcodes and control-flow graphs (CFG):

* **Conditional Jumps:** The auditor scans the emitted machine code for data-dependent jump instructions (e.g., `je`, `jne`, `jg` in x86-64, or `b.eq`, `cbz` in ARM). The presence of any such instruction in authoritative symbols constitutes an immediate violation of the branchless mandate.
* **Loop Backedges:** The auditor analyzes the CFG of each symbol. A loop backedge is detected when a branch instruction targets an address earlier in the instruction stream of the same function block. Since all execution must be bounded and fixed, runtime loops are prohibited. All loops must be macro-unrolled, const-generic unrolled, or proven by the compiler to be fully unrolled in the object code.
* **Indirect Calls:** The auditor flags jump or call instructions that rely on dynamic register values (e.g., `call *%rax` or `blr`). This prevents dynamic dispatch, vtable lookups, and unexpected graph traversals, ensuring the call graph is 100% statically determinable.
* **Panic and Allocator Reachability:** The pipeline inspects call targets to ensure there are zero references to symbols like `core::panicking`, `__rust_alloc`, `malloc`, or internal bounds-check failure handlers.

## Mandatory Evidence Format

A successful audit generates a cryptographic receipt and must produce a rigorous tabular breakdown. The pipeline emits the following permitted evidence format for the target architecture:

| Symbol | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| :--- | --: | --: | --: | --: | --: | :--- |
| `cmca_allocate` | 1 | 0 | 0 | No | No | ALIVE |
| `verify_envelope` | 1 | 0 | 0 | No | No | ALIVE |

If any symbol shows Conditional jumps > 0, Loop backedges > 0, or relies on a Panic path or Allocator, its standing fails, `SIS` drops to 0, and the merge is universally blocked.
