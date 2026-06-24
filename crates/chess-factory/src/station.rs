//! Station boundary contract (HAND-AUTHORED).
//!
//! A `Station` consumes a packed [`PositionView`] and emits a white-relative
//! centipawn [`Score`] plus structured [`Evidence`] for receipts. Station
//! kernels are branchless (CC=1); only the `PositionView` builder may loop.

use crate::position::PositionView;

/// White-relative score in centipawns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Score {
    /// Score in centipawns, positive = good for White.
    pub cp: i32,
}

/// Structured evidence emitted by a station, consumed by receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Evidence {
    /// Stable ordinal identifier of the emitting station.
    pub station_id: u16,
    /// Bitmask of squares/features that fired for this station.
    pub fired_mask: u64,
    /// Raw, pre-weight centipawn contribution.
    pub raw_cp: i32,
    /// Q8.8 fixed-point weight applied during aggregation.
    pub weight_q8: i32,
}

/// Combined output of a single station evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StationResult {
    /// Weighted, white-relative score in centipawns.
    pub score_cp: i32,
    /// Evidence for receipts and conformance replay.
    pub evidence: Evidence,
}

/// A branchless evaluation cell over a packed position.
pub trait Station {
    /// Evaluate the position. MUST be branchless (CC=1).
    fn evaluate(v: &PositionView) -> StationResult;
}
