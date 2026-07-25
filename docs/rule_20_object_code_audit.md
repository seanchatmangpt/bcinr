# Rule 20: Object-Code Audit

In accordance with `AGENTS.md` Rule 20, **every supported release target requires an exact production-profile disassembly audit.**

## Why a Disassembly Audit is Required
The `bcinr` determinism mandate dictates that logic must be strictly expressed as bitwise polynomials without any timing side-channels or data-dependent branches. Because the Rust compiler handles instruction lowering, optimization, and platform-specific codegen differently across targets, code that is branchless on one architecture may not be on another. An exact production-profile disassembly audit is mandatory to verify that the final, compiled machine code adheres to the absolute runtime laws (no branches, no panic paths, zero allocations) across every supported target.

## What the Audit Must Inspect
The structural audit must individually list and inspect the following items to verify the final machine code:

* **All authoritative root symbols**
* **All transitive helper symbols**
* **Panic and bounds-check symbols**
* **Allocator symbols**
* **Conditional jumps**
* **Loop backedges**
* **Indirect calls**
* **Floating-point instructions**
* **Division instructions**
* **Unexpected runtime library calls**

## Permitted Evidence Format
The audit result must list each symbol individually. Any unclassified authoritative symbol automatically blocks the merge. The permitted format for the audit evidence is the following markdown table:

| Symbol            | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
| ----------------- | -: | ----------------: | -------------: | ---------: | --------: | -------- |
| `cmca_allocate`   |  1 |                 0 |              0 |         No |        No | ALIVE    |
| `verify_envelope` |  1 |                 0 |              0 |         No |        No | ALIVE    |

## Why Source-Level `CC=1` is Necessary but Insufficient
Achieving Cyclomatic Complexity 1 (`CC=1`) at the source level ensures the absence of explicit control-flow branches (`if`, `match`, loops, etc.) in the authored logic, which is a strictly necessary baseline. However, it is **insufficient** because the Rust compiler can implicitly inject branches and violations into the final object code. For example:
- **Implicit panics:** Array/slice indexing can introduce bounds-checking branches.
- **Language features:** Checked arithmetic or implicit bounds checks can compile into input-dependent conditional jumps.
- **Hidden abstractions:** Trait monomorphization, macro expansions, or compiler intrinsics could generate concealed branching or loop backedges.

Consequently, claiming "The function contains no `if`, therefore it is branchless" is explicitly prohibited. The only permitted mathematical guarantee is proving that the full authoritative call graph contains no input-dependent conditional branch in the audited release object code.
