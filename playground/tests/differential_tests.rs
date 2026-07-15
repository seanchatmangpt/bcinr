//! Differential tests: runs `playground`'s branchless Petri/POWL/YAWL
//! primitives side by side against the `reference` module's plain-Rust
//! reference implementations and asserts they agree.
#![allow(unsafe_code)]

mod reference;

use playground::{
    petri::{petri_fire_invisible, petri_fire_transition},
    powl::{powl64_execute_step, Powl64Op, Powl64OpKind, PowlState},
    wasm::{wasm_petri_replay, WasmReplayResult},
    yawl::{
        BYawlEngine as BranchlessBYawlEngine, BYawlTask as BranchlessBYawlTask,
        JoinType as BranchlessJoinType, SplitType as BranchlessSplitType,
    },
};
use proptest::prelude::*;

use crate::reference::{
    petri::{
        replay_trace, Arc, Attribute, AttributeValue, Event, Marking, NetBitmask64, PetriNet,
        Place, Trace, Transition,
    },
    yawl::{
        BYawlEngine as RefBYawlEngine, BYawlTask as RefBYawlTask, JoinType as RefJoinType,
        SplitType as RefSplitType,
    },
};

// --- Helper Mappers for YAWL ---

fn map_task(ref_task: &RefBYawlTask) -> BranchlessBYawlTask {
    BranchlessBYawlTask {
        id: ref_task.id,
        join_type: match ref_task.join_type {
            RefJoinType::XOR => BranchlessJoinType::XOR,
            RefJoinType::AND => BranchlessJoinType::AND,
            RefJoinType::OR => BranchlessJoinType::OR,
            RefJoinType::Complex => BranchlessJoinType::Complex,
            RefJoinType::ThreadMerge => BranchlessJoinType::ThreadMerge,
        },
        split_type: match ref_task.split_type {
            RefSplitType::XOR => BranchlessSplitType::XOR,
            RefSplitType::AND => BranchlessSplitType::AND,
            RefSplitType::OR => BranchlessSplitType::OR,
            RefSplitType::MultiInstance => BranchlessSplitType::MultiInstance,
            RefSplitType::DynamicMultiInstance => BranchlessSplitType::DynamicMultiInstance,
            RefSplitType::DeferredChoice => BranchlessSplitType::DeferredChoice,
            RefSplitType::InterleavedRouting => BranchlessSplitType::InterleavedRouting,
            RefSplitType::ThreadSplit => BranchlessSplitType::ThreadSplit,
            RefSplitType::ImplicitTermination => BranchlessSplitType::ImplicitTermination,
            RefSplitType::ExplicitTermination => BranchlessSplitType::ExplicitTermination,
        },
        min_instances: ref_task.min_instances,
        max_instances: ref_task.max_instances,
        threshold_instances: ref_task.threshold_instances,
        join_state_bit: ref_task.join_state_bit,
        flags: ref_task.flags,
        consume_mask: ref_task.consume_mask,
        produce_mask: ref_task.produce_mask,
        cancellation_mask: ref_task.cancellation_mask,
        condition_mask: ref_task.condition_mask,
        reset_mask: ref_task.reset_mask,
        reachability_mask: ref_task.reachability_mask,
        interleaved_lock_mask: ref_task.interleaved_lock_mask,
    }
}

fn map_engine(ref_engine: &RefBYawlEngine) -> BranchlessBYawlEngine {
    BranchlessBYawlEngine {
        state_mask: ref_engine.state_mask,
        active_instances: ref_engine.active_instances,
        active_triggers: ref_engine.active_triggers,
        fired_joins_mask: ref_engine.fired_joins_mask,
        active_locks: ref_engine.active_locks,
    }
}

// --- Proptests ---

proptest! {
    // 1. Petri Net Transition Firing Differential Test
    #[test]
    fn test_petri_fire_transition_differential(
        marking in any::<u64>(),
        in_mask in any::<u64>(),
        out_mask in any::<u64>(),
    ) {
        let mut marking_ref = marking;
        let mut missing_ref = 0u32;
        let mut consumed_ref = 0u32;
        let mut produced_ref = 0u32;

        let mut marking_branchless = marking;
        let mut missing_branchless = 0u32;
        let mut consumed_branchless = 0u32;
        let mut produced_branchless = 0u32;

        // Reference transition firing logic (re-implemented sequentially here)
        let need = in_mask & !marking_ref;
        missing_ref += need.count_ones();
        marking_ref |= need;
        marking_ref = (marking_ref & !in_mask) | out_mask;
        consumed_ref += in_mask.count_ones();
        produced_ref += out_mask.count_ones();

        // Branchless transition firing logic
        petri_fire_transition(
            &mut marking_branchless,
            in_mask,
            out_mask,
            &mut missing_branchless,
            &mut consumed_branchless,
            &mut produced_branchless,
        );

        assert_eq!(marking_branchless, marking_ref);
        assert_eq!(missing_branchless, missing_ref);
        assert_eq!(consumed_branchless, consumed_ref);
        assert_eq!(produced_branchless, produced_ref);
    }

    // 2. Petri Net Invisible Closure Differential Test
    #[test]
    fn test_petri_fire_invisible_differential(
        marking in any::<u64>(),
        inv_in_masks in prop::collection::vec(any::<u64>(), 0..16),
        inv_out_masks in prop::collection::vec(any::<u64>(), 0..16),
    ) {
        let min_len = inv_in_masks.len().min(inv_out_masks.len());
        let in_masks = &inv_in_masks[..min_len];
        let out_masks = &inv_out_masks[..min_len];

        let mut marking_ref = marking;
        let mut marking_branchless = marking;

        // Reference sequential fixed-point firing
        let mut changed = true;
        let mut limit = 0;
        while changed && limit < 16 {
            changed = false;
            for i in 0..min_len {
                let in_m = in_masks[i];
                let out_m = out_masks[i];
                if (marking_ref & in_m) == in_m {
                    marking_ref = (marking_ref & !in_m) | out_m;
                    changed = true;
                    break;
                }
            }
            limit += 1;
        }

        // Branchless closure
        petri_fire_invisible(&mut marking_branchless, in_masks, out_masks);

        // If the execution converges within 16 loops, outputs should match
        if limit < 16 {
            assert_eq!(marking_branchless, marking_ref);
        }
    }

    // 3. YAWL Execution Step Differential Test
    #[test]
    fn test_yawl_execute_task_differential(
        state_mask in any::<u64>(),
        active_triggers in any::<u64>(),
        fired_joins_mask in any::<u64>(),
        active_locks in any::<u64>(),
        active_instances_vec in prop::collection::vec(any::<u8>(), 64),

        task_id in any::<u16>(),
        join_type_idx in 0..5u8,
        split_type_idx in 0..10u8,
        min_instances in any::<u8>(),
        max_instances in any::<u8>(),
        threshold_instances in any::<u8>(),
        join_state_bit in 0..64u8,
        flags in any::<u8>(),
        consume_mask in any::<u64>(),
        produce_mask in any::<u64>(),
        cancellation_mask in any::<u64>(),
        condition_mask in any::<u64>(),
        reset_mask in any::<u64>(),
        reachability_mask in any::<u64>(),
        interleaved_lock_mask in any::<u64>(),
    ) {
        let ref_join = match join_type_idx {
            0 => RefJoinType::XOR,
            1 => RefJoinType::AND,
            2 => RefJoinType::OR,
            3 => RefJoinType::Complex,
            _ => RefJoinType::ThreadMerge,
        };
        let ref_split = match split_type_idx {
            0 => RefSplitType::XOR,
            1 => RefSplitType::AND,
            2 => RefSplitType::OR,
            3 => RefSplitType::MultiInstance,
            4 => RefSplitType::DynamicMultiInstance,
            5 => RefSplitType::DeferredChoice,
            6 => RefSplitType::InterleavedRouting,
            7 => RefSplitType::ThreadSplit,
            8 => RefSplitType::ImplicitTermination,
            _ => RefSplitType::ExplicitTermination,
        };

        let ref_task = RefBYawlTask {
            id: task_id,
            join_type: ref_join,
            split_type: ref_split,
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

        let mut active_instances = [0u8; 64];
        active_instances.copy_from_slice(&active_instances_vec);

        let mut ref_engine = RefBYawlEngine {
            state_mask,
            active_instances,
            active_triggers,
            fired_joins_mask,
            active_locks,
        };

        let mut branchless_engine = map_engine(&ref_engine);
        let branchless_task = map_task(&ref_task);

        let ref_fired = ref_engine.execute_task(&ref_task);
        let branchless_fired_mask = branchless_engine.execute_task_branchless(&branchless_task);
        let branchless_fired = branchless_fired_mask == 0xFFFF_FFFF_FFFF_FFFF;

        assert_eq!(branchless_fired, ref_fired);
        assert_eq!(branchless_engine.state_mask, ref_engine.state_mask);
        assert_eq!(branchless_engine.active_triggers, ref_engine.active_triggers);
        assert_eq!(branchless_engine.fired_joins_mask, ref_engine.fired_joins_mask);
        assert_eq!(branchless_engine.active_locks, ref_engine.active_locks);
        assert_eq!(branchless_engine.active_instances, ref_engine.active_instances);
    }

    // 4. POWL Op Execution Property-Based Test
    #[test]
    fn test_powl64_execute_step_properties(
        completed_ops in any::<u64>(),
        completed_branches in any::<u64>(),
        active_scopes in any::<u64>(),
        scope_stack in prop::array::uniform16(any::<u16>()),
        stack_depth in 1..16u32,
        completed_loops in any::<u64>(),

        op_kind_idx in 0..9u8,
        lane in any::<u8>(),
        activity in any::<u16>(),
        scope in 0..16u16,
        branch in 0..64u16,
        loop_id in 0..64u16,
        pred_mask in any::<u64>(),
        succ_mask in any::<u64>(),
        ctrl_mask in any::<u64>(),
        intensity in any::<u8>(),

        input_choice in any::<u64>(),
        loop_repeat in any::<u64>(),
    ) {
        let kind = match op_kind_idx {
            0 => Powl64OpKind::Activity,
            1 => Powl64OpKind::PartialOrderGate,
            2 => Powl64OpKind::ChoiceGate,
            3 => Powl64OpKind::LoopGate,
            4 => Powl64OpKind::EnterScope,
            5 => Powl64OpKind::ExitScope,
            6 => Powl64OpKind::Promote,
            7 => Powl64OpKind::Demote,
            _ => Powl64OpKind::Watchdog,
        };

        let op = Powl64Op {
            kind,
            lane,
            activity,
            scope,
            branch,
            loop_id,
            pred_mask,
            succ_mask,
            ctrl_mask,
            intensity,
            _pad: [0; 7],
        };

        let mut state = PowlState {
            completed_ops,
            completed_branches,
            active_scopes,
            scope_stack,
            stack_depth,
            completed_loops,
        };

        // Execute branchless step
        powl64_execute_step(&mut state, &op, input_choice, loop_repeat);

        // Determine enablement conditions branchingly
        let is_enter = op.kind as u32 == Powl64OpKind::EnterScope as u32;
        let parent_scope = scope_stack[(stack_depth.wrapping_sub(1) & 15) as usize];
        let scope_to_check = if is_enter { parent_scope } else { op.scope };
        let scope_bit = (active_scopes >> (scope_to_check & 63)) & 1;
        let is_scope_active = scope_bit == 1;

        let diff = (completed_ops & op.pred_mask) ^ op.pred_mask;
        let is_preds_completed = diff == 0;
        let exec = is_scope_active && is_preds_completed;

        if !exec {
            // State fields should remain unchanged if not executable
            assert_eq!(state.completed_ops, completed_ops);
            assert_eq!(state.completed_branches, completed_branches);
            assert_eq!(state.active_scopes, active_scopes);
            assert_eq!(state.stack_depth, stack_depth);
            assert_eq!(state.completed_loops, completed_loops);
        } else {
            match kind {
                Powl64OpKind::Activity | Powl64OpKind::PartialOrderGate | Powl64OpKind::Promote | Powl64OpKind::Demote | Powl64OpKind::Watchdog => {
                    assert_eq!(state.completed_ops, completed_ops | succ_mask);
                }
                Powl64OpKind::EnterScope => {
                    assert_eq!(state.active_scopes, active_scopes | (1u64 << (scope & 63)));
                    assert_eq!(state.stack_depth, stack_depth + 1);
                    assert_eq!(state.scope_stack[stack_depth as usize], scope);
                }
                Powl64OpKind::ExitScope => {
                    assert_eq!(state.active_scopes, active_scopes & !(1u64 << (scope & 63)));
                    assert_eq!(state.stack_depth, stack_depth - 1);
                }
                Powl64OpKind::ChoiceGate => {
                    let self_bit = 1u64 << (branch & 63);
                    let is_start = (ctrl_mask & self_bit) != 0;
                    let has_pred = (completed_branches & ctrl_mask) != 0;
                    let is_choice_enabled = is_start || has_pred;
                    let choice_selected = ((input_choice >> (branch & 63)) & 1) != 0;
                    let fires_and_chosen = is_choice_enabled && choice_selected;

                    if fires_and_chosen {
                        assert_eq!(state.completed_ops, completed_ops);
                        assert_eq!(state.completed_branches, completed_branches | self_bit);
                    } else {
                        assert_eq!(state.completed_ops, completed_ops | succ_mask);
                        assert_eq!(state.completed_branches, completed_branches);
                    }
                }
                Powl64OpKind::LoopGate => {
                    let loop_bit = 1u64 << (loop_id & 63);
                    let is_enter = ctrl_mask != 0;
                    let should_repeat = ((loop_repeat >> (loop_id & 63)) & 1) != 0;

                    if is_enter {
                        if !should_repeat {
                            assert_eq!(state.completed_loops, completed_loops | loop_bit);
                        } else {
                            assert_eq!(state.completed_loops, completed_loops);
                        }
                        assert_eq!(state.completed_ops, completed_ops);
                    } else {
                        // is exit
                        if should_repeat {
                            assert_eq!(state.completed_ops, completed_ops & !pred_mask);
                            assert_eq!(state.completed_loops, completed_loops & !loop_bit);
                        } else {
                            assert_eq!(state.completed_ops, completed_ops);
                            assert_eq!(state.completed_loops, completed_loops | loop_bit);
                        }
                    }
                }
            }
        }
    }
}

// --- E2E / FFI WASM Differential tests ---

#[test]
fn test_wasm_petri_replay_e2e_differential() {
    // Construct a standard workflow net
    let places = vec![Place::new("p0"), Place::new("p1"), Place::new("p2")];
    let transitions = vec![Transition::new("t0", "A"), Transition::new("t1", "B")];
    let arcs = vec![
        Arc::place_to_transition("p0", "t0"),
        Arc::transition_to_place("t0", "p1"),
        Arc::place_to_transition("p1", "t1"),
        Arc::transition_to_place("t1", "p2"),
    ];
    let initial_marking = Marking::new([("p0".to_string(), 1)]);
    let mut net = PetriNet::new(places, transitions, arcs, initial_marking);
    net.final_marking = vec![("p2".to_string(), 1)];

    let bitmask_net = NetBitmask64::from_petri_net(&net);

    // Reference Replay
    let mut trace_events = Vec::new();
    trace_events.push(Event {
        attributes: vec![Attribute {
            key: "concept:name".to_string(),
            value: AttributeValue::String("A".to_string()),
        }],
    });
    trace_events.push(Event {
        attributes: vec![Attribute {
            key: "concept:name".to_string(),
            value: AttributeValue::String("B".to_string()),
        }],
    });
    let ref_trace = Trace { id: "case1".to_string(), attributes: vec![], events: trace_events };
    let ref_result = replay_trace(&bitmask_net, &ref_trace);

    // WASM API Replay
    let in_masks = vec![1u64, 2u64];
    let out_masks = vec![2u64, 4u64];
    let trace_indices = vec![0u32, 1u32];

    let mut branchless_result =
        WasmReplayResult { missing: 99, remaining: 99, produced: 99, consumed: 99 };

    let rc = unsafe {
        wasm_petri_replay(
            bitmask_net.initial_mask,
            bitmask_net.final_mask,
            in_masks.as_ptr(),
            out_masks.as_ptr(),
            2,
            trace_indices.as_ptr(),
            2,
            &mut branchless_result,
        )
    };

    assert_eq!(rc, 0);
    assert_eq!(branchless_result.missing, ref_result.missing);
    assert_eq!(branchless_result.remaining, ref_result.remaining);
    assert_eq!(branchless_result.produced, ref_result.produced);
    assert_eq!(branchless_result.consumed, ref_result.consumed);
}
