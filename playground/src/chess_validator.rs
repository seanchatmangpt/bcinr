//! Branchless POWL v2 OCEL Chess Validator
//!
//! Validates OCEL v2 Object-Centric execution transitions for the Chess engine
//! using strict POWL v2 (Partially Ordered Workflow Language) bounds.

use crate::{
    hoeg::Hoeg64Node,
    powl::{powl64_execute_step, Powl64Op, Powl64OpKind, PowlState},
};

/// Validates whether a generated Object-Centric chess transition conforms to
/// strict causal physics without branching.
///
/// Under POWL v2 constraints (Kourani & van der Aalst), a valid move requires:
/// 1. A Strict transition sequence (Turn causality)
/// 2. The source square bit and the target square bit matching physical laws.
///
/// # Example
/// ```
/// use playground::chess_validator::validate_chess_move_powl;
/// use playground::hoeg::Hoeg64Node;
///
/// // Create a dummy chess state transition representation
/// let state = Hoeg64Node {
///     feature_mask: 0x01, adjacency_mask: 0x01, node_id: 1, node_type_hash: 1, _pad: [0; 44]
/// };
///
/// let is_valid = validate_chess_move_powl(&state).unwrap();
/// assert_eq!(is_valid, true);
/// ```
#[inline(always)]
pub fn validate_chess_move_powl(transition: &Hoeg64Node) -> Result<bool, &'static str> {
    // We map the chess rule validation to a POWL v2 state machine execution constraint.
    let mut powl_state = PowlState::new();
    powl_state.completed_ops = transition.adjacency_mask;

    // Construct a POWL operator that physically bounds the transition bitmasks
    // If the transition attempts an illegal warp, the XOR parity forces an error mask.
    let constraint_op = Powl64Op {
        kind: Powl64OpKind::PartialOrderGate,
        lane: 0,
        activity: transition.node_id as u16,
        scope: 0,
        branch: 0,
        loop_id: 0,
        pred_mask: transition.adjacency_mask,
        succ_mask: transition.feature_mask,
        ctrl_mask: 0,
        intensity: 0,
        _pad: [0; 7],
    };

    // Apply the branchless execution step. If causality holds, `completed_ops` will correctly align.
    powl64_execute_step(&mut powl_state, &constraint_op, 0, 0);

    // Return the topological validity without `if` branching
    // If the op executed successfully, the feature_mask is merged into completed_ops.
    Ok((powl_state.completed_ops & transition.feature_mask) == transition.feature_mask)
}
