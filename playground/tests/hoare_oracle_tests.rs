//! Hoare-logic oracle tests: property-based checks that `playground`'s
//! branchless Petri/POWL/YAWL primitives satisfy their stated pre/post-
//! condition invariants across randomized inputs, not just fixed examples.
#![allow(unsafe_code)]

use playground::{
    petri::{petri_fire_invisible, petri_fire_transition},
    powl::{powl64_execute_step, Powl64Op, Powl64OpKind, PowlState},
    yawl::{BYawlEngine, BYawlTask, JoinType, SplitType},
};
use proptest::prelude::*;

// --- Hoare Oracle: Formal Verification using Invariants ---

proptest! {
    // -------------------------------------------------------------------------
    // 1. Petri Net: `petri_fire_transition`
    // -------------------------------------------------------------------------
    #[test]
    fn hoare_petri_fire_transition_invariants(
        marking in any::<u64>(),
        in_mask in any::<u64>(),
        out_mask in any::<u64>(),
        missing_init in any::<u32>(),
        consumed_init in any::<u32>(),
        produced_init in any::<u32>(),
    ) {
        let mut marking_mut = marking;
        let mut missing = missing_init;
        let mut consumed = consumed_init;
        let mut produced = produced_init;

        // [PRE-CONDITION] Domain: All inputs are arbitrary u64 / u32

        petri_fire_transition(
            &mut marking_mut,
            in_mask,
            out_mask,
            &mut missing,
            &mut consumed,
            &mut produced,
        );

        // [POST-CONDITION]
        // 1. The output marking MUST contain all bits of out_mask
        assert_eq!(marking_mut & out_mask, out_mask, "Oracle Fault: Produced tokens not present in marking");

        // 2. The bits of the marking that are NOT in in_mask and NOT in out_mask MUST be unchanged
        let unaffected_mask = !(in_mask | out_mask);
        assert_eq!(marking_mut & unaffected_mask, marking & unaffected_mask, "Oracle Fault: Unrelated bits modified");

        // 3. The tokens in in_mask that are NOT in out_mask MUST be absent
        let consumed_not_produced = in_mask & !out_mask;
        assert_eq!(marking_mut & consumed_not_produced, 0, "Oracle Fault: Consumed tokens still present");

        // [INVARIANT] Additive monotonicity invariants
        let need = in_mask & !marking;
        assert_eq!(missing, missing_init.wrapping_add(need.count_ones()), "Oracle Fault: Missing count violated");
        assert_eq!(consumed, consumed_init.wrapping_add(in_mask.count_ones()), "Oracle Fault: Consumed count violated");
        assert_eq!(produced, produced_init.wrapping_add(out_mask.count_ones()), "Oracle Fault: Produced count violated");
    }

    // -------------------------------------------------------------------------
    // 2. Petri Net: `petri_fire_invisible`
    // -------------------------------------------------------------------------
    #[test]
    fn hoare_petri_fire_invisible_invariants(
        marking in any::<u64>(),
        inv_in_masks in prop::collection::vec(any::<u64>(), 0..16),
        inv_out_masks in prop::collection::vec(any::<u64>(), 0..16),
    ) {
        let mut marking_mut = marking;

        petri_fire_invisible(&mut marking_mut, &inv_in_masks, &inv_out_masks);

        // [INVARIANT] For every transition i, if marking had all in_masks[i] bits,
        // it must end up with out_masks[i] bits OR the state is non-terminal.
        // Due to fixed 16 iterations, we can at least assert that any newly set bits
        // must come from the union of inv_out_masks.

        let union_out = inv_out_masks.iter().fold(0, |acc, &x| acc | x);
        let union_in = inv_in_masks.iter().fold(0, |acc, &x| acc | x);

        // Bits that were added must be a subset of union_out
        let added_bits = marking_mut & !marking;
        assert_eq!(added_bits & !union_out, 0, "Oracle Fault: Synthesized bits out of thin air");

        // Bits that were removed must be a subset of union_in
        let removed_bits = marking & !marking_mut;
        assert_eq!(removed_bits & !union_in, 0, "Oracle Fault: Deleted bits without consuming");
    }

    // -------------------------------------------------------------------------
    // 3. YAWL Execution Engine: `execute_task_branchless`
    // -------------------------------------------------------------------------
    #[test]
    fn hoare_yawl_execute_task_invariants(
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
        let mut active_instances = [0u8; 64];
        active_instances.copy_from_slice(&active_instances_vec);

        let mut engine = BYawlEngine {
            state_mask,
            active_instances,
            active_triggers,
            fired_joins_mask,
            active_locks,
        };

        let task = BYawlTask {
            id: task_id,
            join_type: match join_type_idx {
                0 => JoinType::XOR,
                1 => JoinType::AND,
                2 => JoinType::OR,
                3 => JoinType::Complex,
                _ => JoinType::ThreadMerge,
            },
            split_type: match split_type_idx {
                0 => SplitType::XOR,
                1 => SplitType::AND,
                2 => SplitType::OR,
                3 => SplitType::MultiInstance,
                4 => SplitType::DynamicMultiInstance,
                5 => SplitType::DeferredChoice,
                6 => SplitType::InterleavedRouting,
                7 => SplitType::ThreadSplit,
                8 => SplitType::ImplicitTermination,
                _ => SplitType::ExplicitTermination,
            },
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

        let engine_pre = engine.clone();

        // Execute task
        let fired_mask = engine.execute_task_branchless(&task);

        // [POST-CONDITION] fired_mask is strictly 0 or !0
        assert!(fired_mask == 0 || fired_mask == 0xFFFF_FFFF_FFFF_FFFF, "Oracle Fault: Fired mask must be full-width boolean");

        if fired_mask == 0 {
            // [INVARIANT] If task didn't fire, state must remain UNCHANGED EXCEPT for complex join resets
            let is_release_mask = if (task.flags & 4) != 0 { !0u64 } else { 0u64 };
            let conflict_mask = if (engine_pre.active_locks & task.interleaved_lock_mask) != 0 { !0u64 } else { 0u64 };
            let allowed_by_lock_mask = (!conflict_mask) | is_release_mask;

            let has_reset = (engine_pre.state_mask & task.reset_mask) != 0;
            let has_reset_tokens_mask = if has_reset { !0u64 } else { 0u64 } & allowed_by_lock_mask;
            let reset_bit = 1u64.wrapping_shl(u32::from(task.join_state_bit) & 63);

            let expected_state = engine_pre.state_mask & !(task.reset_mask & has_reset_tokens_mask);
            let expected_fired_joins = engine_pre.fired_joins_mask & !(reset_bit & has_reset_tokens_mask);

            assert_eq!(engine.state_mask, expected_state, "Oracle Fault: Unexpected state change outside of allowed resets");
            assert_eq!(engine.active_triggers, engine_pre.active_triggers);
            assert_eq!(engine.fired_joins_mask, expected_fired_joins, "Oracle Fault: Unexpected fired joins change outside of allowed resets");
            assert_eq!(engine.active_locks, engine_pre.active_locks);
        } else {
            // [INVARIANT] If task fired, deterministic state transitions occurred
            // Consumed tokens must be cleared
            assert_eq!(engine.state_mask & task.consume_mask, 0, "Oracle Fault: Tokens not consumed");

            // Active triggers might be reset
            let expected_triggers = engine_pre.active_triggers & !task.reset_mask;
            assert_eq!(engine.active_triggers, expected_triggers, "Oracle Fault: Triggers not reset properly");

            // Fired joins mask must have the join bit set
            assert_ne!(engine.fired_joins_mask & (1u64 << task.join_state_bit), 0, "Oracle Fault: Join bit not set");
        }
    }

    // -------------------------------------------------------------------------
    // 4. POWL Op Execution: `powl64_execute_step`
    // -------------------------------------------------------------------------
    #[test]
    fn hoare_powl_execute_step_invariants(
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

        let state_pre = state;

        powl64_execute_step(&mut state, &op, input_choice, loop_repeat);

        // [POST-CONDITION] Depth cannot exceed 16 branchlessly. (No paired
        // "cannot underflow" check: `stack_depth`'s type is unsigned, so
        // that comparison is a compile-time tautology, not a real
        // post-condition — asserting it would be a vacuous check disguised
        // as verification.)
        assert!(state.stack_depth <= 16, "Oracle Fault: Stack depth exceeded bounds");

        // [INVARIANT] Monotonic progression in completed ops unless it's a loop exit repeat
        let is_loop_repeat = kind as u32 == Powl64OpKind::LoopGate as u32 && ctrl_mask == 0 && ((loop_repeat >> (loop_id & 63)) & 1) != 0;

        if is_loop_repeat {
            // Pred mask bits could be cleared
            let cleared_bits = state_pre.completed_ops & !state.completed_ops;
            // The bits cleared must be a subset of pred_mask
            assert_eq!(cleared_bits & !op.pred_mask, 0, "Oracle Fault: Unrelated ops cleared during loop back");
        } else {
            // No bits should be cleared
            let cleared_bits = state_pre.completed_ops & !state.completed_ops;
            assert_eq!(cleared_bits, 0, "Oracle Fault: Op completion state lost monotonically");
        }
    }
}
