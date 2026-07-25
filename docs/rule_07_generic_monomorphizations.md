Based on my review of Rule 7 (Whole-call-graph branchlessness) in `AGENTS.md`, "generic monomorphizations" must be explicitly audited for the following reasons:

1. **Object Code Over Source Claims**: The core philosophy of BCINR dictates that source-level appearance is insufficient (the claim "The function contains no `if`, therefore it is branchless" is explicitly prohibited). The true test of branchlessness is evaluated on the **audited release object code** for the specific target.

2. **Distinct Machine Code per Type**: In Rust, generic functions are monomorphized, meaning the compiler generates unique machine code for every concrete type used. A generic function that appears completely branchless in its source definition might compile into branching logic depending on the specific type it is instantiated with. 

3. **Hidden Branches in Traits and Types**: The concrete type passed into a generic parameter might introduce hidden branches through its specific trait implementations, type-specific bounds checks, fixed-point helpers, or language-generated panic paths. 

4. **Transitive Call-Graph Strictness**: The `CC=1` (cyclomatic complexity of 1) rule applies transitively to the entire call graph. Because each monomorphization produces distinct final machine code, each one must be explicitly audited to ensure that no input-dependent conditional jumps, loop backedges, or panic paths were introduced by the substituted type.
