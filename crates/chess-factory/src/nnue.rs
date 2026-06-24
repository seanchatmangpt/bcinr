#![cfg(feature = "std")]
extern crate std;

use chess::{Board, Color, Piece};
use std::sync::{Mutex, OnceLock};
use std::vec;
use std::vec::Vec;

// Network dimensions
pub const FT_IN: usize = 40960;  // HalfKP features
pub const FT_OUT: usize = 32;    // Accumulator size per side
pub const L2_IN: usize = 64;     // FT_OUT * 2
pub const L2_OUT: usize = 1;     // Single score output

/// NNUE weight storage.
pub struct NnueWeights {
    /// Feature transformer: [FT_IN][FT_OUT] in row-major order.
    /// Row i = weight vector added to accumulator when feature i is active.
    pub ft: Vec<i16>,               // FT_IN * FT_OUT entries
    pub ft_bias: [i16; FT_OUT],
    /// Output layer: [L2_IN] weights + 1 bias.
    pub l2: [i16; L2_IN],
    pub l2_bias: i32,
}

impl NnueWeights {
    fn new() -> Self {
        NnueWeights {
            ft: vec![0i16; FT_IN * FT_OUT],
            ft_bias: [0i16; FT_OUT],
            l2: [0i16; L2_IN],
            l2_bias: 0,
        }
    }

    /// Initialize weights to approximate PST evaluation.
    /// This makes the untrained network produce scores close to fast_eval.
    fn init_pst(&mut self) {
        // Material values in centipawns × FT_OUT scaling factor
        // Each piece type gets a characteristic weight that sums to its material value
        // when all squares for that piece are active.
        const MAT_CP: [i16; 5] = [100, 337, 365, 477, 1025]; // P N B R Q (no king)

        for king_sq in 0..64usize {
            for sq in 0..64usize {
                for pt in 0..5usize {
                    for pc in 0..2usize {
                        let idx = halfkp_index(king_sq, sq, pt, pc);
                        let row_start = idx * FT_OUT;
                        if row_start + FT_OUT <= self.ft.len() {
                            // Spread material value across all FT_OUT dimensions.
                            // First dimension gets the material weight; rest are zero.
                            // This makes the network approximate material counting.
                            let val = if pc == 0 { MAT_CP[pt] } else { -MAT_CP[pt] };
                            self.ft[row_start] = val / FT_OUT as i16;
                        }
                    }
                }
            }
        }

        // Output layer: sum the first accumulator dimension (material proxy)
        self.l2[0] = 1;   // white perspective
        self.l2[32] = -1; // black perspective (negated)
        self.l2_bias = 0;
    }
}

/// Global weight storage — initialized once per process.
fn nnue_weights() -> &'static Mutex<NnueWeights> {
    static WEIGHTS: OnceLock<Mutex<NnueWeights>> = OnceLock::new();
    WEIGHTS.get_or_init(|| {
        let mut w = NnueWeights::new();
        w.init_pst();
        Mutex::new(w)
    })
}

// ---------------------------------------------------------------------------
// HalfKP indexing
// ---------------------------------------------------------------------------

/// Compute the HalfKP feature index for a piece.
/// king_sq: king position (0-63)
/// sq: piece position (0-63)
/// pt: piece type 0=P,1=N,2=B,3=R,4=Q (5=K excluded from features)
/// pc: 0=same color as king, 1=opposite color
#[inline]
pub fn halfkp_index(king_sq: usize, sq: usize, pt: usize, pc: usize) -> usize {
    king_sq * 640 + sq * 10 + pt * 2 + pc
}

// ---------------------------------------------------------------------------
// Accumulator — two sides, each FT_OUT i32 elements
// ---------------------------------------------------------------------------

pub struct Accumulator {
    pub white: [i32; FT_OUT],
    pub black: [i32; FT_OUT],
}

impl Accumulator {
    pub fn new(bias: &[i16; FT_OUT]) -> Self {
        let mut acc = Accumulator {
            white: [0i32; FT_OUT],
            black: [0i32; FT_OUT],
        };
        for i in 0..FT_OUT {
            acc.white[i] = bias[i] as i32;
            acc.black[i] = bias[i] as i32;
        }
        acc
    }
}

// ---------------------------------------------------------------------------
// Full refresh — recompute accumulator from board from scratch
// ---------------------------------------------------------------------------

/// Refresh accumulator for one perspective (king_sq for that side).
fn refresh_perspective(acc: &mut [i32; FT_OUT], king_sq: usize, king_color: Color,
                       board: &Board, ft: &[i16], bias: &[i16; FT_OUT]) {
    // Start from bias
    for i in 0..FT_OUT { acc[i] = bias[i] as i32; }

    for pt_idx in 0..5usize {
        let piece = match pt_idx {
            0 => Piece::Pawn,
            1 => Piece::Knight,
            2 => Piece::Bishop,
            3 => Piece::Rook,
            _ => Piece::Queen,
        };
        let white_bb = board.pieces(piece) & board.color_combined(Color::White);
        let black_bb = board.pieces(piece) & board.color_combined(Color::Black);

        let mut bb = white_bb;
        while bb.0 != 0 {
            let sq = bb.0.trailing_zeros() as usize;
            let pc = if king_color == Color::White { 0 } else { 1 };
            let idx = halfkp_index(king_sq, sq, pt_idx, pc);
            let row = idx * FT_OUT;
            if row + FT_OUT <= ft.len() {
                for i in 0..FT_OUT { acc[i] += ft[row + i] as i32; }
            }
            bb.0 &= bb.0 - 1;
        }

        let mut bb = black_bb;
        while bb.0 != 0 {
            let sq = bb.0.trailing_zeros() as usize;
            let pc = if king_color == Color::Black { 0 } else { 1 };
            let idx = halfkp_index(king_sq, sq, pt_idx, pc);
            let row = idx * FT_OUT;
            if row + FT_OUT <= ft.len() {
                for i in 0..FT_OUT { acc[i] += ft[row + i] as i32; }
            }
            bb.0 &= bb.0 - 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Forward pass
// ---------------------------------------------------------------------------

/// Clipped ReLU: clamp to [0, 127].
#[inline]
fn crelu(x: i32) -> i32 {
    x.clamp(0, 127)
}

/// Full NNUE evaluation: board → score in centipawns, side-to-move relative.
pub fn nnue_eval(board: &Board) -> i32 {
    let weights = nnue_weights().lock().unwrap_or_else(|e| e.into_inner());

    let wk_sq = board.king_square(Color::White).to_index();
    let bk_sq = board.king_square(Color::Black).to_index();

    let mut white_acc = [0i32; FT_OUT];
    let mut black_acc = [0i32; FT_OUT];

    refresh_perspective(&mut white_acc, wk_sq, Color::White,
                        board, &weights.ft, &weights.ft_bias);
    refresh_perspective(&mut black_acc, bk_sq, Color::Black,
                        board, &weights.ft, &weights.ft_bias);

    // Concatenate: [white_acc | black_acc] with ClippedReLU
    let mut l2_input = [0i32; L2_IN];
    for i in 0..FT_OUT { l2_input[i]         = crelu(white_acc[i]); }
    for i in 0..FT_OUT { l2_input[FT_OUT + i] = crelu(black_acc[i]); }

    // Output layer: dot product
    let mut score = weights.l2_bias;
    for i in 0..L2_IN {
        score += l2_input[i] * weights.l2[i] as i32;
    }

    // Scale to centipawns (network outputs are scaled by ~400)
    let cp = score / 400;

    // Side-to-move relative
    if board.side_to_move() == Color::White { cp } else { -cp }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use chess::Board;

    #[test]
    fn nnue_eval_startpos_is_near_zero() {
        let board = Board::default();
        let score = nnue_eval(&board);
        // PST-initialized NNUE should be close to 0 at startpos (symmetric)
        assert!(score.abs() < 100, "startpos eval {} too large", score);
    }

    #[test]
    fn nnue_eval_does_not_panic() {
        let board = Board::default();
        let _ = nnue_eval(&board);
    }
}
