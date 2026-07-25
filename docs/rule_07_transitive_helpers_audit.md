# Analysis of Rule 7: Whole-call-graph branchlessness

Based on the BCINR `AGENTS.md` Deterministic Substrate Constitution, Rule 7 dictates that **"branchlessness applies to the transitive call graph, not merely the public entry point."** This is enforced rigorously because a single source-level branchless function might still compile into machine code containing branches, violating the Radon Law ($CC=1$) and the deterministic nature of the substrate. 

Here is why **"serialization helpers reachable at runtime"** and **"language-generated panic paths"** are explicitly required to be audited across the transitive call graph:

### 1. Language-Generated Panic Paths
In Rust, the compiler automatically inserts implicit branches for safety checks that can lead to panics. 
- **Hidden Branches:** Operations like array indexing (bounds checks), arithmetic overflows, and division by zero result in hidden `if` conditions in the compiled object code (e.g., `if index >= len { panic_bounds_check() }`).
- **Violation of $CC=1$:** These implicit conditions violate the absolute $CC=1$ law (Rule 8 explicitly prohibits "bounds-check panic paths" and "unwrap" operations because they produce control-flow branches).
- **Unwinding and Non-Determinism:** A panic path introduces unwinding and uncontrolled exit semantics which violate the core requirement of "fixed bounded execution work," "no unwinding," and "no panic paths" outlined in the Absolute Runtime Laws (Rule 3).
- **Enforcement:** To be truly branchless, the code must be written such that the compiler can statically prove bounds and omit the panic branch, leaving a pure, straight-line assembly block. Thus, scanning object code for panic symbols (Rule 20) is mandatory.

### 2. Serialization Helpers Reachable at Runtime
Serialization logic is inherently prone to control flow and dynamic behavior, which makes it dangerous if leaked into the authoritative hot path.
- **Dynamic Branching and Looping:** Serialization typically involves iterating over variable-length data, parsing types, or conditionally formatting output. This directly violates laws prohibiting "variable graph traversal," "data-dependent branches," and "data-dependent loop termination" (Rule 3).
- **Allocation and Dynamic Size:** Serialization helpers often allocate memory to format data (e.g., building strings or byte vectors). The authoritative hot path is strictly a `#![no_std]` environment with a "Zero-Allocation Boundary" (0 heap allocations). 
- **Slow Rail Segregation:** `AGENTS.md` (Rule 6) designates "artifact serialization" to the "Slow rail" domain, which is allowed to branch and allocate but **must never be linked into or invoked from the authoritative hot path**. If a serialization helper is reachable in the runtime graph, it indicates a failure in isolating the deterministic core from the slow rail.

By explicitly auditing these two areas, `@turing_machine` (the structural auditor) ensures that no hidden branches, allocations, or variable-time execution bypass the strict constraints of the authoritative substrate.
