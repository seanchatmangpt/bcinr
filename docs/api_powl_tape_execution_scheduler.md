Here is the requested documentation on the execution phase of POWL tapes and how they achieve CC=1 branchless execution.

## POWL Tape Execution Phase

The core execution phase of POWL tapes is primarily found in two locations: the dynamic SWAR scheduler in `crates/bcinr-powl/src/scheduler.rs` and the compile-time topological scheduler in `crates/bcinr-powl/src/const_scheduler.rs`. Both engines eliminate data-dependent conditional jumps (`if`/`else`), achieving strict CC=1 compliance using bitwise arithmetic.

### 1. Dynamic SWAR Scheduler (`scheduler.rs`)

The `scheduler_tick` and `scheduler_tick_guarded` functions advance execution by mutating a fixed-size `PowlRunState` struct containing bitmasks (`done_mask`, `check_mask`, `active_mask`, `choice_taken`). They evaluate and commit transitions for up to 64 operations concurrently using branchless SIMD-within-a-register (SWAR) techniques:

*   **Branchless Condition Evaluation:** Readiness is determined without boolean checks. The `pred_satisfied` function evaluates `unmet = required & !done;`. It maps the outcome to a full-width boolean mask via wrapping subtraction: `0u64.wrapping_sub((unmet == 0) as u64)`, translating `true` to `u64::MAX` and `false` to `0`.
*   **Mask-Based State Mutation:** Rather than using branches to apply updates, variables like `fire_mask` are derived mathematically (e.g., `let fire_mask = u64::wrapping_sub(0, sat_bit) & bit;`) and blindly applied to accumulator masks like `new_done |= fire_mask;`. If the node wasn't ready, the applied mask is `0`, producing a no-op mathematically.
*   **Branchless Control Dispatch:** Control nodes like `XorDispatch` and `LoopRedo` require specialized processing. The scheduler derives a mathematical type-equality mask (`kind_mask` computes `(kind ^ target) >> 7` and mapping to `0` or `u64::MAX`). It computes the state effects of these nodes unconditionally for every slot, but intersects the effects with the `kind_mask` and `fire_mask` before merging them into the state.
*   **Saturating Counters:** Iteration loops handle max-iterations through `iter_under_limit`, producing masks from integer underflow (`(diff >> 15) & 1`) to enable saturating increments without jumps.

### 2. Compile-Time Topologies (`const_scheduler.rs`)

When workflow dependencies are known ahead of time, `const_scheduler.rs` applies "Lever 4" optimization to produce purely straight-line object code.

*   **Ahead-of-Time Graph Sorting:** The topological graph logic is computed via a `const fn topo_order` during compilation (made possible via `#![feature(generic_const_exprs)]`).
*   **Fully Unrolled Elimination:** `const_tick` executes against the static `ConstTopology::ORDER` array. For small bounded sequence sizes, the Rust compiler perfectly unrolls the underlying evaluation loop.
*   **Zero Loop Backedges:** By omitting loop evaluation at runtime entirely, the operation execution collapses into an exact deterministic sequence of `AND`, `SUBS`, `CSINV`, and `ORR` instructions checking a point-in-time snapshot of the `done` state.
