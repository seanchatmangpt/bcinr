# The Radon Law ($CC=1$)

**The Radon Law** is a core architectural mandate within the `bcinr` (BranchlessCInRust) deterministic substrate that strictly enforces a Cyclomatic Complexity (CC) of exactly 1 for all authoritative public primitives. 

### Strict Enforcement on Public Primitives
Under this law, no public primitive is permitted to contain any data-dependent branching. Specifically, it completely outlaws the use of:
- `if` (including `if let` and `else`)
- `match`
- Data-dependent `loop` or `while` statements

In addition, it broadly bans any operations that compile down to conditional jumps, such as early returns, `?`, `unwrap`, and bounds-check panic paths. The execution path must remain perfectly linear regardless of the input.

### Why Logic Must Be Expressed as Bitwise Polynomials
Because traditional control flow is forbidden, sequential semantic decisions must instead be computed mathematically using **bitwise polynomials** (and canonical mask selections). The reasons for this specific approach are:

1. **Eliminating Timing Side-Channels:** Control flow branching introduces execution time variations based on input data. By evaluating logic as bitwise polynomials, the execution time becomes absolutely constant and physically immune to timing side-channels.
2. **Absolute Determinism:** It ensures a fixed, branchless instruction shape in the final compiled object code, avoiding unpredictable jumps and providing bounded execution work for every operation.
3. **Parallel Computation:** It converts sequential, step-by-step logic into parallel bitwise arithmetic (e.g., computing both outcomes unconditionally and applying a bitwise mask to select the correct state).

This approach grounds `bcinr` as the "hard substrate" for AGI, ensuring that execution remains entirely mathematical, deterministic, and branchless.
