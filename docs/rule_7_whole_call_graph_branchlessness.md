Based on Rule 7 (Whole-call-graph branchlessness) in `AGENTS.md`, here are the details regarding the transitive call graph audit and the specific claims around branchlessness:

### Transitive Call Graph Audit Requirements

The audit for whole-call-graph branchlessness must comprehensively include:
* private functions;
* trait methods;
* generic monomorphizations;
* macros;
* generated modules;
* indexing operations;
* fixed-point helpers;
* serialization helpers reachable at runtime;
* language-generated panic paths.

### Branchlessness Claims (Prohibited vs. Permitted)

The rule establishes a strict distinction between superficial source-level branchlessness and actual object-code branchlessness across the entire authoritative call graph:

* **Prohibited Claim:** 
  > "The function contains no `if`, therefore it is branchless."
  *(This is invalid because branches can be hidden in private wrappers, macros, trait implementations, or dependencies, and merely lacking an `if` statement does not guarantee the absence of branches).*

* **Permitted Claim:** 
  > "The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."
  *(This correctly places the burden of proof on analyzing the final release object code across the full transitive call graph, rather than just the source code of a single function).*
