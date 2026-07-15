//! Branchless Bitboard Chess Engine mapping for Binarized GNN evaluation.
//!
//! Enforces CC=1 by resolving piece movements as 64-bit integer physics.

use crate::{
    gnn::{hoeg_gnn_forward, BinarizedGnnLayer},
    hoeg::Hoeg64Node,
};

/// Represents the physical state of a chess board squeezed into a 64-byte boundary.
/// 64 squares = 64 bits = 1 `u64`.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessBitboard {
    /// Bitboard of every square occupied by a white piece.
    pub white_pieces: u64,
    /// Bitboard of every square occupied by a black piece.
    pub black_pieces: u64,
    /// Side to move: 0 = White, 1 = Black.
    pub turn: u16,
    /// Padding to reach the 64-byte cache-line-aligned layout.
    pub _pad: [u8; 46],
}

const _: () = assert!(core::mem::size_of::<ChessBitboard>() == 64);
const _: () = assert!(core::mem::align_of::<ChessBitboard>() == 64);

impl ChessBitboard {
    /// Initializes a standard chess starting position (simplified).
    /// Ranks 1 & 2 for White, Ranks 7 & 8 for Black.
    pub fn starting_position() -> Self {
        Self {
            white_pieces: 0x0000_0000_0000_FFFF, // First 16 squares
            black_pieces: 0xFFFF_0000_0000_0000, // Last 16 squares
            turn: 0,
            _pad: [0; 46],
        }
    }

    /// Generates forward pseudo-legal pawn pushes for White simultaneously.
    /// CC=1 logic: Left-shift all white pawns by 8 squares, mask with empty squares.
    #[inline(always)]
    pub fn white_pawn_pushes(&self, white_pawns: u64) -> u64 {
        let empty_squares = !(self.white_pieces | self.black_pieces);
        (white_pawns << 8) & empty_squares
    }
}

/// Evaluates a board state using the Binarized Graph Neural Network.
/// Maps the `ChessBitboard` to a `Hoeg64Node` natively, computing the
/// advantage score without any branching logic.
///
/// # Example
/// ```
/// use playground::chess::{ChessBitboard, evaluate_board_branchless};
/// use playground::gnn::BinarizedGnnLayer;
///
/// let board = ChessBitboard::starting_position();
/// // A synthetic "brain" evaluating the state
/// let neural_layer = BinarizedGnnLayer { weights: [0xFFFF_0000_0000_FFFF; 64], bias: 0 };
///
/// let advantage = evaluate_board_branchless(&board, &neural_layer).unwrap();
///
/// // The board was mathematically evaluated in O(1) time
/// assert_eq!(advantage > 0, true);
/// ```
#[inline(always)]
pub fn evaluate_board_branchless(
    board: &ChessBitboard,
    neural_layer: &BinarizedGnnLayer,
) -> Result<u64, &'static str> {
    // Map the chess state dynamically into the HOEG node framework
    let node_representation = Hoeg64Node {
        feature_mask: board.white_pieces,
        adjacency_mask: board.black_pieces, // Opponent acts as the structural adversary
        node_id: board.turn,
        node_type_hash: 0, // "BoardState" identifier
        _pad: [0; 44],
    };

    let mut score_buffer = [0u64; 1];
    let nodes = [node_representation];

    // Fire the binarized neural network (0 heap allocations, CC=1)
    hoeg_gnn_forward(&nodes, neural_layer, &mut score_buffer)?;

    Ok(score_buffer[0])
}
