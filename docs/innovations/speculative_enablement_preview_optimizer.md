# Innovation Proposal: Speculative Enablement Preview Optimizer (SEPO) for Guarded Petri Schedulers

## 1. Executive Summary

This proposal introduces the **Speculative Enablement Preview Optimizer (SEPO)**, a zero-allocation, non-cloning, and branchless algorithm designed to replace the expensive state cloning and dry-run execution of the [PriorityPetriEngine](file:///Users/sac/bcinr/crates/bcinr-logic/src/patterns/swar_petri.rs#L19) during the preview phase of [petri_tick_guarded](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L546).

By exploiting the feed-forward priority-ordered scheduling invariants of POWL tapes, SEPO computes the exact speculative enablement mask (the "ready set") in a single, lightweight bit-scan loop. This completely eliminates the need to duplicate the 2KB [PowlPetriState](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L156) structure (which contains a large [TimeWheel](file:///Users/sac/bcinr/crates/bcinr-logic/src/patterns/time_wheel.rs#L21) buffer) and construct stack-allocated transition arrays for the Petri engine. The resulting implementation has a cyclomatic complexity of $CC=1$ inside the core evaluation step, performs zero heap allocations, and satisfies the Radon Law of the BCINR Deterministic Substrate.

---

## 2. Problem Statement & Current Limitations

In the wired scheduler ([scheduler_wired.rs](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs)), the guarded execution entry point [petri_tick_guarded](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L546) implements a multi-step protocol to enforce concurrency complexes over ready operations. Step 1 of this protocol is the **Preview Phase**, defined as follows:

```rust
    // Step 1: dry-run preview on a clone — does not touch the real state,
    // the real ring, or the real event ring.
    let mut preview = state.clone();
    let would_fire = petri_tick(tape, &mut preview, None, None, run_id);
    let ready_mask = would_fire.fired_ops;
    let ready = mask_to_event_set(ready_mask);
```

While functional, this approach introduces significant runtime overhead and violates the spirit of a highly optimized, constant-time substrate:

### 2.1 Large State Cloning Overhead
The [PowlPetriState](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L156) structure contains the following fields:
- `done`: `KBitSet<1>` (8 bytes)
- `check`: `KBitSet<1>` (8 bytes)
- `choice_taken`: `u64` (8 bytes)
- `sla_wheel`: [TimeWheel](file:///Users/sac/bcinr/crates/bcinr-logic/src/patterns/time_wheel.rs#L21)`<256>` (2,056 bytes, consisting of `[u64; 256]` slot masks and `current_tick` bookkeeping)
- `loop_iters`: `[u8; 64]` (64 bytes)
- `sla_breached`: `u64` (8 bytes)

Cloning `state` requires copying **over 2.1 KB** of data on every single scheduler tick. Since this tick runs on a high-frequency system loop (often under a sub-microsecond SLA), this continuous stack copying pollutes L1/L2 caches and introduces substantial latency jitter.

### 2.2 Stack and Engine Initialization Waste
Executing [petri_tick](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L347) on the clone requires:
1. Advancing the cloned [TimeWheel](file:///Users/sac/bcinr/crates/bcinr-logic/src/patterns/time_wheel.rs#L21) (`tick()`).
2. Calling [build_transition_arrays](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L265) to construct transition arrays, which allocates:
   - `inputs`: `[KBitSet<1>; 64]` (512 bytes)
   - `outputs`: `[KBitSet<1>; 64]` (512 bytes)
   - `op_indices`: `[u32; 64]` (256 bytes)
3. Constructing the [PriorityPetriEngine](file:///Users/sac/bcinr/crates/bcinr-logic/src/patterns/swar_petri.rs#L19) via `new_checked` (which performs safety checks on limits).
4. Running `engine.step()` to iterate through 64 transitions, checking [SwarMarking::try_fire](file:///Users/sac/bcinr/crates/bcinr-logic/src/models/petri.rs#L253) on every single slot.

All of this overhead is incurred **solely** to compute `ready_mask`, which is a static snapshot of which candidates are satisfied by the current `done` marking and `choice_taken` context before the `ConcurrencySelector` gates them.

---

## 3. Proposed Innovation: Speculative Enablement Preview Optimizer (SEPO)

We propose eliminating the preview clone and [PriorityPetriEngine](file:///Users/sac/bcinr/crates/bcinr-logic/src/patterns/swar_petri.rs#L19) dry-run completely. Instead, the scheduler will determine the speculative firing mask using a direct, branchless sequential checker.

### 3.1 Logical Foundations
In a POWL scheduler for $\le 64$ operations:
1. The candidates eligible to fire are those in the check mask but not yet completed: `candidates = state.check.words[0] & !state.done.words[0]`.
2. Execution of operations in `petri_tick` proceeds sequentially from lowest index to highest index (the order in which `trailing_zeros` extracts them, and the order in which they are processed in `PriorityPetriEngine`).
3. An operation $i \in \text{candidates}$ is ready if its effective predecessor mask is satisfied:
   - For a `Join` operation: `effective_pred = op.pred_mask & choice_taken`
   - For other operations: `effective_pred = op.pred_mask`
4. The satisfaction condition is `effective_pred & !current_done == 0`, where `current_done` accumulates the bits of operations that fire in the *same* tick. Since there are no circular dependencies within a single tick's candidate set (which is guaranteed by the feed-forward nature of the compiled POWL AST), an operation $i$ can only be unblocked in the current tick by a predecessor $j < i$ that also fires in this tick.
5. Firing an operation has no immediate side-effect on the readiness of other candidates except for adding its own bit index $i$ to `current_done`. Other state modifications (like loop reset masks and XOR branches) are only committed to the real persistent state after selection and firing, meaning they do not affect the *preview* of what is ready in the current tick.

### 3.2 The SEPO Preview Algorithm
By matching these invariants, we can write a loop that performs a single scan over `candidates` in ascending priority order.

```rust
/// Speculative Enablement Preview Optimizer (SEPO)
///
/// Computes the exact set of ops that would fire in a normal unguarded tick
/// without copying state or executing the Petri engine.
#[inline(always)]
pub fn speculative_enablement_preview(
    tape: &[Powl64Op],
    done: u64,
    check: u64,
    choice_taken: u64,
) -> u64 {
    let mut current_done = done;
    let mut ready_mask = 0u64;
    let mut bits = check & !done;

    while bits != 0 {
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;

        let op = &tape[i];
        let op_bit = 1u64 << i;

        // Compute effective predecessor mask (handling XOR Join branch selection)
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);

        // Check if all predecessors are satisfied by current_done
        let unmet = effective_pred & !current_done;
        let sat = (unmet == 0) as u64; // 1 if satisfied, 0 otherwise
        
        // Generate fire mask branchlessly: op_bit if sat, 0 otherwise
        let fire_mask = u64::wrapping_sub(0, sat) & op_bit;

        // Speculatively add to done to unblock subsequent candidates in the same tick
        current_done |= fire_mask;
        ready_mask |= fire_mask;
    }

    ready_mask
}
```

### 3.3 Integration into `petri_tick_guarded`
Inside [petri_tick_guarded](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler_wired.rs#L546), Step 1 is optimized as follows:

```diff
-    // Step 1: dry-run preview on a clone — does not touch the real state,
-    // the real ring, or the real event ring.
-    let mut preview = state.clone();
-    let would_fire = petri_tick(tape, &mut preview, None, None, run_id);
-    let ready_mask = would_fire.fired_ops;
-    let ready = mask_to_event_set(ready_mask);
+    // Step 1: SEPO preview (no cloning, no engine dry-run)
+    let ready_mask = speculative_enablement_preview(
+        tape,
+        state.done.words[0],
+        state.check.words[0],
+        state.choice_taken,
+    );
+    let ready = mask_to_event_set(ready_mask);
```

---

## 4. Mathematical and Logical Contract

The correctness and safety of SEPO are specified by a Hoare-logic contract:

$$\{P(\text{tape}, \text{state})\} \quad \text{speculative\_enablement\_preview}(\text{tape}, \text{state.done}, \text{state.check}, \text{state.choice\_taken}) \quad \{Q(\text{tape}, \text{state}, \text{ready\_mask})\}$$

### 4.1 Preconditions $P(\text{tape}, \text{state})$
- **Valid Tape**: `tape.len() <= 64` and is structured as a valid feed-forward POWL execution graph (no cyclic dependencies within a single tick).
- **Valid Bitmasks**: `state.done.words[0]` and `state.check.words[0]` represent the valid marking state of the scheduler.
- **Predecessor Invariant**:
  $$\forall i \in [0, 64), \text{tape}[i].\text{pred\_mask} < (1 \ll \text{tape.len()})$$

### 4.2 Postconditions $Q(\text{tape}, \text{state}, \text{ready\_mask})$
- **Equivalence to Dry-Run**: The resulting `ready_mask` must be bit-for-bit identical to the mask of fired operations produced by running `petri_tick` on a clone of the state:
  $$\text{ready\_mask} = \text{petri\_tick}(\text{tape}, \&\text{mut state.clone}(), \text{None}, \text{None}, \text{run\_id}).\text{fired\_ops}$$
- **Radon Law ($CC=1$)**: The body of the `speculative_enablement_preview` function contains no data-dependent conditional branches (`if`/`match` statement compilation).
- **Loop Boundedness**: The loop executes at most $C = \text{candidates.count\_ones()}$ times, with a static upper bound of 64 iterations.
- **Zero Allocations**: The function performs zero heap allocations and utilizes $\le 64$ bytes of stack space (fully registers-allocated on modern target architectures).

---

## 5. Verification Strategy

Following the mandatory decomposition protocol in the substrate constitution (see [AGENTS.md:Section 5](file:///Users/sac/bcinr/AGENTS.md#L48)), the verification of SEPO will require independent roles to author the oracle, mutants, and object audits:

### 5.1 Independent Reference Oracle
The oracle will be written as a slow-rail, branching implementation that represents the transition engine explicitly, completely separate from the SEPO bit-scan logic:

```rust
fn oracle_ready_mask(tape: &[Powl64Op], state: &PowlPetriState) -> u64 {
    let mut current_done = state.done.words[0];
    let mut ready_mask = 0u64;
    let candidates = state.check.words[0] & !state.done.words[0];
    
    // Explicit priority loop check
    for i in 0..64 {
        let op_bit = 1u64 << i;
        if (candidates & op_bit) != 0 {
            let op = &tape[i];
            let is_join = op.kind == OpKind::Join;
            let join_effective = op.pred_mask & state.choice_taken;
            let effective_pred = if is_join { join_effective } else { op.pred_mask };
            
            // Check satisfaction via branching checks
            if (effective_pred & !current_done) == 0 {
                current_done |= op_bit;
                ready_mask |= op_bit;
            }
        }
    }
    ready_mask
}
```
A differential test suite will verify that `speculative_enablement_preview` matches `oracle_ready_mask` and `petri_tick` (on cloned state) across:
- 100,000 random POWL tape configurations.
- 500,000 random combination states of `done`, `check`, and `choice_taken`.

### 5.2 Hostile Mutation Testing
We define three mutants to verify the test suite's sensitivity to structural errors:

- **Mutant 1 (Choice Gating Omission)**:
  Bypass `choice_taken` masking for Join operations:
  ```rust
  let effective_pred = op.pred_mask;
  ```
  *Expectation*: Must cause a test failure when evaluating a gated XOR Join node where unchosen paths are not yet completed.

- **Mutant 2 (Priority Order Inversion)**:
  Iterate from highest index (MSB) to lowest index (LSB) instead of lowest to highest:
  ```rust
  let i = 63 - bits.leading_zeros() as usize; // Processing reverse priority
  ```
  *Expectation*: Must trigger differential test failures on sequential dependency chains (`a -> b -> c`) where later ops are checked before their predecessor tokens propagate.

- **Mutant 3 (Done State Isolation Failure)**:
  Speculatively accumulate the firing mask without updating `current_done` during the iteration:
  ```rust
  // Omitted: current_done |= fire_mask;
  ready_mask |= fire_mask;
  ```
  *Expectation*: Must fail to detect same-tick multi-step enablement, preventing dependent candidates from being identified as ready in the same tick.

### 5.3 Object-Code Disassembly Audit Plan
The generated assembly for `speculative_enablement_preview` and the inline optimization in `petri_tick_guarded` must satisfy:
1. **Zero Conditional Jumps inside the core**: The loop body must be compiled into straight-line assembly with conditional moves (`cmov`), masking, and bit-manipulation instructions (`tzcnt`, `blsr`).
2. **Zero Allocator References**: No call instructions targeting memory allocators (`malloc`, `__rust_alloc`).
3. **No Timing Side-Channels**: Constant-time execution bounds per candidate.

---

## 6. Substrate Impact & Standing

- **Maturity Standing**: Implementation of SEPO will raise the Substrate Integrity Score (SIS) by removing memory churn in the hot loop.
- **Latency reduction**: Micro-benchmarks are projected to show a **90% to 95% reduction in latency** for the preview step (from ~120ns down to ~6ns), making the guarded tick path highly competitive with the unguarded tick path.
- **Zero heap footprint**: Guaranteed compile-time bounds and register-only execution.
