# Rule 7: Whole-Call-Graph Branchlessness

According to Rule 7 in `AGENTS.md`, branchlessness applies to the entire transitive call graph, not merely the public entry point. 

## Transitive Call Graph Audit Requirements

For each authoritative root, the audit must trace and produce the full execution path:
```text
root function
→ direct callees
→ transitive callees
→ compiler intrinsics
→ linked runtime symbols
```

To ensure absolutely no branches exist in the execution path, the audit is explicitly required to include:
* Private functions
* Trait methods
* Generic monomorphizations
* Macros
* Generated modules
* Indexing operations
* Fixed-point helpers
* Serialization helpers reachable at runtime
* Language-generated panic paths

## Why "The function contains no 'if', therefore it is branchless" is Prohibited

The claim "The function contains no `if`, therefore it is branchless" is explicitly prohibited because source-level absence of an `if` statement in a single function does not guarantee that the executing machine code will be branchless. Branches can easily be hidden and introduced through abstractions, such as:
- Private wrappers, trait methods, or external dependencies
- Macro expansions
- Array indexing operations (which may introduce bounds-checking branches)
- Language-generated panic paths (e.g., from unwrap, expect, or checked arithmetic)
- Generic monomorphizations

Because BCINR is a deterministic computational substrate, branchlessness must be proven at the machine level. Therefore, the only permitted claim is:

> "The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."

This ensures that verification relies on rigorous object-code and disassembly inspection of the entire transitive call graph rather than surface-level syntax assumptions.
