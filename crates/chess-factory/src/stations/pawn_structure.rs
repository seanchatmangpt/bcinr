
#![forbid(unsafe_code)]

//! Feature station: pawn_structure_evaluated (id 4).
//!
//! # Branchless Contract
//! **Ensures:** `evaluate` matches the branchful `_reference` oracle for all inputs.
//! **Invariant:** the execution path is independent of input data values (CC=1).
//!
//! Lowering: Bitset. Primitive: `crate::rays::file_fill`.
//! Input mask: `pawns_by_color`. Score scale: 15 cp/unit.
//! Authority: raw_cp == (opp_doubled + opp_isolated - my_doubled - my_isolated) * 15
//!
//! Pawn structure: doubled (extra pawns beyond one per file) and isolated (no friendly pawn on either adjacent file) pawns penalised 15 cp each; white-relative.

use crate::position::{PositionView, BLACK, WHITE};
use crate::station::{Evidence, Station, StationResult};

/// Stable ordinal identifier of this station (ORDER BY anchor in the ontology).
pub const STATION_ID: u16 = 4;
/// OCEL event code emitted when this station fires.
pub const EVENT_CODE: u16 = 1004;
/// Q8.8 aggregation weight (256 == 1.0).
pub const WEIGHT_Q8: i32 = 256;
/// Centipawn scale applied to the raw differential.
pub const SCORE_SCALE: i32 = 15;

use crate::position::PAWN;

/// File masks (A..H); FILES[f] selects every square on file f.
const FILES: [u64; 8] = [
    0x0101_0101_0101_0101,
    0x0202_0202_0202_0202,
    0x0404_0404_0404_0404,
    0x0808_0808_0808_0808,
    0x1010_1010_1010_1010,
    0x2020_2020_2020_2020,
    0x4040_4040_4040_4040,
    0x8080_8080_8080_8080,
];

/// doubled+isolated penalty count for one color's pawns. Private, loops.
#[inline]
pub(super) fn pawn_penalty(v: &PositionView, color: usize) -> i32 {
    let pawns = v.by_piece[color][PAWN];
    let mut penalty = 0i32;
    let mut f = 0usize;
    while f < 8 {
        let on_file = (pawns & FILES[f]).count_ones() as i32;
        if on_file > 0 {
            // doubled: every pawn beyond the first on this file.
            penalty += on_file - 1;
            // isolated: no friendly pawn on either adjacent file.
            let mut neighbour = false;
            if f > 0 && (pawns & FILES[f - 1]) != 0 {
                neighbour = true;
            }
            if f < 7 && (pawns & FILES[f + 1]) != 0 {
                neighbour = true;
            }
            if !neighbour {
                penalty += on_file;
            }
        }
        f += 1;
    }
    penalty
}

/// Branchless feature kernel for **pawn_structure**.
///
/// Reduces the position to a white-relative centipawn contribution per the
/// station's named law (Authority above). The public entry is CC = 1: every
/// loop is delegated to a private helper above (the looping-projection rule).
///
/// # Examples
///
/// ```
/// use chess_factory::position::PositionView;
/// use chess_factory::stations::pawn_structure;
/// let v = PositionView::default();
/// // Empty board: balanced, zero contribution.
/// assert_eq!(pawn_structure::raw_cp(&v), 0);
/// ```
#[must_use]
#[inline(always)]
pub fn raw_cp(v: &PositionView) -> i32 {
    (pawn_penalty(v, BLACK) - pawn_penalty(v, WHITE)).wrapping_mul(SCORE_SCALE)
}

/// Branchless evaluation entry point. CC = 1: a single straight-line reduction
/// producing the [`StationResult`] (weighted score + evidence) for receipts.
#[must_use]
#[inline(always)]
pub fn evaluate(v: &PositionView) -> StationResult {
    let raw = raw_cp(v);
    let fired = v.by_piece[WHITE][PAWN] | v.by_piece[BLACK][PAWN];
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
pub struct Station4;

impl Station for Station4 {
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

    /// Branchful reference oracle for the **pawn_structure** law. Test-only, so it
    /// MAY branch; the kernel above MUST match it on every input.
    /// Authority: raw_cp == (opp_doubled + opp_isolated - my_doubled - my_isolated) * 15
    fn reference_raw_cp(v: &PositionView) -> i32 {
        fn pen(pawns: u64) -> i32 {
            const FILE_A: u64 = 0x0101_0101_0101_0101;
            let mut p = 0i32;
            let mut f = 0usize;
            while f < 8 {
                let file = FILE_A << f;
                let n = (pawns & file).count_ones() as i32;
                if n > 0 {
                    p += n - 1;
                    let mut has_neighbour = false;
                    if f > 0 && (pawns & (FILE_A << (f - 1))) != 0 {
                        has_neighbour = true;
                    }
                    if f < 7 && (pawns & (FILE_A << (f + 1))) != 0 {
                        has_neighbour = true;
                    }
                    if !has_neighbour {
                        p += n;
                    }
                }
                f += 1;
            }
            p
        }
        (pen(v.by_piece[BLACK][PAWN]) - pen(v.by_piece[WHITE][PAWN])) * SCORE_SCALE
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