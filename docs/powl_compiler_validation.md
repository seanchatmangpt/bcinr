# Compilation Phase in `compiler.rs`

The `compiler.rs` module compiles a Partially Ordered Workflow Language (POWL) Abstract Syntax Tree (AST) into a flat execution tape (`PowlTape`). 

## Compilation Strategy
The compiler performs a single recursive descent over the AST, allocating slots sequentially on a flat `PowlTape`. Each compilation step returns an internal `Segment` containing:
- **Entry Mask**: A bitmask of slots representing the start of execution for a subtree.
- **Exit Mask**: A bitmask of slots representing termination points.
Node wiring is done via predecessor and successor bitmasks by linking the exit mask of previous subtrees to the entry mask of subsequent subtrees.

## Validation Protocol
After the flat tape is built, the graph passes through a Two-Phase Verification Protocol. 

### Kahn's Cycle Detection (Phase 1)
This phase uses Kahn's topological sort algorithm to detect non-loop cycles. It computes the in-degree of non-`LoopRedo` nodes (ignoring `LoopRedo` inputs) and performs a BFS traversal. If visited nodes are fewer than the total non-`LoopRedo` nodes, a cycle exists, failing compilation.

**Where it happens:** Kahn's Check occurs in the compiler on the **slow rail**. The code uses branching (`if`, `while`) and vectors/queues, which means it is NOT part of the strictly bounded, branchless hot path.

### Bit-Parallel Transitive Closure Reachability Validation (Phase 2)
The Bit-Parallel Transitive Closure Reachability Validation (BP-TCRV) ensures that all active non-`LoopRedo` nodes are reachable from the tape's `entry_mask`.

**Where it happens:** This phase is specifically designed for the **hot path** with branchless constraints. The implementation (`bp_tcrv_validate_reachability`) uses bitwise masks and wrapping arithmetic on fixed-size `64x64` arrays, enforcing constant-time execution without timing leaks, `CC=1`, avoiding `if` branches and allocations entirely.
