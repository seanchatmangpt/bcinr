
#![forbid(unsafe_code)]

//! Tactical motif: hanging_detected (id 0).
//!
//! # Branchless Contract
//! **Ensures:** `detect` matches the independent branchful `_reference` oracle.
//! **Invariant:** the public execution path is data-independent (CC=1); all
//! looping detection is delegated to private `pub(super)` helpers.
//!
//! Lowering: Mask. Primitive: `crate::rays::attack_sets`.
//! Input mask: `my & opp_attacks & !own_defends`. Authority: fired == (stm_pieces & enemy_attacks & !stm_attacks)
//!
//! Hanging piece: side-to-move pieces attacked by the enemy and not defended by a friendly piece (stm & enemy_attacks & !stm_defends).

#[allow(unused_imports)]
use crate::position::{PositionView, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};

/// Stable ordinal identifier of this motif (ORDER BY anchor in the ontology).
pub const MOTIF_ID: u16 = 0;
/// OCEL event code emitted when the motif fires.
pub const EVENT_CODE: u16 = 2000;

use crate::rays::{bishop_attacks, queen_attacks, rook_attacks, KING_MASKS, KNIGHT_MASKS};

/// Full attack span of one color: slider rays (Kogge-Stone over `empty`) plus
/// knight/king leaper masks. Private (loops), outside the CC=1 boundary.
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

/// Hanging squares for the side to move: own pieces hit by enemy attacks and
/// not defended by a friendly attack. Private (calls looping spans).
#[inline]
pub(super) fn hanging_mask(v: &PositionView) -> u64 {
    let stm = v.stm;
    let enemy = stm ^ 1;
    v.by_color[stm] & attack_span(v, enemy) & !attack_span(v, stm)
}

/// Branchless tactical detector for **hanging**.
///
/// Returns the bitmask of board squares that participate in the motif for the
/// side to move (`v.stm`). Pure delegation to private looping helpers; the
/// public body is a single expression (CC = 1).
#[must_use]
#[inline(always)]
pub fn detect(v: &PositionView) -> u64 {
    hanging_mask(v)
}

/// Whether the motif fired at all (all-ones if any square set, else zero).
/// Branchless: CC = 1.
#[must_use]
#[inline(always)]
pub fn fired(v: &PositionView) -> u64 {
    let m = hanging_mask(v); (((m | m.wrapping_neg()) >> 63) & 1).wrapping_neg()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    extern crate std;
    use super::*;
    use proptest::prelude::*;

    /// Structurally INDEPENDENT branchful reference oracle for the **hanging**
    /// law. Test-only, MAY branch; `detect` MUST match it on every position.
    /// Authority: fired == (stm_pieces & enemy_attacks & !stm_attacks)
    fn reference_detect(v: &PositionView) -> u64 {
        // Independent oracle: recompute each color's full attack set square by
        // square via per-piece masks, then test every stm square for the law.
        fn span(v: &PositionView, color: usize) -> u64 {
            use crate::rays::{bishop_attacks, queen_attacks, rook_attacks, KING_MASKS, KNIGHT_MASKS};
            let mut out = 0u64;
            let mut sq = 0usize;
            while sq < 64 {
                let bit = 1u64 << sq;
                if v.by_piece[color][ROOK] & bit != 0 { out |= rook_attacks(bit, v.empty); }
                if v.by_piece[color][BISHOP] & bit != 0 { out |= bishop_attacks(bit, v.empty); }
                if v.by_piece[color][QUEEN] & bit != 0 { out |= queen_attacks(bit, v.empty); }
                if v.by_piece[color][KNIGHT] & bit != 0 { out |= KNIGHT_MASKS[sq]; }
                if v.by_piece[color][KING] & bit != 0 { out |= KING_MASKS[sq]; }
                sq += 1;
            }
            out
        }
        let stm = v.stm;
        let enemy = stm ^ 1;
        let atk = span(v, enemy);
        let def = span(v, stm);
        let mut fired = 0u64;
        let mut sq = 0usize;
        while sq < 64 {
            let bit = 1u64 << sq;
            let mine = v.by_color[stm] & bit != 0;
            let attacked = atk & bit != 0;
            let defended = def & bit != 0;
            if mine && attacked && !defended {
                fired |= bit;
            }
            sq += 1;
        }
        fired
    }

    /// Build a [`PositionView`] from a chess board reached by a short random
    /// legal game, so every test position is legal (no impossible bitboards).
    #[cfg(feature = "std")]
    fn legal_position(seed: u64) -> PositionView {
        use chess::{Board, MoveGen};
        let mut board = Board::default();
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
        let mut ply = 0;
        while ply < (seed % 40) as usize + 4 {
            let gen = MoveGen::new_legal(&board);
            let n = gen.len();
            if n == 0 {
                break;
            }
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let pick = (state as usize) % n;
            let mv = MoveGen::new_legal(&board).nth(pick).unwrap();
            board = board.make_move_new(mv);
            ply += 1;
        }
        PositionView::from_board(&board)
    }

    #[cfg(feature = "std")]
    proptest! {
        /// Kernel == independent oracle over legal positions reached by random play.
        #[test]
        fn prop_kernel_matches_oracle(seed in any::<u64>()) {
            let v = legal_position(seed);
            prop_assert_eq!(detect(&v), reference_detect(&v), "fen-seed {}", seed);
        }
    }

    #[test]
    fn empty_position_never_fires() {
        let v = PositionView::default();
        assert_eq!(detect(&v), 0);
        assert_eq!(reference_detect(&v), 0);
        assert_eq!(fired(&v), 0);
    }
}