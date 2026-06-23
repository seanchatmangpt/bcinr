#![no_std]

/// Branchless Incremental NNUE Accumulator (Stockfish Technique)
/// Instead of re-evaluating the whole board ($O(N)$), we apply an $O(1)$ branchless update.

pub struct Accumulator {
    pub hidden: [i32; 16], // L1 Activation
}

impl Accumulator {
    /// Branchless update: When a piece moves, we subtract its old position weights 
    /// and add its new position weights.
    #[inline(always)]
    pub fn update_branchless(
        &mut self, 
        old_sq: usize, 
        new_sq: usize, 
        piece_type_idx: usize, 
        l1_weights: &[[i32; 768]; 16] // [hidden_size][pieces * squares]
    ) {
        let old_feature_idx = piece_type_idx * 64 + old_sq;
        let new_feature_idx = piece_type_idx * 64 + new_sq;

        // $CC=1$ Unrolled parallel vector update
        // On Apple Silicon, this auto-vectorizes to NEON SIMD instructions
        for i in 0..16 {
            self.hidden[i] = self.hidden[i] 
                - l1_weights[i][old_feature_idx] 
                + l1_weights[i][new_feature_idx];
        }
    }
}

/// Branchless Zobrist Hashing for Transposition Tables
pub struct Zobrist {
    pub hash: u64,
}

impl Zobrist {
    /// Update the hash strictly branchlessly using XOR polynomials
    #[inline(always)]
    pub fn update_branchless(
        &mut self,
        old_sq: usize,
        new_sq: usize,
        piece_idx: usize,
        zobrist_table: &[u64; 768] // 12 pieces * 64 squares
    ) {
        let old_key = piece_idx * 64 + old_sq;
        let new_key = piece_idx * 64 + new_sq;
        
        // XOR out the old piece position, XOR in the new piece position
        self.hash ^= zobrist_table[old_key] ^ zobrist_table[new_key];
    }
}
