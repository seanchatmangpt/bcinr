#![allow(warnings, clippy::all)]
#![allow(warnings)]
//! Honest Elo measurement harness for the BCINR UCI engine.
//!
//! Plays a match of real games against Stockfish (capped to a known `UCI_Elo`),
//! adjudicating every game with the `chess` crate as an independent arbiter:
//! illegal moves forfeit, checkmate/stalemate are detected from the board
//! state, and a long game is adjudicated as a draw at the ply cap.
//!
//! The reported Elo is a *performance rating* derived from the actual score,
//! not a hardcoded constant.
//!
//! Usage: `bcinr_tournament [opponent_elo] [games] [bcinr_movetime_ms] [sf_go_mode]`
//!
//! `sf_go_mode` controls how Stockfish is queried for each move:
//!   - `depth1`     => `go depth 1`     (absolute fastest; ~58µs warmed up) [DEFAULT]
//!   - `fast`       => `go movetime 10`
//!   - `calibrated` => `go movetime 100`
//!
//! HONESTY CAVEAT: in `depth1` and `fast` modes Stockfish plays at a fixed,
//! search-limited strength. The `UCI_LimitStrength`/`UCI_Elo` setting becomes
//! largely irrelevant under a hard depth/movetime cap, so the reported
//! "performance Elo" is RELATIVE TO the fixed "Stockfish depth-1" reference
//! opponent, NOT to `opponent_elo`. Only `calibrated` mode gives Stockfish
//! enough time for the `UCI_Elo` throttle to actually anchor strength, making
//! the performance Elo meaningful against `opponent_elo`.

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use chess::{Board, BoardStatus, ChessMove, Color, File, Piece, Rank, Square};

const STOCKFISH: &str = "/opt/homebrew/bin/stockfish";
const BCINR: &str = "/Users/sac/bcinr/target/release/bcinr_uci";
const PLY_CAP: usize = 240;

/// Curated, roughly balanced opening lines (UCI moves from startpos). Deterministic
/// engines replay an identical game from a fixed start, so a varied book is the
/// ONLY way to obtain independent games. Each line is played twice (both colors)
/// to cancel any opening color bias.
const OPENINGS: &[&[&str]] = &[
    &[],                                               // startpos
    &["e2e4", "e7e5"],                                 // Open game
    &["e2e4", "c7c5"],                                 // Sicilian
    &["e2e4", "e7e6"],                                 // French
    &["e2e4", "c7c6"],                                 // Caro-Kann
    &["e2e4", "d7d5"],                                 // Scandinavian
    &["d2d4", "d7d5"],                                 // Closed d4
    &["d2d4", "g8f6"],                                 // Indian
    &["d2d4", "f7f5"],                                 // Dutch
    &["c2c4", "e7e5"],                                 // English
    &["g1f3", "d7d5"],                                 // Reti
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6"], // Ruy Lopez
    &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4"], // Open Sicilian
    &["d2d4", "g8f6", "c2c4", "e7e6"],                 // Nimzo/QID complex
    &["d2d4", "d7d5", "c2c4", "c7c6"],                 // Slav
    &["e2e4", "e7e5", "g1f3", "g8f6"],                 // Petroff
    &["c2c4", "g8f6", "b1c3", "e7e5"],                 // English four knights
    &["d2d4", "g8f6", "c2c4", "g7g6"],                 // King's Indian / Grünfeld
    &["e2e4", "g8f6"],                                 // Alekhine
    &["e2e4", "d7d6"],                                 // Pirc
    &["g1f3", "g8f6", "c2c4", "c7c5"],                 // Symmetric English
    &["d2d4", "e7e6", "c2c4", "f8b4"],                 // Nimzo-ish
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"],         // Italian
    &["d2d4", "d7d5", "g1f3", "g8f6"],                 // Symmetrical d4
    // --- extended book for >100-game samples (tighter CIs) ---
    &["e2e4", "c7c5", "b1c3"],                         // Closed Sicilian
    &["e2e4", "c7c5", "c2c3"],                         // Alapin
    &["e2e4", "e7e5", "g1f3", "b8c6", "d2d4"],         // Scotch
    &["e2e4", "e7e5", "f2f4"],                         // King's Gambit
    &["e2e4", "e7e6", "d2d4", "d7d5", "b1c3"],         // French Winawer setup
    &["e2e4", "c7c6", "d2d4", "d7d5", "b1c3"],         // Caro main
    &["d2d4", "g8f6", "c2c4", "c7c5"],                 // Benoni
    &["d2d4", "g8f6", "c2c4", "e7e6", "g1f3"],         // QID
    &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5"], // Grünfeld
    &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7"], // King's Indian
    &["d2d4", "d7d5", "c2c4", "e7e6"],                 // QGD
    &["d2d4", "d7d5", "c2c4", "d5c4"],                 // QGA
    &["c2c4", "c7c5"],                                 // Symmetrical English
    &["c2c4", "e7e6", "g1f3", "d7d5"],                 // English -> Catalan
    &["g1f3", "g8f6", "g2g3"],                         // King's Indian Attack
    &["d2d4", "f7f5", "g2g3"],                         // Leningrad Dutch
    &["e2e4", "d7d5", "e4d5", "d8d5", "b1c3"],         // Scandinavian main
    &["e2e4", "g8f6", "e4e5", "f6d5"],                 // Alekhine main
    &["e2e4", "d7d6", "d2d4", "g8f6", "b1c3"],         // Pirc main
    &["b2b3"],                                         // Larsen
    &["g2g3", "d7d5"],                                 // Hypermodern
    &["e2e4", "e7e5", "g1f3", "g8f6", "f3e5"],         // Petroff main
    &["e2e4", "c7c5", "g1f3", "b8c6"],                 // Sicilian ...Nc6
    &["e2e4", "c7c5", "g1f3", "e7e6"],                 // Taimanov
    &["d2d4", "e7e6", "c2c4", "g8f6", "b1c3", "f8b4"], // Nimzo-Indian
    &["d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6"], // Slav main
    &["c2c4", "g8f6", "g2g3", "g7g6"],                 // English fianchetto
    &["e2e4", "e7e5", "b1c3"],                         // Vienna
];

struct Engine {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Engine {
    fn spawn(path: &str) -> Engine {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn {path}: {e}"));
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut e = Engine { child, stdin, stdout };
        e.send("uci");
        e.wait_for("uciok");
        e
    }

    fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{cmd}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn wait_for(&mut self, token: &str) {
        let mut line = String::new();
        while self.stdout.read_line(&mut line).unwrap() > 0 {
            if line.contains(token) {
                return;
            }
            line.clear();
        }
    }

    /// Ask for the best move in the given position; returns UCI string ("0000" if none).
    fn bestmove(&mut self, position_cmd: &str, go_cmd: &str) -> String {
        self.send(position_cmd);
        self.send(go_cmd);
        let mut line = String::new();
        while self.stdout.read_line(&mut line).unwrap() > 0 {
            if line.starts_with("bestmove") {
                return line.split_whitespace().nth(1).unwrap_or("0000").to_string();
            }
            line.clear();
        }
        "0000".to_string()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.send("quit");
        let _ = self.child.kill();
    }
}

/// Parse a UCI move string ("e2e4", "e7e8q") into a `ChessMove`.
fn parse_uci(s: &str) -> Option<ChessMove> {
    let b = s.as_bytes();
    if s.len() < 4 {
        return None;
    }
    let sq = |file: u8, rank: u8| -> Option<Square> {
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return None;
        }
        Some(Square::make_square(
            Rank::from_index((rank - b'1') as usize),
            File::from_index((file - b'a') as usize),
        ))
    };
    let from = sq(b[0], b[1])?;
    let to = sq(b[2], b[3])?;
    let promo = if s.len() >= 5 {
        match b[4] {
            b'q' => Some(Piece::Queen),
            b'r' => Some(Piece::Rook),
            b'b' => Some(Piece::Bishop),
            b'n' => Some(Piece::Knight),
            _ => None,
        }
    } else {
        None
    };
    Some(ChessMove::new(from, to, promo))
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Outcome {
    BcinrWin,
    StockfishWin,
    Draw,
}

/// Play one game. `bcinr_white` selects BCINR's color.
/// Returns the outcome from BCINR's perspective.
fn play_game(
    bcinr: &mut Engine,
    sf: &mut Engine,
    bcinr_white: bool,
    opening: &[&str],
    bcinr_go: &str,
    sf_go: &str,
    bcinr_secs: &mut f64,
    sf_secs: &mut f64,
) -> Outcome {
    bcinr.send("ucinewgame");
    sf.send("ucinewgame");
    let mut board = Board::default();
    let mut moves: Vec<String> = Vec::new();
    // Seed the curated opening line (book moves, not played by either engine).
    for &om in opening {
        if let Some(mv) = parse_uci(om) {
            if board.legal(mv) {
                board = board.make_move_new(mv);
                moves.push(om.to_string());
            }
        }
    }

    for ply in 0..PLY_CAP {
        let white_to_move = board.side_to_move() == Color::White;
        let bcinr_to_move = white_to_move == bcinr_white;

        let position_cmd = if moves.is_empty() {
            "position startpos".to_string()
        } else {
            format!("position startpos moves {}", moves.join(" "))
        };

        let t_move = std::time::Instant::now();
        let mv_str = if bcinr_to_move {
            let s = bcinr.bestmove(&position_cmd, bcinr_go);
            *bcinr_secs += t_move.elapsed().as_secs_f64();
            s
        } else {
            let s = sf.bestmove(&position_cmd, sf_go);
            *sf_secs += t_move.elapsed().as_secs_f64();
            s
        };

        let mv = match parse_uci(&mv_str) {
            Some(m) => m,
            None => {
                // No legal move offered (resign / "(none)"): mover loses.
                return if bcinr_to_move { Outcome::StockfishWin } else { Outcome::BcinrWin };
            }
        };

        // Independent legality check: an illegal move forfeits the game.
        if !board.legal(mv) {
            eprintln!(
                "  [ply {ply}] ILLEGAL move {mv_str} by {} -> forfeit",
                if bcinr_to_move { "BCINR" } else { "Stockfish" }
            );
            return if bcinr_to_move { Outcome::StockfishWin } else { Outcome::BcinrWin };
        }

        board = board.make_move_new(mv);
        moves.push(mv_str);

        match board.status() {
            BoardStatus::Checkmate => {
                // Side to move is checkmated == loser; the mover won.
                return if bcinr_to_move { Outcome::BcinrWin } else { Outcome::StockfishWin };
            }
            BoardStatus::Stalemate => return Outcome::Draw,
            BoardStatus::Ongoing => {}
        }
    }
    // Reached the ply cap without resolution: adjudicate as a draw.
    Outcome::Draw
}

/// Standard Elo performance-rating formula from a fractional score in (0,1).
fn performance_elo(opponent: f64, score: f64, n: usize) -> f64 {
    let p = score / n as f64;
    if p >= 1.0 {
        opponent + 800.0
    } else if p <= 0.0 {
        opponent - 800.0
    } else {
        opponent - 400.0 * (1.0 / p - 1.0).log10()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opponent_elo: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1800);
    let games: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let bcinr_ms: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(300);
    // DEFAULT to depth1: the user asked for Stockfish's absolute-fastest move.
    let sf_go_mode: String = args.get(4).cloned().unwrap_or_else(|| "depth1".to_string());

    // Resolve the Stockfish go-command. `go depth 1` is the absolute-fastest
    // meaningful query (~58µs warmed up). Note: under depth1/fast, the UCI_Elo
    // throttle is largely irrelevant (see module-level HONESTY CAVEAT).
    let (sf_go, sf_calibrated) = match sf_go_mode.as_str() {
        "depth1" => ("go depth 1".to_string(), false),
        "fast" => ("go movetime 10".to_string(), false),
        "calibrated" => ("go movetime 100".to_string(), true),
        // `depthN` / `nodesN` — raw fixed-strength Stockfish, used to find the
        // LOWEST level at which it still plays a decent (non-blundering) game.
        s if s.starts_with("depth") && s[5..].parse::<u32>().is_ok() => {
            (format!("go depth {}", &s[5..]), false)
        }
        s if s.starts_with("nodes") && s[5..].parse::<u64>().is_ok() => {
            (format!("go nodes {}", &s[5..]), false)
        }
        other => {
            eprintln!(
                "unknown sf_go_mode '{other}' (use depth1|fast|calibrated|depthN|nodesN); defaulting to depth1"
            );
            ("go depth 1".to_string(), false)
        }
    };

    println!("=== BCINR Elo Measurement (real arbiter via `chess` crate) ===");
    println!("Opponent: Stockfish @ UCI_Elo {opponent_elo}");
    println!("Games: {games}  |  BCINR movetime: {bcinr_ms}ms  |  ply cap: {PLY_CAP}");
    println!("Stockfish go-mode: {sf_go_mode} ({sf_go})");
    if sf_calibrated {
        println!(
            "Reference: CALIBRATED. Stockfish has time for UCI_Elo to anchor strength; \
             the performance Elo below is measured RELATIVE TO UCI_Elo {opponent_elo}."
        );
    } else {
        println!(
            "Reference: HONESTY CAVEAT -- under `{sf_go}` Stockfish plays at a fixed, \
             search-limited strength and the UCI_Elo {opponent_elo} throttle is largely \
             IRRELEVANT. The performance Elo below is RELATIVE TO the fixed \
             \"Stockfish depth-1\" reference opponent, NOT to UCI_Elo {opponent_elo}."
        );
    }

    // Allow overriding the BCINR-side engine (e.g. the GPU MCTS `bcinr_az`) so
    // both paradigms can be measured against the same Stockfish anchor.
    let bcinr_path = std::env::var("BCINR_ENGINE").unwrap_or_else(|_| BCINR.to_string());
    println!("BCINR engine: {bcinr_path}");
    let mut bcinr = Engine::spawn(&bcinr_path);
    let sf_path = std::env::var("STOCKFISH_ENGINE").unwrap_or_else(|_| STOCKFISH.to_string());
    println!("Opponent engine: {sf_path}");
    let mut sf = Engine::spawn(&sf_path);

    // Only throttle Stockfish by UCI_Elo in calibrated mode. For depthN/nodesN
    // (the speed-chess sweep) we want RAW Stockfish at a fixed shallow level.
    if sf_calibrated {
        sf.send("setoption name UCI_LimitStrength value true");
        sf.send(&format!("setoption name UCI_Elo value {opponent_elo}"));
    }
    sf.send("isready");
    sf.wait_for("readyok");

    // BCINR_GO overrides the go-command sent to our engine, e.g. "go depth 1"
    // for microsecond-scale fixed-depth moves (speed-parity mode).
    let bcinr_go = std::env::var("BCINR_GO").unwrap_or_else(|_| format!("go movetime {bcinr_ms}"));
    // SF_GO overrides Stockfish's go-command, e.g. "go movetime 5" for an EQUAL
    // time budget vs our engine (true speed chess against full-strength Stockfish).
    let sf_go = std::env::var("SF_GO").unwrap_or(sf_go);
    println!("BCINR go-command: {bcinr_go}  |  Opponent go-command: {sf_go}");

    // `games` selects how many distinct opening lines to use (capped to the book);
    // each line is played twice (both colors), so total games = 2 * n_openings.
    let n_openings = games.clamp(1, OPENINGS.len());
    let total = n_openings * 2;
    println!("Opening book: {n_openings} lines x 2 colors = {total} independent games\n");

    let (mut wins, mut draws, mut losses) = (0usize, 0usize, 0usize);
    let mut game_no = 0usize;
    let mut total_secs = 0.0f64;
    // Cumulative engine think-time and move counts, to compare per-move latency.
    let (mut bcinr_total, mut sf_total) = (0.0f64, 0.0f64);
    for line in OPENINGS.iter().take(n_openings) {
        for &bcinr_white in &[true, false] {
            game_no += 1;
            let (mut b_secs, mut s_secs) = (0.0f64, 0.0f64);
            let t0 = std::time::Instant::now();
            let outcome = play_game(
                &mut bcinr,
                &mut sf,
                bcinr_white,
                line,
                &bcinr_go,
                &sf_go,
                &mut b_secs,
                &mut s_secs,
            );
            let secs = t0.elapsed().as_secs_f64();
            total_secs += secs;
            bcinr_total += b_secs;
            sf_total += s_secs;
            let tag = if line.is_empty() { "startpos".to_string() } else { line.join(" ") };
            let r = match outcome {
                Outcome::BcinrWin => {
                    wins += 1;
                    "BCINR wins"
                }
                Outcome::StockfishWin => {
                    losses += 1;
                    "Stockfish wins"
                }
                Outcome::Draw => {
                    draws += 1;
                    "draw"
                }
            };
            println!(
                "  [{game_no}/{total}] BCINR {} | {tag} -> {r}  (BCINR {:.0}ms vs SF {:.0}ms think)",
                if bcinr_white { "W" } else { "B" },
                b_secs * 1000.0,
                s_secs * 1000.0,
            );
        }
    }
    println!(
        "\nTotal think time -- BCINR: {bcinr_total:.2}s  Stockfish: {sf_total:.2}s  (wall {total_secs:.1}s)"
    );

    let score = wins as f64 + 0.5 * draws as f64;
    let perf = performance_elo(f64::from(opponent_elo), score, total);
    // Wald 95% CI on the score rate (rough; assumes independent games).
    let p = score / total as f64;
    let se = (p * (1.0 - p) / total as f64).sqrt();
    let (lo, hi) = ((p - 1.96 * se).max(0.0), (p + 1.96 * se).min(1.0));

    println!("\n=== RESULT ===");
    println!("Record (BCINR): +{wins} ={draws} -{losses}  (score {score}/{total})");
    println!(
        "Score rate: {:.1}%  (95% CI {:.1}%..{:.1}%)  | avg game {:.2}s",
        p * 100.0,
        lo * 100.0,
        hi * 100.0,
        total_secs / total as f64
    );
    if score > 0.0 && score < games as f64 {
        println!("Measured performance Elo: ~{perf:.0}");
    } else if score >= games as f64 {
        println!(
            "BCINR scored 100% vs Elo {opponent_elo}: performance is >= {opponent_elo}; \
             raise opponent_elo to bracket it."
        );
    } else {
        println!(
            "BCINR scored 0% vs Elo {opponent_elo}: performance is <= {opponent_elo}; \
             lower opponent_elo to bracket it."
        );
    }
}
