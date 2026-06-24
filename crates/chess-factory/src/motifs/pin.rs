
#![forbid(unsafe_code)]

//! Tactical motif: pin_detected (id 2).
//!
//! # Branchless Contract
//! **Ensures:** `detect` matches the independent branchful `_reference` oracle.
//! **Invariant:** the public execution path is data-independent (CC=1); all
//! looping detection is delegated to private `pub(super)` helpers.
//!
//! Lowering: Network. Primitive: `crate::rays::xray_attacks`.
//! Input mask: `xray_span_to_king`. Authority: fired == { enemy non-king p1 | a stm slider ray hits p1 then the enemy king p2 with no piece between }
//!
//! Pin: an enemy non-king piece p1 such that, walking a friendly slider's ray, p1 is the first piece and the enemy king is the second — p1 is pinned to its king. fired = pinned enemy-piece squares.

#[allow(unused_imports)]
use crate::position::{PositionView, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};

/// Stable ordinal identifier of this motif (ORDER BY anchor in the ontology).
pub const MOTIF_ID: u16 = 2;
/// OCEL event code emitted when the motif fires.
pub const EVENT_CODE: u16 = 2002;

/// The 8 ray steps as (file delta, rank delta). First 4 are orthogonal
/// (rook/queen), last 4 diagonal (bishop/queen).
pub(super) const DIRS: [(i32, i32); 8] = [
    (1, 0), (-1, 0), (0, 1), (0, -1),
    (1, 1), (1, -1), (-1, 1), (-1, -1),
];

/// Whether a (file, rank) delta direction is a rook line (orthogonal).
#[inline]
pub(super) fn is_orth(df: i32, dr: i32) -> bool {
    df == 0 || dr == 0
}

/// Walk from square `sq` in direction `dir` and return the first two occupied
/// squares encountered as `(p1, p2)` bit masks (0 if the ray runs off-board or
/// finds fewer than two pieces). Private (loops).
#[inline]
pub(super) fn first_two(v: &PositionView, sq: usize, dir: (i32, i32)) -> (u64, u64) {
    let (df, dr) = dir;
    let mut file = (sq % 8) as i32;
    let mut rank = (sq / 8) as i32;
    let mut found = [0u64; 2];
    let mut n = 0usize;
    loop {
        file += df;
        rank += dr;
        if !(0..8).contains(&file) || !(0..8).contains(&rank) {
            break;
        }
        let bit = 1u64 << (rank * 8 + file);
        if v.occ & bit != 0 {
            found[n] = bit;
            n += 1;
            if n == 2 {
                break;
            }
        }
    }
    (found[0], found[1])
}

/// stm slider bitmask matching a direction: rook+queen for orthogonal lines,
/// bishop+queen for diagonal lines.
#[inline]
pub(super) fn sliders_for(v: &PositionView, color: usize, orth: bool) -> u64 {
    if orth {
        v.by_piece[color][ROOK] | v.by_piece[color][QUEEN]
    } else {
        v.by_piece[color][BISHOP] | v.by_piece[color][QUEEN]
    }
}

/// Pinned enemy pieces: for each stm slider, walk its lines; if the first piece
/// is an enemy non-king and the second is the enemy king, the first is pinned.
#[inline]
pub(super) fn pin_mask(v: &PositionView) -> u64 {
    let stm = v.stm;
    let enemy = stm ^ 1;
    let enemy_king = v.by_piece[enemy][KING];
    let enemy_nonking = v.by_color[enemy] & !enemy_king;
    let mut fired = 0u64;
    let mut bb = v.by_piece[stm][ROOK] | v.by_piece[stm][BISHOP] | v.by_piece[stm][QUEEN];
    while bb != 0 {
        let bit = bb & bb.wrapping_neg();
        let sq = bit.trailing_zeros() as usize;
        let mut d = 0usize;
        while d < 8 {
            let dir = DIRS[d];
            let orth = is_orth(dir.0, dir.1);
            if sliders_for(v, stm, orth) & bit != 0 {
                let (p1, p2) = first_two(v, sq, dir);
                if p1 & enemy_nonking != 0 && p2 & enemy_king != 0 {
                    fired |= p1;
                }
            }
            d += 1;
        }
        bb &= bb - 1;
    }
    fired
}

/// Branchless tactical detector for **pin**.
///
/// Returns the bitmask of board squares that participate in the motif for the
/// side to move (`v.stm`). Pure delegation to private looping helpers; the
/// public body is a single expression (CC = 1).
#[must_use]
#[inline(always)]
pub fn detect(v: &PositionView) -> u64 {
    pin_mask(v)
}

/// Whether the motif fired at all (all-ones if any square set, else zero).
/// Branchless: CC = 1.
#[must_use]
#[inline(always)]
pub fn fired(v: &PositionView) -> u64 {
    let m = pin_mask(v); (((m | m.wrapping_neg()) >> 63) & 1).wrapping_neg()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    extern crate std;
    use super::*;
    use proptest::prelude::*;

    /// Structurally INDEPENDENT branchful reference oracle for the **pin**
    /// law. Test-only, MAY branch; `detect` MUST match it on every position.
    /// Authority: fired == { enemy non-king p1 | a stm slider ray hits p1 then the enemy king p2 with no piece between }
    fn reference_detect(v: &PositionView) -> u64 {
        // Independent oracle: anchor the walk at the ENEMY KING instead of the
        // slider. From the king, step outward in each direction; if the first
        // piece is an enemy non-king and the next is a stm slider of the
        // matching type, the first piece is pinned.
        let stm = v.stm;
        let enemy = stm ^ 1;
        let enemy_king_bb = v.by_piece[enemy][KING];
        if enemy_king_bb == 0 {
            return 0;
        }
        let ksq = enemy_king_bb.trailing_zeros() as usize;
        let kf = (ksq % 8) as i32;
        let kr = (ksq / 8) as i32;
        let dirs: [(i32, i32); 8] = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];
        let mut fired = 0u64;
        let mut d = 0usize;
        while d < 8 {
            let (df, dr) = dirs[d];
            let orth = df == 0 || dr == 0;
            let stm_sliders = if orth {
                v.by_piece[stm][ROOK] | v.by_piece[stm][QUEEN]
            } else {
                v.by_piece[stm][BISHOP] | v.by_piece[stm][QUEEN]
            };
            let mut f = kf;
            let mut r = kr;
            let mut first = 0u64;
            let mut second = 0u64;
            let mut n = 0;
            loop {
                f += df;
                r += dr;
                if !(0..8).contains(&f) || !(0..8).contains(&r) {
                    break;
                }
                let bit = 1u64 << (r * 8 + f);
                if v.occ & bit != 0 {
                    if n == 0 { first = bit; } else { second = bit; break; }
                    n += 1;
                }
            }
            let p1_enemy_nonking = first & v.by_color[enemy] != 0 && first & v.by_piece[enemy][KING] == 0;
            let p2_stm_slider = second & stm_sliders != 0;
            if p1_enemy_nonking && p2_stm_slider {
                fired |= first;
            }
            d += 1;
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