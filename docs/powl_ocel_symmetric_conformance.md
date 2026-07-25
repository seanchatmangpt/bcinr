# Symmetric Run-Bounded Conformance Gating (SRBCG) in `ocel.rs`

The Symmetric Run-Bounded Conformance Gating (SRBCG) mechanism in `crates/bcinr-powl/src/ocel.rs` provides a deterministic, zero-allocation (`no_std` compatible) approach to recording execution traces of POWL workflows and validating them against a compiled `PowlTape`.

It is strictly designed to follow the project's **deterministic substrate** rules, specifically the **Radon Law (CC=1)** which prohibits all data-dependent branching (e.g., `if`, `match`, data-dependent loops), and the **Zero-Allocation Boundary**, meaning all logic runs in bounded stack memory using fixed-size arrays.

## 1. Branchless Slot Allocation (Comparison Networks)

Because heap allocations are forbidden, mapping dynamic `run_id` values to state vectors is handled using a statically allocated array of exactly 64 slots. 

To determine which slot corresponds to an incoming `run_id` *without* branching, the core function `process_event_srbcg` uses a **comparison network**. This compiles down into unrolled, straight-line assembly employing conditional selection instructions (e.g., `CSEL` on ARM, `CMOV` on x86).

### How the Comparison Network Works:
The algorithm uses bitwise polynomials and arithmetic masks rather than control flow:

1. **Fixed-Size Search Loop**: It iterates over all 64 slots in a fixed loop.
2. **Arithmetic Masking**: It generates an `is_match` mask and arithmetically updates `match_idx`.
   ```rust
   for i in 0..64 {
       let is_match = (run_ids[i] == incoming_rid) as usize;
       // If match, use `i`, else keep `match_idx`
       match_idx = (is_match * i) + ((1 - is_match) * match_idx);
   }
   ```
3. **Branchless State Updates**: Decisions about allocating a new slot or updating counts are translated into multiplicative masks:
   ```rust
   let found = (match_idx < 64) as usize;
   let can_allocate = (current_count < 64) as usize;

   // Arithmetic selection of target slot
   let target_idx = (found * match_idx) 
       + ((1 - found) * (can_allocate * allocate_idx + (1 - can_allocate) * 64));

   // Arithmetic update to run_count
   *run_count = current_count + ((1 - found) * can_allocate);
   ```
4. **Guaranteed Constant Time**: If the target run is not found and the network is full (64 slots used), an overflow mask is accumulated bitwise instead of panicking or throwing an early return error.

## 2. The 5 Conformance Validation Checks

The `validate_against_tape` function aggregates these discrete events and evaluates them using 5 deterministic checks:

1. **Empty Log Check**: Rejects logs with 0 events (`ConformanceResult::EmptyLog`).
2. **Run Limit Check**: Evaluates the `overflow_mask` produced by the SRBCG slot allocator. If more than 64 unique runs were observed, it refuses the validation (`ConformanceResult::RunLimitExceeded`) instead of silently dropping runs or overflowing memory.
3. **Duplicate Fire Check**: Uses bitwise `trailing_zeros()` and shift operations to detect if the same operation index (`op_idx`) fired more than once within the same run.
4. **Seal Mismatch Check**: Validates that the bitmask of operations declared during the `run_sealed` event perfectly matches the bitmask of aggregated `op_fired` events.
5. **Predecessor Constraint Check**: Computes missing predecessors exclusively using bitwise logic against the tape's compiled `pred_mask` for each fired operation:
   ```rust
   let missing = pred_mask & !op_trace;
   if missing != 0 {
       // Return ConformanceResult::Violation
   }
   ```

## Summary
By expressing all logical state transitions as bit-parallel mechanics and arithmetic selection across a strictly bounded (64-slot) domain, SRBCG succeeds in managing dynamic, multi-tenant workflow execution histories without violating `CC=1` determinism or triggering heap allocations.
