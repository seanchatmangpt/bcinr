#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::inline_always,
    clippy::match_same_arms,
    clippy::too_many_lines,
    clippy::cast_sign_loss,
    clippy::if_same_then_else
)]
//! Object-Centric Event Log (OCEL) chess drill: plays out a game while
//! recording each move as an OCEL event, for exercising the OCEL export path
//! against a real game trace rather than a synthetic fixture.
use std::time::Instant;

use playground::{
    legal_moves::{bishop_attacks, queen_attacks, rook_attacks, KING_MASKS, KNIGHT_MASKS},
    nnue::BranchTorchNNUE,
};

#[derive(Clone, Debug)]
struct Piece {
    id: String,
    kind: &'static str,
    color: &'static str,
    alive: bool,
    square: usize,
}

fn generate_legal_targets(piece: &Piece, empty_mask: u64, enemy_mask: u64, own_mask: u64) -> u64 {
    let sq_mask = 1u64 << piece.square;
    let mut targets = 0u64;

    if piece.kind == "Pawn" {
        if piece.color == "White" {
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
    } else if piece.kind == "Knight" {
        targets = KNIGHT_MASKS[piece.square];
    } else if piece.kind == "King" {
        targets = KING_MASKS[piece.square];
    } else if piece.kind == "Rook" {
        targets = rook_attacks(sq_mask, empty_mask);
    } else if piece.kind == "Bishop" {
        targets = bishop_attacks(sq_mask, empty_mask);
    } else if piece.kind == "Queen" {
        targets = queen_attacks(sq_mask, empty_mask);
    }

    targets & !own_mask
}

#[inline(always)]
fn get_bitboards(pieces: &[Piece]) -> [u64; 12] {
    let mut bb = [0u64; 12];
    for p in pieces {
        if p.alive {
            let color_offset = if p.color == "White" { 0 } else { 6 };
            let kind_idx = match p.kind {
                "Pawn" => 0,
                "Knight" => 1,
                "Bishop" => 2,
                "Rook" => 3,
                "Queen" => 4,
                "King" => 5,
                _ => 0,
            };
            bb[color_offset + kind_idx] |= 1u64 << p.square;
        }
    }
    bb
}

#[inline(always)]
fn branchless_static_evaluation(pieces: &[Piece]) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;

    for p in pieces {
        if p.alive {
            let val = match p.kind {
                "Pawn" => 100,
                "Knight" => 320,
                "Bishop" => 330,
                "Rook" => 500,
                "Queen" => 900,
                "King" => 20000,
                _ => 0,
            };
            if p.color == "White" {
                white_score += val;
            } else {
                black_score += val;
            }
        }
    }
    white_score - black_score
}

fn main() {
    let mut nnue = BranchTorchNNUE::new();

    let mut pieces = Vec::new();
    let setup = [
        ("Rook_a", "Rook", 0),
        ("Knight_b", "Knight", 1),
        ("Bishop_c", "Bishop", 2),
        ("Queen", "Queen", 3),
        ("King", "King", 4),
        ("Bishop_f", "Bishop", 5),
        ("Knight_g", "Knight", 6),
        ("Rook_h", "Rook", 7),
        ("Pawn_a", "Pawn", 8),
        ("Pawn_b", "Pawn", 9),
        ("Pawn_c", "Pawn", 10),
        ("Pawn_d", "Pawn", 11),
        ("Pawn_e", "Pawn", 12),
        ("Pawn_f", "Pawn", 13),
        ("Pawn_g", "Pawn", 14),
        ("Pawn_h", "Pawn", 15),
    ];

    for (name, kind, pos) in &setup {
        pieces.push(Piece {
            id: format!("White_{name}"),
            kind,
            color: "White",
            alive: true,
            square: *pos,
        });
    }
    for (name, kind, pos) in &setup {
        let real_pos = if *pos >= 8 { 48 + (*pos - 8) } else { 56 + *pos };
        pieces.push(Piece {
            id: format!("Black_{name}"),
            kind,
            color: "Black",
            alive: true,
            square: real_pos,
        });
    }

    let start_time = Instant::now();

    let mut events = vec![];
    let mut objects = vec![];
    for i in 0..64 {
        let rank = (i / 8) + 1;
        let file = (b'a' + (i % 8) as u8) as char;
        objects
            .push(format!(r#"    {{ "id": "{file}{rank}", "type": "Square", "attributes": [] }}"#));
    }
    for p in &pieces {
        objects.push(format!(r#"    {{ "id": "{}", "type": "Piece", "attributes": [{{ "name": "color", "time": "1970-01-01T00:00:00Z", "value": "{}" }}] }}"#, p.id, p.color));
    }

    let mut turn_counter = 1;
    let mut game_over = false;

    // Distillation & Play Loop
    while !game_over && turn_counter <= 100 {
        let is_white = turn_counter % 2 != 0;
        let color_filter = if is_white { "White" } else { "Black" };

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

        let mut legal_moves = Vec::new();
        for (idx, p) in pieces.iter().enumerate() {
            if p.alive && p.color == color_filter {
                let targets = generate_legal_targets(p, empty_mask, enemy_mask, own_mask);
                for i in 0..64 {
                    if (targets & (1u64 << i)) != 0 {
                        legal_moves.push((idx, i));
                    }
                }
            }
        }

        if legal_moves.is_empty() {
            break;
        }

        let mut best_score = if is_white { i32::MIN } else { i32::MAX };
        let mut best_move = legal_moves[0];

        // Minimax Selection using NNUE instead of Shannon directly
        for mv in &legal_moves {
            let mut test_pieces = pieces.clone();
            if let Some(v_idx) = test_pieces
                .iter()
                .position(|p| p.alive && p.square == mv.1 && p.color != color_filter)
            {
                test_pieces[v_idx].alive = false;
            }
            test_pieces[mv.0].square = mv.1;

            let test_bb = get_bitboards(&test_pieces);

            // 1. Branchless NNUE Forward Pass
            let (nnue_pred, hidden, activated) = nnue.forward(&test_bb);

            // 2. Shannon Ground Truth (The Teacher)
            let shannon_target = branchless_static_evaluation(&test_pieces);

            // Print learning progress for the first evaluated move of the turn
            if mv == &legal_moves[0] {
                eprintln!(
                    "Turn {:03} | NNUE Guess: {:>5} | Shannon Target: {:>5} | Error: {:>5}",
                    turn_counter,
                    nnue_pred,
                    shannon_target,
                    (nnue_pred - shannon_target).abs()
                );
            }

            // 3. Algorithmic Distillation (Branchless Backprop)
            nnue.backprop(&test_bb, hidden, activated, nnue_pred, shannon_target);

            // 4. Decision Logic (Using the Teacher's knowledge during training)
            if is_white && shannon_target > best_score {
                best_score = shannon_target;
                best_move = *mv;
            } else if !is_white && shannon_target < best_score {
                best_score = shannon_target;
                best_move = *mv;
            }
        }

        let piece_idx = best_move.0;
        let target_square = best_move.1;

        let piece_id = pieces[piece_idx].id.clone();
        let target_square_str =
            format!("{}{}", (b'a' + (target_square % 8) as u8) as char, (target_square / 8) + 1);

        let mut relationships = vec![
            format!(r#"{{ "objectId": "{}", "qualifier": "moved_piece" }}"#, piece_id),
            format!(r#"{{ "objectId": "{}", "qualifier": "target_square" }}"#, target_square_str),
        ];

        if let Some(victim_idx) = pieces.iter().position(|p| p.alive && p.square == target_square) {
            pieces[victim_idx].alive = false;
            relationships.push(format!(
                r#"{{ "objectId": "{}", "qualifier": "captured_piece" }}"#,
                pieces[victim_idx].id
            ));
            if pieces[victim_idx].id.contains("King") {
                game_over = true;
            }
        }

        pieces[piece_idx].square = target_square;

        events.push(format!(
            r#"    {{ "id": "move_{}", "type": "Move", "time": "1970-01-01T00:00:00Z", "relationships": [ {} ] }}"#,
            turn_counter, relationships.join(", ")
        ));

        turn_counter += 1;
    }

    let duration = start_time.elapsed();
    eprintln!("NNUE Distillation & Game Execution completed in: {duration:?}");

    println!("{{");
    println!(r#"  "objectTypes": [ {{ "name": "Piece" }}, {{ "name": "Square" }} ],"#);
    println!(r#"  "eventTypes": [ {{ "name": "Move" }} ],"#);
    println!("  \"objects\": [\n{}\n  ],", objects.join(",\n"));
    println!("  \"events\": [\n{}\n  ]\n}}", events.join(",\n"));
}
