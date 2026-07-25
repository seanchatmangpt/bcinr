# Rule 7: Transitive Callees & Whole-Call-Graph Branchlessness

Under the BCINR constitution (`AGENTS.md` Rule 7), branchlessness is a strict physical law applied to the **transitive call graph**, not a mere syntactic preference for source code. 

## The Insufficiency of a Shallow Check
Claiming "the function contains no `if`, therefore it is branchless" is explicitly prohibited. A shallow source-level check of the public entry point is insufficient because an outwardly $CC=1$ (Cyclomatic Complexity = 1) function can trivially violate deterministic execution by delegating to a branching helper, an allocation-backed dependency, or a panicking standard library trait method. 

In BCINR, the authoritative runtime requires deterministic, bounded, allocation-free execution where timing side-channels are physically impossible. A branch hidden deep in the execution path violates this mandate just as completely as a top-level `if` statement. Therefore, verification must prove that the *entire* authoritative call graph contains no input-dependent conditional branches in the final release object code.

## Auditing Deep Call-Graph Components

To guarantee true whole-call-graph branchlessness, the audit must recursively inspect the following layers of the compilation stack:

### 1. Generic Monomorphizations
In Rust, generic functions are instantiated (monomorphized) by the compiler for each concrete type used. A generic algorithm might appear structurally branchless in its source AST, but the specific monomorphized instance can secretly introduce branches depending on the concrete type's trait implementations (e.g., `<T as Trait>::method()`). Auditing monomorphizations ensures that every concrete type substituted into the authoritative runtime produces straight-line, bit-parallel object code without introducing type-specific control flow.

### 2. Compiler Intrinsics
The Rust compiler and LLVM routinely lower seemingly innocent operations into compiler intrinsics (e.g., bounds checks, complex arithmetic, memory copying). While these appear as single operations in the source, the backend might expand them into conditional branches, loops, or traps—especially on architectures that lack native hardware instructions for the requested operation (such as software-emulated division or floating-point routines). Auditing intrinsics verifies that LLVM maps these operations directly to branchless hardware instructions.

### 3. Linked Runtime Symbols
Even in `#![no_std]` environments, binaries often link against implicit routines from `compiler_builtins` (like `memcpy`, `memset`, `memcmp`, or math fallbacks). These foundational symbols are typically written in C or assembly and contain highly optimized, branch-heavy logic designed to handle alignment checks, variable chunk sizes, and loop terminations. If an authoritative function implicitly links to one of these routines, it inherits its branches and loop backedges, violating the $CC=1$ mandate.

### 4. Serialization Helpers
Standard serialization and deserialization frameworks (like Serde) rely heavily on state machines, dynamic variant matching (`match`), and loop-based parsing over variable-length inputs. If traditional serialization helpers are reachable in the authoritative runtime, they introduce strictly prohibited variable graph traversal and data-dependent loop termination. In BCINR, state encoding must be executed via fixed-width bitwise packing and masks. Any serialization helper reachable at runtime must undergo an object-code audit to prove it has been flattened into branchless, bounded machine instructions.
