Based on a review of `AGENTS.md` (specifically Rule 3, Rule 6, and Rule 7), here are the details regarding why benchmark orchestration is placed in the slow rail and why it must remain disconnected from the hot path:

### Why benchmark orchestration is placed in the "Slow rail"
Rule 6 ("Authoritative versus non-authoritative code") classifies code into strict categories based on its operational constraints. The **Authoritative runtime** is bound by the project's **absolute runtime laws** (Rule 3), which strictly require:
- `#![no_std]` execution
- Zero heap allocation (`no alloc`)
- Branchless logic (`CC=1` transitively, no data-dependent branches or loops)
- Fixed bounded execution work

**Benchmark orchestration** inherently requires unbounded iteration, varying inputs, metric collection, and dynamic setup, none of which fit the rigid fixed-width and branchless constraints. Rule 6 explicitly acknowledges that code in the **Slow rail** (which includes benchmark orchestration, code generation, CLI display, etc.) "may branch and allocate," making it the only suitable place for such logic. 

### Why it must never be linked into or invoked from the hot path
Rule 6 explicitly dictates: *"The slow rail may branch and allocate, but it must never be linked into or invoked from the authoritative hot path."*

This is enforced by **Rule 7 (Whole-call-graph branchlessness)**, which specifies that the strict constraints (branchlessness, allocation-free execution) apply to the *entire transitive call graph*, including any linked symbols. 
If benchmark orchestration code were linked into or invoked from the hot path:
1. It would introduce heap allocations and branches to the call graph, violating the `CC=1` and `no alloc` laws.
2. The authoritative object-code audit (Rule 20) and structural enforcer (`@turing_machine`) would detect unexpected conditional jumps, loop backedges, or allocator symbols.
3. This would instantly violate the deterministic, constant-time, bounded guarantees of the substrate, breaking its constitutional invariants.
