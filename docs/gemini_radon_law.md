# The Radon Law ($CC=1$)

Based on the rules set in `GEMINI.md` and `AGENTS.md`, here are the detailed constraints of **The Radon Law ($CC=1$)**.

## What it Mandates
- **Absolute Branchlessness**: No public primitive may contain any branching logic or data-dependent loops.
- **Arithmetic Logic**: Sequential semantic decisions and logic must be expressed as bitwise polynomials, masks, and arithmetic selection (e.g., SWAR, SIMD shuffles) rather than control flow. 
- **Full Call-Graph Compliance**: The law applies transitively to the entire authoritative call graph. This includes private functions, trait methods, generic monomorphizations, macros, generated modules, indexing operations, fixed-point helpers, and language-generated panic paths. The final release object code must contain zero input-dependent conditional branches or loop backedges.
- **Mask-based Execution**: Runtime predicates must be transformed into full-width masks (e.g., $m \in \{0, 2^w-1\}$). State selection must take an arithmetic form equivalent to `(m & a) | (!m & b)`.

## What it Forbids
The following constructs are strictly prohibited in authoritative code, as they produce control-flow branches:
- `if`, `if let`, `else`
- `match`
- `while`, `loop`, `break`, `continue`
- early returns and `?` operator
- `unwrap`, `unwrap_or`, `unwrap_or_else`, `expect`
- Checked arithmetic with branch-bearing handling
- `Option`-based and `Result`-based control flow
- Iterator short-circuiting
- Variable-bound iteration (e.g., `while value > 0` or `for item in variable_slice`)
- Bounds-check panic paths

*Note: Asserting that code contains no `if` is insufficient. The standard mandates that the authoritative instruction shape must not depend on semantic input, verified via object-code inspection.*
