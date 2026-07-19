#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::unwrap_used,
    clippy::inline_always,
    clippy::too_many_lines,
    clippy::if_same_then_else,
    clippy::needless_range_loop,
)]
//! Multi-threaded variant of `nnue_1sec_drill`: runs the same one-second NNUE
//! evaluation drill across multiple worker threads and reports aggregate
//! evaluations/sec.
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use playground::{
    legal_moves::{bishop_attacks, queen_attacks, rook_attacks, KING_MASKS, KNIGHT_MASKS},
    nnue::BranchTorchNNUE,
};
use rayon::prelude::*;

#[derive(Clone, Copy)]
struct Piece {
    kind: u8,
    color: u8,
    alive: bool,
    square: usize,
}

#[inline(always)]
fn generate_legal_targets(piece: &Piece, empty_mask: u64, enemy_mask: u64, own_mask: u64) -> u64 {
    let sq_mask = 1u64 << piece.square;
    let mut targets = 0u64;

    if piece.kind == 0 {
        if piece.color == 0 {
            let single = (sq_mask << 8) & empty_mask;
            targets |= single;
            if (piece.square / 8) == 1 && single != 0 {
                targets |= (sq_mask << 16) & empty_mask;
            }
            targets |= (sq_mask << 7) & 0x7f7f_7f7f_7f7f_7f7f & enemy_mask;
            targets |= (sq_mask << 9) & 0xfefe_fefe_fefe_fefe & enemy_mask;
        } else {
            let single = (sq_mask >> 8) & empty_mask;
            targets |= single;
            if (piece.square / 8) == 6 && single != 0 {
                targets |= (sq_mask >> 16) & empty_mask;
            }
            targets |= (sq_mask >> 9) & 0x7f7f_7f7f_7f7f_7f7f & enemy_mask;
            targets |= (sq_mask >> 7) & 0xfefe_fefe_fefe_fefe & enemy_mask;
        }
    } else if piece.kind == 1 {
        targets = KNIGHT_MASKS[piece.square];
    } else if piece.kind == 5 {
        targets = KING_MASKS[piece.square];
    } else if piece.kind == 3 {
        targets = rook_attacks(sq_mask, empty_mask);
    } else if piece.kind == 2 {
        targets = bishop_attacks(sq_mask, empty_mask);
    } else if piece.kind == 4 {
        targets = queen_attacks(sq_mask, empty_mask);
    }

    targets & !own_mask
}

#[inline(always)]
fn get_bitboards(pieces: &[Piece; 32]) -> [u64; 12] {
    let mut bb = [0u64; 12];
    for p in pieces {
        if p.alive {
            bb[(p.color * 6 + p.kind) as usize] |= 1u64 << p.square;
        }
    }
    bb
}

#[inline(always)]
fn branchless_static_evaluation(pieces: &[Piece; 32]) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;
    let vals = [100, 320, 330, 500, 900, 20000];
    for p in pieces {
        if p.alive {
            let val = vals[p.kind as usize];
            if p.color == 0 {
                white_score += val;
            } else {
                black_score += val;
            }
        }
    }
    white_score - black_score
}

fn main() {
    let initial_setup = [
        (3, 0, 0),
        (1, 0, 1),
        (2, 0, 2),
        (4, 0, 3),
        (5, 0, 4),
        (2, 0, 5),
        (1, 0, 6),
        (3, 0, 7),
        (0, 0, 8),
        (0, 0, 9),
        (0, 0, 10),
        (0, 0, 11),
        (0, 0, 12),
        (0, 0, 13),
        (0, 0, 14),
        (0, 0, 15),
        (3, 1, 56),
        (1, 1, 57),
        (2, 1, 58),
        (4, 1, 59),
        (5, 1, 60),
        (2, 1, 61),
        (1, 1, 62),
        (3, 1, 63),
        (0, 1, 48),
        (0, 1, 49),
        (0, 1, 50),
        (0, 1, 51),
        (0, 1, 52),
        (0, 1, 53),
        (0, 1, 54),
        (0, 1, 55),
    ];

    let mut start_pieces = [Piece { kind: 0, color: 0, alive: false, square: 0 }; 32];
    for i in 0..32 {
        start_pieces[i] = Piece {
            kind: initial_setup[i].0,
            color: initial_setup[i].1,
            alive: true,
            square: initial_setup[i].2,
        };
    }

    let start_time = Instant::now();
    let games_played = AtomicUsize::new(0);
    let moves_played = AtomicUsize::new(0);
    let nodes_evaluated = AtomicUsize::new(0);

    // Run parallel iterations mapping to CPU cores
    (0..1_000_000).into_par_iter().for_each(|_| {
        if start_time.elapsed().as_secs_f64() >= 1.0 {
            return;
        }

        let mut nnue = BranchTorchNNUE::new();
        let mut pieces = start_pieces;
        let mut turn_counter = 1;
        let mut game_over = false;
        let mut local_moves = 0;
        let mut local_nodes = 0;

        while !game_over && turn_counter <= 100 {
            let is_white = turn_counter % 2 != 0;
            let color_filter = u8::from(!is_white);

            let bb = get_bitboards(&pieces);
            let mut white_mask = 0u64;
            let mut black_mask = 0u64;
            for i in 0..6 {
                white_mask |= bb[i];
                black_mask |= bb[i + 6];
            }

            let empty_mask = !(white_mask | black_mask);
            let own_mask = if is_white { white_mask } else { black_mask };
            let enemy_mask = if is_white { black_mask } else { white_mask };

            let mut best_score = if is_white { i32::MIN } else { i32::MAX };
            let mut best_move = (99, 99);
            let mut move_found = false;

            for idx in 0..32 {
                let p = &pieces[idx];
                if p.alive && p.color == color_filter {
                    let mut t = generate_legal_targets(p, empty_mask, enemy_mask, own_mask);
                    while t != 0 {
                        let sq = t.trailing_zeros() as usize;
                        t &= t - 1;

                        local_nodes += 1;
                        let mut test_pieces = pieces;
                        for v_idx in 0..32 {
                            if test_pieces[v_idx].alive
                                && test_pieces[v_idx].square == sq
                                && test_pieces[v_idx].color != color_filter
                            {
                                test_pieces[v_idx].alive = false;
                                break;
                            }
                        }
                        test_pieces[idx].square = sq;

                        let test_bb = get_bitboards(&test_pieces);
                        let (nnue_pred, hidden, activated) = nnue.forward(&test_bb);
                        let shannon_target = branchless_static_evaluation(&test_pieces);
                        nnue.backprop(&test_bb, hidden, activated, nnue_pred, shannon_target);

                        if !move_found {
                            best_score = shannon_target;
                            best_move = (idx, sq);
                            move_found = true;
                        } else if is_white && shannon_target > best_score {
                            best_score = shannon_target;
                            best_move = (idx, sq);
                        } else if !is_white && shannon_target < best_score {
                            best_score = shannon_target;
                            best_move = (idx, sq);
                        }
                    }
                }
            }

            if !move_found {
                break;
            }

            let target_square = best_move.1;
            for v_idx in 0..32 {
                if pieces[v_idx].alive && pieces[v_idx].square == target_square {
                    pieces[v_idx].alive = false;
                    if pieces[v_idx].kind == 5 {
                        game_over = true;
                    }
                    break;
                }
            }
            pieces[best_move.0].square = target_square;

            turn_counter += 1;
            local_moves += 1;
        }

        games_played.fetch_add(1, Ordering::Relaxed);
        moves_played.fetch_add(local_moves, Ordering::Relaxed);
        nodes_evaluated.fetch_add(local_nodes, Ordering::Relaxed);
    });

    let nps = nodes_evaluated.load(Ordering::Relaxed);
    println!("--- M3 MAX MULTI-CORE 1-SECOND NNUE DISTILLATION ---");
    println!("Total Training Games Played: {}", games_played.load(Ordering::Relaxed));
    println!("Total Plies (Half-Moves) Played: {}", moves_played.load(Ordering::Relaxed));
    println!("Total CPU NNUE Nodes Evaluated: {nps}");
    println!("CPU Nodes per second (NPS): {nps}");
}
