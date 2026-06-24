
//! Generated Q8.8 station weight table — manufactured from `ontology/chess.ttl`.
//!
//! First-class source (GGEN-SRC law): edit `cf:weight_q8` in the ontology and
//! re-run `ggen sync`. Each entry is a fixed-point weight where 256 == 1.0; the
//! aggregator applies `(raw_cp * STATION_WEIGHTS_Q8[id]) >> 8` branchlessly.

/// Q8.8 fixed-point weight that represents 1.0.
pub const Q8_ONE: i32 = 256;

/// Per-station Q8.8 aggregation weights, indexed by station id.
///
/// Generated ORDER BY id, so `STATION_WEIGHTS_Q8[station_id]` is the weight for
/// that station. 256 == 1.0.
pub static STATION_WEIGHTS_Q8: &[i32] = &[
    256, // [0] material
    256, // [1] pst
    256, // [2] mobility
    256, // [3] king_safety
    256, // [4] pawn_structure
    256, // [5] center_control
];

/// Number of weighted stations.
pub const WEIGHT_COUNT: usize = 6;

/// Apply a Q8.8 weight to a raw centipawn contribution. Branchless: CC = 1.
#[must_use]
#[inline(always)]
pub fn apply_weight(raw_cp: i32, weight_q8: i32) -> i32 {
    raw_cp.wrapping_mul(weight_q8) >> 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_length_matches_count() {
        assert_eq!(STATION_WEIGHTS_Q8.len(), WEIGHT_COUNT);
    }

    #[test]
    fn unit_weight_is_identity() {
        assert_eq!(apply_weight(123, Q8_ONE), 123);
        assert_eq!(apply_weight(-77, Q8_ONE), -77);
    }
}