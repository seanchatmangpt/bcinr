# Rule 7: Whole-Call-Graph Branchlessness

In the BCINR Deterministic Substrate, Rule 7 dictates that branchlessness must apply to the **entire transitive call graph**, not merely the public entry point. True determinism and constant-time execution cannot be guaranteed by surface-level inspections of a single function.

## Why Looking Only at the Public Entry Point is Insufficient

Evaluating only the public-facing function for branchlessness creates a false sense of security. Modern programming languages, including Rust, aggressively abstract control flow. A function that appears linear at the source level may invoke dependencies or language features that introduce hidden branches. 

To ensure absolute runtime determinism, the audit must encompass every execution path reachable from the authoritative root. The audit must explicitly include:
* **Private functions:** Branches cannot be hidden by moving them into private helpers.
* **Trait methods and generic monomorphizations:** Dynamic or abstracted behavior must resolve to branchless machine code for the specific type instantiations used.
* **Macros and generated modules:** Macro expansions can easily conceal conditional logic or early returns.
* **Indexing operations:** Array or slice indexing can introduce language-generated panic paths (bounds checking).
* **Fixed-point and serialization helpers:** Any utility reachable at runtime must adhere to the same zero-branch laws.
* **Language-generated panic paths:** Unseen branches inserted by the compiler for safety must be mathematically eliminated.

## The Prohibited Claim: "No `if`, therefore it is branchless"

The constitution explicitly prohibits the claim: 
> "The function contains no `if`, therefore it is branchless."

This claim is fundamentally flawed because control flow is not limited to the `if` keyword. Rust provides numerous mechanisms that generate conditional branches, such as:
- `match` statements and pattern matching.
- The `?` operator for error propagation.
- Methods like `unwrap()`, `expect()`, and `unwrap_or_else()`.
- Iteration constructs (`while`, `for`, `take_while`).
- Checked arithmetic or implicit overflow checks.

Source-level simplicity is not evidence of structural branchlessness. The only permitted claim must be based on empirical evidence at the machine code level:
> "The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."

## Auditing Transitive Callees, Intrinsics, and Runtime Symbols

To enforce the absolute $CC=1$ law, the audit trace must rigorously follow the complete execution chain:
`root function → direct callees → transitive callees → compiler intrinsics → linked runtime symbols`

1. **Transitive Callees:** Every function called, however deeply nested in the dependency tree, must be verified. A single branching callee invalidates the entire call graph.
2. **Compiler Intrinsics:** Operations that compile down to branchless instructions on one architecture might fall back to branching software routines on another. The exact target's instruction set capabilities must be verified.
3. **Linked Runtime Symbols:** Core language operations or linked libraries may introduce unexpected jumps or calls. Disassembly evidence is required to prove that the compiler or linker has not injected conditional jumps, loop backedges, or calls to panicking/allocating symbols.

By expanding the jurisdiction of the audit to the final linked object code of the entire transitive graph, BCINR guarantees mathematically rigid, branchless execution that is impervious to source-level abstractions.
