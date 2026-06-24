
#![forbid(unsafe_code)]

//! Feature station: pst_evaluated (id 1).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Lut. Primitive: `crate::tables::PST_Q0`.
//! Input mask: `occupancy_by_piece_square`. Score scale: 1 cp/unit.
//! Authority: raw_cp == sum_over_pieces_squares(white_bit ? +pst[p][sq^56] : 0) - (black_bit ? +pst[p][sq] : 0)
//!
//! Piece-square-table positional bonus (PeSTO-style middlegame): central advancement rewarded, rim/edge penalised; black mirrors rank (sq^56) for white-relative score.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 1;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1001;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 256;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 1;

/// PeSTO-style middlegame piece-square tables, listed a8..h1 (table row 0
/// = rank 8). White reads `PST[p][sq ^ 56]` (rank-flip), black reads
/// `PST[p][sq]`, giving a white-relative positional score.
const PST: [[i32; 64]; 6] = [
    // Pawn
    [  0,  0,  0,  0,  0,  0,  0,  0,
      98,134, 61, 95, 68,126, 34,-11,
      -6,  7, 26, 31, 65, 56, 25,-20,
     -14, 13,  6, 21, 23, 12, 17,-23,
     -27, -2, -5, 12, 17,  6, 10,-25,
     -26, -4, -4,-10,  3,  3, 33,-12,
     -35, -1,-20,-23,-15, 24, 38,-22,
       0,  0,  0,  0,  0,  0,  0,  0],
    // Knight
    [-167,-89,-34,-49, 61,-97,-15,-107,
      -73,-41, 72, 36, 23, 62,  7, -17,
      -47, 60, 37, 65, 84,129, 73,  44,
       -9, 17, 19, 53, 37, 69, 18,  22,
      -13,  4, 16, 13, 28, 19, 21,  -8,
      -23, -9, 12, 10, 19, 17, 25, -16,
      -29,-53,-12, -3, -1, 18,-14, -19,
     -105,-21,-58,-33,-17,-28,-19, -23],
    // Bishop
    [-29,  4,-82,-37,-25,-42,  7, -8,
     -26, 16,-18,-13, 30, 59, 18,-47,
     -16, 37, 43, 40, 35, 50, 37, -2,
      -4,  5, 19, 50, 37, 37,  7, -2,
      -6, 13, 13, 26, 34, 12, 10,  4,
       0, 15, 15, 15, 14, 27, 18, 10,
       4, 15, 16,  0,  7, 21, 33,  1,
     -33, -3,-14,-21,-13,-12,-39,-21],
    // Rook
    [ 32, 42, 32, 51, 63,  9, 31, 43,
      27, 32, 58, 62, 80, 67, 26, 44,
      -5, 19, 26, 36, 17, 45, 61, 16,
     -24,-11,  7, 26, 24, 35, -8,-20,
     -36,-26,-12, -1,  9, -7,  6,-23,
     -45,-25,-16,-17,  3,  0, -5,-33,
     -44,-16,-20, -9, -1, 11, -6,-71,
     -19,-13,  1, 17, 16,  7,-37,-26],
    // Queen
    [-28,  0, 29, 12, 59, 44, 43, 45,
     -24,-39, -5,  1,-16, 57, 28, 54,
     -13,-17,  7,  8, 29, 56, 47, 57,
     -27,-27,-16,-16, -1, 17, -2,  1,
      -9,-26, -9,-10, -2, -4,  3, -3,
     -14,  2,-11, -2, -5,  2, 14,  5,
     -35, -8, 11,  2,  8, 15, -3,  1,
      -1,-18, -9, 10,-15,-25,-31,-50],
    // King
    [-65, 23, 16,-15,-56,-34,  2, 13,
      29, -1,-20, -7, -8, -4,-38,-29,
      -9, 24,  2,-16,-20,  6, 22,-22,
     -17,-20,-12,-27,-30,-25,-14,-36,
     -49, -1,-27,-39,-46,-44,-33,-51,
     -14,-14,-22,-46,-44,-30,-15,-27,
       1,  7, -8,-64,-43,-16,  9,  8,
     -15, 36, 12,-54,  8,-28, 24, 14],
];

/// Folded white-relative PST score over all pieces/squares (private; loops).
#[inline]
fn pst_score(v: &PositionView) -> i32 {
    let mut acc = 0i32;
    let mut p = 0usize;
    while p < 6 {
        let mut w = v.by_piece[WHITE][p];
        while w != 0 {
            let sq = w.trailing_zeros() as usize;
            acc += PST[p][sq ^ 56];
            w &= w - 1;
        }
        let mut b = v.by_piece[BLACK][p];
        while b != 0 {
            let sq = b.trailing_zeros() as usize;
            acc -= PST[p][sq];
            b &= b - 1;
        }
        p += 1;
    }
    acc
}

/// Branchless feature kernel for **pst**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::pst;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(pst::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    pst_score(v).wrapping_mul(SCORE_SCALE)
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
pub struct Station1;

impl Station for Station1 {
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

    /// Branchful reference oracle for the **pst** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == sum_over_pieces_squares(white_bit ? +pst[p][sq^56] : 0) - (black_bit ? +pst[p][sq] : 0)
    fn reference_raw_cp(v: &PositionView) -> i32 {
        const T: [[i32; 64]; 6] = [
            [  0,  0,  0,  0,  0,  0,  0,  0, 98,134, 61, 95, 68,126, 34,-11, -6,  7, 26, 31, 65, 56, 25,-20,-14, 13,  6, 21, 23, 12, 17,-23,-27, -2, -5, 12, 17,  6, 10,-25,-26, -4, -4,-10,  3,  3, 33,-12,-35, -1,-20,-23,-15, 24, 38,-22,  0,  0,  0,  0,  0,  0,  0,  0],
            [-167,-89,-34,-49, 61,-97,-15,-107,-73,-41, 72, 36, 23, 62,  7, -17,-47, 60, 37, 65, 84,129, 73,  44, -9, 17, 19, 53, 37, 69, 18,  22,-13,  4, 16, 13, 28, 19, 21,  -8,-23, -9, 12, 10, 19, 17, 25, -16,-29,-53,-12, -3, -1, 18,-14, -19,-105,-21,-58,-33,-17,-28,-19, -23],
            [-29,  4,-82,-37,-25,-42,  7, -8,-26, 16,-18,-13, 30, 59, 18,-47,-16, 37, 43, 40, 35, 50, 37, -2, -4,  5, 19, 50, 37, 37,  7, -2, -6, 13, 13, 26, 34, 12, 10,  4,  0, 15, 15, 15, 14, 27, 18, 10,  4, 15, 16,  0,  7, 21, 33,  1,-33, -3,-14,-21,-13,-12,-39,-21],
            [ 32, 42, 32, 51, 63,  9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,-24,-11,  7, 26, 24, 35, -8,-20,-36,-26,-12, -1,  9, -7,  6,-23,-45,-25,-16,-17,  3,  0, -5,-33,-44,-16,-20, -9, -1, 11, -6,-71,-19,-13,  1, 17, 16,  7,-37,-26],
            [-28,  0, 29, 12, 59, 44, 43, 45,-24,-39, -5,  1,-16, 57, 28, 54,-13,-17,  7,  8, 29, 56, 47, 57,-27,-27,-16,-16, -1, 17, -2,  1, -9,-26, -9,-10, -2, -4,  3, -3,-14,  2,-11, -2, -5,  2, 14,  5,-35, -8, 11,  2,  8, 15, -3,  1, -1,-18, -9, 10,-15,-25,-31,-50],
            [-65, 23, 16,-15,-56,-34,  2, 13, 29, -1,-20, -7, -8, -4,-38,-29, -9, 24,  2,-16,-20,  6, 22,-22,-17,-20,-12,-27,-30,-25,-14,-36,-49, -1,-27,-39,-46,-44,-33,-51,-14,-14,-22,-46,-44,-30,-15,-27,  1,  7, -8,-64,-43,-16,  9,  8,-15, 36, 12,-54,  8,-28, 24, 14],
        ];
        let mut acc = 0i32;
        let mut p = 0usize;
        while p < 6 {
            let mut sq = 0usize;
            while sq < 64 {
                let m = 1u64 << sq;
                if v.by_piece[WHITE][p] & m != 0 {
                    acc += T[p][sq ^ 56];
                }
                if v.by_piece[BLACK][p] & m != 0 {
                    acc -= T[p][sq];
                }
                sq += 1;
            }
            p += 1;
        }
        acc * SCORE_SCALE
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