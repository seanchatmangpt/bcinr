# Branchless Process Intelligence Analysis & Design Specification

This document provides a comprehensive structural analysis of reference repositories for the branchless process intelligence library suite (`petri`, `yawl`, `powl`, and `wasm`) and proposes a unified, branchless ($CC=1$, `#![no_std]`, zero heap allocations) architecture utilizing `bcinr` primitives.

---

## 1. Reference Repositories Analysis

### 1. Petri Net Engine (`petri.rs` & `bitmask_replay.rs`)
- **Key Structures**:
  - `Marking`: Maps place ID string to count (`Vec<(String, usize)>`).
  - `ReplayResult`: Encapsulates metrics `missing`, `remaining`, `produced`, and `consumed`. Calculates fitness: $1.0 - (missing + remaining) / (consumed + missing + produced)$.
  - `NetBitmask64`: Contains packed masks for up to 64 places (`initial_mask`, `final_mask`), along with a `Vec<TransMask>` (where `TransMask` stores `in_mask` and `out_mask` as `u64`).
  - `MarkingSet`: Stack-allocated container avoiding heap allocations by caching up to 64 markings in BFS/EPS closure frontiers.
- **Logic Flows**:
  - **Trace Replay**: Greedy transition firing along with a nested fixpoint loop for invisible transitions (`fire_invisible`). If an activity is matched, it is fired. Missing tokens are supplied on demand.
  - **Language Acceptance**: Computes the full frontier of reachable states under silent transitions (`epsilon_close` BFS) and matches trace events.
- **Edge Cases**:
  - Unbounded or unsafe Petri nets resulting in multiple tokens or infinite loop triggers.
  - Missing initial/final markings or object-type preservation mismatches.
  - Heap spilling in BFS if reachable state frontier exceeds 64 markings.

### 2. YAWL Routing Engine (`engine.rs` & `format.rs`)
- **Key Structures**:
  - `BYawlEngine`: Manages execution state using 64-bit words: `state_mask` (active tokens), `active_instances` (instance counts array), `active_triggers` (triggers), `fired_joins_mask` (discriminator history), and `active_locks` (interleaved route mutex locks).
  - `BYawlTask`: A 64-byte aligned C-compatible struct containing metadata (`JoinType`, `SplitType`, flags, bounds) and multiple `u64` bitmasks: `consume_mask`, `produce_mask`, `cancellation_mask`, `condition_mask`, `reset_mask`, `reachability_mask`, and `interleaved_lock_mask`.
- **Logic Flows**:
  - **Join Execution**: Implements `AND`, `XOR`, `OR` (using synchronizing merge), `Complex` (N-out-of-M/Discriminator), and `ThreadMerge`.
  - **Cancellation & Resets**: Cancels regions/instances specified in `cancellation_mask`, and resets complex join registers if `reset_mask` matches active tokens.
  - **Split Routing**: Directs output tokens to place/instance structures based on split type.
- **Edge Cases**:
  - Mutex lock conflicts in interleaved parallel routing.
  - Re-entry of complex joins (discriminator token vacuuming when already fired in the same cycle).
  - Explicit termination annihilating all tokens and resetting instances.

### 3. POWL Compiler (`lib.rs` & `executor.rs`)
- **Key Structures**:
  - `Powl64OpKind`: Represents compiled execution opcodes (Activity, PartialOrderGate, ChoiceGate, LoopGate, EnterScope, ExitScope, Promote, Demote, Watchdog).
  - `Powl64Op`: Cache-aligned (64-byte) instruction representation featuring `pred_mask`, `succ_mask`, and `ctrl_mask`.
  - `ScopeDesc`: Specifies scope parent, ranges, and `TruthBlock` bit offsets.
- **Logic Flows**:
  - **Sequential Walk**: The executor walks the flat `Powl64Op` array. When encountering `EnterScope`/`ExitScope`, it manages a scope projection stack. Under a `Watchdog` event, it checks deadlines and performs scope-level drains.
  - **Concur marker**: Evaluates `PartialOrderGate` with `ctrl_mask = u64::MAX`, scheduling parallel branches onto dispatcher slots.
- **Edge Cases**:
  - Scope stack depth overflow (> 16 levels) or scope index bounds exceeding 64.
  - Watchdog deadline expiration forcing denial-marked events.
  - Unbalanced scope boundaries or missing left arms for concurrent markers.

### 4. WASM API Boundary (`wasm.rs` & `streaming_wasm.rs`)
- **Key Structures & JS Interop**:
  - State management uses a global `StoredObject` pool and string handles.
  - API boundary uses `wasm_bindgen` and JSON serialization/deserialization across the JS/Rust boundary.
- **Logic Flows**:
  - Ingestion processes event streams via handle lookup, modifying stateful builders (`StreamingDfgBuilder`, etc.) on the heap.
- **Edge Cases**:
  - Dynamic allocation overhead and serialization bottlenecks.
  - Memory leaks if JS side fails to call `delete_object`.

---

## 2. Axiomatic Branchless Design Specification (Radon Law $CC=1$)

To eliminate timing side-channels and maximize throughput, we define constant-time operations utilizing branchless bitwise polynomials.

### 1. Helper Branchless Masks
- **nz_mask** (Non-zero mask): Returns `0xFFFFFFFFFFFFFFFF` if input is non-zero, `0` otherwise.
  $$\text{nz\_mask}(x) = \left(((x \mid -x) \text{ as i64}) \gg 63\right) \text{ as u64}$$
- **z_mask** (Zero mask): Returns `0xFFFFFFFFFFFFFFFF` if input is zero, `0` otherwise.
  $$\text{z\_mask}(x) = \text{!nz\_mask}(x)$$

---

## 3. Rust API Design Proposals

### 1. `petri`: Bitmask-based Token Replay
A constant-time replay engine processing trace events branchlessly:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayResult {
    pub missing: u32,
    pub remaining: u32,
    pub produced: u32,
    pub consumed: u32,
}

#[inline(always)]
fn nz_mask(x: u64) -> u64 {
    (((x | x.wrapping_neg()) as i64) >> 63) as u64
}

#[inline(always)]
fn z_mask(x: u64) -> u64 {
    !nz_mask(x)
}

/// Constant-time Petri transition firing step (CC = 1).
#[inline(always)]
pub fn petri_fire_transition(
    marking: &mut u64,
    in_mask: u64,
    out_mask: u64,
    missing: &mut u32,
    consumed: &mut u32,
    produced: &mut u32,
) {
    let need = in_mask & !(*marking);
    *missing += need.count_ones();
    *marking |= need;

    *marking = (*marking & !in_mask) | out_mask;
    *consumed += in_mask.count_ones();
    *produced += out_mask.count_ones();
}

/// Constant-time invisible transition closure.
/// Bounded to a fixed count (e.g. 16) to comply with CC = 1.
#[inline(always)]
pub fn petri_fire_invisible(
    marking: &mut u64,
    inv_in_masks: &[u64],
    inv_out_masks: &[u64],
) {
    let n = inv_in_masks.len();
    for _ in 0..16 {
        for i in 0..16 {
            let in_bounds = ((i < n) as u64).wrapping_neg();
            let in_mask = inv_in_masks[i & 15] & in_bounds;
            let out_mask = inv_out_masks[i & 15] & in_bounds;

            let is_enabled = z_mask((*marking & in_mask) ^ in_mask) & in_bounds;
            *marking = (*marking & !(in_mask & is_enabled)) | (out_mask & is_enabled);
        }
    }
}
```

---

### 2. `yawl`: Branchless Routing calculus (AND/XOR/OR splits/joins)

```rust
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JoinType {
    XOR = 0,
    AND = 1,
    OR = 2,
    Complex = 3,
    ThreadMerge = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitType {
    XOR = 0,
    AND = 1,
    OR = 2,
    MultiInstance = 3,
    DynamicMultiInstance = 4,
    DeferredChoice = 5,
    InterleavedRouting = 6,
    ThreadSplit = 7,
    ImplicitTermination = 8,
    ExplicitTermination = 9,
}

pub struct BYawlEngine {
    pub state_mask: u64,
    pub active_instances: [u8; 64],
    pub active_triggers: u64,
    pub fired_joins_mask: u64,
    pub active_locks: u64,
}

pub struct BYawlTask {
    pub id: u16,
    pub join_type: JoinType,
    pub split_type: SplitType,
    pub min_instances: u8,
    pub max_instances: u8,
    pub threshold_instances: u8,
    pub join_state_bit: u8,
    pub flags: u8, // bit 0: Transient trigger, bit 2: Interleaved release, bit 3: Complete MI
    pub consume_mask: u64,
    pub produce_mask: u64,
    pub cancellation_mask: u64,
    pub condition_mask: u64,
    pub reset_mask: u64,
    pub reachability_mask: u64,
    pub interleaved_lock_mask: u64,
}

impl BYawlEngine {
    /// Fully branchless task execution (CC = 1, #![no_std]).
    pub fn execute_task_branchless(&mut self, task: &BYawlTask) -> u64 {
        // --- 1. Reset Complex Joins ---
        let has_reset_tokens = nz_mask(self.state_mask & task.reset_mask);
        self.fired_joins_mask &= !((1u64.wrapping_shl(task.join_state_bit as u32 & 63)) & has_reset_tokens);
        self.state_mask &= !(task.reset_mask & has_reset_tokens);

        // --- 2. Evaluate Lock & Conditions ---
        let lock_conflict = nz_mask(self.active_locks & task.interleaved_lock_mask);
        let allowed_by_lock = !lock_conflict;

        let cond_not_met = nz_mask((self.state_mask & task.condition_mask) ^ task.condition_mask);
        let allowed_by_cond = !cond_not_met;

        // --- 3. Join Predicate Evaluations ---
        let join_xor = {
            let c = self.state_mask & task.consume_mask;
            nz_mask(c) & z_mask(c & c.wrapping_sub(1))
        };

        let join_and = z_mask((self.state_mask & task.consume_mask) ^ task.consume_mask);

        let join_or = {
            let val = self.state_mask & task.consume_mask;
            let aux = self.state_mask & task.reachability_mask;
            nz_mask(val) & z_mask(aux & !val)
        };

        let join_complex = {
            let pt = (self.state_mask & task.consume_mask).count_ones() as u8;
            let has_fired = nz_mask(self.fired_joins_mask & (1u64.wrapping_shl(task.join_state_bit as u32 & 63)));
            let threshold_met = !nz_mask((((pt as i16 - task.threshold_instances as i16) >> 15) & 1) as u64);
            !has_fired & threshold_met;
        };

        let join_thread_merge = nz_mask(self.state_mask & task.consume_mask);

        // Multiplex Join Types
        let mask_xor = z_mask((task.join_type as u64) ^ (JoinType::XOR as u64));
        let mask_and = z_mask((task.join_type as u64) ^ (JoinType::AND as u64));
        let mask_or = z_mask((task.join_type as u64) ^ (JoinType::OR as u64));
        let mask_complex = z_mask((task.join_type as u64) ^ (JoinType::Complex as u64));
        let mask_thread_merge = z_mask((task.join_type as u64) ^ (JoinType::ThreadMerge as u64));

        let can_join = (join_xor & mask_xor)
            | (join_and & mask_and)
            | (join_or & mask_or)
            | (join_complex & mask_complex)
            | (join_thread_merge & mask_thread_merge);

        // Fired mask: task executes successfully
        let fired = allowed_by_lock & allowed_by_cond & can_join;

        // Complex Join: token consumption on bypass
        let is_complex = mask_complex;
        let complex_has_fired = nz_mask(self.fired_joins_mask & (1u64.wrapping_shl(task.join_state_bit as u32 & 63)));
        let consume_on_blocked = (!fired) & is_complex & complex_has_fired;
        let do_consume = fired | consume_on_blocked;

        // --- 4. State Updates ---
        // Consume tokens
        self.state_mask &= !(task.consume_mask & do_consume);

        // Active triggers
        let is_transient = nz_mask((task.flags & 1) as u64);
        self.active_triggers &= !(task.consume_mask & fired & is_transient);

        // Active locks
        self.active_locks |= task.interleaved_lock_mask & fired;
        let is_release_lock = nz_mask((task.flags & 4) as u64);
        self.active_locks &= !(task.interleaved_lock_mask & fired & is_release_lock);

        // Fired complex joins
        self.fired_joins_mask |= (1u64.wrapping_shl(task.join_state_bit as u32 & 63)) & fired & is_complex;

        // Cancellations
        self.state_mask &= !(task.cancellation_mask & fired);
        for i in 0..64 {
            let clear_cancel = nz_mask(task.cancellation_mask & (1u64 << i)) & fired;
            self.active_instances[i] &= !(clear_cancel as u8);
        }

        // Complete MI Activity
        let is_complete_mi = nz_mask((task.flags & 8) as u64);
        let clear_mi = fired & is_complete_mi;
        for i in 0..64 {
            let clear_instance = nz_mask(task.produce_mask & (1u64 << i)) & clear_mi;
            self.active_instances[i] &= !(clear_instance as u8);
        }

        // --- 5. Splits & Produces ---
        let split_it = z_mask((task.split_type as u64) ^ (SplitType::ImplicitTermination as u64));
        let split_et = z_mask((task.split_type as u64) ^ (SplitType::ExplicitTermination as u64));
        let split_mi = z_mask((task.split_type as u64) ^ (SplitType::MultiInstance as u64));

        let should_produce = fired & !(split_it | split_et);
        self.state_mask |= task.produce_mask & should_produce;

        // Explicit Termination
        let et_mask = !(fired & split_et);
        self.state_mask &= et_mask;
        self.fired_joins_mask &= et_mask;
        self.active_locks &= et_mask;
        for i in 0..64 {
            self.active_instances[i] &= et_mask as u8;
        }

        // Multi-Instance produce limits
        let target_idx = task.produce_mask.trailing_zeros() as usize;
        for i in 0..64 {
            let is_target = z_mask((target_idx as u64) ^ (i as u64)) & fired & split_mi;
            self.active_instances[i] = (self.active_instances[i] & !(is_target as u8))
                | (task.max_instances & (is_target as u8));
        }

        fired
    }
}
```

---

### 3. `powl`: Non-recursive flat opcode executor

```rust
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Powl64OpKind {
    Activity = 0,
    PartialOrderGate = 1,
    ChoiceGate = 2,
    LoopGate = 3,
    EnterScope = 4,
    ExitScope = 5,
    Promote = 6,
    Demote = 7,
    Watchdog = 8,
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Powl64Op {
    pub kind: Powl64OpKind,
    pub lane: u8,
    pub activity: u16,
    pub scope: u16,
    pub branch: u16,
    pub loop_id: u16,
    pub pred_mask: u64,
    pub succ_mask: u64,
    pub ctrl_mask: u64,
    pub intensity: u8,
    _pad: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PowlState {
    pub completed: u64,
    pub enabled: u64,
    pub active_choices: u64,
    pub active_loops: u64,
}

/// Constant-time step execution of flat POWL opcodes (CC = 1).
#[inline(always)]
pub fn powl64_execute_step(
    state: &mut PowlState,
    op: &Powl64Op,
    input_choice: u64,
    loop_repeat: u64,
) {
    let is_activity = z_mask(op.kind as u64 ^ Powl64OpKind::Activity as u64);
    let is_po_gate = z_mask(op.kind as u64 ^ Powl64OpKind::PartialOrderGate as u64);
    let is_choice_gate = z_mask(op.kind as u64 ^ Powl64OpKind::ChoiceGate as u64);
    let is_loop_gate = z_mask(op.kind as u64 ^ Powl64OpKind::LoopGate as u64);

    // 1. Activity Execution
    let act_enabled = nz_mask(state.enabled & (1u64.wrapping_shl(op.activity as u32 & 63)));
    let act_fire = is_activity & act_enabled;
    state.completed |= (1u64.wrapping_shl(op.activity as u32 & 63)) & act_fire;
    state.enabled &= !((1u64.wrapping_shl(op.activity as u32 & 63)) & act_fire);

    // 2. PartialOrderGate
    let pred_done = z_mask((state.completed & op.pred_mask) ^ op.pred_mask);
    let po_fire = is_po_gate & pred_done;
    state.enabled |= op.succ_mask & po_fire;

    // 3. ChoiceGate
    let choice_enabled = nz_mask(state.active_choices & op.ctrl_mask);
    let choice_chosen = nz_mask(input_choice & (1u64.wrapping_shl(op.branch as u32 & 63)));
    let choice_fire = is_choice_gate & choice_enabled & choice_chosen;
    state.active_choices |= (1u64.wrapping_shl(op.branch as u32 & 63)) & choice_fire;
    state.enabled |= op.succ_mask & choice_fire;

    // 4. LoopGate (Enter and Exit)
    let is_loop_enter = is_loop_gate & nz_mask(op.ctrl_mask);
    let is_loop_exit = is_loop_gate & z_mask(op.ctrl_mask);

    let loop_enter_enabled = nz_mask(state.enabled & op.ctrl_mask);
    let enter_fire = is_loop_enter & loop_enter_enabled;
    state.active_loops |= op.ctrl_mask & enter_fire;

    let body_done = z_mask((state.completed & op.pred_mask) ^ op.pred_mask);
    let exit_fire = is_loop_exit & body_done;

    let should_repeat = nz_mask(loop_repeat & (1u64.wrapping_shl(op.loop_id as u32 & 63)));
    state.enabled |= op.succ_mask & exit_fire & should_repeat;

    let should_exit = !should_repeat;
    state.active_loops &= !((1u64.wrapping_shl(op.loop_id as u32 & 63)) & exit_fire & should_exit);
}
```

---

### 4. `wasm`: no_std WASM API interface
Provides raw pointers and direct primitive integration:

```rust
#[repr(C)]
pub struct WasmReplayResult {
    pub missing: u32,
    pub remaining: u32,
    pub produced: u32,
    pub consumed: u32,
}

#[repr(C)]
pub struct WasmBYawlState {
    pub state_mask: u64,
    pub active_instances: [u8; 64],
    pub active_triggers: u64,
    pub fired_joins_mask: u64,
    pub active_locks: u64,
}

/// WASM boundary for running trace token replay checker.
#[no_mangle]
pub unsafe extern "C" fn wasm_petri_replay(
    initial_mask: u64,
    final_mask: u64,
    transitions_in: *const u64,
    transitions_out: *const u64,
    n_transitions: u32,
    trace_events: *const u32,
    trace_len: u32,
    out_result: *mut WasmReplayResult,
) -> i32 {
    if transitions_in.is_null() || transitions_out.is_null() || trace_events.is_null() || out_result.is_null() {
        return -1;
    }

    let mut marking = initial_mask;
    let mut missing = 0u32;
    let mut consumed = 0u32;
    let mut produced = initial_mask.count_ones();

    let trans_in = core::slice::from_raw_parts(transitions_in, n_transitions as usize);
    let trans_out = core::slice::from_raw_parts(transitions_out, n_transitions as usize);
    let trace = core::slice::from_raw_parts(trace_events, trace_len as usize);

    for &t_idx in trace {
        if t_idx >= n_transitions {
            return -2;
        }
        let in_mask = trans_in[t_idx as usize];
        let out_mask = trans_out[t_idx as usize];
        petri_fire_transition(&mut marking, in_mask, out_mask, &mut missing, &mut consumed, &mut produced);
    }

    // Final marking consumption
    let final_needed = final_mask.count_ones();
    let final_have = (marking & final_mask).count_ones();
    let diff = final_needed.saturating_sub(final_have);
    missing += diff;
    consumed += final_needed;

    (*out_result).missing = missing;
    (*out_result).remaining = (marking & !final_mask).count_ones();
    (*out_result).produced = produced;
    (*out_result).consumed = consumed;

    0
}

/// WASM boundary for executing a YAWL task step.
#[no_mangle]
pub unsafe extern "C" fn wasm_yawl_execute_task(
    state_ptr: *mut WasmBYawlState,
    task_ptr: *const BYawlTask,
) -> i32 {
    if state_ptr.is_null() || task_ptr.is_null() {
        return -1;
    }

    let state = &mut *(state_ptr as *mut BYawlEngine);
    let task = &*task_ptr;
    let result = state.execute_task_branchless(task);

    result as i32
}
```
