//! `sanity_random` — a tiny seeded UCI engine: the sanity floor.
//!
//! It speaks the minimal UCI subset the benchmark runner needs:
//!
//!     uci            -> id/uciok
//!     isready        -> readyok
//!     ucinewgame     -> (ignored)
//!     position startpos [moves ...]   -> rebuild the board
//!     position fen <FEN> [moves ...]  -> rebuild the board
//!     go ...         -> bestmove <uci>   (a SEEDED pick of a legal move)
//!     quit           -> exit
//!
//! Determinism: the move index is a function of a fixed seed mixed with the
//! number of plies played so far, so a given game replays identically. No
//! search, no evaluation — this is deliberately the weakest lawful opponent.

use std::io::{self, BufRead, Write};
use std::str::FromStr;

use chess::{Board, ChessMove, MoveGen};

/// SplitMix64 step — a fast, well-distributed seeded mixer (no external dep).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Rebuild the board from a `position ...` command, applying any `moves`.
fn board_from_position(rest: &str) -> Board {
    let (mut board, after) = if let Some(tail) = rest.strip_prefix("startpos") {
        (Board::default(), tail.trim_start())
    } else if let Some(tail) = rest.strip_prefix("fen ") {
        // Split off an optional " moves ..." suffix.
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

/// Pick the seeded legal move for `board`. Returns "0000" if no legal move.
fn pick_move(board: &Board, seed: u64) -> String {
    let legal: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if legal.is_empty() {
        return "0000".to_string();
    }
    // Mix the seed with the position hash so the choice is position-dependent
    // yet fully reproducible from the fixed seed.
    let mut state = seed ^ board.get_hash();
    let idx = (splitmix64(&mut state) % legal.len() as u64) as usize;
    legal[idx].to_string()
}

fn main() {
    let seed: u64 = std::env::var("SANITY_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0xC0FF_EE12_3456_789A);

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
            writeln!(stdout, "id name sanity_random").unwrap();
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
