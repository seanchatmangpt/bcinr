
//! Generated tactical-motif registry — manufactured from `ontology/chess.ttl`.
//!
//! First-class source (GGEN-SRC law): edit the ontology and re-run `ggen sync`.
//! Each sub-module is one branchless bitwise detector whose public `detect`
//! kernel is held to cyclomatic complexity 1 by the contract gate.

/// [0] hanging_detected — Mask lowering.
pub mod hanging;
/// [1] fork_detected — Bitset lowering.
pub mod fork;
/// [2] pin_detected — Network lowering.
pub mod pin;
/// [3] skewer_detected — Network lowering.
pub mod skewer;

pub use hanging::detect as detect_hanging;
pub use fork::detect as detect_fork;
pub use pin::detect as detect_pin;
pub use skewer::detect as detect_skewer;


use crate::position::PositionView;

/// Static metadata for one generated tactical motif.
#[derive(Debug, Clone, Copy)]
pub struct MotifSpec {
    /// Stable ordinal identifier (ORDER BY anchor).
    pub id: u16,
    /// snake_case motif name (matches the sub-module).
    pub name: &'static str,
    /// OCEL event code emitted when the motif fires.
    pub event_code: u16,
    /// OCEL event name.
    pub event_name: &'static str,
    /// Branchless detector kernel: `&PositionView -> fired square mask`.
    pub detect: fn(&PositionView) -> u64,
}

/// The full catalog of generated tactical motifs, ordered by id.
pub static MOTIF_REGISTRY: &[MotifSpec] = &[
    MotifSpec {
        id: 0,
        name: "hanging",
        event_code: 2000,
        event_name: "hanging_detected",
        detect: hanging::detect,
    },
    MotifSpec {
        id: 1,
        name: "fork",
        event_code: 2001,
        event_name: "fork_detected",
        detect: fork::detect,
    },
    MotifSpec {
        id: 2,
        name: "pin",
        event_code: 2002,
        event_name: "pin_detected",
        detect: pin::detect,
    },
    MotifSpec {
        id: 3,
        name: "skewer",
        event_code: 2003,
        event_name: "skewer_detected",
        detect: skewer::detect,
    },
];

/// Number of generated tactical motifs.
pub const MOTIF_COUNT: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_ordered_and_complete() {
        assert_eq!(MOTIF_REGISTRY.len(), MOTIF_COUNT);
        for (i, spec) in MOTIF_REGISTRY.iter().enumerate() {
            assert_eq!(spec.id as usize, i, "registry must be ORDER BY id");
        }
    }
}