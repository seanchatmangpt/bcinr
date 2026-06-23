#![no_std]

/// A simplified 768 -> 16 -> 1 NNUE architecture
pub struct BranchTorchNNUE {
    pub l1_weights: [[i32; 768]; 16],
    pub l1_biases: [i32; 16],
    pub l2_weights_value: [i32; 16],
    pub l2_weights_policy: [[i32; 64]; 16],
    pub l2_bias_value: i32,
    pub _pad: [i32; 3],
}

impl BranchTorchNNUE {
    pub const fn new() -> Self {
        let mut nnue = Self {
            l1_weights: [[0; 768]; 16],
            l1_biases: [0; 16],
            l2_weights_value: [0; 16],
            l2_weights_policy: [[0; 64]; 16],
            l2_bias_value: 0,
            _pad: [0; 3],
        };
        
        // Emulate classical Material and PST inside Hidden Node 0
        nnue.l2_weights_value[0] = 1;
        
        // Pawn, Knight, Bishop, Rook, Queen, King
        let piece_values = [100, 320, 330, 500, 900, 20000];
        
        let mut piece_idx = 0;
        while piece_idx < 6 {
            let mut sq = 0;
            while sq < 64 {
                let rank = sq / 8;
                let file = sq % 8;
                
                // Manhattan distance from center
                let center_dist_rank = if rank > 3 { rank - 3 } else { 4 - rank };
                let center_dist_file = if file > 3 { file - 3 } else { 4 - file };
                let center_dist = center_dist_rank + center_dist_file;
                
                let mut val = piece_values[piece_idx];
                
                // Piece-specific positional bonuses
                if piece_idx == 0 { // Pawn
                    // Bonus for advancing, but penalize a2-a4 reckless pushing
                    if rank >= 4 { val += (rank - 3) * 15; }
                    // Center pawns
                    if (file == 3 || file == 4) && rank >= 3 { val += 20; }
                } else if piece_idx == 1 { // Knight
                    // Knights hate the rim
                    val -= center_dist * 5;
                } else if piece_idx == 2 { // Bishop
                    // Bishops like center
                    val -= center_dist * 3;
                } else if piece_idx == 3 { // Rook
                    // Rooks like the 7th rank
                    if rank == 6 { val += 30; }
                } else if piece_idx == 5 { // King
                    // King safety: stay on 1st rank, heavily penalize moving to center in opening/middlegame
                    if rank > 0 { val -= 50 * rank; }
                    // Prefer castling positions (g1, c1)
                    if rank == 0 && (file == 6 || file == 2) { val += 30; }
                }
                
                nnue.l1_weights[0][piece_idx * 64 + sq] = val as i32;
                
                // Black pieces (mirror rank)
                let b_rank = 7 - rank;
                let b_center_dist_rank = if b_rank > 3 { b_rank - 3 } else { 4 - b_rank };
                let b_center_dist = b_center_dist_rank + center_dist_file;
                
                let mut b_val = piece_values[piece_idx];
                if piece_idx == 0 {
                    if b_rank >= 4 { b_val += (b_rank - 3) * 15; }
                    if (file == 3 || file == 4) && b_rank >= 3 { b_val += 20; }
                } else if piece_idx == 1 {
                    b_val -= b_center_dist * 5;
                } else if piece_idx == 2 {
                    b_val -= b_center_dist * 3;
                } else if piece_idx == 3 {
                    if b_rank == 6 { b_val += 30; }
                } else if piece_idx == 5 {
                    if b_rank > 0 { b_val -= 50 * b_rank; }
                    if b_rank == 0 && (file == 6 || file == 2) { b_val += 30; }
                }
                
                nnue.l1_weights[0][(piece_idx + 6) * 64 + sq] = -(b_val as i32);
                
                sq += 1;
            }
            piece_idx += 1;
        }
        nnue
    }
}
