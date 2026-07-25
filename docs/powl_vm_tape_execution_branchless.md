# POWL Tape Execution and Branchless State Advancement

In `bcinr-powl`, there is no traditional scalar "instruction pointer" (IP). Instead, execution state is managed concurrently across a flat, cache-line-aligned array of operations ("tape") using bitmasks. This design enables SIMD-within-a-register (SWAR) evaluation without control-flow branches, ensuring deterministic, allocation-free, and timing-side-channel-free execution.

## 1. Tape Layout (`tape.rs`)

The `PowlTape` consists of up to 64 operations (slots) packed into an array. Each operation (`Powl64Op`) is 64-byte cache-line-aligned and defines its dependencies using bitmasks:
- `pred_mask`: A bitmask of predecessor slot indices that must be completed before this op can fire.
- `succ_mask`: A bitmask of successor slot indices that will be activated when this op completes.
- `kind`: The operation's semantic type (e.g., `Atom`, `XorDispatch`, `Join`, `LoopRedo`, `Silent`).

Because the state uses `u64` bitmasks, a tape can natively track 64 concurrent operations in a single register. For larger programs (up to 512 ops), `PowlTapeLarge` uses `[u64; 8]` bitmask arrays.

## 2. Branchless Execution State (`scheduler.rs`)

The dynamic scheduler state is maintained in `PowlRunState`, which replaces a single IP with three primary bitmasks:
- `done_mask`: Operations that have successfully fired.
- `check_mask`: Operations currently pending readiness evaluation.
- `active_mask`: Operations firing in the current tick.

### The Execution Pipeline

Each scheduler tick (`scheduler_tick`) evaluates readiness and advances the state entirely through branchless bitwise arithmetic:

1. **Candidate Selection**: `candidates = check_mask & !done_mask`
2. **Readiness Evaluation**: For each candidate slot, the runtime evaluates if `pred_mask` is satisfied. This is done without `if` statements:
   ```rust
   let unmet = required & !done;
   let satisfied = 0u64.wrapping_sub((unmet == 0) as u64); // returns u64::MAX if satisfied, else 0
   ```
3. **State Commit**: Fired slots update the `done_mask` and transition their `succ_mask` into the `check_mask` for the next tick.
4. **Control Dispatch**: Complex control flow like `XorDispatch` and `LoopRedo` is also handled branchlessly. For example, XOR branch choices are recorded into a `choice_taken` mask, which acts as a dynamic filter for `Join` operations later in the tape.

## 3. Compile-Time Topologies (`const_scheduler.rs`)

For workflows where the topology is known at compile time, the `const_scheduler` eliminates the runtime check-loop entirely (Lever 4 optimization).

Instead of maintaining dynamic `check_mask` queues, Kahn's algorithm is applied in a `const fn` at compile time to produce a fixed `ORDER` array (a topological sort of the operations). 

At runtime, the `const_tick` function becomes a straight-line, unrolled sequence of bitwise checks. For a simple linear chain of operations, the compiler unrolls the evaluation into roughly 4 native instructions per op (AND + SUBS + CSINV + OR). This results in zero loop overhead, zero branching, and execution times on the order of picoseconds per operation.
