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
    /// Branchless forward pass: 12 bitboards -> (value, hidden, activated)
    pub fn forward(&self, bb: &[u64; 12]) -> (i32, [i32; 16], [i32; 16]) {
        let mut hidden = [0i32; 16];
        // Accumulate L1: weight[neuron][feature] * feature_active
        for neuron in 0..16 {
            let mut acc = self.l1_biases[neuron];
            for piece in 0..12 {
                let mut bits = bb[piece];
                while bits != 0 {
                    let sq = bits.trailing_zeros() as usize;
                    acc = acc.wrapping_add(self.l1_weights[neuron][piece * 64 + sq]);
                    bits &= bits - 1;
                }
            }
            hidden[neuron] = acc;
        }
        // ReLU activation (branchless: max(0, x) = (x + |x|) >> 1 for positive range)
        let mut activated = [0i32; 16];
        for i in 0..16 {
            activated[i] = hidden[i].max(0);
        }
        // L2 value head
        let mut value = self.l2_bias_value;
        for i in 0..16 {
            value = value.wrapping_add(activated[i].wrapping_mul(self.l2_weights_value[i]));
        }
        (value, hidden, activated)
    }

    /// Branchless SGD backprop step (learning rate 1/256 approximated with shift)
    pub fn backprop(
        &mut self,
        bb: &[u64; 12],
        _hidden: [i32; 16],
        activated: [i32; 16],
        pred: i32,
        target: i32,
    ) {
        let err = pred.wrapping_sub(target);
        // Update L2 weights
        for i in 0..16 {
            let grad = err.wrapping_mul(activated[i]) >> 8;
            self.l2_weights_value[i] = self.l2_weights_value[i].wrapping_sub(grad);
        }
        // Update L1 weights (only for active ReLU neurons)
        for neuron in 0..16 {
            if activated[neuron] > 0 {
                let l2_w = self.l2_weights_value[neuron];
                let delta = err.wrapping_mul(l2_w) >> 8;
                for piece in 0..12 {
                    let mut bits = bb[piece];
                    while bits != 0 {
                        let sq = bits.trailing_zeros() as usize;
                        self.l1_weights[neuron][piece * 64 + sq] =
                            self.l1_weights[neuron][piece * 64 + sq].wrapping_sub(delta);
                        bits &= bits - 1;
                    }
                }
            }
        }
    }

    pub const fn new() -> Self {
        let mut nnue = Self {
            l1_weights: [[0; 768]; 16],
            l1_biases: [0; 16],
            l2_weights_value: [0; 16],
            l2_weights_policy: [[0; 64]; 16],
            l2_bias_value: 0,
            _pad: [0; 3],
        };

        // Emulate classical Material and PST inside Hidden Node 0.
        nnue.l2_weights_value[0] = 1;

        // Pawn, Knight, Bishop, Rook, Queen, King base material values.
        let piece_values = [100, 320, 330, 500, 900, 20000];

        // Textbook middlegame piece-square tables (Simplified Evaluation
        // Function / PeSTO-style), in centipawns, ADDED to base material.
        //
        // IMPORTANT INDEXING: each table is written rank-8-first (the way the
        // tables appear in the literature, white's perspective). Our board
        // index is `sq = rank * 8 + file` with a1 = sq 0 (rank 0 = white back
        // rank). So table row `r` (0 = rank 8 .. 7 = rank 1) maps to board rank
        // `7 - r`. We index the table with `(7 - rank) * 8 + file`.
        let pst: [[i32; 64]; 6] = [
            // Pawn: reward central advancement, discourage edge pawns.
            [
                0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10,
                10, 5, 5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10,
                -5, 5, 5, 10, 10, -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            // Knight: "on the rim is dim" — strong rim penalty, central reward.
            [
                -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10,
                15, 15, 10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30,
                -30, 5, 10, 15, 15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30,
                -30, -30, -40, -50,
            ],
            // Bishop: long diagonals / central, slight back-rank penalty.
            [
                -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10,
                10, 5, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10,
                10, 10, 10, 10, 10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10,
                -10, -10, -20,
            ],
            // Rook: reward 7th rank and central files, slight a/h penalty.
            [
                0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5,
                0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
                0, 0, 0, 0, -5, 0, 0, 0, 5, 5, 0, 0, 0,
            ],
            // Queen: small central preference, discourage early sorties.
            [
                -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5,
                5, 0, -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5,
                0, -10, -10, 0, 5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
            ],
            // King (middlegame): reward castled corners (g1/c1), punish center.
            [
                -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
                -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
                -20, -30, -30, -40, -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20,
                20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0, 10, 30, 20,
            ],
        ];

        let mut piece_idx = 0;
        while piece_idx < 6 {
            let mut sq = 0;
            while sq < 64 {
                let rank = sq / 8;
                let file = sq % 8;

                // White: PST row 0 is rank 8, so board rank `rank` -> table row
                // `7 - rank`.
                let w_pst = pst[piece_idx][(7 - rank) * 8 + file];
                let val = piece_values[piece_idx] + w_pst;
                nnue.l1_weights[0][piece_idx * 64 + sq] = val as i32;

                // Black: a black piece on board `sq` mirrors to the white PST
                // value of the rank-flipped square (rank -> 7 - rank), then is
                // negated (white-relative signed eval).
                let b_rank = 7 - rank;
                let b_pst = pst[piece_idx][(7 - b_rank) * 8 + file];
                let b_val = piece_values[piece_idx] + b_pst;
                nnue.l1_weights[0][(piece_idx + 6) * 64 + sq] = -(b_val as i32);

                sq += 1;
            }
            piece_idx += 1;
        }

        // Mirror-neuron trick so the output ReLU can represent SIGNED evaluations.
        //
        // The GPU value head computes `sum_h relu(hidden[h]) * l2_weights_value[h]`.
        // With a single neuron, `relu(e)` clamps every losing position to 0. By
        // adding neuron 1 = -neuron 0 and weighting it -1, the head computes
        // `relu(e) - relu(-e) = e` for all e: a faithful signed score, still fully
        // branchless. (This is the classic split-ReLU identity for signed linear
        // units.)
        let mut k = 0;
        while k < 768 {
            nnue.l1_weights[1][k] = -nnue.l1_weights[0][k];
            k += 1;
        }
        nnue.l1_biases[1] = -nnue.l1_biases[0];
        nnue.l2_weights_value[0] = 1;
        nnue.l2_weights_value[1] = -1;

        nnue
    }
}
