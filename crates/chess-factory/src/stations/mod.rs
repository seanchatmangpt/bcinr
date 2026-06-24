
//! Generated feature-station registry — manufactured from `ontology/chess.ttl`.
//!
//! First-class source (GGEN-SRC law): edit the ontology and re-run `ggen sync`;
//! never hand-edit. Each sub-module is one branchless feature station whose
//! public `evaluate` kernel is held to cyclomatic complexity 1 by the contract
//! gate and proptest-verified against a branchful oracle.

use crate::station::StationResult;
use crate::position::PositionView;

/// [0] material_evaluated — Saturating lowering.
pub mod material;
/// [1] pst_evaluated — Lut lowering.
pub mod pst;
/// [2] mobility_evaluated — Bitset lowering.
pub mod mobility;
/// [3] king_safety_evaluated — Mask lowering.
pub mod king_safety;
/// [4] pawn_structure_evaluated — Bitset lowering.
pub mod pawn_structure;
/// [5] center_control_evaluated — Mask lowering.
pub mod center_control;

pub use material::evaluate as evaluate_material;
pub use pst::evaluate as evaluate_pst;
pub use mobility::evaluate as evaluate_mobility;
pub use king_safety::evaluate as evaluate_king_safety;
pub use pawn_structure::evaluate as evaluate_pawn_structure;
pub use center_control::evaluate as evaluate_center_control;


/// Static metadata for one generated feature station.
#[derive(Debug, Clone, Copy)]
pub struct StationSpec {
    /// Stable ordinal identifier (ORDER BY anchor).
    pub id: u16,
    /// snake_case station name (matches the sub-module).
    pub name: &'static str,
    /// OCEL event code emitted when the station fires.
    pub event_code: u16,
    /// OCEL event name.
    pub event_name: &'static str,
    /// Centipawn scale applied to the raw differential.
    pub score_scale: i32,
    /// Q8.8 aggregation weight (256 == 1.0).
    pub weight_q8: i32,
    /// Branchless evaluation kernel.
    pub evaluate: fn(&PositionView) -> StationResult,
}

/// The full catalog of generated feature stations, ordered by id.
pub static STATION_REGISTRY: &[StationSpec] = &[
    StationSpec {
        id: 0,
        name: "material",
        event_code: 1000,
        event_name: "material_evaluated",
        score_scale: 1,
        weight_q8: 256,
        evaluate: material::evaluate,
    },
    StationSpec {
        id: 1,
        name: "pst",
        event_code: 1001,
        event_name: "pst_evaluated",
        score_scale: 1,
        weight_q8: 256,
        evaluate: pst::evaluate,
    },
    StationSpec {
        id: 2,
        name: "mobility",
        event_code: 1002,
        event_name: "mobility_evaluated",
        score_scale: 6,
        weight_q8: 256,
        evaluate: mobility::evaluate,
    },
    StationSpec {
        id: 3,
        name: "king_safety",
        event_code: 1003,
        event_name: "king_safety_evaluated",
        score_scale: 20,
        weight_q8: 256,
        evaluate: king_safety::evaluate,
    },
    StationSpec {
        id: 4,
        name: "pawn_structure",
        event_code: 1004,
        event_name: "pawn_structure_evaluated",
        score_scale: 15,
        weight_q8: 256,
        evaluate: pawn_structure::evaluate,
    },
    StationSpec {
        id: 5,
        name: "center_control",
        event_code: 1005,
        event_name: "center_control_evaluated",
        score_scale: 10,
        weight_q8: 256,
        evaluate: center_control::evaluate,
    },
];

/// Number of generated feature stations.
pub const STATION_COUNT: usize = 6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_ordered_and_complete() {
        assert_eq!(STATION_REGISTRY.len(), STATION_COUNT);
        for (i, spec) in STATION_REGISTRY.iter().enumerate() {
            assert_eq!(spec.id as usize, i, "registry must be ORDER BY id");
        }
    }
}