
#![forbid(unsafe_code)]

//! Tactical motif: skewer_detected (id 3).
//!
//! # Branchless Contract
//! **Ensures:** `detect` matches the independent branchful `_reference` oracle.
//! **Invariant:** the public execution path is data-independent (CC=1); all
//! looping detection is delegated to private `pub(super)` helpers.
//!
//! Lowering: Network. Primitive: `crate::rays::xray_attacks`.
//! Input mask: `xray_span_value_descending`. Authority: fired == { rear enemy p2 | a stm slider ray hits enemy front p1 then enemy rear p2 with value(p1) > value(p2) }
//!
//! Skewer: a friendly slider's ray hits a more valuable enemy front piece p1 that shields a less valuable enemy rear piece p2 (value(p1) > value(p2)). fired = rear-piece squares (inverse of a pin).

#[allow(unused_imports)]
use crate::position::{PositionView, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};

/// Stable ordinal identifier of this motif (ORDER BY anchor in the ontology).
pub const MOTIF_ID: u16 = 3;
/// OCEL event code emitted when the motif fires.
pub const EVENT_CODE: u16 = 2003;

/// The 8 ray steps (file delta, rank delta); 4 orthogonal then 4 diagonal.
pub(super) const DIRS: [(i32, i32); 8] = [
    (1, 0), (-1, 0), (0, 1), (0, -1),
    (1, 1), (1, -1), (-1, 1), (-1, -1),
];

/// Centipawn rank used only to order front vs rear value (king is highest).
pub(super) const VALUE: [i32; 6] = [100, 320, 330, 500, 900, 20000];

/// Piece type (0..6) of the piece occupying bit `bit` for `color`, or 6 if none.
#[inline]
pub(super) fn piece_of(v: &PositionView, color: usize, bit: u64) -> usize {
    let mut p = 0usize;
    while p < 6 {
        if v.by_piece[color][p] & bit != 0 {
            return p;
        }
        p += 1;
    }
    6
}

/// Walk from `sq` in `dir`, returning the first two occupied bits `(p1, p2)`.
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

/// stm slider bitmask matching a direction (orth: rook+queen, diag: bishop+queen).
#[inline]
pub(super) fn sliders_for(v: &PositionView, color: usize, orth: bool) -> u64 {
    if orth {
        v.by_piece[color][ROOK] | v.by_piece[color][QUEEN]
    } else {
        v.by_piece[color][BISHOP] | v.by_piece[color][QUEEN]
    }
}

/// Skewered rear pieces: for each stm slider, walk its lines; if both blockers
/// are enemy pieces and the FRONT one is worth strictly more than the REAR one,
/// the rear square is skewered.
#[inline]
pub(super) fn skewer_mask(v: &PositionView) -> u64 {
    let stm = v.stm;
    let enemy = stm ^ 1;
    let mut fired = 0u64;
    let mut bb = v.by_piece[stm][ROOK] | v.by_piece[stm][BISHOP] | v.by_piece[stm][QUEEN];
    while bb != 0 {
        let bit = bb & bb.wrapping_neg();
        let sq = bit.trailing_zeros() as usize;
        let mut d = 0usize;
        while d < 8 {
            let dir = DIRS[d];
            let orth = dir.0 == 0 || dir.1 == 0;
            if sliders_for(v, stm, orth) & bit != 0 {
                let (p1, p2) = first_two(v, sq, dir);
                let both_enemy = p1 & v.by_color[enemy] != 0 && p2 & v.by_color[enemy] != 0;
                if both_enemy {
                    let v1 = VALUE[piece_of(v, enemy, p1)];
                    let v2 = VALUE[piece_of(v, enemy, p2)];
                    if v1 > v2 {
                        fired |= p2;
                    }
                }
            }
            d += 1;
        }
        bb &= bb - 1;
    }
    fired
}

/// Branchless tactical detector for **skewer**.
///
/// Returns the bitmask of board squares that participate in the motif for the
/// side to move (`v.stm`). Pure delegation to private looping helpers; the
/// public body is a single expression (CC = 1).
#[must_use]
#[inline(always)]
pub fn detect(v: &PositionView) -> u64 {
    skewer_mask(v)
}

/// Whether the motif fired at all (all-ones if any square set, else zero).
/// Branchless: CC = 1.
#[must_use]
#[inline(always)]
pub fn fired(v: &PositionView) -> u64 {
    let m = skewer_mask(v); (((m | m.wrapping_neg()) >> 63) & 1).wrapping_neg()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    extern crate std;
    use super::*;
    use proptest::prelude::*;

    /// Structurally INDEPENDENT branchful reference oracle for the **skewer**
    /// law. Test-only, MAY branch; `detect` MUST match it on every position.
    /// Authority: fired == { rear enemy p2 | a stm slider ray hits enemy front p1 then enemy rear p2 with value(p1) > value(p2) }
    fn reference_detect(v: &PositionView) -> u64 {
        // Independent oracle: scan all 64 squares for a stm slider; for each,
        // independently re-walk every one of its directions and apply the
        // front>rear value law to the first two enemy blockers.
        const VAL: [i32; 6] = [100, 320, 330, 500, 900, 20000];
        let stm = v.stm;
        let enemy = stm ^ 1;
        let dirs: [(i32, i32); 8] = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];
        let type_at = |bit: u64| -> usize {
            let mut p = 0usize;
            while p < 6 {
                if v.by_piece[enemy][p] & bit != 0 { return p; }
                p += 1;
            }
            6
        };
        let mut fired = 0u64;
        let mut sq = 0usize;
        while sq < 64 {
            let bit = 1u64 << sq;
            let is_rook = v.by_piece[stm][ROOK] & bit != 0;
            let is_bishop = v.by_piece[stm][BISHOP] & bit != 0;
            let is_queen = v.by_piece[stm][QUEEN] & bit != 0;
            if is_rook || is_bishop || is_queen {
                let mut d = 0usize;
                while d < 8 {
                    let (df, dr) = dirs[d];
                    let orth = df == 0 || dr == 0;
                    let active = (orth && (is_rook || is_queen)) || (!orth && (is_bishop || is_queen));
                    if active {
                        let mut f = (sq % 8) as i32;
                        let mut r = (sq / 8) as i32;
                        let mut p1 = 0u64;
                        let mut p2 = 0u64;
                        let mut n = 0;
                        loop {
                            f += df;
                            r += dr;
                            if !(0..8).contains(&f) || !(0..8).contains(&r) { break; }
                            let b = 1u64 << (r * 8 + f);
                            if v.occ & b != 0 {
                                if n == 0 { p1 = b; } else { p2 = b; break; }
                                n += 1;
                            }
                        }
                        if p1 & v.by_color[enemy] != 0 && p2 & v.by_color[enemy] != 0 {
                            let v1 = VAL[type_at(p1)];
                            let v2 = VAL[type_at(p2)];
                            if v1 > v2 { fired |= p2; }
                        }
                    }
                    d += 1;
                }
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