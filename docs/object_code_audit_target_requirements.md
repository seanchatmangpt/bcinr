# Object-Code Audit and Target Requirements

## The Necessity of Production-Profile Disassembly

Rule 20 of `AGENTS.md` mandates an exact production-profile disassembly audit for every supported release target. This rigorous requirement exists because **source-level `CC=1` (Cyclomatic Complexity = 1) is necessary but insufficient** to guarantee a deterministic, constant-time runtime environment. 

While a Rust function may be written without explicit branching constructs (`if`, `match`, `while`, etc.), the compiler can still introduce branches or prohibited instructions during its translation to machine code. Furthermore, compiler optimizations and target-specific lowering can vary drastically across different Instruction Set Architectures (ISAs). An operation that compiles to a single, branchless hardware instruction on one target might be lowered into a branching software fallback or a standard library call on another. Therefore, an audit must evaluate the exact production-profile machine code generated for *each* supported release target to assure absolute compliance with the BCINR runtime laws.

## Transcending Source-Level `CC=1` Compliance

Relying solely on source-code inspection creates vulnerabilities to "hidden" branches and behaviors inserted by the compiler. The object-code audit transcends source-level compliance by addressing the following realities:

* **Bounds Checks and Panics**: Array indexing or checked arithmetic in Rust may be transparently wrapped with panic paths (and therefore branches) if the compiler cannot statically prove safety.
* **Compiler-Generated Branches**: Seemingly branchless logical operations or math patterns might be "optimized" by LLVM into conditional jumps depending on the target's heuristics.
* **Incomplete Unrolling**: A fixed-bound source loop might not be fully unrolled by the compiler, leaving a loop backedge in the compiled binary.
* **Hardware Fallbacks**: Missing hardware features on a specific target architecture (e.g., PDEP/PEXT) might cause the compiler to generate runtime function calls (which may branch internally) to emulate the behavior.

The object-code audit closes this gap by proving that the final executable artifact mathematically adheres to the structural laws, replacing source claims with hard disassembly evidence. As stated in Rule 7, the permitted claim is never "the function contains no `if`," but rather: 

> *"The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."*

## Forbidden Instruction Types and Symbols

During the disassembly audit, the following specific instruction types and symbol categories must be inspected and are strictly forbidden within the authoritative call graph:

1. **Conditional Jumps**: Violate the law of branchless execution and introduce data-dependent execution times, opening up timing side-channels.
2. **Loop Backedges**: Indicate incomplete loop unrolling or dynamic bounds, violating the requirement for fixed bounded execution work.
3. **Indirect Calls**: Violate the prohibition on dynamic dispatch, preventing static analysis of the complete authoritative call graph and breaking predictability.
4. **Floating-Point Instructions**: Forbidden by the deterministic math requirements; all logic must use fixed-point arithmetic to guarantee bit-for-bit reproducibility across all hardware.
5. **Division Instructions**: Generally forbidden due to their variable hardware latency on many microarchitectures.
6. **Panic and Bounds-Check Symbols**: Introduce early termination and unwinding paths, violating the mandate for fixed execution flow and bounded error handling (Typed Refusals).
7. **Allocator Symbols**: Violate the absolute zero-heap-allocation boundary and the `#![no_std]` constraint.
8. **Unexpected Runtime Library Calls**: Can hide arbitrary complexity, branching, and allocation behind opaque function boundaries.

## Audit Output and Enforcement

The audit result must explicitly list every authoritative root symbol and transitive helper symbol. A symbol's standing is only considered "ALIVE" if it provides verifiable metrics demonstrating 0 conditional jumps, 0 loop backedges, and no panic or allocator paths. Any unclassified or failing authoritative symbol in the disassembly automatically fails the structural enforcement gate and blocks the merge.
