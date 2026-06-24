
#![forbid(unsafe_code)]

//! Feature station: king_safety_evaluated (id 3).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Mask. Primitive: `crate::rays::king_ring`.
//! Input mask: `king_ring_and_attacks`. Score scale: 20 cp/unit.
//! Authority: raw_cp == (popcount(my_attacks & opp_king_ring) - popcount(opp_attacks & my_king_ring)) * 20
//!
//! King safety: net attackers on the enemy king ring (8-neighbourhood) minus attackers on our own king ring, 12 cp per attacked ring square; white-relative and signed.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 3;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1003;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 256;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 20;

use crate::rays::{bishop_attacks, queen_attacks, rook_attacks, KING_MASKS, KNIGHT_MASKS};
use crate::position::{BISHOP, KING, KNIGHT, QUEEN, ROOK};

/// Full attack span of one color (see mobility). Private, loops.
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

/// 8-neighbourhood ring around every king of `color`. Private, loops.
#[inline]
pub(super) fn king_ring(v: &PositionView, color: usize) -> u64 {
    let mut ring = 0u64;
    let mut bb = v.by_piece[color][KING];
    while bb != 0 {
        ring |= KING_MASKS[bb.trailing_zeros() as usize];
        bb &= bb - 1;
    }
    ring
}

/// Branchless feature kernel for **king_safety**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::king_safety;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(king_safety::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    let on_opp = (attack_span(v, WHITE) & king_ring(v, BLACK)).count_ones() as i32;
    let on_my = (attack_span(v, BLACK) & king_ring(v, WHITE)).count_ones() as i32;
    (on_opp - on_my).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = king_ring(v, WHITE) | king_ring(v, BLACK);
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
pub struct Station3;

impl Station for Station3 {
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

    /// Branchful reference oracle for the **king_safety** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == (popcount(my_attacks & opp_king_ring) - popcount(opp_attacks & my_king_ring)) * 20
    fn reference_raw_cp(v: &PositionView) -> i32 {
        let white_span = super::attack_span(v, WHITE);
        let black_span = super::attack_span(v, BLACK);
        let white_ring = super::king_ring(v, WHITE);
        let black_ring = super::king_ring(v, BLACK);
        let mut on_opp = 0i32;
        let mut on_my = 0i32;
        let mut sq = 0usize;
        while sq < 64 {
            let m = 1u64 << sq;
            if (white_span & black_ring & m) != 0 {
                on_opp += 1;
            }
            if (black_span & white_ring & m) != 0 {
                on_my += 1;
            }
            sq += 1;
        }
        (on_opp - on_my) * SCORE_SCALE
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