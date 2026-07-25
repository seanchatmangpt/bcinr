# Rule 7: Whole-Call-Graph Branchlessness

In the BCINR deterministic substrate, the mandate for branchless execution is not merely a syntactic preference; it is a strict mathematical and physical requirement to ensure determinism and eliminate timing side-channels. Rule 7 explicitly states that branchlessness applies transitively to the entire call graph, not just the public entry point.

## Why Branchlessness Must Be Transitive

A system is only as deterministic as its least deterministic component. The requirement for constant-time, bounded execution means that no part of the hot path can depend on data-driven control flow. Branchlessness must apply to the entire transitive call graph for the following reasons:

1. **Abstractions Hide Branches**: Modern languages like Rust offer powerful abstractions—such as private helper functions, trait method implementations, and generic monomorphizations. If a public, apparently branchless function delegates work to a branching private helper, the execution time and instruction sequence will still vary based on input, violating the deterministic substrate.
2. **Implicit Control Flow**: Standard operations can introduce branches invisibly at the source level. Indexing into a slice (`slice[i]`), fixed-point helpers, serialization helpers, and macro expansions can inject conditional logic (like bounds checking and language-generated panic paths).
3. **Compiler and Runtime Intrusions**: Compiler intrinsics, generated modules, or linked runtime symbols might employ conditional logic under the hood to handle edge cases or optimize certain hardware paths. Transitive auditing ensures these hidden layers are exposed and verified.

The authoritative root must maintain a structural guarantee: 
`root function → direct callees → transitive callees → compiler intrinsics → linked runtime symbols`.
If any node in this graph contains an input-dependent conditional branch, the entire path is compromised.

## The Insufficiency of Source-Level Claims

Rule 7 prohibits the claim:
> *The function contains no `if`, therefore it is branchless.*

This claim is fundamentally insufficient because source code does not execute on the CPU—object code does. A source file devoid of `if`, `match`, or `loop` keywords can still produce branching machine code due to:

- **Compiler Injected Branches**: The Rust compiler routinely inserts conditional branches for bounds checks, overflow checks, and panic unwinding paths.
- **Optimization Artifacts**: The compiler might optimize branchless source code into branching assembly if it deems it faster for a specific target, or conversely, it might fail to emit branchless instructions (like `cmov`) when expected.
- **Target Architecture Variability**: A mathematical operation might compile to a single deterministic instruction on one architecture but require a branching fallback routine on another.

Because of the "Contract with Teeth" and the strict zero-tolerance policy for side-channels, the only permitted claim is:
> *The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target.*

Disassembly evidence is the only authoritative proof. Without inspecting the exact production-profile object code for all symbols reachable from the authoritative root, one cannot guarantee that the compiler hasn't betrayed the branchless intent of the source code.
