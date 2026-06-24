
#![forbid(unsafe_code)]

//! Feature station: bishop_pair_evaluated (id 8).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Mask. Primitive: `crate::rays::BISHOP_MASKS`.
//! Input mask: `bishops & color_bit`. Score scale: 30 cp/unit.
//! Authority: raw_cp == (white_bishops>=2?30:0) - (black_bishops>=2?30:0)
//!
//! Bishop pair bonus: 30 cp if a side has two or more bishops. Branchless: (count_ones() as i32 - 1).clamp(0, 1) * 30. White minus black.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 8;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1008;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 128;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 30;

use crate::position::BISHOP;

/// Branchless bishop-pair bonus for one color. CC=1 sub-expression.
#[inline]
pub(super) fn bp_score(v: &PositionView, color: usize) -> i32 {
    let cnt = v.by_piece[color][BISHOP].count_ones() as i32;
    (cnt - 1).clamp(0, 1) * 30
}

/// Branchless feature kernel for **bishop_pair**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::bishop_pair;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(bishop_pair::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    (bp_score(v, WHITE) - bp_score(v, BLACK)).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = v.by_piece[WHITE][crate::position::BISHOP] | v.by_piece[BLACK][crate::position::BISHOP];
    StationResult {
        score_cp: raw.wrapping_mul(WEIGHT_Q8) >> 8,
        evidence: Evidence {
            station_id: STATION_ID,
            fired_mask: fired,
            raw_cp: raw,
            weight_q8: WEIGHT_Q8,
        },
    }
}

/// Zero-sized station marker implementing the [`Station`] trait.
/// Named `Station<id>` for a deterministic, collision-free type identity.
#[derive(Debug, Clone, Copy, Default)]
pub struct Station8;

impl Station for Station8 {
    #[inline(always)]
    fn evaluate(v: &PositionView) -> StationResult {
        evaluate(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::PositionView;
    use proptest::prelude::*;

    /// Branchful reference oracle for the **bishop_pair** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == (white_bishops>=2?30:0) - (black_bishops>=2?30:0)
    fn reference_raw_cp(v: &PositionView) -> i32 {
        let white_b = v.by_piece[WHITE][crate::position::BISHOP].count_ones();
        let black_b = v.by_piece[BLACK][crate::position::BISHOP].count_ones();
        let white = if white_b >= 2 { 30i32 } else { 0 };
        let black = if black_b >= 2 { 30i32 } else { 0 };
        (white - black) * SCORE_SCALE
    }

    fn view_from(by_piece: [[u64; 6]; 2], stm: usize) -> PositionView {
        PositionView::from_bitboards(by_piece, stm)
    }

    proptest! {
        /// Kernel == oracle for arbitrary packed positions.
        #[test]
        fn prop_kernel_matches_oracle(
            wp in any::<[u64; 6]>(),
            bp in any::<[u64; 6]>(),
            stm in 0usize..2,
        ) {
            let v = view_from([wp, bp], stm);
            prop_assert_eq!(raw_cp(&v), reference_raw_cp(&v));
        }
    }

    #[test]
    fn empty_board_is_balanced() {
        let v = PositionView::default();
        assert_eq!(raw_cp(&v), 0);
        assert_eq!(reference_raw_cp(&v), 0);
    }

    #[test]
    fn evidence_carries_station_identity() {
        let v = PositionView::default();
        let r = evaluate(&v);
        assert_eq!(r.evidence.station_id, STATION_ID);
        assert_eq!(r.evidence.weight_q8, WEIGHT_Q8);
    }
}