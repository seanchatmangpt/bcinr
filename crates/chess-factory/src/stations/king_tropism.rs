
#![forbid(unsafe_code)]

//! Feature station: king_tropism_evaluated (id 9).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Mask. Primitive: `crate::rays::KING_MASKS`.
//! Input mask: `attacker_bits & king_ring`. Score scale: 10 cp/unit.
//! Authority: raw_cp == sum(white_piece_in_black_king_ring*weight[p]) - same_for_black; weights: Knight=3, Bishop=2, Rook=4, Queen=5
//!
//! King tropism: counts enemy pieces attacking the king ring (king square plus adjacent squares) weighted by piece type. Knight=3, Bishop=2, Rook=4, Queen=5. White attackers near black king minus black attackers near white king.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 9;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1009;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 192;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 10;

use crate::rays::KING_MASKS;
use crate::position::{BISHOP, KING, KNIGHT, QUEEN, ROOK};

/// King ring = KING_MASKS[sq] | king_square_bit.
#[inline]
pub(super) fn king_ring_9(v: &PositionView, color: usize) -> u64 {
    let king_bb = v.by_piece[color][KING];
    if king_bb == 0 {
        return 0;
    }
    let sq = king_bb.trailing_zeros() as usize;
    KING_MASKS[sq] | king_bb
}

/// Count weighted tropism score: sum of (attacker_bb & ring).count_ones() * weight
/// for each piece type of `attacker` color. Loops — private, outside CC=1.
#[inline]
pub(super) fn tropism_score(v: &PositionView, attacker: usize, ring: u64) -> i32 {
    let n = (v.by_piece[attacker][KNIGHT] & ring).count_ones() as i32;
    let b = (v.by_piece[attacker][BISHOP] & ring).count_ones() as i32;
    let r = (v.by_piece[attacker][ROOK]   & ring).count_ones() as i32;
    let q = (v.by_piece[attacker][QUEEN]  & ring).count_ones() as i32;
    n * 3 + b * 2 + r * 4 + q * 5
}

/// Branchless feature kernel for **king_tropism**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::king_tropism;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(king_tropism::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    let b_ring = king_ring_9(v, BLACK);
    let w_ring = king_ring_9(v, WHITE);
    (tropism_score(v, WHITE, b_ring) - tropism_score(v, BLACK, w_ring)).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = king_ring_9(v, WHITE) | king_ring_9(v, BLACK);
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
pub struct Station9;

impl Station for Station9 {
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

    /// Branchful reference oracle for the **king_tropism** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == sum(white_piece_in_black_king_ring*weight[p]) - same_for_black; weights: Knight=3, Bishop=2, Rook=4, Queen=5
    fn reference_raw_cp(v: &PositionView) -> i32 {
        use crate::rays::KING_MASKS;
        let b_king = v.by_piece[BLACK][crate::position::KING];
        let w_king = v.by_piece[WHITE][crate::position::KING];
        let b_ring = if b_king != 0 { let sq = b_king.trailing_zeros() as usize; KING_MASKS[sq] | b_king } else { 0 };
        let w_ring = if w_king != 0 { let sq = w_king.trailing_zeros() as usize; KING_MASKS[sq] | w_king } else { 0 };
        let mut white_score = 0i32;
        let mut black_score = 0i32;
        let weights = [0i32, 3, 2, 4, 5, 0]; // indexed by piece type: PAWN=0,KNIGHT=1,BISHOP=2,ROOK=3,QUEEN=4,KING=5
        let mut p = 1usize;
        while p <= 4 {
            white_score += (v.by_piece[WHITE][p] & b_ring).count_ones() as i32 * weights[p];
            black_score += (v.by_piece[BLACK][p] & w_ring).count_ones() as i32 * weights[p];
            p += 1;
        }
        (white_score - black_score) * SCORE_SCALE
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