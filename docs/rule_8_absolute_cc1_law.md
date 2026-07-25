# Rule 8: The Absolute `CC=1` Law (Branchless Execution)

In the `bcinr` deterministic computational substrate, **Rule 8** establishes the absolute law of **Cyclomatic Complexity = 1** (`CC=1`) for authoritative code. This ensures branchless, bounded, and side-channel-free execution (also referred to as the **Radon Law**). 

Logic must be expressed strictly as arithmetic operations, bitwise polynomials, and mask-based state selection rather than semantic control flow.

## Prohibited Control-Flow Constructs

To guarantee exactly one unified execution path, the following Rust constructs are strictly prohibited in the authoritative call graph:

* `if`, `if let`, and `else`
* `match`
* `while` and `loop`
* `break` and `continue`
* `early return`
* `?` (Try operator)
* `unwrap`, `unwrap_or`, `unwrap_or_else`, and `expect`
* Checked arithmetic with branch-bearing handling
* `Option`-based control flow
* `Result`-based control flow
* Iterator short-circuiting
* Variable-bound iteration
* Bounds-check panic paths

## Why Implicit Branches Violate the Law

The `CC=1` law applies transitively to the entire call graph (the source code, macro expansions, trait implementations, and compiled object code). The goal is to enforce **zero data-dependent conditional jumps** and **zero panic paths**.

Constructs that seem declarative or innocuous often conceal runtime branches:

1. **Iterator Short-Circuiting**:
   Operations that terminate an iteration early (such as `take_while`, `find`, or manual `break` logic) introduce data-dependent conditional jumps. The execution time and instruction path become dependent on the runtime data, violating constant-time guarantees and introducing timing side-channels. All loops in `bcinr` must be fixed-width, bounded, fully unrolled, or proven to be free of loop backedges at the object code level.

2. **`unwrap`, `expect`, `unwrap_or_else`, `?`**:
   While ergonomic in standard Rust, these methods internally expand to `match` or `if let` statements. They evaluate the runtime state of an `Option` or `Result` and conditionally branch to a separate execution path (e.g., invoking a panic handler or returning early). In `bcinr`, such sequential semantic decisions must instead be transformed into branchless bitwise masks and arithmetic selection.

3. **Bounds-Check Panic Paths**:
   Native slice or array indexing (`slice[index]`) inherently introduces an implicit bounds check. If the compiler cannot prove the index is in-bounds, it emits an invisible branch (`if index >= len { panic() }`). This means the function can branch into a panic symbol. To remain lawful under `CC=1`, code must use strictly masked arrays, fixed bounds, or arithmetic bounds-clamping so that the compiler generates contiguous, uninterrupted machine code with no reachable panic pathways.

## Enforcement Details

To prevent any evasion of the branchless contract, structural auditors (like `@turing_machine`) enforce the following constraints:
* **AST Scanning**: The scanner inspects the parsed Abstract Syntax Tree (AST), not just source lines, to catch hidden operators.
* **Transitive Reachability**: Branches hidden inside macros, trait implementations, or third-party dependencies count as violations if they are reachable from the authoritative call path.
* **Object-Code Audits**: Source-level compliance is not enough. The authoritative runtime strictly forbids any conditional jump, loop backedge, indirect call, or panic handler access at the compiled assembly level.
