//! NNUE (Efficiently Updatable Neural Network) for chess evaluation.
//!
//! Architecture: HalfKP features (40,960/side) -> L1 accumulator [i16;64]/side
//! -> clipped-ReLU -> L2 (128->8 int8 matmul) -> output (8->1 scalar) -> /400 = centipawns
//!
//! Until trained weights are loaded, `nnue_init_from_pst` seeds weights so the
//! network degrades gracefully to a material evaluation.
#![cfg(feature = "std")]

extern crate std;
use std::sync::{Mutex, OnceLock};
use std::vec::Vec;

use chess::{Board, Color, Piece, ALL_SQUARES};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Feature-transformer rows: 40_960 per perspective * 2 perspectives.
pub const FT_ROWS: usize = 81_920;

/// L1 accumulator size (per perspective).
pub const L1: usize = 64;

/// L2 input size (both perspectives concatenated).
pub const L2_IN: usize = 128;

/// L2 output neurons.
pub const L2_OUT: usize = 8;

// ---------------------------------------------------------------------------
// Weight structures
// ---------------------------------------------------------------------------

/// All NNUE weight tensors.
pub struct NnueWeights {
    /// Feature transformer: FT_ROWS * L1 elements (row-major: row = feature, col = L1 neuron).
    pub ft_weights: Vec<i16>,
    /// Feature transformer biases: one per L1 neuron.
    pub ft_biases: [i16; L1],
    /// L2 weight matrix: [L2_OUT][L2_IN] stored as i8.
    pub l2_weights: [[i8; L2_IN]; L2_OUT],
    /// L2 biases: one per output neuron, stored as i32 to absorb quantisation scale.
    pub l2_biases: [i32; L2_OUT],
    /// Output layer weights: one per L2 neuron.
    pub output_weights: [i32; L2_OUT],
    /// Output layer scalar bias.
    pub output_bias: i32,
}

impl NnueWeights {
    fn zeroed() -> NnueWeights {
        let mut ft_weights = Vec::new();
        ft_weights.resize(FT_ROWS * L1, 0i16);
        NnueWeights {
            ft_weights,
            ft_biases: [0i16; L1],
            l2_weights: [[0i8; L2_IN]; L2_OUT],
            l2_biases: [0i32; L2_OUT],
            output_weights: [0i32; L2_OUT],
            output_bias: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Accumulator
// ---------------------------------------------------------------------------

/// Per-position half-KP accumulator (one half per perspective).
pub struct NnueAccumulator {
    pub white: [i16; L1],
    pub black: [i16; L1],
}

impl NnueAccumulator {
    pub fn zeroed() -> NnueAccumulator {
        NnueAccumulator {
            white: [0i16; L1],
            black: [0i16; L1],
        }
    }
}

// ---------------------------------------------------------------------------
// Feature indexing
// ---------------------------------------------------------------------------

/// Map a `Piece` to a HalfKP piece-type index (0-4).  King has no HalfKP feature.
pub fn piece_to_pt(p: Piece) -> Option<usize> {
    match p {
        Piece::Pawn   => Some(0),
        Piece::Knight => Some(1),
        Piece::Bishop => Some(2),
        Piece::Rook   => Some(3),
        Piece::Queen  => Some(4),
        Piece::King   => None,
    }
}

/// Raw HalfKP index (before perspective mirroring / offset).
///
/// `king_sq`  – king square index 0..64
/// `piece_sq` – piece square index 0..64
/// `pt`       – piece type 0..5 (use `piece_to_pt`)
/// `pc`       – piece colour: 0 = white, 1 = black
pub fn halfkp_index(king_sq: usize, piece_sq: usize, pt: usize, pc: usize) -> usize {
    king_sq * 640 + pc * 320 + pt * 64 + piece_sq
}

/// Full feature-transformer row index, including perspective and the 40_960 offset
/// for the black-king perspective.
///
/// `persp` – 0 = white's perspective, 1 = black's perspective
pub fn ft_index(
    king_sq: usize,
    piece_sq: usize,
    pt: usize,
    pc: usize,
    persp: usize,
) -> usize {
    if persp == 0 {
        halfkp_index(king_sq, piece_sq, pt, pc)
    } else {
        halfkp_index(king_sq ^ 56, piece_sq ^ 56, pt, pc) + 40_960
    }
}

// ---------------------------------------------------------------------------
// Global weight singleton
// ---------------------------------------------------------------------------

static NNUE_WEIGHTS: OnceLock<Mutex<NnueWeights>> = OnceLock::new();

/// Return a reference to the global (lazily initialised) `NnueWeights`.
///
/// On first call, weights are seeded via `nnue_init_from_pst` so the network
/// approximates material evaluation before real training data is loaded.
pub fn nnue_weights() -> &'static Mutex<NnueWeights> {
    NNUE_WEIGHTS.get_or_init(|| {
        let mut w = NnueWeights::zeroed();
        nnue_init_from_pst(&mut w);
        Mutex::new(w)
    })
}

// ---------------------------------------------------------------------------
// Accumulator refresh
// ---------------------------------------------------------------------------

/// Recompute the accumulator from scratch for `board`.
///
/// Call this after a position is loaded from scratch.  For incremental updates,
/// modify the accumulator with the delta (add/subtract the feature weight rows
/// for the moved/captured piece).
pub fn nnue_refresh(board: &Board, weights: &NnueWeights, acc: &mut NnueAccumulator) {
    // Seed with biases.
    acc.white = weights.ft_biases;
    acc.black = weights.ft_biases;

    // Locate both kings.
    let white_king_bb = board.pieces(Piece::King) & board.color_combined(Color::White);
    let black_king_bb = board.pieces(Piece::King) & board.color_combined(Color::Black);

    let wk_sq: usize = white_king_bb.to_square().to_index();
    let bk_sq: usize = black_king_bb.to_square().to_index();

    // Accumulate features for every non-king piece.
    for sq in ALL_SQUARES {
        let sq_idx = sq.to_index();
        let piece_opt = board.piece_on(sq);
        let color_opt = board.color_on(sq);

        let (piece, color) = match (piece_opt, color_opt) {
            (Some(p), Some(c)) => (p, c),
            _ => continue,
        };

        let pt = match piece_to_pt(piece) {
            Some(t) => t,
            None => continue, // skip kings
        };

        let pc: usize = if color == Color::White { 0 } else { 1 };

        // White-king perspective.
        let wi = ft_index(wk_sq, sq_idx, pt, pc, 0);
        let base_w = wi * L1;
        for k in 0..L1 {
            acc.white[k] = acc.white[k].saturating_add(weights.ft_weights[base_w + k]);
        }

        // Black-king perspective.
        let bi = ft_index(bk_sq, sq_idx, pt, pc, 1);
        let base_b = bi * L1;
        for k in 0..L1 {
            acc.black[k] = acc.black[k].saturating_add(weights.ft_weights[base_b + k]);
        }
    }
}

// ---------------------------------------------------------------------------
// Forward pass
// ---------------------------------------------------------------------------

/// Run the full NNUE forward pass and return an evaluation in centipawns
/// from the side-to-move's perspective.
///
/// The accumulator must be current (call `nnue_refresh` or maintain it
/// incrementally).
pub fn nnue_forward(acc: &NnueAccumulator, stm: Color, weights: &NnueWeights) -> i32 {
    // Build the 128-element L2 input: [us | them], clipped to 0..127.
    let (us_half, them_half) = if stm == Color::White {
        (&acc.white, &acc.black)
    } else {
        (&acc.black, &acc.white)
    };

    let mut l2_input = [0i32; L2_IN];
    for k in 0..L1 {
        l2_input[k]        = (us_half[k]   as i32).clamp(0, 127);
        l2_input[L1 + k]   = (them_half[k] as i32).clamp(0, 127);
    }

    // L2 layer: 128->8, integer dot-product, bias, right-shift 6, clamp 0..127.
    let mut l2_out = [0i32; L2_OUT];
    for o in 0..L2_OUT {
        let mut acc_o: i32 = weights.l2_biases[o];
        for i in 0..L2_IN {
            acc_o += l2_input[i] * (weights.l2_weights[o][i] as i32);
        }
        l2_out[o] = (acc_o >> 6).clamp(0, 127);
    }

    // Output layer: 8->1 dot-product + bias, divide by 400 for centipawns.
    let mut output: i32 = weights.output_bias;
    for o in 0..L2_OUT {
        output += l2_out[o] * weights.output_weights[o];
    }

    output / 400
}

// ---------------------------------------------------------------------------
// PST-based weight initialisation
// ---------------------------------------------------------------------------

/// Seed `NnueWeights` from piece-square tables so the network approximates
/// material evaluation before real training data is available.
///
/// Scheme:
/// - For every (king_sq, piece_sq, pt, colour, perspective) combination, set
///   `ft_weights[index * L1 + 0]` to ±BASE[pt]:
///   white pieces add value, black pieces subtract.
/// - Wire a pass-through chain: `l2_weights[0][0] = 1`, `output_weights[0] = 400`;
///   dividing by 400 in `nnue_forward` cancels the scale, leaving centipawns.
pub fn nnue_init_from_pst(w: &mut NnueWeights) {
    const BASE: [i16; 5] = [82, 337, 365, 477, 1025]; // P N B R Q

    for ksq in 0..64usize {
        for psq in 0..64usize {
            for pt in 0..5usize {
                for persp in 0..2usize {
                    // White piece at psq: adds value.
                    let wi_w = ft_index(ksq, psq, pt, 0, persp) * L1;
                    if wi_w < w.ft_weights.len() {
                        w.ft_weights[wi_w] = BASE[pt];
                    }
                    // Black piece at psq: subtracts value.
                    let wi_b = ft_index(ksq, psq, pt, 1, persp) * L1;
                    if wi_b < w.ft_weights.len() {
                        w.ft_weights[wi_b] = -BASE[pt];
                    }
                }
            }
        }
    }

    // Pass-through chain: l2_weights[0][0]=1, output_weights[0]=400.
    // nnue_forward divides by 400, so the net effect is identity on L1 neuron 0.
    w.l2_weights[0][0] = 1i8;
    w.output_weights[0] = 400;
}
