
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
const RANK_BONUS: [i32; 8] = [0, 0, 0, 20, 40, 80, 160, 0];

/// Fill all squares south of (and including) occupied bits — produces the
/// "already controlled" span below the enemy pawn frontspan.
#[inline]
fn south_fill(mut bb: u64) -> u64 {
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    bb
}

/// North-fill: propagates bits toward higher ranks (rank 7).
#[inline]
fn north_fill(mut bb: u64) -> u64 {
    bb |= bb << 8;
    bb |= bb << 16;
    bb |= bb << 32;
    bb
}

/// Spread a bitboard one file left and right (masking wrap-around).
#[inline]
fn adj_files(bb: u64) -> u64 {
    bb | ((bb & 0xFEFEFEFEFEFEFEFEu64) >> 1) | ((bb & 0x7F7F7F7F7F7F7F7Fu64) << 1)
}

/// Block mask for WHITE passed-pawn detection.
///
/// A white pawn is blocked if any black pawn exists strictly north of it on
/// the same or adjacent file. We mark "strictly south of each black pawn"
/// by south-filling from one rank below the black pawns, then spreading to
/// adjacent files. Same-rank black pawns are excluded (they cannot block).
#[inline]
fn white_block(black_pawns: u64) -> u64 {
    // Shift black pawns one rank south before filling so we get squares
    // strictly south of each black pawn (i.e., squares where a white pawn
    // would have a black blocker ahead of it).
    adj_files(south_fill(black_pawns >> 8))
}

/// Block mask for BLACK passed-pawn detection.
///
/// A black pawn is blocked if any white pawn exists strictly south of it on
/// the same or adjacent file. North-fill from one rank above the white pawns
/// then spread to adjacent files.
#[inline]
fn black_block(white_pawns: u64) -> u64 {
    adj_files(north_fill(white_pawns << 8))
}

/// Sum passed-pawn bonuses for one color. Loops — private, outside CC=1 boundary.
#[inline]
pub(super) fn passed_score(v: &PositionView, color: usize) -> i32 {
    let opp = 1 - color;
    let my_pawns = v.by_piece[color][PAWN];
    let opp_pawns = v.by_piece[opp][PAWN];
    // For white (color=0), passed = pawn with no black pawn on same/adj file ahead (north).
    // For black (color=1), mirror: we reflect rank so rank increases towards promotion.
    let block = if color == 0 {
        white_block(opp_pawns)
    } else {
        black_block(opp_pawns)
    };
    let passed = my_pawns & !block;
    let mut bb = passed;
    let mut score = 0i32;
    while bb != 0 {
        let sq = bb.trailing_zeros() as usize;
        let rank = if color == 0 { sq / 8 } else { 7 - sq / 8 };
        score += RANK_BONUS[rank];
        bb &= bb - 1;
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
        let mut bb = v.by_piece[WHITE][crate::position::PAWN];
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
                if (v.by_piece[BLACK][crate::position::PAWN] & (bit | left | right)) != 0 {
                    blocked = true;
                }
                r += 1;
            }
            if !blocked {
                white += rank_bonus[rank];
            }
            bb &= bb - 1;
        }
        let mut bb = v.by_piece[BLACK][crate::position::PAWN];
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
                if (v.by_piece[WHITE][crate::position::PAWN] & (bit | left | right)) != 0 {
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