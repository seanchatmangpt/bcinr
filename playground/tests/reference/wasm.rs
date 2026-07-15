#![allow(unsafe_code)]
//! WASM API C-Interface Wrappers Reference Implementation

use std::{ffi::CStr, os::raw::c_char};

use super::{
    petri::{
        in_language, replay_trace, Arc, Attribute, AttributeValue, Event, NetBitmask64, PetriNet,
        Place, Trace, Transition,
    },
    powl::{Dispatcher, Powl64Executor, Powl64Program, Watchdog},
    yawl::{BYawlEngine, BYawlTask, JoinType, SplitType},
};

// ── Petri Net FFI ───────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn ref_petri_create() -> *mut PetriNet {
    Box::into_raw(Box::new(PetriNet {
        places: Vec::new(),
        transitions: Vec::new(),
        arcs: Vec::new(),
        initial_marking: Vec::new(),
        final_marking: Vec::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_free(net: *mut PetriNet) {
    if !net.is_null() {
        let _ = Box::from_raw(net);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_add_place(net: *mut PetriNet, id: *const c_char) {
    if net.is_null() || id.is_null() {
        return;
    }
    let id_str = CStr::from_ptr(id).to_string_lossy().into_owned();
    (*net).places.push(Place { id: id_str });
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_add_transition(
    net: *mut PetriNet,
    id: *const c_char,
    label: *const c_char,
    is_invisible: bool,
) {
    if net.is_null() || id.is_null() || label.is_null() {
        return;
    }
    let id_str = CStr::from_ptr(id).to_string_lossy().into_owned();
    let label_str = CStr::from_ptr(label).to_string_lossy().into_owned();
    (*net).transitions.push(Transition {
        id: id_str,
        label: label_str,
        is_invisible: Some(is_invisible),
    });
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_add_arc(
    net: *mut PetriNet,
    from: *const c_char,
    to: *const c_char,
) {
    if net.is_null() || from.is_null() || to.is_null() {
        return;
    }
    let from_str = CStr::from_ptr(from).to_string_lossy().into_owned();
    let to_str = CStr::from_ptr(to).to_string_lossy().into_owned();
    (*net).arcs.push(Arc { from: from_str, to: to_str, weight: None, object_type: None });
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_set_initial_marking(
    net: *mut PetriNet,
    place_id: *const c_char,
    count: usize,
) {
    if net.is_null() || place_id.is_null() {
        return;
    }
    let p_str = CStr::from_ptr(place_id).to_string_lossy().into_owned();
    (*net).initial_marking.push((p_str, count));
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_set_final_marking(
    net: *mut PetriNet,
    place_id: *const c_char,
    count: usize,
) {
    if net.is_null() || place_id.is_null() {
        return;
    }
    let p_str = CStr::from_ptr(place_id).to_string_lossy().into_owned();
    (*net).final_marking.push((p_str, count));
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_replay_trace(
    net: *mut PetriNet,
    activities: *const *const c_char,
    len: usize,
    out_missing: *mut u32,
    out_remaining: *mut u32,
    out_produced: *mut u32,
    out_consumed: *mut u32,
) -> bool {
    if net.is_null() || activities.is_null() {
        return false;
    }
    let mut trace_events = Vec::new();
    for i in 0..len {
        let act_ptr = *activities.add(i);
        if act_ptr.is_null() {
            continue;
        }
        let act_str = CStr::from_ptr(act_ptr).to_string_lossy().into_owned();
        trace_events.push(Event {
            attributes: vec![Attribute {
                key: "concept:name".to_string(),
                value: AttributeValue::String(act_str),
            }],
        });
    }
    let trace = Trace { id: "ffi_trace".to_string(), attributes: Vec::new(), events: trace_events };
    let bm = NetBitmask64::from_petri_net(&*net);
    let res = replay_trace(&bm, &trace);
    if !out_missing.is_null() {
        *out_missing = res.missing;
    }
    if !out_remaining.is_null() {
        *out_remaining = res.remaining;
    }
    if !out_produced.is_null() {
        *out_produced = res.produced;
    }
    if !out_consumed.is_null() {
        *out_consumed = res.consumed;
    }
    res.is_perfect()
}

#[no_mangle]
pub unsafe extern "C" fn ref_petri_in_language(
    net: *mut PetriNet,
    activities: *const *const c_char,
    len: usize,
) -> bool {
    if net.is_null() || activities.is_null() {
        return false;
    }
    let mut trace_events = Vec::new();
    for i in 0..len {
        let act_ptr = *activities.add(i);
        if act_ptr.is_null() {
            continue;
        }
        let act_str = CStr::from_ptr(act_ptr).to_string_lossy().into_owned();
        trace_events.push(Event {
            attributes: vec![Attribute {
                key: "concept:name".to_string(),
                value: AttributeValue::String(act_str),
            }],
        });
    }
    let trace = Trace { id: "ffi_trace".to_string(), attributes: Vec::new(), events: trace_events };
    let bm = NetBitmask64::from_petri_net(&*net);
    in_language(&bm, &trace)
}

// ── YAWL FFI ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn ref_yawl_create() -> *mut BYawlEngine {
    Box::into_raw(Box::new(BYawlEngine::new()))
}

#[no_mangle]
pub unsafe extern "C" fn ref_yawl_free(engine: *mut BYawlEngine) {
    if !engine.is_null() {
        let _ = Box::from_raw(engine);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ref_yawl_execute_task(
    engine: *mut BYawlEngine,
    id: u16,
    join_type: u8,
    split_type: u8,
    consume_mask: u64,
    produce_mask: u64,
    cancellation_mask: u64,
    condition_mask: u64,
    reset_mask: u64,
    reachability_mask: u64,
    interleaved_lock_mask: u64,
    min_instances: u8,
    max_instances: u8,
    threshold_instances: u8,
    join_state_bit: u8,
    flags: u8,
) -> bool {
    if engine.is_null() {
        return false;
    }
    let join_t = match join_type {
        0 => JoinType::XOR,
        1 => JoinType::AND,
        2 => JoinType::OR,
        3 => JoinType::Complex,
        _ => JoinType::ThreadMerge,
    };
    let split_t = match split_type {
        0 => SplitType::XOR,
        1 => SplitType::AND,
        2 => SplitType::OR,
        3 => SplitType::MultiInstance,
        4 => SplitType::DynamicMultiInstance,
        5 => SplitType::InterleavedRouting,
        6 => SplitType::ThreadSplit,
        7 => SplitType::ImplicitTermination,
        8 => SplitType::ExplicitTermination,
        _ => SplitType::DeferredChoice,
    };

    let task = BYawlTask {
        id,
        join_type: join_t,
        split_type: split_t,
        min_instances,
        max_instances,
        threshold_instances,
        join_state_bit,
        flags,
        consume_mask,
        produce_mask,
        cancellation_mask,
        condition_mask,
        reset_mask,
        reachability_mask,
        interleaved_lock_mask,
    };

    (*engine).execute_task(&task)
}

// ── POWL FFI ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn ref_powl_program_create() -> *mut Powl64Program {
    Box::into_raw(Box::new(Powl64Program::default()))
}

#[no_mangle]
pub unsafe extern "C" fn ref_powl_program_free(prog: *mut Powl64Program) {
    if !prog.is_null() {
        let _ = Box::from_raw(prog);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ref_powl_execute(
    prog: *mut Powl64Program,
    watchdog_deadline: u64,
    out_concur_claims: *mut u32,
    out_scopes_entered: *mut u32,
) -> bool {
    if prog.is_null() {
        return false;
    }
    let watchdog = Watchdog::with_deadline(watchdog_deadline);
    let dispatcher = Dispatcher::new();
    let executor = Powl64Executor::new(&*prog);
    if let Ok(report) = executor.run(&watchdog, &dispatcher) {
        if !out_concur_claims.is_null() {
            *out_concur_claims = report.concur_slot_claims;
        }
        if !out_scopes_entered.is_null() {
            *out_scopes_entered = report.scopes_entered;
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_petri_lifecycle() {
        unsafe {
            let net = ref_petri_create();
            assert!(!net.is_null());

            let p0 = CStr::from_bytes_with_nul(b"p0\0").unwrap().as_ptr();
            let p1 = CStr::from_bytes_with_nul(b"p1\0").unwrap().as_ptr();
            let t0 = CStr::from_bytes_with_nul(b"t0\0").unwrap().as_ptr();
            let label = CStr::from_bytes_with_nul(b"a\0").unwrap().as_ptr();

            ref_petri_add_place(net, p0);
            ref_petri_add_place(net, p1);
            ref_petri_add_transition(net, t0, label, false);
            ref_petri_add_arc(net, p0, t0);
            ref_petri_add_arc(net, t0, p1);

            ref_petri_set_initial_marking(net, p0, 1);
            ref_petri_set_final_marking(net, p1, 1);

            let act = CStr::from_bytes_with_nul(b"a\0").unwrap().as_ptr();
            let acts = [act];

            let mut missing = 0u32;
            let mut remaining = 0u32;
            let mut produced = 0u32;
            let mut consumed = 0u32;

            let ok = ref_petri_replay_trace(
                net,
                acts.as_ptr(),
                1,
                &mut missing,
                &mut remaining,
                &mut produced,
                &mut consumed,
            );

            assert!(ok);
            assert_eq!(missing, 0);
            assert_eq!(remaining, 0);
            assert_eq!(produced, 2); // 1 initial + 1 produced by t0
            assert_eq!(consumed, 2); // 1 consumed by t0 + 1 final consumed

            ref_petri_free(net);
        }
    }
}
