# Rule 13: No Unbounded Execution

## The Prohibition of Variable, Short-Circuiting, and Panicking Iteration

In the BCINR authoritative runtime, all execution paths must exhibit a Cyclomatic Complexity (CC) of exactly 1. This "Radon Law" requires the transitive execution graph to be entirely branchless. Because the runtime serves as a deterministic computational substrate where timing side-channels must be physically impossible and execution work is fixed, the following constructs are strictly illegal:

1. **Variable-Bound Iteration (`for item in variable_slice`, `while value > 0`)** 
   Iteration that terminates based on runtime data introduces dynamic loop backedges and execution time that varies by input. This violates the fixed bounded execution work law and introduces timing variations (side channels), destroying the mathematical guarantee of deterministic fixed-time execution.

2. **Iterator Short-Circuiting (`take_while`, `loop { if done { break; } }`)**
   Short-circuiting fundamentally relies on data-dependent branching. Evaluating a condition and deciding whether to continue or break introduces conditional jumps in the object code. This violates the core principle that the authoritative instruction shape must not depend on semantic input.

3. **Bounds-Check Panic Paths**
   Standard slice indexing implicitly introduces branches (`if index >= len { panic!() }`). Panic paths entail unwinding or aborting, resulting in unverified control flow deviations and multiple exit points. The runtime must instead handle edge cases via bitwise masks, branchless clamping, or structural typed refusals, maintaining $CC=1$.

## Lawful Iteration Management

To guarantee that the final machine code contains **no loop backedges** (backward jump instructions), all iteration must be transformed into straight-line instruction sequences. A `for` or `while` loop written in Rust is not inherently accepted, even if its bounds seem fixed, unless the final compiled output is mathematically verified as completely flat.

Iteration must be managed through one of the following structural methods:

1. **Compile-Time Fixed (Const Generics)**: Using fixed-length arrays or `const` bounds where the compiler evaluates the bounds ahead of time and consistently flattens the loop during the release profile optimization.
2. **Macro-Unrolled**: Using procedural or declarative macros to expand loops into sequential, branchless source statements before it even reaches the compiler's backend.
3. **Generated Code**: Utilizing code generation tools (e.g., build scripts on the slow rail) to statically emit fully unrolled, straight-line Rust code.
4. **Demonstrated as Fully Unrolled**: If an explicit loop construct remains in the source code, it must be proven via rigorous object-code disassembly (certified by the `@turing_machine` role) that the release-profile machine code for all supported targets is entirely unrolled, containing exactly zero conditional loop jumps or backedges.

By strictly adhering to these patterns, the BCINR runtime guarantees that execution time and computational resources are strictly bounded, universally predictable, and physically independent of the payload.
