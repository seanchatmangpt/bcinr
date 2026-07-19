# BCINR-POWL: Branchless Pipelines, Kahn's Compilation, and BSS/SRBCG Mechanics

## 1. Executive Summary

This report details the inner workings of the `bcinr-powl` crate, which provides the Partially Ordered Workflow Language (POWL) scheduler and compilation substrate. Adhering to the Radon Law ($CC=1$), the components guarantee deterministic, timing side-channel-free execution by transforming dynamic control flow into bitwise algebraic operations.

The components explored include:
- **`scheduler.rs`**: Bitmask-driven SWAR execution pipelines.
- **`compiler.rs`**: Kahn's cycle detection and Roy-Warshall reachability matrices.
- **`typestate.rs`**: Compile-time phase isolation and Branchless Linear Execution Tokens (BLET).
- **`ocel.rs`**: Symmetric Run-Bounded Conformance Gating (SRBCG) comparison networks.
- **`enterprise.rs`**: Bounded Saga Stack (BSS) using indices multiplexing.

---

## 2. POWL Compiler: Kahn's Cycle Detection and Bit-Parallel Reachability

The POWL compiler translates Abstract Syntax Trees into a flat array of 64 operations (`PowlTape`). To ensure structural validity without iterating during runtime, the compiler performs a two-phase graph validation (`bcinr-powl/src/compiler.rs`).

### Phase 1: Kahn's Topological Sort (Cycle Detection)
The compiler runs Kahn's algorithm over the tape. To support deliberate loops, it masks out back-edges designated as `OpKind::LoopRedo`.

```rust
// In `crates/bcinr-powl/src/compiler.rs`
fn run_kahn_walk(tape: &PowlTape, n: usize, mut in_deg: [u32; 64]) -> Result<(), CompileError> {
    // ... setup and BFS walk ...
    let non_redo_count = (0..n)
        .filter(|&i| tape.ops[i].kind != OpKind::LoopRedo)
        .count();
    if visited < non_redo_count {
        Err(CompileError::Cycle)
    } else {
        Ok(())
    }
}
```

### Phase 2: Roy-Warshall Transitive Closure (Reachability)
The compiler verifies that all non-LoopRedo nodes are reachable from the tape's entry mask using a completely branchless Bit-Parallel Roy-Warshall algorithm. The algorithm relies entirely on bit-shifts and `wrapping_sub`.

```rust
// Bitwise Transitive Closure Propagation
for k in 0..64 {
    let r_k = r[k];
    for i in 0..64 {
        let can_reach_k = (r[i] >> k) & 1;
        let mask = 0u64.wrapping_sub(can_reach_k);
        r[i] |= r_k & mask;
    }
}
```

---

## 3. The SWAR Branchless Pipeline Scheduler

Execution occurs through the SWAR (SIMD-within-a-register) scheduler inside `bcinr-powl/src/scheduler.rs`. Control flow dependencies are represented via `done_mask`, `active_mask`, and `check_mask` variables housed in `PowlRunState`.

```rust
#[derive(Clone)]
#[repr(C, align(8))]
pub struct PowlRunState {
    pub done_mask: u64,
    pub active_mask: u64,
    pub check_mask: u64,
    pub choice_taken: u64,
    pub loop_iters: [u8; 64],
    pub tick: u32,
    _pad: [u8; 4],
}
```

**State Transitions Pipeline:**
1. Evaluates readiness: Active candidates in the `check_mask` compare their `pred_mask` against the current `done_mask`.
2. Gating: A concurrency-aware guard filters the candidates.
3. Firing: Selected candidates progress to the `fire_mask`, eventually modifying the `done_mask` and populating the `check_mask` with new successors.

---

## 4. Phase-Indexed Typestates and Branchless Linear Execution Tokens (BLET)

Safety constraints are guaranteed statically and dynamically through `crates/bcinr-powl/src/typestate.rs`. The `PowlRunner` advances through states: `Unvalidated -> Compiled -> Scheduled -> Executing -> Receipted`.

When transitioning to `Executing`, the runner yields an `ExecutionToken`. The token acts as an emulation of linear types, enforcing `no_alloc` tracking of the tape's progress.

### `CC=1` Branchless Token Consumption
Defects (double-firing, invalid out-of-bounds fires, malformed masks) are evaluated using boolean logic without short-circuiting control flow:

```rust
// In `crates/bcinr-powl/src/typestate.rs`
pub fn consume_op(&mut self, op_bit: u64) {
    // Accumulate invalid bit fires (bits outside the valid boundary)
    let invalid = op_bit & !self.valid_mask;
    self.defect_invalid |= invalid;

    // Accumulate double-fire defects
    let target_valid = op_bit & self.valid_mask;
    let present = self.remaining & target_valid;
    let double_fired = target_valid ^ present;
    self.defect_double_fire |= double_fired;

    // Update the remaining mask (idempotent write-through)
    self.remaining &= !op_bit;
}
```

---

## 5. Symmetric Run-Bounded Conformance Gating (SRBCG)

The Object-Centric Event Log (OCEL) is managed via SRBCG inside `bcinr-powl/src/ocel.rs`. To track up to 64 concurrent run IDs without any dynamic allocation or variable bounds loops, `process_event_srbcg` uses a fully unrolled comparison network:

```rust
for i in 0..64 {
    let is_match = (run_ids[i] == incoming_rid) as usize;
    // CSEL/CMOV conditional selection
    match_idx = (is_match * i) + ((1 - is_match) * match_idx);
}
```
Depending on the computed `match_idx`, the function branchlessly routes values, updates `run_count`, and triggers the `overflow_mask` if the capacity of 64 unique runs is exhausted.

---

## 6. Enterprise Bounded Saga Stack (BSS)

To facilitate system rollbacks and compensations without heap-backed undo logs, `bcinr-powl/src/enterprise.rs` introduces the `SagaStack`. The `SagaStack` uses **Indices Multiplexing** to achieve branchless, constant-time Push/Pop semantics.

### Multiplexing Logic
The capacity of the stack is 32 slots, with an additional 33rd slot serving as a bit-bucket (sink) for overflow.
- **Push**: `write_idx = (top & !mask) | (sink_idx & mask)`. When full, values are cleanly dumped into the sink slot.
- **Pop**: Reads fallback to the sink slot if the stack is empty, returning a `BranchlessPop` envelope containing a valid status mask:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchlessPop {
    pub value: u16,
    pub valid_mask: u16,
}
```

Capability masking relies on the same mathematical branchlessness, ensuring that tenant tier SLA privileges are assessed purely as polynomial evaluations over `u64`.
