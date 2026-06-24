//! Packed position view (HAND-AUTHORED boundary).
//!
//! `PositionView` projects a `chess::Board` into raw bitboards once, so that
//! station kernels stay branchless. `from_board` is the ONLY looping builder
//! and is intentionally not part of any CC=1 station kernel.

/// Piece index: pawn, knight, bishop, rook, queen, king.
pub const PAWN: usize = 0;
/// Knight index.
pub const KNIGHT: usize = 1;
/// Bishop index.
pub const BISHOP: usize = 2;
/// Rook index.
pub const ROOK: usize = 3;
/// Queen index.
pub const QUEEN: usize = 4;
/// King index.
pub const KING: usize = 5;

/// White color index.
pub const WHITE: usize = 0;
/// Black color index.
pub const BLACK: usize = 1;

/// A fully-projected, branchless-friendly view of a chess position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PositionView {
    /// All occupied squares.
    pub occ: u64,
    /// All empty squares (`!occ`).
    pub empty: u64,
    /// Occupancy per color: `[white, black]`.
    pub by_color: [u64; 2],
    /// Occupancy per `[color][piece]`.
    pub by_piece: [[u64; 6]; 2],
    /// Side to move: `WHITE` (0) or `BLACK` (1).
    pub stm: usize,
}

impl PositionView {
    /// Build a view from raw per-color, per-piece bitboards.
    ///
    /// This constructor may loop; it is the projection boundary, not a station.
    #[must_use]
    pub fn from_bitboards(by_piece: [[u64; 6]; 2], stm: usize) -> Self {
        let mut by_color = [0u64; 2];
        let mut color = 0;
        while color < 2 {
            let mut piece = 0;
            let mut acc = 0u64;
            while piece < 6 {
                acc |= by_piece[color][piece];
                piece += 1;
            }
            by_color[color] = acc;
            color += 1;
        }
        let occ = by_color[WHITE] | by_color[BLACK];
        Self {
            occ,
            empty: !occ,
            by_color,
            by_piece,
            stm,
        }
    }

    /// Project a `chess::Board` into a packed [`PositionView`].
    ///
    /// This is the std/`chess`-crate boundary: it loops over the 2x6 piece
    /// planes once and hands fully-packed bitboards to the branchless stations.
    /// It is the only `chess`-aware constructor and is not a CC=1 kernel.
    #[cfg(feature = "std")]
    #[must_use]
    pub fn from_board(board: &chess::Board) -> Self {
        use chess::{Color, Piece};
        let pieces = [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ];
        let colors = [Color::White, Color::Black];
        let mut by_piece = [[0u64; 6]; 2];
        let mut c = 0usize;
        while c < 2 {
            let mut p = 0usize;
            while p < 6 {
                by_piece[c][p] =
                    (*board.color_combined(colors[c]) & *board.pieces(pieces[p])).0;
                p += 1;
            }
            c += 1;
        }
        let stm = match board.side_to_move() {
            Color::White => WHITE,
            Color::Black => BLACK,
        };
        Self::from_bitboards(by_piece, stm)
    }
}
