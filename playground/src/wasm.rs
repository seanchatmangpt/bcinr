#![allow(unsafe_code)]

use bcinr::mask::{select_u32, nonzero_mask_u32};
use bcinr::int::popcount_u64;
use crate::yawl::{BYawlEngine, BYawlTask};
use crate::powl::{Powl64Op, PowlState, powl64_execute_step};

/// Results of the token replay operation exported to C/WASM.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmReplayResult {
    /// Count of missing tokens supplied on demand.
    pub missing: u32,
    /// Count of remaining tokens after replay completion.
    pub remaining: u32,
    /// Count of produced tokens.
    pub consumed: u32,
    /// Count of consumed tokens.
    pub produced: u32,
}

/// YAWL Engine state exported to C/WASM.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmBYawlState {
    /// Bitmask of the current active places.
    pub state_mask: u64,
    /// Number of active instances per place (up to 64 places).
    pub active_instances: [u8; 64],
    /// Bitmask of active triggers.
    pub active_triggers: u64,
    /// Bitmask of fired joins.
    pub fired_joins_mask: u64,
    /// Bitmask of active locks.
    pub active_locks: u64,
}

/// POWL Engine state exported to C/WASM.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmPowlState {
    pub completed_ops: u64,
    pub completed_branches: u64,
    pub active_scopes: u64,
    pub scope_stack: [u16; 16],
    pub stack_depth: u32,
    pub completed_loops: u64,
}

/// WASM boundary for running trace token replay checker.
///
/// Returns 0 on success, -1 if any input pointer is null, and -2 if any out-of-bounds transition is encountered.
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
    let is_null_in = transitions_in.is_null();
    let is_null_out = transitions_out.is_null();
    let is_null_trace = trace_events.is_null();
    let is_null_res = out_result.is_null();
    
    let is_any_null = is_null_in | is_null_out | is_null_trace | is_null_res;
    
    let any_null_mask_u32 = nonzero_mask_u32(is_any_null as u32);
    let null_mask = (any_null_mask_u32 as i32 as i64) as u64;
    let valid_mask = !null_mask as usize;
    
    let mut marking = initial_mask;
    let mut missing = 0u32;
    let mut consumed = 0u32;
    let mut produced = popcount_u64(initial_mask) as u32;

    let safe_in = ((transitions_in as usize & valid_mask) | (8 & null_mask as usize)) as *const u64;
    let safe_out = ((transitions_out as usize & valid_mask) | (8 & null_mask as usize)) as *const u64;
    let safe_trace = ((trace_events as usize & valid_mask) | (4 & null_mask as usize)) as *const u32;
    let safe_res = ((out_result as usize & valid_mask) | (&mut marking as *mut _ as usize & null_mask as usize)) as *mut WasmReplayResult;
    
    let clean_n_transitions = select_u32(any_null_mask_u32, 0, n_transitions);
    let clean_trace_len = select_u32(any_null_mask_u32, 0, trace_len);
    
    let trans_in = core::slice::from_raw_parts(safe_in, clean_n_transitions as usize);
    let trans_out = core::slice::from_raw_parts(safe_out, clean_n_transitions as usize);
    let trace = core::slice::from_raw_parts(safe_trace, clean_trace_len as usize);
    
    let mut out_of_bounds_detected = false;
    
    for i in 0..(clean_trace_len as usize) {
        let t_idx = trace[i];
        let is_valid_idx = t_idx < clean_n_transitions;
        out_of_bounds_detected |= !is_valid_idx;
        
        let valid_idx_mask = nonzero_mask_u32(is_valid_idx as u32);
        let safe_t_idx = select_u32(valid_idx_mask, t_idx, 0) as usize;
        
        let in_mask = trans_in[safe_t_idx];
        let out_mask = trans_out[safe_t_idx];
        
        let fire_mask = (valid_idx_mask as i32 as i64) as u64;
        
        let actual_in_mask = in_mask & fire_mask;
        let actual_out_mask = out_mask & fire_mask;
        
        crate::petri::petri_fire_transition(
            &mut marking,
            actual_in_mask,
            actual_out_mask,
            &mut missing,
            &mut consumed,
            &mut produced,
        );
    }
    
    // Final marking consumption
    let final_needed = popcount_u64(final_mask) as u32;
    let final_have = popcount_u64(marking & final_mask) as u32;
    let diff = final_needed.saturating_sub(final_have);
    missing += diff;
    consumed += final_needed;
    
    (*safe_res).missing = missing;
    (*safe_res).remaining = popcount_u64(marking & !final_mask) as u32;
    (*safe_res).produced = produced;
    (*safe_res).consumed = consumed;
    
    let out_of_bounds_mask = nonzero_mask_u32(out_of_bounds_detected as u32);
    let code_oob = select_u32(out_of_bounds_mask, -2i32 as u32, 0);
    
    select_u32(any_null_mask_u32, -1i32 as u32, code_oob) as i32
}

/// WASM boundary for executing a YAWL task step.
///
/// Returns the engine's fired mask as an i32 (0 for not fired, -1 if fired), or -1 if any pointer is null.
#[no_mangle]
pub unsafe extern "C" fn wasm_yawl_execute_task(
    state_ptr: *mut WasmBYawlState,
    task_ptr: *const BYawlTask,
) -> i32 {
    let is_null_state = state_ptr.is_null();
    let is_null_task = task_ptr.is_null();
    let is_any_null = is_null_state | is_null_task;
    
    let any_null_mask_u32 = nonzero_mask_u32(is_any_null as u32);
    let null_mask = (any_null_mask_u32 as i32 as i64) as u64;
    let valid_mask = !null_mask as usize;
    
    let mut engine = BYawlEngine {
        state_mask: 0,
        active_instances: [0; 64],
        active_triggers: 0,
        fired_joins_mask: 0,
        active_locks: 0,
    };
    
    let safe_state = ((state_ptr as usize & valid_mask) | (&mut engine as *mut _ as usize & null_mask as usize)) as *mut WasmBYawlState;
    let safe_task = ((task_ptr as usize & valid_mask) | (&mut engine as *mut _ as usize & null_mask as usize)) as *const BYawlTask;
    
    engine.state_mask = (*safe_state).state_mask;
    engine.active_instances = (*safe_state).active_instances;
    engine.active_triggers = (*safe_state).active_triggers;
    engine.fired_joins_mask = (*safe_state).fired_joins_mask;
    engine.active_locks = (*safe_state).active_locks;
    
    let result = engine.execute_task_branchless(&*safe_task);
    
    (*safe_state).state_mask = engine.state_mask;
    (*safe_state).active_instances = engine.active_instances;
    (*safe_state).active_triggers = engine.active_triggers;
    (*safe_state).fired_joins_mask = engine.fired_joins_mask;
    (*safe_state).active_locks = engine.active_locks;
    
    select_u32(any_null_mask_u32, -1i32 as u32, result as u32) as i32
}

/// WASM boundary for executing a POWL step.
///
/// Returns 0 on success, -1 if any pointer is null.
#[no_mangle]
pub unsafe extern "C" fn wasm_powl_execute_step(
    state_ptr: *mut WasmPowlState,
    op_ptr: *const Powl64Op,
    input_choice: u64,
    loop_repeat: u64,
) -> i32 {
    let is_null_state = state_ptr.is_null();
    let is_null_op = op_ptr.is_null();
    let is_any_null = is_null_state | is_null_op;
    
    let any_null_mask_u32 = nonzero_mask_u32(is_any_null as u32);
    let null_mask = (any_null_mask_u32 as i32 as i64) as u64;
    let valid_mask = !null_mask as usize;
    
    let mut engine_state = PowlState {
        completed_ops: 0,
        completed_branches: 0,
        active_scopes: 0,
        scope_stack: [0; 16],
        stack_depth: 0,
        completed_loops: 0,
    };
    
    let safe_state = ((state_ptr as usize & valid_mask) | (&mut engine_state as *mut _ as usize & null_mask as usize)) as *mut WasmPowlState;
    let safe_op = ((op_ptr as usize & valid_mask) | (&mut engine_state as *mut _ as usize & null_mask as usize)) as *const Powl64Op;
    
    engine_state.completed_ops = (*safe_state).completed_ops;
    engine_state.completed_branches = (*safe_state).completed_branches;
    engine_state.active_scopes = (*safe_state).active_scopes;
    engine_state.scope_stack = (*safe_state).scope_stack;
    engine_state.stack_depth = (*safe_state).stack_depth;
    engine_state.completed_loops = (*safe_state).completed_loops;
    
    powl64_execute_step(&mut engine_state, &*safe_op, input_choice, loop_repeat);
    
    (*safe_state).completed_ops = engine_state.completed_ops;
    (*safe_state).completed_branches = engine_state.completed_branches;
    (*safe_state).active_scopes = engine_state.active_scopes;
    (*safe_state).scope_stack = engine_state.scope_stack;
    (*safe_state).stack_depth = engine_state.stack_depth;
    (*safe_state).completed_loops = engine_state.completed_loops;
    
    select_u32(any_null_mask_u32, -1i32 as u32, 0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_petri_replay_success() {
        let trans_in = [1u64, 2u64];
        let trans_out = [2u64, 4u64];
        let trace = [0u32, 1u32];
        let mut result = WasmReplayResult {
            missing: 99,
            remaining: 99,
            produced: 99,
            consumed: 99,
        };

        let rc = unsafe {
            wasm_petri_replay(
                1, // initial_mask (token at place 0)
                4, // final_mask (token at place 2)
                trans_in.as_ptr(),
                trans_out.as_ptr(),
                2,
                trace.as_ptr(),
                2,
                &mut result,
            )
        };

        assert_eq!(rc, 0);
        assert_eq!(result.missing, 0);
        assert_eq!(result.remaining, 0);
        assert_eq!(result.produced, 3);
        assert_eq!(result.consumed, 3);
    }

    #[test]
    fn test_wasm_petri_replay_null_pointers() {
        let rc = unsafe {
            wasm_petri_replay(
                1,
                4,
                core::ptr::null(),
                core::ptr::null(),
                0,
                core::ptr::null(),
                0,
                core::ptr::null_mut(),
            )
        };
        assert_eq!(rc, -1);
    }

    #[test]
    fn test_wasm_petri_replay_out_of_bounds() {
        let trans_in = [1u64];
        let trans_out = [2u64];
        let trace = [5u32]; // Out of bounds!
        let mut result = WasmReplayResult {
            missing: 99,
            remaining: 99,
            produced: 99,
            consumed: 99,
        };

        let rc = unsafe {
            wasm_petri_replay(
                1,
                2,
                trans_in.as_ptr(),
                trans_out.as_ptr(),
                1,
                trace.as_ptr(),
                1,
                &mut result,
            )
        };
        assert_eq!(rc, -2);
    }

    #[test]
    fn test_wasm_yawl_execute_task_success() {
        let mut state = WasmBYawlState {
            state_mask: 0b11, // Places 0 and 1 active
            active_instances: [0; 64],
            active_triggers: 0,
            fired_joins_mask: 0,
            active_locks: 0,
        };
        state.active_instances[0] = 1;
        state.active_instances[1] = 1;

        let task = BYawlTask {
            id: 1,
            join_type: JoinType::AND,
            split_type: SplitType::AND,
            min_instances: 1,
            max_instances: 1,
            threshold_instances: 0,
            join_state_bit: 0,
            flags: 0,
            consume_mask: 0b11,
            produce_mask: 0b100, // Produces to place 2
            cancellation_mask: 0,
            condition_mask: 0,
            reset_mask: 0,
            reachability_mask: 0,
            interleaved_lock_mask: 0,
        };

        let rc = unsafe { wasm_yawl_execute_task(&mut state, &task) };
        assert_eq!(rc, -1); // Fired mask in i32 is -1 (u64::MAX)
        assert_eq!(state.state_mask, 0b100);
    }

    #[test]
    fn test_wasm_yawl_execute_task_null_pointers() {
        let rc = unsafe { wasm_yawl_execute_task(core::ptr::null_mut(), core::ptr::null()) };
        assert_eq!(rc, -1);
    }

    #[test]
    fn test_wasm_powl_execute_step_success() {
        let mut state = WasmPowlState {
            completed_ops: 0,
            completed_branches: 0,
            active_scopes: 1, // scope 0
            scope_stack: [0; 16],
            stack_depth: 1,
            completed_loops: 0,
        };

        let op = Powl64Op {
            kind: Powl64OpKind::Activity,
            lane: 0,
            activity: 5,
            scope: 0,
            branch: 0,
            loop_id: 0,
            pred_mask: 0,
            succ_mask: 1 << 5,
            ctrl_mask: 0,
            intensity: 0,
            _pad: [0; 7],
        };

        let rc = unsafe { wasm_powl_execute_step(&mut state, &op, 0, 0) };
        assert_eq!(rc, 0);
        assert_eq!(state.completed_ops, 1 << 5);
    }

    #[test]
    fn test_wasm_powl_execute_step_null_pointers() {
        let rc = unsafe { wasm_powl_execute_step(core::ptr::null_mut(), core::ptr::null(), 0, 0) };
        assert_eq!(rc, -1);
    }
}
