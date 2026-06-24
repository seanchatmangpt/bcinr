
//! Generated OCEL event schema — manufactured from `ontology/chess.ttl`.
//!
//! First-class source (GGEN-SRC law): edit the ontology and re-run `ggen sync`.
//! Each [`OcelSchemaEdge`] asserts that a given evaluation cell emits an OCEL
//! activity of type `activity` linked to objects of kind `object_kind`. The
//! verifier validates `(cell, activity)` membership against this flat table at
//! zero heap cost.

/// One (cell, activity, object-kind) edge of the OCEL schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OcelSchemaEdge {
    /// Ordinal id of the emitting evaluation cell.
    pub cell_id: u16,
    /// snake_case cell name.
    pub cell_name: &'static str,
    /// OCEL activity (event) code.
    pub activity: u16,
    /// OCEL activity name.
    pub activity_name: &'static str,
    /// Object kind partition: "feature" or "tactical".
    pub object_kind: &'static str,
}

impl OcelSchemaEdge {
    /// Returns `true` if this edge matches the given activity code.
    #[must_use]
    #[inline(always)]
    pub fn matches(&self, activity: u16) -> bool {
        self.activity == activity
    }
}

/// The flat OCEL schema: every (cell, activity, object-kind) edge in the
/// catalog, ordered by activity code for deterministic iteration.
pub static OCEL_SCHEMA: &[OcelSchemaEdge] = &[
    // feature: material -> material_evaluated
    OcelSchemaEdge {
        cell_id: 0,
        cell_name: "material",
        activity: 1000,
        activity_name: "material_evaluated",
        object_kind: "feature",
    },
    // feature: pst -> pst_evaluated
    OcelSchemaEdge {
        cell_id: 1,
        cell_name: "pst",
        activity: 1001,
        activity_name: "pst_evaluated",
        object_kind: "feature",
    },
    // feature: mobility -> mobility_evaluated
    OcelSchemaEdge {
        cell_id: 2,
        cell_name: "mobility",
        activity: 1002,
        activity_name: "mobility_evaluated",
        object_kind: "feature",
    },
    // feature: king_safety -> king_safety_evaluated
    OcelSchemaEdge {
        cell_id: 3,
        cell_name: "king_safety",
        activity: 1003,
        activity_name: "king_safety_evaluated",
        object_kind: "feature",
    },
    // feature: pawn_structure -> pawn_structure_evaluated
    OcelSchemaEdge {
        cell_id: 4,
        cell_name: "pawn_structure",
        activity: 1004,
        activity_name: "pawn_structure_evaluated",
        object_kind: "feature",
    },
    // feature: center_control -> center_control_evaluated
    OcelSchemaEdge {
        cell_id: 5,
        cell_name: "center_control",
        activity: 1005,
        activity_name: "center_control_evaluated",
        object_kind: "feature",
    },
    // feature: passed_pawn -> passed_pawn_evaluated
    OcelSchemaEdge {
        cell_id: 6,
        cell_name: "passed_pawn",
        activity: 1006,
        activity_name: "passed_pawn_evaluated",
        object_kind: "feature",
    },
    // feature: rook_open_file -> rook_open_file_evaluated
    OcelSchemaEdge {
        cell_id: 7,
        cell_name: "rook_open_file",
        activity: 1007,
        activity_name: "rook_open_file_evaluated",
        object_kind: "feature",
    },
    // feature: bishop_pair -> bishop_pair_evaluated
    OcelSchemaEdge {
        cell_id: 8,
        cell_name: "bishop_pair",
        activity: 1008,
        activity_name: "bishop_pair_evaluated",
        object_kind: "feature",
    },
    // feature: king_tropism -> king_tropism_evaluated
    OcelSchemaEdge {
        cell_id: 9,
        cell_name: "king_tropism",
        activity: 1009,
        activity_name: "king_tropism_evaluated",
        object_kind: "feature",
    },
    // tactical: hanging -> hanging_detected
    OcelSchemaEdge {
        cell_id: 0,
        cell_name: "hanging",
        activity: 2000,
        activity_name: "hanging_detected",
        object_kind: "tactical",
    },
    // tactical: fork -> fork_detected
    OcelSchemaEdge {
        cell_id: 1,
        cell_name: "fork",
        activity: 2001,
        activity_name: "fork_detected",
        object_kind: "tactical",
    },
    // tactical: pin -> pin_detected
    OcelSchemaEdge {
        cell_id: 2,
        cell_name: "pin",
        activity: 2002,
        activity_name: "pin_detected",
        object_kind: "tactical",
    },
    // tactical: skewer -> skewer_detected
    OcelSchemaEdge {
        cell_id: 3,
        cell_name: "skewer",
        activity: 2003,
        activity_name: "skewer_detected",
        object_kind: "tactical",
    },
];

/// Number of OCEL schema edges.
pub const OCEL_EDGE_COUNT: usize = 14;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_ordered_and_complete() {
        assert_eq!(OCEL_SCHEMA.len(), OCEL_EDGE_COUNT);
        for w in OCEL_SCHEMA.windows(2) {
            assert!(w[0].activity < w[1].activity, "OCEL_SCHEMA must be ORDER BY activity");
        }
    }
}