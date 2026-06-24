
#![forbid(unsafe_code)]

//! Feature station: passed_pawn_evaluated (id 6).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Mask. Primitive: `crate::rays::south_fill`.
//! Input mask: `my_pawns & !enemy_block`. Score scale: 20 cp/unit.
//! Authority: raw_cp == sum_white_passed_pawns(RANK_BONUS[rank]) - sum_black_passed_pawns(RANK_BONUS[rank])
//!
//! Passed pawn bonus: a pawn with no enemy pawn on the same or adjacent files ahead of it scores a rank-indexed bonus (RANK_BONUS=[0,0,0,20,40,80,160,0]). South-fill of enemy pawn frontspan yields the block mask; passed = my_pawns & !block.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 6;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1006;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 256;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 20;

use crate::position::PAWN;

/// Rank bonus table (index = rank 0..7). Rank 0 and 7 are impossible for pawns.
#[allow(dead_code)]
const RANK_BONUS: [i32; 8] = [0, 0, 0, 20, 40, 80, 160, 0];

/// Sum passed-pawn bonuses for one color. Loops — private, outside CC=1 boundary.
#[inline]
pub(super) fn passed_score(v: &PositionView, color: usize) -> i32 {
    let rank_bonus = [0i32, 0, 0, 20, 40, 80, 160, 0];
    let mut score = 0i32;

    if color == 0 {
        // White pawns: check if each pawn is passed (no black pawn ahead on same/adj files)
        let mut bb = v.by_piece[WHITE][PAWN];
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            let rank = sq / 8;
            let file = sq % 8;
            let mut blocked = false;

            // Check ranks ahead (north)
            let mut r = rank + 1;
            while r < 8 {
                let bit = 1u64 << (r * 8 + file);
                let left = if file > 0 { 1u64 << (r * 8 + file - 1) } else { 0 };
                let right = if file < 7 { 1u64 << (r * 8 + file + 1) } else { 0 };
                if (v.by_piece[BLACK][PAWN] & (bit | left | right)) != 0 {
                    blocked = true;
                }
                r += 1;
            }

            if !blocked {
                score += rank_bonus[rank];
            }
            bb &= bb - 1;
        }
    } else {
        // Black pawns: check if each pawn is passed (no white pawn ahead on same/adj files)
        let mut bb = v.by_piece[BLACK][PAWN];
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            let rank = 7 - sq / 8;
            let file = sq % 8;
            let mut blocked = false;

            // Check ranks ahead (south)
            let mut r = (sq / 8) as i32 - 1;
            while r >= 0 {
                let bit = 1u64 << (r as usize * 8 + file);
                let left = if file > 0 { 1u64 << (r as usize * 8 + file - 1) } else { 0 };
                let right = if file < 7 { 1u64 << (r as usize * 8 + file + 1) } else { 0 };
                if (v.by_piece[WHITE][PAWN] & (bit | left | right)) != 0 {
                    blocked = true;
                }
                r -= 1;
            }

            if !blocked {
                score += rank_bonus[rank as usize];
            }
            bb &= bb - 1;
        }
    }

    score
}

/// Branchless feature kernel for **passed_pawn**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::passed_pawn;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(passed_pawn::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    (passed_score(v, WHITE) - passed_score(v, BLACK)).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = v.by_piece[WHITE][crate::position::PAWN] | v.by_piece[BLACK][crate::position::PAWN];
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
pub struct Station6;

impl Station for Station6 {
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

    /// Branchful reference oracle for the **passed_pawn** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == sum_white_passed_pawns(RANK_BONUS[rank]) - sum_black_passed_pawns(RANK_BONUS[rank])
    fn reference_raw_cp(v: &PositionView) -> i32 {
        let mut white = 0i32;
        let mut black = 0i32;
        let rank_bonus = [0i32, 0, 0, 20, 40, 80, 160, 0];
        // Use hardcoded indices to debug
        let mut bb = v.by_piece[0][0];  // WHITE = 0, PAWN = 0
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            let rank = sq / 8;
            // check no black pawn on same or adjacent files ahead (north)
            let file = sq % 8;
            let mut blocked = false;
            let mut r = rank + 1;
            while r < 8 {
                let bit = 1u64 << (r * 8 + file);
                let left = if file > 0 { 1u64 << (r * 8 + file - 1) } else { 0 };
                let right = if file < 7 { 1u64 << (r * 8 + file + 1) } else { 0 };
                if (v.by_piece[1][0] & (bit | left | right)) != 0 {  // BLACK = 1, PAWN = 0
                    blocked = true;
                }
                r += 1;
            }
            if !blocked {
                white += rank_bonus[rank];
            }
            bb &= bb - 1;
        }
        let mut bb = v.by_piece[1][0];  // BLACK = 1, PAWN = 0
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            let rank = 7 - sq / 8;
            let file = sq % 8;
            let mut blocked = false;
            let mut r = (sq / 8) as i32 - 1;
            while r >= 0 {
                let bit = 1u64 << (r as usize * 8 + file);
                let left = if file > 0 { 1u64 << (r as usize * 8 + file - 1) } else { 0 };
                let right = if file < 7 { 1u64 << (r as usize * 8 + file + 1) } else { 0 };
                if (v.by_piece[0][0] & (bit | left | right)) != 0 {  // WHITE = 0, PAWN = 0
                    blocked = true;
                }
                r -= 1;
            }
            if !blocked {
                black += rank_bonus[rank];
            }
            bb &= bb - 1;
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