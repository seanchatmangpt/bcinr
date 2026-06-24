
#![forbid(unsafe_code)]

//! Feature station: mobility_evaluated (id 2).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Bitset. Primitive: `crate::rays::kogge_stone_attacks`.
//! Input mask: `sliders_and_leapers`. Score scale: 6 cp/unit.
//! Authority: raw_cp == (popcount(my_attacks & empty) - popcount(opp_attacks & empty)) * 6
//!
//! Mobility differential: Kogge-Stone rook/bishop/queen attack spans plus knight/king masks, counting reachable empty squares; 4 cp per square (reuses legal_moves).

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 2;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1002;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 256;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 6;

use crate::rays::{bishop_attacks, queen_attacks, rook_attacks, KING_MASKS, KNIGHT_MASKS};
use crate::position::{BISHOP, KING, KNIGHT, QUEEN, ROOK};

/// Full attack span of one color: slider rays (Kogge-Stone, over `empty`)
/// plus knight/king leaper masks. Private (loops), outside CC=1.
#[inline]
pub(super) fn attack_span(v: &PositionView, color: usize) -> u64 {
    let empty = v.empty;
    let mut span = 0u64;
    let mut bb = v.by_piece[color][ROOK];
    while bb != 0 {
        span |= rook_attacks(bb & bb.wrapping_neg(), empty);
        bb &= bb - 1;
    }
    let mut bb = v.by_piece[color][BISHOP];
    while bb != 0 {
        span |= bishop_attacks(bb & bb.wrapping_neg(), empty);
        bb &= bb - 1;
    }
    let mut bb = v.by_piece[color][QUEEN];
    while bb != 0 {
        span |= queen_attacks(bb & bb.wrapping_neg(), empty);
        bb &= bb - 1;
    }
    let mut bb = v.by_piece[color][KNIGHT];
    while bb != 0 {
        span |= KNIGHT_MASKS[bb.trailing_zeros() as usize];
        bb &= bb - 1;
    }
    let mut bb = v.by_piece[color][KING];
    while bb != 0 {
        span |= KING_MASKS[bb.trailing_zeros() as usize];
        bb &= bb - 1;
    }
    span
}

/// Branchless feature kernel for **mobility**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::mobility;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(mobility::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    let my = (attack_span(v, WHITE) & v.empty).count_ones() as i32;
    let opp = (attack_span(v, BLACK) & v.empty).count_ones() as i32;
    (my - opp).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = attack_span(v, WHITE) | attack_span(v, BLACK);
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
pub struct Station2;

impl Station for Station2 {
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

    /// Branchful reference oracle for the **mobility** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == (popcount(my_attacks & empty) - popcount(opp_attacks & empty)) * 6
    fn reference_raw_cp(v: &PositionView) -> i32 {
        let white_span = super::attack_span(v, WHITE);
        let black_span = super::attack_span(v, BLACK);
        let mut white = 0i32;
        let mut black = 0i32;
        let mut sq = 0usize;
        while sq < 64 {
            let m = 1u64 << sq;
            if (white_span & v.empty & m) != 0 {
                white += 1;
            }
            if (black_span & v.empty & m) != 0 {
                black += 1;
            }
            sq += 1;
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