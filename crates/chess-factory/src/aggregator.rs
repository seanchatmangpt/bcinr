//! Weighted station aggregation (straight-line, manufactured-from-registry).
//!
//! Applies `(station.raw_cp * weights::STATION_WEIGHTS_Q8[i]) >> 8` over every
//! generated feature station and sums the contributions into a single
//! white-relative centipawn score. The reduction is fully unrolled (no loop,
//! no branch) so the aggregate kernel stays straight-line; the per-station
//! `evaluate` kernels are themselves CC=1.

use crate::motifs;
use crate::position::{PositionView, WHITE};
use crate::stations;
use crate::weights::{apply_weight, STATION_WEIGHTS_Q8};

/// Centipawn bonus per fired tactical motif (modest, load-bearing not dominant).
const MOTIF_CP: i32 = 30;

/// Small white-relative tactical bonus from the manufactured motif detectors.
///
/// Each offensive motif (fork/pin/skewer) that fires for the side to move adds
/// `MOTIF_CP`; a hanging-piece detection (the side to move's own piece is
/// hanging) subtracts it. Oriented white-relative so the aggregate stays in
/// white's frame. Straight-line: CC = 1.
#[must_use]
#[inline(always)]
pub fn motif_bonus(v: &PositionView) -> i32 {
    let offense = ((motifs::fork::detect(v) != 0) as i32
        + (motifs::pin::detect(v) != 0) as i32
        + (motifs::skewer::detect(v) != 0) as i32)
        * MOTIF_CP;
    let hanging = (motifs::hanging::detect(v) != 0) as i32 * MOTIF_CP;
    let stm_relative = offense - hanging;
    let sign = 1 - 2 * (v.stm != WHITE) as i32;
    stm_relative * sign
}

/// Aggregate every feature station into a single white-relative centipawn score.
///
/// Straight-line: each station's `raw_cp` evidence is re-weighted by the Q8.8
/// weight table and summed. No conditional control flow.
#[must_use]
#[inline(always)]
pub fn aggregate(v: &PositionView) -> i32 {
    let r0 = stations::evaluate_material(v).evidence.raw_cp;
    let r1 = stations::evaluate_pst(v).evidence.raw_cp;
    let r2 = stations::evaluate_mobility(v).evidence.raw_cp;
    let r3 = stations::evaluate_king_safety(v).evidence.raw_cp;
    let r4 = stations::evaluate_pawn_structure(v).evidence.raw_cp;
    let r5 = stations::evaluate_center_control(v).evidence.raw_cp;
    let r6 = stations::evaluate_passed_pawn(v).evidence.raw_cp;
    let r7 = stations::evaluate_rook_open_file(v).evidence.raw_cp;
    let r8 = stations::evaluate_bishop_pair(v).evidence.raw_cp;
    let r9 = stations::evaluate_king_tropism(v).evidence.raw_cp;

    apply_weight(r0, STATION_WEIGHTS_Q8[0])
        .wrapping_add(apply_weight(r1, STATION_WEIGHTS_Q8[1]))
        .wrapping_add(apply_weight(r2, STATION_WEIGHTS_Q8[2]))
        .wrapping_add(apply_weight(r3, STATION_WEIGHTS_Q8[3]))
        .wrapping_add(apply_weight(r4, STATION_WEIGHTS_Q8[4]))
        .wrapping_add(apply_weight(r5, STATION_WEIGHTS_Q8[5]))
        .wrapping_add(apply_weight(r6, STATION_WEIGHTS_Q8[6]))
        .wrapping_add(apply_weight(r7, STATION_WEIGHTS_Q8[7]))
        .wrapping_add(apply_weight(r8, STATION_WEIGHTS_Q8[8]))
        .wrapping_add(apply_weight(r9, STATION_WEIGHTS_Q8[9]))
        .wrapping_add(motif_bonus(v))
}

/// Apply a Q8.8 weight to a raw centipawn contribution. Branchless.
#[inline(always)]
#[must_use]
pub fn weight_q8(raw_cp: i32, weight_q8: i32) -> i32 {
    apply_weight(raw_cp, weight_q8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_is_balanced() {
        let v = PositionView::default();
        assert_eq!(aggregate(&v), 0);
    }
}
