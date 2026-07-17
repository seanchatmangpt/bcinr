# Innovation Proposal: Unrolled Bit-Parallel Tick Scheduler (UBPTS) for Compact POWL v2 Tapes

## 1. Executive Summary

This proposal introduces the **Unrolled Bit-Parallel Tick Scheduler (UBPTS)**, a constant-time, branch-free, and zero-allocation scheduling engine designed for compact Partially Ordered Workflow Language (POWL) v2 tapes (up to 64 operations).

The primary objective is to replace the current data-dependent loops (e.g., `while candidates != 0`) and heavy set-conversions (e.g., [`mask_to_event_set`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L344-L353) and [`event_set_to_mask`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L356-L360)) in the hot-path scheduler with fully unrolled, branchless, bitmask-parallel passes. UBPTS achieves:
1. **Strict Radon Compliance ($CC=1$)**: Zero data-dependent control branches, zero data-dependent loop terminations, and exactly $CC=1$ per authoritative function.
2. **Zero Heap Allocation**: Completely avoids heap-allocated collections (e.g., `Vec`, `EventSet`) and data conversions on the scheduling hot path.
3. **Loop-Backedge Elimination**: Executes loop iterations over a compile-time fixed bound of exactly 64 slots, allowing the compiler to completely unroll the loops into straight-line machine instructions containing zero loop backedges.
4. **Timing-Channel Immunity**: Guarantees identical execution latency (WCET = BCET) and identical assembly instruction sequences for all input configurations.

---

## 2. Problem Statement & Current Limitations

The current POWL v2 scheduler in [`crates/bcinr-powl/src/scheduler.rs`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs) implements two main scheduling entry points: [`scheduler_tick`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L199-L246) (gated only by control dependencies) and [`scheduler_tick_guarded`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L384-L449) (gated also by concurrency guard tables).

Both implementations violate the strict **BCINR Deterministic Substrate Constitution** ([`AGENTS.md`](file:///Users/sac/bcinr/AGENTS.md)) in several ways:

### 2.1 Data-Dependent Loop Termination
The core tick loops in [`scheduler_tick`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L206) and [`scheduler_tick_guarded`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L412) utilize a standard CTZ bit-scan loop:
```rust
let mut candidates = state.check_mask & !state.done_mask;
while candidates != 0 {
    let i = candidates.trailing_zeros() as usize;
    candidates &= candidates - 1;
    // ...
}
```
**Violation**: The iteration count depends entirely on the popcount of the `candidates` mask. In a high-integrity substrate, variable loop boundaries produce input-dependent timing profiles, exposing the system to timing side-channels.

### 2.2 Conversion and Iteration Overhead
Before executing the concurrency-aware selection, the scheduler converts `u64` masks into `EventSet` collections:
```rust
let ready = mask_to_event_set(ready_mask);
let selected = selector.select_checked(&ready, guards);
let selected_mask = event_set_to_mask(&selected);
```
Where [`mask_to_event_set`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L344) and [`event_set_to_mask`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L356) are defined as:
```rust
pub(crate) fn mask_to_event_set(mask: u64) -> EventSet {
    let mut set = EventSet::empty();
    let mut bits = mask;
    while bits != 0 {
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        set.insert(i);
    }
    set
}
```
**Violation**:
1. `EventSet` uses a multi-word bitset internally and can trigger helper functions with variable execution times.
2. The loop `while bits != 0` terminates early based on input bits, introducing further timing side-channels and branching.
3. `StableMaximalSelector::select` iterates over the `ready` set with `for id in ready.iter_stable()`, which is a data-dependent loop.

### 2.3 Branching Fast-Path Gating
In [`scheduler_tick_guarded`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs#L401-L403), the fast-path check:
```rust
if selected_mask == ready_mask {
    return scheduler_tick(tape, state);
}
```
introduces a conditional branch in the control flow, violating the Radon Law's requirement for flat, straight-line execution paths.

---

## 3. Proposed Innovation: Unrolled Bit-Parallel Tick Scheduler (UBPTS)

We propose eliminating all variable loops, set conversions, and control branches by restructuring the scheduling tick into a three-stage pipeline of fully unrolled, compile-time bounded loops.

Since compact tapes are strictly limited to 64 operations, every set can be represented natively as a `u64` bitmask, and all loops can be bounded to exactly 64 iterations, which LLVM optimizes into branch-free, straight-line instruction sequences.

### 3.1 Stage 1: Branchless Tick Preview (Pass 1)
Pass 1 simulates execution on a logical copy of the state to identify which operations *would* fire under control-flow rules alone (this set is the `ready_mask`).
Instead of checking candidates sequentially in a variable loop, we evaluate all 64 potential slots in a fixed loop. A slot is only processed if its bit is set in the `candidates` mask, handled branchlessly using masks:

```rust
#[inline(always)]
pub fn preview_tick_unrolled(tape: &[Powl64Op; 64], state: &PowlRunState) -> u64 {
    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut choice_taken = state.choice_taken;
    
    let candidates = state.check_mask & !state.done_mask;
    
    // Loop has a compile-time fixed bound of 64.
    // The compiler unrolls this loop completely.
    for i in 0..64 {
        let op = &tape[i];
        let bit = 1u64 << i;
        let is_candidate = (candidates >> i) & 1;
        let is_candidate_mask = 0u64.wrapping_sub(is_candidate); // 0x0 or 0xFFFFFFFFFFFFFFFF
        
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);
        
        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = sat & 1;
        
        // Gate the fire mask by whether this slot is a candidate
        let fire_mask = u64::wrapping_sub(0, sat_bit) & bit & is_candidate_mask;
        
        fired |= fire_mask;
        new_done |= fire_mask;
        
        // Update temporary choice_taken to simulate XOR dispatch branch satisfaction
        let is_xor = kind_mask(op.kind, OpKind::XorDispatch);
        let fire_nz = 0u64.wrapping_sub((fire_mask | fire_mask.wrapping_neg()) >> 63);
        let active = is_xor & fire_nz;
        let chosen = op.branch_mask & op.branch_mask.wrapping_neg();
        choice_taken |= chosen & active;
    }
    
    fired
}
```

### 3.2 Stage 2: Bit-Parallel Selection (Pass 2)
Using the Bit-Parallel Concurrency Guard Gating (BP-CGG) mechanism, we perform greedy stable-maximal selection entirely on `u64` bitmasks in exactly 64 unrolled iterations, bypassing `EventSet` entirely:

```rust
impl ConcurrencyGuardTable {
    /// Determines branchlessly if the candidate set (as a u64 mask) is admitted.
    /// Returns `u64::MAX` if admitted, `0` if rejected.
    #[inline(always)]
    pub fn admits_mask(&self, candidate: u64) -> u64 {
        let u = !candidate;
        let mut s = 0u64;
        for e in 0..64 {
            let bit = (u >> e) & 1;
            let mask = 0u64.wrapping_sub(bit);
            s |= mask & self.cols[e];
        }
        let unmet = self.active_mask & !s;
        0u64.wrapping_sub((unmet == 0) as u64)
    }
}

impl StableMaximalSelector {
    /// Selects a maximal subset of ready_mask branchlessly.
    #[inline(always)]
    pub fn select_mask(&self, ready_mask: u64, guards: &ConcurrencyGuardTable) -> u64 {
        let mut selected = 0u64;
        for e in 0..64 {
            let bit = 1u64 << e;
            let is_ready = (ready_mask >> e) & 1;
            let is_ready_mask = 0u64.wrapping_sub(is_ready);
            
            let candidate_next = selected | bit;
            let admitted = guards.admits_mask(candidate_next);
            
            selected |= bit & is_ready_mask & admitted;
        }
        selected
    }
}
```

### 3.3 Stage 3: Gated Execution & Commit (Pass 3)
The real execution is performed by running a second unrolled pass over the 64 slots, gating each slot's firing by `selected_mask`. This updates the persistent `state` in a single transaction:

```rust
#[inline(always)]
pub fn execute_tick_unrolled(
    tape: &[Powl64Op; 64],
    state: &mut PowlRunState,
    selected_mask: u64,
) -> FiredSet {
    let mut fired = 0u64;
    let mut new_done = state.done_mask;
    let mut new_check = 0u64;
    
    let candidates = state.check_mask & !state.done_mask;
    
    for i in 0..64 {
        let op = &tape[i];
        let bit = 1u64 << i;
        let is_candidate = (candidates >> i) & 1;
        let is_candidate_mask = 0u64.wrapping_sub(is_candidate);
        
        let is_join = kind_mask(op.kind, OpKind::Join);
        let join_effective = op.pred_mask & state.choice_taken;
        let effective_pred = (join_effective & is_join) | (op.pred_mask & !is_join);
        
        let sat = pred_satisfied(new_done, effective_pred);
        let sat_bit = sat & 1;
        let is_selected = (selected_mask >> i) & 1;
        let is_selected_mask = 0u64.wrapping_sub(is_selected);
        
        // Slot i fires iff it is a candidate, satisfied, AND selected by the concurrency guard
        let fire_mask = u64::wrapping_sub(0, sat_bit) & bit & is_candidate_mask & is_selected_mask;
        
        fired |= fire_mask;
        new_done |= fire_mask;
        
        let fired_this = fire_mask >> i;
        new_check |= op.succ_mask & u64::wrapping_sub(0, fired_this);
        
        // --- Branchless XorDispatch ---
        new_done |= apply_xor_dispatch(op, fire_mask, &mut state.choice_taken);
        
        // --- Branchless LoopRedo ---
        let (redo_clear, redo_check) = apply_loop_redo(op, fire_mask, &mut state.loop_iters[i]);
        new_done &= !redo_clear;
        new_check |= redo_check;
    }
    
    // Carry forward ready-but-unselected ops to next tick
    let ready_mask = preview_tick_unrolled(tape, state);
    new_check |= ready_mask & !selected_mask;
    
    state.done_mask = new_done;
    state.check_mask = new_check & !new_done;
    
    FiredSet(fired)
}
```

---

## 4. Mathematical and Logical Contract

The UBPTS implementation satisfies a strict mathematical contract defined under `@hoare_oracle` jurisdiction.

### 4.1 Hoare Contract Representation
$$\{P(\text{tape}, \text{state})\} \quad \text{scheduler\_tick\_guarded\_unrolled}(\text{tape}, \text{state}, \text{guards}) \quad \{Q(\text{tape}, \text{state}_{\text{pre}}, \text{state}_{\text{post}}, \text{fired})\}$$

### 4.2 Preconditions $P(\text{tape}, \text{state})$
- **Tape Integrity**: `tape` must contain exactly 64 cache-aligned [`Powl64Op`](file:///Users/sac/bcinr/crates/bcinr-powl/src/tape.rs#L157) structures. Unused slots beyond the active tape length $N \le 64$ must be populated with `OpKind::Silent` operations (predecessors = 0, successors = 0).
- **State Validity**: `state.done_mask`, `state.check_mask`, and `state.choice_taken` must be well-formed 64-bit integers.
- **State Intersection Constraint**: `state.check_mask` and `state.done_mask` must not intersect:
  $$\text{state.check\_mask} \land \text{state.done\_mask} = 0$$

### 4.3 Postconditions $Q(\text{tape}, \text{state}_{\text{pre}}, \text{state}_{\text{post}}, \text{fired})$
- **Deterministic Complexity**: The cyclomatic complexity of `execute_tick_unrolled` is exactly $CC=1$.
- **Object-Code Invariant**: The generated release assembly contains exactly 0 conditional jumps and 0 loop backedges.
- **Zero-Allocation**: No heap allocations or dynamic stack sizing are performed.
- **Correct Fired Set**:
  $$\forall i \in [0, 64), \text{bit}_i \in \text{fired} \iff \text{bit}_i \in \text{candidates} \land \text{effective\_pred}_i \subseteq \text{state}_{\text{pre}}.\text{done\_mask} \land \text{bit}_i \in \text{selected\_mask}$$
- **State Evolution Invariant**:
  - Done commits:
    $$\text{state}_{\text{post}}.\text{done\_mask} = \text{state}_{\text{pre}}.\text{done\_mask} \cup \text{fired} \cup \text{suppressed\_mask}$$
  - Check propagation:
    $$\text{state}_{\text{post}}.\text{check\_mask} = (\text{state}_{\text{pre}}.\text{check\_mask} \cup \text{succ\_fold} \cup (\text{ready\_mask} \setminus \text{selected\_mask})) \setminus \text{state}_{\text{post}}.\text{done\_mask}$$
- **Stability and Refusal Protection**: Any unadmitted input or internal constraint violation results in a clean refusal mask return without specular mutations of `state`.

---

## 5. Verification Strategy

Following the constitutional mandates of [`AGENTS.md`](file:///Users/sac/bcinr/AGENTS.md), UBPTS is verified using a tiered testing infrastructure.

### 5.1 Independent Reference Oracle
An independent, slow-rail reference oracle is implemented using standard branching and standard iterator-based set operations:

```rust
pub fn oracle_scheduler_tick_guarded(
    tape: &[Powl64Op],
    state: &mut PowlRunState,
    guards: &ConcurrencyGuardTable,
) -> u64 {
    let mut fired = 0u64;
    let mut done = state.done_mask;
    let mut check = state.check_mask;
    let mut choice = state.choice_taken;
    
    // Simulate candidate check sequentially using standard branches
    let candidates = check & !done;
    let mut ready_mask = 0u64;
    
    for i in 0..tape.len() {
        if (candidates & (1 << i)) != 0 {
            let op = &tape[i];
            let is_join = op.kind == OpKind::Join;
            let effective_pred = if is_join { op.pred_mask & choice } else { op.pred_mask };
            if (effective_pred & !done) == 0 {
                ready_mask |= 1 << i;
            }
        }
    }
    
    // Greedy selection simulation
    let mut selected_mask = 0u64;
    for i in 0..64 {
        if (ready_mask & (1 << i)) != 0 {
            let candidate_next = selected_mask | (1 << i);
            let mut admits = true;
            for nf in &guards.nonfaces {
                let nf_mask = event_set_to_mask(&nf.members);
                if (nf_mask & !candidate_next) == 0 {
                    admits = false;
                    break;
                }
            }
            if admits {
                selected_mask |= 1 << i;
            }
        }
    }
    
    // Real fire simulation
    for i in 0..tape.len() {
        if (candidates & (1 << i)) != 0 && (selected_mask & (1 << i)) != 0 {
            let op = &tape[i];
            fired |= 1 << i;
            done |= 1 << i;
            if op.kind == OpKind::XorDispatch {
                let chosen = op.branch_mask & op.branch_mask.wrapping_neg();
                choice |= chosen;
                let suppressed = op.branch_mask & !chosen;
                done |= suppressed;
            }
        }
    }
    
    // Update state fields
    state.done_mask = done;
    state.choice_taken = choice;
    // ... calculate next check mask
    fired
}
```

A differential testing harness executes 1,000,000 randomized execution steps, asserting bit-identical state updates and fire sets between the unrolled implementation and the oracle.

### 5.2 Hostile Mutants
Three independent mutants are defined under `@armstrong_fault` rules:

1. **Mutant 1 (Loop Bound Offset)**:
   Modify the loop bound in Pass 3 to terminate early:
   ```rust
   for i in 0..63 { // Mutated: missing last slot check
   ```
   *Expected outcome*: Operations on slot 63 fail to fire or propagate check masks. Differential testing will identify the missing fire event and raise a validation error.
2. **Mutant 2 (Gating Mask Inversion)**:
   Invert the selection check:
   ```rust
   let is_selected_mask = 0u64.wrapping_sub(!is_selected); // Mutated: gates on unselected instead
   ```
   *Expected outcome*: Operations gate in reverse, causing conflict violations. This triggers a `StabilityRefusal::EnvelopeViolated` refusal.
3. **Mutant 3 (XorDispatch Chosen Mask Shift)**:
   Alter XOR branch selection to choose the highest set bit:
   ```rust
   let chosen = op.branch_mask; // Mutated: selects all branch masks instead of lowest-indexed chosen
   ```
   *Expected outcome*: Fails to isolate branch choice, marking all branches as taken, which violates the XOR-mutual-exclusion invariant and triggers an assertion failure.

### 5.3 Object-Code Disassembly Audit Plan
The production release build is compiled and disassembled:
```bash
cargo objdump --bin bcinr-powl --release -- --disassemble --symbol="execute_tick_unrolled"
```
The disassembly must pass the `@turing_machine` audit:
1. **Zero Conditional Jumps**: Total count of conditional jump instructions (`je`, `jne`, etc.) must be exactly $0$.
2. **Loop Backedge Count = 0**: The instruction trace must contain no backward jumps, proving the 64-iteration loop is fully unrolled.
3. **No External Library Calls**: The symbol list must contain no references to heap allocator libraries or unwinding helper functions.

---

## 6. Implementation Architecture & Integration Plan

The UBPTS structures will be integrated as follows:
1. **[`crates/bcinr-powl/src/scheduler.rs`](file:///Users/sac/bcinr/crates/bcinr-powl/src/scheduler.rs)**:
   - Deprecate the old branching `scheduler_tick` and `scheduler_tick_guarded` methods.
   - Insert `preview_tick_unrolled`, `execute_tick_unrolled`, and their associated branchless helpers.
   - Refactor `StableMaximalSelector` to implement `select_mask(ready_mask, guards) -> u64`.
2. **[`crates/bcinr-powl/src/lib.rs`](file:///Users/sac/bcinr/crates/bcinr-powl/src/lib.rs)**:
   - Expose the unrolled tick entry points to the API.
3. **Receipt Hashing Integration**:
   - Update `bcinr-powl-receipt` to hash the new `u64` fire masks directly without set conversion, satisfying `direct_bitmask_receipt_hashing`.
