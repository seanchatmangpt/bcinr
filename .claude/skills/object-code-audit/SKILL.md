---
name: object-code-audit
description: Use when verifying that an authoritative bcinr function is actually branchless in the compiled release artifact (not just source), per AGENTS.md §7/§13/§20 — disassemble the release build and produce a per-symbol audit table. Triggers on "object code audit", "disassembly", "is this really branchless", or before claiming BRANCHLESS_ALIVE standing.
---

# Object-code audit

Implements AGENTS.md §7 (whole-call-graph branchlessness), §13 (no unbounded execution), and §20
(object-code audit). Owned by `@turing-machine`; source-level `CC=1` is necessary but never
sufficient on its own.

## Steps

1. **Build the exact release profile** for the target(s) actually shipped:
   `cargo build --release -p <crate>` (repeat per feature/target combination per AGENTS.md §22 —
   default features, no-default-features, all-features, and each supported architecture).
2. **Enumerate the authoritative call graph** for each root symbol: direct callees → transitive
   callees → compiler intrinsics → linked runtime symbols. Include private functions, trait
   methods, generic monomorphizations, macro expansions, generated modules, indexing operations,
   fixed-point helpers, and any runtime-reachable serialization helpers.
3. **Disassemble** (`objdump -d`, `cargo asm`, or equivalent for the target) each symbol in that
   graph.
4. **Inspect for, per symbol:**
   - conditional jump instructions (data-dependent branches)
   - loop backedges
   - panic/bounds-check symbols (`core::panicking::*`, `Option::unwrap`, etc.)
   - allocator symbols (`__rust_alloc`, `alloc::*`)
   - indirect calls (vtable dispatch)
   - floating-point instructions
   - division instructions (unless explicitly admitted per contract)
5. **Report per-symbol** using the AGENTS.md §20 table:

   | Symbol | CC | Conditional jumps | Loop backedges | Panic path | Allocator | Standing |
   |---|---:|---:|---:|---|---|---|

6. **Assign standing.** Any unclassified authoritative symbol blocks merge. A symbol with zero
   conditional jumps/backedges/panic/allocator hits earns `ALIVE`/`BRANCHLESS_ALIVE`; anything
   short of full inspection stays `SOURCE_BRANCHLESS_PARTIAL` or `UNKNOWN` — never round up.

## Output

`OBJECT_CODE_AUDIT.md` and `AUTHORITATIVE_CALL_GRAPH.md` for the feature.

## Boundaries

- Never claim "contains no `if`, therefore branchless" — only the object-code result over the
  declared target is admissible evidence.
- A green source scan without a matching disassembly pass is not evidence of `BRANCHLESS_ALIVE`.
