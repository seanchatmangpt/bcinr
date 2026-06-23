# Scope: Implementation Track of Process Intelligence Project

## Architecture
We are implementing the process intelligence suite inside the `playground` crate in `#![no_std]` and with zero heap allocations under the Radon Law ($CC=1$).
Files under `playground/src/`:
- `lib.rs`: Library root exporting the modules and specifying `#![no_std]`.
- `petri.rs`: Petri net token replay engine.
- `yawl.rs`: YAWL routing semantics.
- `powl.rs`: POWL compiler/executor.
- `wasm.rs`: WASM C API boundary.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Petri Net Engine | Implement branchless Petri net token replay (`petri`) | none | DONE |
| 2 | YAWL Routing Engine | Implement branchless YAWL routing semantics (`yawl`) | M1 | IN_PROGRESS (Conv: e5d97ef2-2082-4cca-be8c-4571aea55dde) |
| 3 | POWL Compiler | Implement flat non-recursive POWL execution (`powl`) | M2 | PLANNED |
| 4 | WASM API Boundary | Implement no_std WASM C-interface wrappers (`wasm`) | M1, M2, M3 | PLANNED |
| 5 | Final Integration & E2E | Run E2E tests, add adversarial checks, run forensic auditor | M1, M2, M3, M4 | PLANNED |

## Interface Contracts

### 1. `petri`:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayResult {
    pub missing: u32,
    pub remaining: u32,
    pub produced: u32,
    pub consumed: u32,
}

pub fn petri_fire_transition(
    marking: &mut u64,
    in_mask: u64,
    out_mask: u64,
    missing: &mut u32,
    consumed: &mut u32,
    produced: &mut u32,
);

pub fn petri_fire_invisible(
    marking: &mut u64,
    inv_in_masks: &[u64],
    inv_out_masks: &[u64],
);
```

### 2. `yawl`:
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

#[repr(C)]
pub struct BYawlEngine {
    pub state_mask: u64,
    pub active_instances: [u8; 64],
    pub active_triggers: u64,
    pub fired_joins_mask: u64,
    pub active_locks: u64,
}

#[repr(C)]
pub struct BYawlTask {
    pub id: u16,
    pub join_type: JoinType,
    pub split_type: SplitType,
    pub min_instances: u8,
    pub max_instances: u8,
    pub threshold_instances: u8,
    pub join_state_bit: u8,
    pub flags: u8,
    pub consume_mask: u64,
    pub produce_mask: u64,
    pub cancellation_mask: u64,
    pub condition_mask: u64,
    pub reset_mask: u64,
    pub reachability_mask: u64,
    pub interleaved_lock_mask: u64,
}

impl BYawlEngine {
    pub fn execute_task_branchless(&mut self, task: &BYawlTask) -> u64;
}
```

### 3. `powl`:
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

pub fn powl64_execute_step(
    state: &mut PowlState,
    op: &Powl64Op,
    input_choice: u64,
    loop_repeat: u64,
);
```

### 4. `wasm`:
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

pub unsafe extern "C" fn wasm_petri_replay(
    initial_mask: u64,
    final_mask: u64,
    transitions_in: *const u64,
    transitions_out: *const u64,
    n_transitions: u32,
    trace_events: *const u32,
    trace_len: u32,
    out_result: *mut WasmReplayResult,
) -> i32;

pub unsafe extern "C" fn wasm_yawl_execute_task(
    state_ptr: *mut WasmBYawlState,
    task_ptr: *const BYawlTask,
) -> i32;
```
