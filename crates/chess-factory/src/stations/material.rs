
#![forbid(unsafe_code)]

//! Feature station: material_evaluated (id 0).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Saturating. Primitive: `bcinr_logic::bitset::popcount_u64`.
//! Input mask: `occupancy_by_piece`. Score scale: 1 cp/unit.
//! Authority: raw_cp == sum_over_pieces(popcount(my[p]) - popcount(opp[p])) * baseValue[p]
//!
//! White-relative material balance: signed Q0 popcount of each piece type scaled by its centipawn base value (Pawn 100 .. King 20000).

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 0;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1000;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 256;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 1;

/// Per-piece centipawn base values (PeSTO midgame), indexed
/// pawn..king. King is valued at 20000 so material dominates.
const BASE_VALUE: [i32; 6] = [82, 337, 365, 477, 1025, 20000];

/// Weighted popcount fold for one color (private; loops, so outside CC=1).
#[inline]
fn weighted_material(v: &PositionView, color: usize) -> i32 {
    let mut acc = 0i32;
    let mut p = 0usize;
    while p < 6 {
        acc += (v.by_piece[color][p].count_ones() as i32) * BASE_VALUE[p];
        p += 1;
    }
    acc
}

/// Branchless feature kernel for **material**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::material;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(material::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    (weighted_material(v, WHITE) - weighted_material(v, BLACK)).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = v.by_color[WHITE] | v.by_color[BLACK];
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
pub struct Station0;

impl Station for Station0 {
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

    /// Branchful reference oracle for the **material** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == sum_over_pieces(popcount(my[p]) - popcount(opp[p])) * baseValue[p]
    fn reference_raw_cp(v: &PositionView) -> i32 {
        const BASE: [i32; 6] = [82, 337, 365, 477, 1025, 20000];
        let mut white = 0i32;
        let mut black = 0i32;
        let mut p = 0usize;
        while p < 6 {
            white += v.by_piece[WHITE][p].count_ones() as i32 * BASE[p];
            black += v.by_piece[BLACK][p].count_ones() as i32 * BASE[p];
            p += 1;
        }
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