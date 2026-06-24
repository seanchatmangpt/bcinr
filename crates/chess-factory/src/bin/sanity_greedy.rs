//! `sanity_greedy` — a tiny seeded UCI engine that grabs the most material.
//!
//! Same minimal UCI subset as `sanity_random`, but instead of a random pick it
//! chooses the legal move that captures the highest-value piece (centipawn
//! material values lifted from `playground/src/nnue.rs`). Ties are broken by a
//! seeded mixer so play is deterministic. No quiescence, no recapture analysis
//! — a one-ply material grabber, a slightly stronger sanity floor than random.

use std::io::{self, BufRead, Write};
use std::str::FromStr;

use chess::{Board, ChessMove, MoveGen, Piece};

/// Centipawn material values (Pawn..King), matching nnue.rs.
fn piece_value(p: Piece) -> i32 {
    match p {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20_000,
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn board_from_position(rest: &str) -> Board {
    let (mut board, after) = if let Some(tail) = rest.strip_prefix("startpos") {
        (Board::default(), tail.trim_start())
    } else if let Some(tail) = rest.strip_prefix("fen ") {
        let (fen, mv_tail) = match tail.find(" moves ") {
            Some(i) => (&tail[..i], &tail[i + 1..]),
            None => (tail, ""),
        };
        let b = Board::from_str(fen.trim()).unwrap_or_default();
        (b, mv_tail)
    } else {
        (Board::default(), "")
    };

    if let Some(moves) = after.strip_prefix("moves ") {
        for tok in moves.split_whitespace() {
            if let Ok(mv) = ChessMove::from_str(tok) {
                if board.legal(mv) {
                    board = board.make_move_new(mv);
                }
            }
        }
    }
    board
}

/// Material captured by `mv` on `board` (0 for a quiet/non-capture move).
fn capture_gain(board: &Board, mv: ChessMove) -> i32 {
    match board.piece_on(mv.get_dest()) {
        Some(p) => piece_value(p),
        // No piece on the destination: still a capture if it's en passant.
        None => {
            let is_ep = board.piece_on(mv.get_source()) == Some(Piece::Pawn)
                && Some(mv.get_dest()) == board.en_passant();
            if is_ep {
                piece_value(Piece::Pawn)
            } else {
                0
            }
        }
    }
}

/// Choose the highest-material-capturing legal move (seeded tiebreak).
fn pick_move(board: &Board, seed: u64) -> String {
    let legal: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if legal.is_empty() {
        return "0000".to_string();
    }
    let mut state = seed ^ board.get_hash();
    let mut best = legal[0];
    let mut best_gain = i64::MIN;
    for mv in legal {
        // Promote the gain into a 64-bit key whose low bits are a seeded jitter,
        // so equal-material moves get a deterministic-but-varied tiebreak.
        let jitter = (splitmix64(&mut state) & 0xFFFF) as i64;
        let key = (capture_gain(board, mv) as i64) << 20 | jitter;
        if key > best_gain {
            best_gain = key;
            best = mv;
        }
    }
    best.to_string()
}

fn main() {
    let seed: u64 = std::env::var("SANITY_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x1234_5678_9ABC_DEF0);

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut board = Board::default();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim();

        if line == "uci" {
            writeln!(stdout, "id name sanity_greedy").unwrap();
            writeln!(stdout, "id author bcinr-chess-factory").unwrap();
            writeln!(stdout, "uciok").unwrap();
        } else if line == "isready" {
            writeln!(stdout, "readyok").unwrap();
        } else if line == "ucinewgame" {
            board = Board::default();
        } else if let Some(rest) = line.strip_prefix("position ") {
            board = board_from_position(rest);
        } else if line.starts_with("go") {
            writeln!(stdout, "bestmove {}", pick_move(&board, seed)).unwrap();
        } else if line == "quit" {
            break;
        }
        stdout.flush().unwrap();
    }
}
