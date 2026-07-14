//! Branchless POWL ontology matrix compiler and executor.
//!
//! Fully branchless (CC = 1), zero-allocation, and `#![no_std]` compliant.

use bcinr::mask::{select_u32, select_u64};

/// A single compiled POWL64 opcode.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Powl64OpKind {
    /// An activity execution (AEF at a lane).
    Activity = 0,
    /// A partial-order join/activation gate.
    PartialOrderGate = 1,
    /// A choice-graph gate.
    ChoiceGate = 2,
    /// A loop entry/exit gate.
    LoopGate = 3,
    /// Enter a nested scope.
    EnterScope = 4,
    /// Exit the current scope.
    ExitScope = 5,
    /// A tier promotion.
    Promote = 6,
    /// A tier demotion.
    Demote = 7,
    /// Watchdog tick point.
    Watchdog = 8,
}

/// One compiled op in the flat program.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Powl64Op {
    /// What this op does.
    pub kind: Powl64OpKind,
    /// Which field lane (only meaningful for `Activity`).
    pub lane: u8,
    /// Dense activity id (only meaningful for `Activity`).
    pub activity: u16,
    /// Scope this op belongs to.
    pub scope: u16,
    /// Branch id (only meaningful for `ChoiceGate`).
    pub branch: u16,
    /// Loop id (only meaningful for `LoopGate`).
    pub loop_id: u16,
    /// Predecessor completion mask (bits cleared as predecessors finish).
    pub pred_mask: u64,
    /// Successor activation mask (bits set as this op completes).
    pub succ_mask: u64,
    /// Branch/loop control mask.
    pub ctrl_mask: u64,
    /// Intensity for activities.
    pub intensity: u8,
    pub _pad: [u8; 7],
}

const _: () = assert!(core::mem::align_of::<Powl64Op>() == 64);
const _: () = assert!(core::mem::size_of::<Powl64Op>() == 64);

/// Represents the execution state of the POWL engine.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowlState {
    /// Completed operations mask.
    pub completed_ops: u64,
    /// Completed branches mask.
    pub completed_branches: u64,
    /// Active scopes mask.
    pub active_scopes: u64,
    /// Stack of active scopes.
    pub scope_stack: [u16; 16],
    /// Current depth of the scope stack.
    pub stack_depth: u32,
    /// Completed loops mask.
    pub completed_loops: u64,
}

impl PowlState {
    /// Creates a new `PowlState` with scope 0 active.
    pub fn new() -> Self {
        let mut scope_stack = [0u16; 16];
        scope_stack[0] = 0;
        Self {
            completed_ops: 0,
            completed_branches: 0,
            active_scopes: 1, // Scope 0 is active initially
            scope_stack,
            stack_depth: 1,
            completed_loops: 0,
        }
    }
}

impl Default for PowlState {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes one compiled operation step completely branchlessly (CC = 1).
#[inline(always)]
pub fn powl64_execute_step(
    state: &mut PowlState,
    op: &Powl64Op,
    input_choice: u64,
    loop_repeat: u64,
) {
    let kind_val = op.kind as u64;

    // Bitmask categories using SWAR (1 << kind_val)
    let is_enter_scope_mask = 0u64.wrapping_sub((16u64 >> kind_val) & 1);
    let is_exit_scope_mask = 0u64.wrapping_sub((32u64 >> kind_val) & 1);
    let is_act_or_po_mask = 0u64.wrapping_sub((451u64 >> kind_val) & 1);
    let is_choice_mask = 0u64.wrapping_sub((4u64 >> kind_val) & 1);
    let is_loop_mask = 0u64.wrapping_sub((8u64 >> kind_val) & 1);

    // 1. Determine if this op's scope is active.
    let parent_scope = state.scope_stack[(state.stack_depth.wrapping_sub(1) & 15) as usize] as u64;
    let scope_to_check = select_u64(is_enter_scope_mask, parent_scope, op.scope as u64);

    let scope_bit = (state.active_scopes >> (scope_to_check & 63)) & 1;
    let is_scope_active_mask = 0u64.wrapping_sub(scope_bit);

    // 2. Determine if predecessors are completed.
    let diff = (!state.completed_ops) & op.pred_mask;
    let diff_non_zero_msb = (diff | diff.wrapping_neg()) >> 63;
    let is_preds_completed_mask = diff_non_zero_msb.wrapping_sub(1);

    // 3. Execution mask: active if both scope is active and predecessors are completed.
    let exec_mask = is_scope_active_mask & is_preds_completed_mask;

    // Prepare candidate values for state fields.

    // --- completed_ops update ---
    let next_ops_succ = state.completed_ops | op.succ_mask;

    // For ChoiceGate:
    let self_bit = 1u64 << (op.branch & 63);
    let start_val = op.ctrl_mask & self_bit;
    let is_start_mask = 0u64.wrapping_sub((start_val | start_val.wrapping_neg()) >> 63);
    let has_pred_val = state.completed_branches & op.ctrl_mask;
    let has_pred_mask = 0u64.wrapping_sub((has_pred_val | has_pred_val.wrapping_neg()) >> 63);

    let is_choice_enabled_mask = is_start_mask | has_pred_mask;
    let choice_selected_mask = 0u64.wrapping_sub((input_choice >> (op.branch & 63)) & 1);
    let fires_and_chosen_mask = is_choice_enabled_mask & choice_selected_mask;

    let choice_ops = state.completed_ops | (op.succ_mask & !fires_and_chosen_mask);

    // For LoopGate:
    let is_exit_loop_mask =
        0u64.wrapping_sub(((op.ctrl_mask | op.ctrl_mask.wrapping_neg()) >> 63) ^ 1);
    let should_repeat_mask = 0u64.wrapping_sub((loop_repeat >> (op.loop_id & 63)) & 1);
    let exit_and_repeat_mask = is_exit_loop_mask & should_repeat_mask;
    let loop_ops =
        select_u64(exit_and_repeat_mask, state.completed_ops & !op.pred_mask, state.completed_ops);

    // Select the new completed_ops candidate based on op.kind
    let ops_1 = select_u64(is_act_or_po_mask, next_ops_succ, state.completed_ops);
    let ops_2 = select_u64(is_choice_mask, choice_ops, ops_1);
    let next_completed_ops_raw = select_u64(is_loop_mask, loop_ops, ops_2);

    state.completed_ops = select_u64(exec_mask, next_completed_ops_raw, state.completed_ops);

    // --- completed_branches update ---
    let choice_branches = state.completed_branches | (self_bit & fires_and_chosen_mask);
    let next_completed_branches_raw =
        select_u64(is_choice_mask, choice_branches, state.completed_branches);
    state.completed_branches =
        select_u64(exec_mask, next_completed_branches_raw, state.completed_branches);

    // --- completed_loops update ---
    let loop_bit = 1u64 << (op.loop_id & 63);
    let is_enter_loop_mask = 0u64.wrapping_sub((op.ctrl_mask | op.ctrl_mask.wrapping_neg()) >> 63);

    // Enter loop gate: if not repeat, mark loop completed.
    let enter_loop_loops =
        select_u64(!should_repeat_mask, state.completed_loops | loop_bit, state.completed_loops);
    // Exit loop gate: if repeat, clear loop completed status, else set loop completed status.
    let exit_loop_loops = select_u64(
        should_repeat_mask,
        state.completed_loops & !loop_bit,
        state.completed_loops | loop_bit,
    );

    let loop_loops = select_u64(is_enter_loop_mask, enter_loop_loops, exit_loop_loops);
    let next_completed_loops_raw = select_u64(is_loop_mask, loop_loops, state.completed_loops);
    state.completed_loops = select_u64(exec_mask, next_completed_loops_raw, state.completed_loops);

    // --- active_scopes, scope_stack, and stack_depth update ---
    let enter_scopes = state.active_scopes | (1u64 << (op.scope & 63));
    let exit_scopes = state.active_scopes & !(1u64 << (op.scope & 63));

    let scopes_1 = select_u64(is_enter_scope_mask, enter_scopes, state.active_scopes);
    let next_active_scopes_raw = select_u64(is_exit_scope_mask, exit_scopes, scopes_1);
    state.active_scopes = select_u64(exec_mask, next_active_scopes_raw, state.active_scopes);

    // Write scope stack
    let idx = state.stack_depth as usize & 15;
    state.scope_stack[idx] = select_u32(
        (exec_mask & is_enter_scope_mask) as u32,
        op.scope as u32,
        state.scope_stack[idx] as u32,
    ) as u16;

    let depth_change = select_u32(
        exec_mask as u32,
        select_u32(
            is_enter_scope_mask as u32,
            1,
            select_u32(is_exit_scope_mask as u32, 0xFFFF_FFFF, 0),
        ),
        0,
    ) as u32;
    state.stack_depth = state.stack_depth.wrapping_add(depth_change);
}

use wasm4pm_compat::powl::{Powl, PowlNodeKind};

/// Compiles a strictly verified POWL 2.0 AST into a flat branchless execution array.
/// Adheres strictly to `#![no_std]` and zero-allocation boundaries.
pub fn compile_powl_to_swar(powl: &Powl, out: &mut [Powl64Op]) -> Result<usize, &'static str> {
    let mut op_count = 0;

    for node in &powl.nodes {
        if op_count >= out.len() {
            return Err("Output buffer too small");
        }

        match &node.kind {
            PowlNodeKind::Atom(_) => {
                out[op_count] = Powl64Op {
                    kind: Powl64OpKind::Activity,
                    lane: 0,
                    activity: node.id.0 as u16,
                    scope: 0,
                    branch: 0,
                    loop_id: 0,
                    pred_mask: 0, // Inferred dynamically from PO edges if needed
                    succ_mask: 1 << (node.id.0 & 63),
                    ctrl_mask: 0,
                    intensity: 1,
                    _pad: [0; 7],
                };
                op_count += 1;
            }
            PowlNodeKind::Silent => {
                // Silent transitions compiled out in execution layer
            }
            PowlNodeKind::PartialOrder(_) => {
                // Emits PartialOrderGates based on global precedence edges
                for edge in &powl.edges {
                    if op_count >= out.len() {
                        return Err("Output buffer too small");
                    }
                    out[op_count] = Powl64Op {
                        kind: Powl64OpKind::PartialOrderGate,
                        lane: 0,
                        activity: 0,
                        scope: 0,
                        branch: 0,
                        loop_id: 0,
                        pred_mask: 1 << (edge.from.0 & 63),
                        succ_mask: 1 << (edge.to.0 & 63),
                        ctrl_mask: 0,
                        intensity: 0,
                        _pad: [0; 7],
                    };
                    op_count += 1;
                }
            }
            PowlNodeKind::Start | PowlNodeKind::End => {
                // Start/End sentinel nodes compile to no-ops
            }
            PowlNodeKind::ChoiceGraph { nodes: _, edges: cg_edges } => {
                // Flatten unified choice/cyclic topology directly into branchless jump masks!
                for edge in cg_edges {
                    if op_count >= out.len() {
                        return Err("Output buffer too small");
                    }

                    let is_back_edge = edge.from.0 >= edge.to.0;

                    if is_back_edge {
                        out[op_count] = Powl64Op {
                            kind: Powl64OpKind::LoopGate,
                            lane: 0,
                            activity: 0,
                            scope: 0,
                            branch: 0,
                            loop_id: edge.to.0 as u16,
                            pred_mask: 1 << (edge.from.0 & 63),
                            succ_mask: 1 << (edge.to.0 & 63),
                            ctrl_mask: 1 << 63, // Loop re-entry mask
                            intensity: 0,
                            _pad: [0; 7],
                        };
                    } else {
                        out[op_count] = Powl64Op {
                            kind: Powl64OpKind::ChoiceGate,
                            lane: 0,
                            activity: 0,
                            scope: 0,
                            branch: edge.to.0 as u16,
                            loop_id: 0,
                            pred_mask: 1 << (edge.from.0 & 63),
                            succ_mask: 1 << (edge.to.0 & 63),
                            ctrl_mask: 1 << (edge.from.0 & 63), // Guard mask
                            intensity: 0,
                            _pad: [0; 7],
                        };
                    }
                    op_count += 1;
                }
            }
        }
    }

    Ok(op_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_activity_step() {
        let mut state = PowlState::new();
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

        powl64_execute_step(&mut state, &op, 0, 0);
        assert_eq!(state.completed_ops, 1 << 5);
    }

    #[test]
    fn test_execute_partial_order_gate() {
        let mut state = PowlState::new();
        state.completed_ops = 1 << 3;

        let op = Powl64Op {
            kind: Powl64OpKind::PartialOrderGate,
            lane: 0,
            activity: 0,
            scope: 0,
            branch: 0,
            loop_id: 0,
            pred_mask: 1 << 3,
            succ_mask: 1 << 4,
            ctrl_mask: 0,
            intensity: 0,
            _pad: [0; 7],
        };

        powl64_execute_step(&mut state, &op, 0, 0);
        assert_eq!(state.completed_ops, (1 << 3) | (1 << 4));
    }

    #[test]
    fn test_execute_partial_order_gate_blocked() {
        let mut state = PowlState::new();
        state.completed_ops = 0; // Predecessor not completed

        let op = Powl64Op {
            kind: Powl64OpKind::PartialOrderGate,
            lane: 0,
            activity: 0,
            scope: 0,
            branch: 0,
            loop_id: 0,
            pred_mask: 1 << 3,
            succ_mask: 1 << 4,
            ctrl_mask: 0,
            intensity: 0,
            _pad: [0; 7],
        };

        powl64_execute_step(&mut state, &op, 0, 0);
        assert_eq!(state.completed_ops, 0); // Remain blocked
    }

    #[test]
    fn test_execute_enter_exit_scope() {
        let mut state = PowlState::new();
        assert_eq!(state.active_scopes, 1);
        assert_eq!(state.stack_depth, 1);
        assert_eq!(state.scope_stack[0], 0);

        let enter_op = Powl64Op {
            kind: Powl64OpKind::EnterScope,
            lane: 0,
            activity: 0,
            scope: 2,
            branch: 0,
            loop_id: 0,
            pred_mask: 0,
            succ_mask: 0,
            ctrl_mask: 0,
            intensity: 0,
            _pad: [0; 7],
        };

        powl64_execute_step(&mut state, &enter_op, 0, 0);
        assert_eq!(state.active_scopes, 1 | (1 << 2));
        assert_eq!(state.stack_depth, 2);
        assert_eq!(state.scope_stack[1], 2);

        let exit_op = Powl64Op {
            kind: Powl64OpKind::ExitScope,
            lane: 0,
            activity: 0,
            scope: 2,
            branch: 0,
            loop_id: 0,
            pred_mask: 0,
            succ_mask: 0,
            ctrl_mask: 0,
            intensity: 0,
            _pad: [0; 7],
        };

        powl64_execute_step(&mut state, &exit_op, 0, 0);
        assert_eq!(state.active_scopes, 1);
        assert_eq!(state.stack_depth, 1);
    }
}
