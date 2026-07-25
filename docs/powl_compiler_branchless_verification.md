# POWL Compiler & Branchless (CC=1) Verification

The POWL compiler transforms a Partially Ordered Workflow Language (POWL) Abstract Syntax Tree (AST) into a flat, 64-slot execution tape (`PowlTape`) for runtime execution. 

In the BCINR architecture, code is strictly divided between the **slow rail** and the **authoritative runtime (hot path)**:

## 1. Parsing & Tape Allocation (Slow Rail)
The initial parsing and mapping of the AST or `PowlModel` into tape slots happens on the **slow rail**. This process (via `compile_powl` and `compile_node`) is allowed to allocate and use branches (e.g., recursive descent, pattern matching) to traverse the AST. The goal is to generate a fixed-size, flat tape of operations (`PowlTape`) that the hot path can execute without branching. Tape slots are explicitly connected using bitmasks (`pred_mask` and `succ_mask`) rather than references or pointers.

## 2. Reachability Validation (Phase 2 - CC=1)
To ensure the compiled tape is safe for execution, the compiler enforces a two-phase validation process. Phase 2 of this process, the **Bit-Parallel Transitive Closure Reachability Validation (BP-TCRV)**, guarantees that all active non-loop nodes are reachable from the tape's entry mask. 

Because this algorithm is a core invariant check, it is implemented entirely without data-dependent branching ($CC=1$) to prevent timing leaks. It achieves this via **Mask-Based Execution**:

*   **Fixed Bounded Execution:** All matrices are hardcoded to $64 \times 64$ limits. Loops iterate exactly 64 times (`for k in 0..64`), allowing the compiler to fully unroll them and eliminate loop backedges.
*   **Branchless Mask Generation:** Instead of using `if` statements to conditionally propagate reachability, the code transforms semantic logic into bitwise polynomials. A boolean state `(0 or 1)` is converted to a full-width mask (`0x000...` or `0xFFF...`) using `wrapping_sub`:
    ```rust
    // Instead of: if can_reach_k { r[i] |= r_k; }
    let can_reach_k = (r[i] >> k) & 1;
    let mask = 0u64.wrapping_sub(can_reach_k); // All 1s if true, all 0s if false
    r[i] |= r_k & mask;
    ```
*   **Masked State Selection:** Validating node boundaries and reachability containment is done purely via bitwise intersection. For example, boundary checks are masked out branchlessly (`let in_bounds = (i < tape_len) as u64; let bounds_mask = 0u64.wrapping_sub(in_bounds);`), ensuring that unallocated slots naturally zero out without triggering conditional panic paths.

By flattening the AST during the slow-rail phase and relying purely on bitmask propagation during the branchless validation phase, the resulting `PowlTape` is verified and ready for bounded, branchless, allocation-free execution in the authoritative runtime.
