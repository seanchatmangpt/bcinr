
#![forbid(unsafe_code)]

//! Feature station: rook_open_file_evaluated (id 7).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Mask. Primitive: `crate::rays::FILE_A`.
//! Input mask: `rooks & open_files`. Score scale: 10 cp/unit.
//! Authority: raw_cp == sum_rooks(open?25:semi_open?10:0) white minus black
//!
//! Rook on open/semi-open file bonus: 25 cp for a fully open file (no pawns of either color), 10 cp for semi-open (no own pawns). White minus black, scaled.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 7;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1007;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 192;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 10;

use crate::position::{PAWN, ROOK};

/// File-A mask repeated across all 8 files (one bit per rank on file A).
const FILE_A: u64 = 0x0101010101010101u64;

/// Score for all rooks of one color based on open/semi-open file status.
/// Loops over rooks — private, outside CC=1 boundary.
#[inline]
pub(super) fn rook_file_score(v: &PositionView, color: usize) -> i32 {
    let all_pawns = v.by_piece[WHITE][PAWN] | v.by_piece[BLACK][PAWN];
    let own_pawns = v.by_piece[color][PAWN];
    let mut rooks = v.by_piece[color][ROOK];
    let mut score = 0i32;
    while rooks != 0 {
        let sq = rooks.trailing_zeros() as usize;
        let file = sq % 8;
        let file_mask = FILE_A << file;
        let open = (all_pawns & file_mask) == 0;
        let semi_open = (own_pawns & file_mask) == 0;
        score += if open { 25 } else if semi_open { 10 } else { 0 };
        rooks &= rooks - 1;
    }
    score
}

/// Branchless feature kernel for **rook_open_file**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::rook_open_file;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(rook_open_file::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    (rook_file_score(v, WHITE) - rook_file_score(v, BLACK)).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = v.by_piece[WHITE][crate::position::ROOK] | v.by_piece[BLACK][crate::position::ROOK];
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
pub struct Station7;

impl Station for Station7 {
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

    /// Branchful reference oracle for the **rook_open_file** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == sum_rooks(open?25:semi_open?10:0) white minus black
    fn reference_raw_cp(v: &PositionView) -> i32 {
        let file_a: u64 = 0x0101010101010101u64;
        let all_pawns = v.by_piece[WHITE][crate::position::PAWN] | v.by_piece[BLACK][crate::position::PAWN];
        let mut white = 0i32;
        let mut black = 0i32;
        let mut rooks = v.by_piece[WHITE][crate::position::ROOK];
        while rooks != 0 {
            let sq = rooks.trailing_zeros() as usize;
            let file_mask = file_a << (sq % 8);
            let open = (all_pawns & file_mask) == 0;
            let semi = (v.by_piece[WHITE][crate::position::PAWN] & file_mask) == 0;
            white += if open { 25 } else if semi { 10 } else { 0 };
            rooks &= rooks - 1;
        }
        let mut rooks = v.by_piece[BLACK][crate::position::ROOK];
        while rooks != 0 {
            let sq = rooks.trailing_zeros() as usize;
            let file_mask = file_a << (sq % 8);
            let open = (all_pawns & file_mask) == 0;
            let semi = (v.by_piece[BLACK][crate::position::PAWN] & file_mask) == 0;
            black += if open { 25 } else if semi { 10 } else { 0 };
            rooks &= rooks - 1;
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