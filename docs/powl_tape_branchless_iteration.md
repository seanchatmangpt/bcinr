# Branchless Tape Execution in `PowlTape`

The `PowlTape` is a core data structure in `bcinr-powl`, implemented as a flat, cache-line-aligned execution array capped at 64 slots (`[Powl64Op; 64]`). To comply with the BCINR Radon Law ($CC=1$) and timing-invariant constraints, iterating and executing operations within this tape avoids traditional branching mechanisms (like `if`, `break`, or early returns) in favor of **full iteration** and **mask-based arithmetic**.

## 1. Full Iteration Without Early Exits

When performing static analysis or reachability validation on the tape, loops iterate over the maximum capacity (64 times) rather than dynamically stopping at the tape's actual length `tape.len`.

A prime example is the Bit-Parallel Roy-Warshall transitive closure algorithm inside `compiler.rs` (`bp_tcrv_validate_reachability`):

```rust
// Fixed loop bound of 64 allows complete compiler unrolling.
for i in 0..64 {
    let in_bounds = (i < tape_len) as u64;
    let bounds_mask = 0u64.wrapping_sub(in_bounds); // !0u64 if valid, 0 otherwise

    let succs = tape.ops[i].succ_mask & bounds_mask;
    r[i] = succs | (1u64 << i);
}
```

Instead of an `if i >= tape_len { break; }`, the code calculates an `in_bounds` boolean, converts it to a `u64` bitmask (using `0u64.wrapping_sub(in_bounds)`), and applies this mask to safely ignore out-of-bounds operations. This guarantees a constant number of clock cycles and timing immunity.

## 2. Mask-Based Tape Execution

Execution of operations on the tape (e.g., `scheduler_tick` in `scheduler.rs`) avoids iterating over empty or waiting slots by employing a candidate bitmask and `trailing_zeros()`:

```rust
let mut candidates = state.check_mask & !state.done_mask;

while candidates != 0 {
    let i = candidates.trailing_zeros() as usize;
    candidates &= candidates - 1; // Clear lowest set bit
    let op = &tape[i];
    
    // ... branchless processing ...
}
```

Inside the loop, the logic evaluates conditions branchlessly using SWAR (SIMD-within-a-register) arithmetic. All potential side effects (XOR choice taking, loop redoing, etc.) are computed unconditionally, and predicated using bitwise masking:

*   **Type Matching:** Instead of matching on `op.kind`, a `kind_mask` function calculates `u64::MAX` if the types match and `0` otherwise.
*   **Effective Predecessors:**
    ```rust
    let is_join = kind_mask(op.kind, OpKind::Join);
    let join_effective = op.pred_mask & state.choice_taken;
    // Evaluates both branches simultaneously:
    let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);
    ```
*   **Firing Operations:** Satisfying constraints computes a boolean `sat_bit`, which is transformed into a `fire_mask` (`u64::MAX` or `0`). Side effects are then applied by simply applying a bitwise `AND` against the `fire_mask`.

By projecting conditional control flow into algebraic bit manipulation, `PowlTape` execution remains strictly constant-time, bounded, and allocation-free.
