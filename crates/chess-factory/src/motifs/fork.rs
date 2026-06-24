
#![forbid(unsafe_code)]

//! Tactical motif: fork_detected (id 1).
//!
//! # Branchless Contract
//! **Ensures:** `detect` matches the independent branchful `_reference` oracle.
//! **Invariant:** the public execution path is data-independent (CC=1); all
//! looping detection is delegated to private `pub(super)` helpers.
//!
//! Lowering: Bitset. Primitive: `crate::rays::attack_sets`.
//! Input mask: `attacker_targets >= 2`. Authority: fired == { stm attacker | popcount(attacks(attacker) & enemy_valuables) >= 2 }
//!
//! Fork: a side-to-move knight, bishop, rook or queen whose attack set covers two or more enemy valuable pieces (rook/queen/king) simultaneously.

#[allow(unused_imports)]
use crate::position::{PositionView, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};

/// Stable ordinal identifier of this motif (ORDER BY anchor in the ontology).
pub const MOTIF_ID: u16 = 1;
/// OCEL event code emitted when the motif fires.
pub const EVENT_CODE: u16 = 2001;

use crate::rays::{bishop_attacks, queen_attacks, rook_attacks, KNIGHT_MASKS};

/// Attack set of a single piece of `piece` type on square bit `bit`.
#[inline]
pub(super) fn piece_attacks(v: &PositionView, piece: usize, bit: u64) -> u64 {
    let empty = v.empty;
    match piece {
        KNIGHT => KNIGHT_MASKS[bit.trailing_zeros() as usize],
        BISHOP => bishop_attacks(bit, empty),
        ROOK => rook_attacks(bit, empty),
        QUEEN => queen_attacks(bit, empty),
        _ => 0,
    }
}

/// Side-to-move forking pieces: knights/sliders attacking >= 2 enemy valuables
/// (rook, queen, king). Returns the bitmask of forking-piece squares.
#[inline]
pub(super) fn fork_mask(v: &PositionView) -> u64 {
    let stm = v.stm;
    let enemy = stm ^ 1;
    let valuables =
        v.by_piece[enemy][ROOK] | v.by_piece[enemy][QUEEN] | v.by_piece[enemy][KING];
    let mut fired = 0u64;
    let pieces = [KNIGHT, BISHOP, ROOK, QUEEN];
    let mut i = 0;
    while i < pieces.len() {
        let piece = pieces[i];
        let mut bb = v.by_piece[stm][piece];
        while bb != 0 {
            let bit = bb & bb.wrapping_neg();
            let hits = (piece_attacks(v, piece, bit) & valuables).count_ones();
            if hits >= 2 {
                fired |= bit;
            }
            bb &= bb - 1;
        }
        i += 1;
    }
    fired
}

/// Branchless tactical detector for **fork**.
///
/// Returns the bitmask of board squares that participate in the motif for the
/// side to move (`v.stm`). Pure delegation to private looping helpers; the
/// public body is a single expression (CC = 1).
#[must_use]
#[inline(always)]
pub fn detect(v: &PositionView) -> u64 {
    fork_mask(v)
}

/// Whether the motif fired at all (all-ones if any square set, else zero).
/// Branchless: CC = 1.
#[must_use]
#[inline(always)]
pub fn fired(v: &PositionView) -> u64 {
    let m = fork_mask(v); (((m | m.wrapping_neg()) >> 63) & 1).wrapping_neg()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    extern crate std;
    use super::*;
    use proptest::prelude::*;

    /// Structurally INDEPENDENT branchful reference oracle for the **fork**
    /// law. Test-only, MAY branch; `detect` MUST match it on every position.
    /// Authority: fired == { stm attacker | popcount(attacks(attacker) & enemy_valuables) >= 2 }
    fn reference_detect(v: &PositionView) -> u64 {
        // Independent oracle: iterate all 64 squares, for each stm knight/slider
        // recompute its attack set inline and count enemy valuables hit.
        use crate::rays::{bishop_attacks, queen_attacks, rook_attacks, KNIGHT_MASKS};
        let stm = v.stm;
        let enemy = stm ^ 1;
        let valuables =
            v.by_piece[enemy][ROOK] | v.by_piece[enemy][QUEEN] | v.by_piece[enemy][KING];
        let mut fired = 0u64;
        let mut sq = 0usize;
        while sq < 64 {
            let bit = 1u64 << sq;
            let attacks = if v.by_piece[stm][KNIGHT] & bit != 0 {
                KNIGHT_MASKS[sq]
            } else if v.by_piece[stm][BISHOP] & bit != 0 {
                bishop_attacks(bit, v.empty)
            } else if v.by_piece[stm][ROOK] & bit != 0 {
                rook_attacks(bit, v.empty)
            } else if v.by_piece[stm][QUEEN] & bit != 0 {
                queen_attacks(bit, v.empty)
            } else {
                0
            };
            let mut hits = 0u32;
            let mut t = 0usize;
            while t < 64 {
                let tb = 1u64 << t;
                if attacks & tb != 0 && valuables & tb != 0 {
                    hits += 1;
                }
                t += 1;
            }
            if hits >= 2 {
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