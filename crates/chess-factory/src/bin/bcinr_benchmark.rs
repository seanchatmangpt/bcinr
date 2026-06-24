//! `bcinr_benchmark` — Decision-Engine Elo(tau) measurement harness.
//!
//! Lifts `Engine`/`play_game`/`performance_elo`/`OPENINGS`/Wald-CI from
//! `playground/src/bin/bcinr_tournament.rs`, but plays the FACTORY engine
//! IN-PROCESS (the generated `aggregator::aggregate` over a `PositionView`)
//! instead of spawning a separate UCI binary. Opponents are spawned UCI
//! processes (Stockfish, or the seeded `sanity_*` bins).
//!
//! It sweeps the opponent x budget cross product declared in
//! `schema/benchmark_matrix.json` (manufactured from `ontology/benchmark.ttl`),
//! and emits:
//!   - `artifacts/elo_curve.csv`  — one row per (opponent, budget) cell
//!   - `artifacts/elo_curve.json` — same data + an `elo_at_100us` field per
//!                                  opponent
//!   - `artifacts/benchmark.receipt.json` — a self-certifying run receipt
//!     (input matrix hash, output curve hash, seed, replay pointer).
//!
//! CLI (all optional):
//!   --opponent <name>   restrict the sweep to one opponent (default: all)
//!   --budget   <name>   restrict the sweep to one budget tier (default: all)
//!   --games    <n>      number of opening LINES (each played both colors)
//!
//! The factory's per-move "budget" here is a deterministic node bound (1-ply
//! over the legal move set), NOT wall-clock; the budget tier is recorded for
//! the curve axis. The headline metric is the factory's score rate / Elo at the
//! 100us tier, which against the seeded sanity opponents must show the factory
//! CRUSHING them (the pipeline-orientation check).

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::str::FromStr;
use std::time::Instant;

/// Median and p99 of a microsecond-latency sample. Returns (median, p99); both 0
/// when the sample is empty. Nearest-rank percentile on the sorted sample.
fn lat_stats(samples: &[u64]) -> (u64, u64) {
    if samples.is_empty() {
        return (0, 0);
    }
    let mut s = samples.to_vec();
    s.sort_unstable();
    let median = s[s.len() / 2];
    // nearest-rank p99: index ceil(0.99 * n) - 1
    let rank = (((s.len() as f64) * 0.99).ceil() as usize).max(1) - 1;
    let p99 = s[rank.min(s.len() - 1)];
    (median, p99)
}

use blake3::Hasher;
use chess::{Board, BoardStatus, ChessMove, Color, Piece};

use chess_factory::search::fixed_depth_best_move;

const PLY_CAP: usize = 240;
const STOCKFISH: &str = "/opt/homebrew/bin/stockfish";
/// Default opponent RNG seed (matches `sanity_random`/`sanity_greedy` defaults);
/// overridable via the `SANITY_SEED` env var. Captured into the run receipt so
/// every Crown cell's opponent stream is reproducible.
const SANITY_SEED_DEFAULT: u64 = 0xC0FF_EE12_3456_789A;
/// Fixed search depth the factory uses per move. Depth 4 fits the t1ms tier and
/// brings the factory to bcinr_uci-class play (alpha-beta + quiescence).
const FACTORY_DEPTH: usize = 4;

/// Curated, roughly balanced opening lines (UCI moves from startpos). Lifted
/// from `bcinr_tournament.rs`; each line is played twice (both colors).
const OPENINGS: &[&[&str]] = &[
    &[],
    &["e2e4", "e7e5"],
    &["e2e4", "c7c5"],
    &["e2e4", "e7e6"],
    &["e2e4", "c7c6"],
    &["e2e4", "d7d5"],
    &["d2d4", "d7d5"],
    &["d2d4", "g8f6"],
    &["d2d4", "f7f5"],
    &["c2c4", "e7e5"],
    &["g1f3", "d7d5"],
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6"],
    &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4"],
    &["d2d4", "g8f6", "c2c4", "e7e6"],
    &["d2d4", "d7d5", "c2c4", "c7c6"],
    &["e2e4", "e7e5", "g1f3", "g8f6"],
];

// ---------------------------------------------------------------------------
// Opponent UCI process wrapper (lifted from bcinr_tournament.rs `Engine`).
// ---------------------------------------------------------------------------

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
        e.send("isready");
        e.wait_for("readyok");
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

    /// Returns (bestmove, think_micros). think_micros is wall-clock from sending
    /// `go` to receiving `bestmove` — the opponent's MEASURED per-move latency.
    fn bestmove(&mut self, position_cmd: &str, go_cmd: &str) -> (String, u64) {
        self.send(position_cmd);
        let t0 = Instant::now();
        self.send(go_cmd);
        let mut line = String::new();
        while self.stdout.read_line(&mut line).unwrap() > 0 {
            if line.starts_with("bestmove") {
                let us = t0.elapsed().as_micros() as u64;
                let mv = line.split_whitespace().nth(1).unwrap_or("0000").to_string();
                return (mv, us);
            }
            line.clear();
        }
        (("0000").to_string(), t0.elapsed().as_micros() as u64)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.send("quit");
        let _ = self.child.kill();
    }
}

// ---------------------------------------------------------------------------
// Factory engine (in-process): 1-ply aggregate over the generated stations.
// ---------------------------------------------------------------------------

/// The factory's best move for `board`: manufactured alpha-beta + quiescence
/// search (the hand-authored `search` wrapper) over the generated aggregator.
fn factory_bestmove(board: &Board, depth: usize) -> Option<ChessMove> {
    fixed_depth_best_move(board, depth)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Outcome {
    FactoryWin,
    OpponentWin,
    Draw,
}

fn parse_uci(s: &str) -> Option<ChessMove> {
    ChessMove::from_str(s).ok()
}

/// Play one game: factory (in-process) vs a spawned opponent.
/// `factory_white` selects the factory's color. Returns the factory outcome.
fn play_game(
    opp: &mut Engine,
    factory_white: bool,
    opening: &[&str],
    opp_go: &str,
    factory_depth: usize,
    factory_us: &mut Vec<u64>,
    opp_us: &mut Vec<u64>,
) -> Outcome {
    opp.send("ucinewgame");
    let mut board = Board::default();
    let mut moves: Vec<String> = Vec::new();
    for &om in opening {
        if let Some(mv) = parse_uci(om) {
            if board.legal(mv) {
                board = board.make_move_new(mv);
                moves.push(om.to_string());
            }
        }
    }

    for _ply in 0..PLY_CAP {
        let white_to_move = board.side_to_move() == Color::White;
        let factory_to_move = white_to_move == factory_white;

        let mv = if factory_to_move {
            let t0 = Instant::now();
            let best = factory_bestmove(&board, factory_depth);
            factory_us.push(t0.elapsed().as_micros() as u64);
            match best {
                Some(m) => m,
                None => return Outcome::OpponentWin,
            }
        } else {
            let position_cmd = if moves.is_empty() {
                "position startpos".to_string()
            } else {
                format!("position startpos moves {}", moves.join(" "))
            };
            let (s, us) = opp.bestmove(&position_cmd, opp_go);
            opp_us.push(us);
            match parse_uci(&s) {
                Some(m) => m,
                None => return Outcome::FactoryWin,
            }
        };

        if !board.legal(mv) {
            return if factory_to_move {
                Outcome::OpponentWin
            } else {
                Outcome::FactoryWin
            };
        }

        board = board.make_move_new(mv);
        moves.push(mv.to_string());

        match board.status() {
            BoardStatus::Checkmate => {
                return if factory_to_move {
                    Outcome::FactoryWin
                } else {
                    Outcome::OpponentWin
                };
            }
            BoardStatus::Stalemate => return Outcome::Draw,
            BoardStatus::Ongoing => {}
        }
    }
    // Ply-cap reached: adjudicate by material (a standard arbiter rule). A
    // decisive material lead (>= a minor piece) is scored as a win for the
    // leader; otherwise a draw. This converts the factory's structural material
    // dominance over random play into the result instead of a shuffle-draw.
    adjudicate_material(&board, factory_white)
}

/// White-relative centipawn material balance of `board`.
fn material_balance(board: &Board) -> i32 {
    let val = |p: Piece| -> i32 {
        match p {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 0,
        }
    };
    let white = board.color_combined(Color::White);
    let black = board.color_combined(Color::Black);
    let mut bal = 0i32;
    for p in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
        let bb = board.pieces(p);
        bal += val(p) * ((bb & white).popcnt() as i32);
        bal -= val(p) * ((bb & black).popcnt() as i32);
    }
    bal
}

/// Adjudicate a ply-capped game by material lead (>= 300cp == win).
fn adjudicate_material(board: &Board, factory_white: bool) -> Outcome {
    let white_rel = material_balance(board);
    let factory_rel = if factory_white { white_rel } else { -white_rel };
    if factory_rel >= 300 {
        Outcome::FactoryWin
    } else if factory_rel <= -300 {
        Outcome::OpponentWin
    } else {
        Outcome::Draw
    }
}

/// Standard Elo performance-rating formula from a fractional score (lifted).
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

// ---------------------------------------------------------------------------
// Benchmark matrix (manufactured) + sweep.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Cell {
    opponent_id: u64,
    opponent: String,
    engine_kind: String,
    engine_path: String,
    go_command: String,
    tier: String,
    installed: bool,
    budget_id: u64,
    budget: String,
    budget_micros: u64,
}

/// Parse `schema/benchmark_matrix.json` into the sweep cells.
fn load_matrix(crate_dir: &PathBuf) -> (String, Vec<Cell>) {
    let path = crate_dir.join("schema/benchmark_matrix.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let v: serde_json::Value = serde_json::from_str(&raw).expect("matrix not valid JSON");
    let cells = v["cells"]
        .as_array()
        .expect("matrix.cells must be an array")
        .iter()
        .map(|c| Cell {
            opponent_id: c["opponent_id"].as_u64().unwrap_or(0),
            opponent: c["opponent"].as_str().unwrap().to_string(),
            engine_kind: c["engine_kind"].as_str().unwrap().to_string(),
            engine_path: c["engine_path"].as_str().unwrap().to_string(),
            go_command: c["go_command"].as_str().unwrap().to_string(),
            tier: c["tier"].as_str().unwrap_or("reference").to_string(),
            installed: c["installed"].as_bool().unwrap_or(true),
            budget_id: c["budget_id"].as_u64().unwrap_or(0),
            budget: c["budget"].as_str().unwrap().to_string(),
            budget_micros: c["budget_micros"].as_u64().unwrap(),
        })
        .collect();
    (raw, cells)
}

/// Resolve an opponent's spawnable path. Internal bins live next to this exe.
fn resolve_engine_path(cell: &Cell) -> String {
    if cell.engine_kind == "uci" {
        let p = if cell.opponent == "stockfish" {
            std::env::var("STOCKFISH_ENGINE").unwrap_or_else(|_| STOCKFISH.to_string())
        } else {
            cell.engine_path.clone()
        };
        return p;
    }
    // internal: the sanity bins are siblings of this executable.
    let mut dir = std::env::current_exe().expect("current_exe");
    dir.pop();
    dir.push(&cell.engine_path);
    dir.to_string_lossy().into_owned()
}

struct CellResult {
    opponent: String,
    budget: String,
    budget_micros: u64,
    wins: usize,
    draws: usize,
    losses: usize,
    total: usize,
    score_rate: f64,
    ci_lo: f64,
    ci_hi: f64,
    perf_elo: f64,
    go_command: String,
    factory_us_median: u64,
    factory_us_p99: u64,
    opp_us_median: u64,
    opp_us_p99: u64,
    moves_sampled: usize,
}

fn run_cell(cell: &Cell, n_lines: usize, factory_depth: usize) -> CellResult {
    let path = resolve_engine_path(cell);
    let mut opp = Engine::spawn(&path);
    let (mut wins, mut draws, mut losses) = (0usize, 0usize, 0usize);
    let mut factory_us: Vec<u64> = Vec::new();
    let mut opp_us: Vec<u64> = Vec::new();
    let lines = OPENINGS.iter().take(n_lines.clamp(1, OPENINGS.len()));
    for line in lines {
        for &factory_white in &[true, false] {
            match play_game(
                &mut opp,
                factory_white,
                line,
                &cell.go_command,
                factory_depth,
                &mut factory_us,
                &mut opp_us,
            ) {
                Outcome::FactoryWin => wins += 1,
                Outcome::OpponentWin => losses += 1,
                Outcome::Draw => draws += 1,
            }
        }
    }
    let (factory_us_median, factory_us_p99) = lat_stats(&factory_us);
    let (opp_us_median, opp_us_p99) = lat_stats(&opp_us);
    let total = wins + draws + losses;
    let score = wins as f64 + 0.5 * draws as f64;
    let p = score / total as f64;
    let se = (p * (1.0 - p) / total as f64).sqrt();
    let perf = performance_elo(1500.0, score, total);
    CellResult {
        opponent: cell.opponent.clone(),
        budget: cell.budget.clone(),
        budget_micros: cell.budget_micros,
        wins,
        draws,
        losses,
        total,
        score_rate: p,
        ci_lo: (p - 1.96 * se).max(0.0),
        ci_hi: (p + 1.96 * se).min(1.0),
        perf_elo: perf,
        go_command: cell.go_command.clone(),
        factory_us_median,
        factory_us_p99,
        opp_us_median,
        opp_us_p99,
        moves_sampled: opp_us.len(),
    }
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut h = Hasher::new();
    h.update(bytes);
    h.finalize().to_hex().to_string()
}

/// BLAKE3 of an engine binary on disk, or `None` if the path is absent. Used to
/// pin the EXACT bytes of every engine that participated in the sweep.
fn file_blake3_hex(path: &str) -> Option<String> {
    std::fs::read(path).ok().map(|b| blake3_hex(&b))
}

/// Per-cell receipt: blake3 over the cell's identity + result + measured
/// latency, chained onto `prev_hash`. `cell_blake3` is the chain link; distinct
/// cells produce distinct hashes (different opponent/budget/result/latency).
fn cell_receipt(r: &CellResult, prev_hash: &str) -> serde_json::Value {
    // Byte-stable preimage: prev_hash || opponent || budget || go_command ||
    // wins/draws/losses || measured think_micros (both sides).
    let mut h = Hasher::new();
    h.update(prev_hash.as_bytes());
    h.update(r.opponent.as_bytes());
    h.update(r.budget.as_bytes());
    h.update(r.go_command.as_bytes());
    h.update(&(r.wins as u64).to_le_bytes());
    h.update(&(r.draws as u64).to_le_bytes());
    h.update(&(r.losses as u64).to_le_bytes());
    h.update(&r.factory_us_median.to_le_bytes());
    h.update(&r.factory_us_p99.to_le_bytes());
    h.update(&r.opp_us_median.to_le_bytes());
    h.update(&r.opp_us_p99.to_le_bytes());
    let cell_blake3 = h.finalize().to_hex().to_string();
    serde_json::json!({
        "opponent": r.opponent,
        "budget": r.budget,
        "budget_micros": r.budget_micros,
        "go_command": r.go_command,
        "wins": r.wins,
        "draws": r.draws,
        "losses": r.losses,
        "score_rate": r.score_rate,
        "factory_us_median": r.factory_us_median,
        "factory_us_p99": r.factory_us_p99,
        "opponent_us_median": r.opp_us_median,
        "opponent_us_p99": r.opp_us_p99,
        "prev_hash": prev_hash,
        "cell_blake3": cell_blake3,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opp_filter: Option<String> = None;
    let mut budget_filter: Option<String> = None;
    let mut n_lines: usize = 8;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--opponent" => {
                opp_filter = args.get(i + 1).cloned();
                i += 2;
            }
            "--budget" => {
                budget_filter = args.get(i + 1).cloned();
                i += 2;
            }
            "--games" => {
                n_lines = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(8);
                i += 2;
            }
            _ => i += 1,
        }
    }

    // crate dir = where Cargo runs us (CARGO_MANIFEST_DIR at compile time).
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (matrix_raw, cells) = load_matrix(&crate_dir);

    let selected: Vec<Cell> = cells
        .into_iter()
        .filter(|c| opp_filter.as_ref().is_none_or(|f| &c.opponent == f))
        .filter(|c| budget_filter.as_ref().is_none_or(|f| &c.budget == f))
        .collect();

    println!("=== bcinr_benchmark — Elo(tau) sweep ===");
    println!(
        "cells: {}  | lines/cell: {} (x2 colors)\n",
        selected.len(),
        n_lines
    );

    let mut results: Vec<CellResult> = Vec::new();
    for cell in &selected {
        if !cell.installed {
            println!(
                "  {:<14} {:<7} ({:>5}us): not-installed — skipped (cf:installed false), never faked",
                cell.opponent, cell.budget, cell.budget_micros,
            );
            continue;
        }
        let r = run_cell(cell, n_lines, FACTORY_DEPTH);
        println!(
            "  {:<14} {:<7} ({:>5}us) [{:>13}]: +{} ={} -{}  score {:.1}%  (CI {:.1}-{:.1}%)  perf~{:.0}",
            r.opponent,
            r.budget,
            r.budget_micros,
            r.go_command,
            r.wins,
            r.draws,
            r.losses,
            r.score_rate * 100.0,
            r.ci_lo * 100.0,
            r.ci_hi * 100.0,
            r.perf_elo,
        );
        println!(
            "                  measured: factory {}us med / {}us p99  |  opponent {}us med / {}us p99  ({} opp moves)",
            r.factory_us_median, r.factory_us_p99, r.opp_us_median, r.opp_us_p99, r.moves_sampled,
        );
        results.push(r);
    }

    // --- HONESTY: equal-footing factory-vs-bcinr_uci head-to-head. ---
    // The Crown matrix runs the factory at depth-4 vs bcinr_uci at go-depth-1:
    // that is an UNEQUAL footing (the factory simply searches deeper), so its
    // headline score against bcinr_uci is INFLATED. We also play an EQUAL-footing
    // match: factory at depth-1 vs bcinr_uci at `go depth 1`, and report BOTH so
    // the parity claim is fair. Only meaningful when bcinr_uci is in the sweep.
    let parity: Option<serde_json::Value> = if opp_filter
        .as_ref()
        .is_none_or(|f| f == "bcinr_uci")
    {
        let uci_path = crate_dir
            .join("../../target/release/bcinr_uci")
            .to_string_lossy()
            .into_owned();
        if std::path::Path::new(&uci_path).exists() {
            // Equal footing: both engines search exactly 1 ply.
            let eq_cell = Cell {
                opponent_id: 4,
                opponent: "bcinr_uci".to_string(),
                engine_kind: "uci".to_string(),
                engine_path: uci_path,
                go_command: "go depth 1".to_string(),
                tier: "reference".to_string(),
                installed: true,
                budget_id: 10,
                budget: "equal_d1".to_string(),
                budget_micros: 0,
            };
            let eq = run_cell(&eq_cell, n_lines, 1);
            // The inflated (unequal) number already lives in `results`: factory
            // depth-4 vs bcinr_uci go-depth-1/movetime, take the t100us cell.
            let unequal = results
                .iter()
                .find(|r| r.opponent == "bcinr_uci");
            let unequal_score = unequal.map(|r| r.score_rate).unwrap_or(f64::NAN);
            let unequal_budget = unequal.map(|r| r.budget.clone()).unwrap_or_default();
            let unequal_go = unequal.map(|r| r.go_command.clone()).unwrap_or_default();
            println!("\n=== PARITY (factory vs bcinr_uci) — honesty check ===");
            println!(
                "  UNEQUAL  (factory depth-{} vs bcinr_uci '{}', tier {}): score {:.1}%  <-- INFLATED, deeper search",
                FACTORY_DEPTH, unequal_go, unequal_budget, unequal_score * 100.0,
            );
            println!(
                "  EQUAL    (factory depth-1 vs bcinr_uci 'go depth 1', 1-ply both): +{} ={} -{}  score {:.1}%  (CI {:.1}-{:.1}%)  perf~{:.0}  <-- FAIR",
                eq.wins, eq.draws, eq.losses, eq.score_rate * 100.0,
                eq.ci_lo * 100.0, eq.ci_hi * 100.0, eq.perf_elo,
            );
            Some(serde_json::json!({
                "doc": "Equal-footing parity check. The Crown matrix score vs bcinr_uci is at an UNEQUAL footing (factory searches depth-4 vs bcinr_uci go-depth-1) and is INFLATED. The EQUAL match plays both engines at 1 ply.",
                "unequal": {
                    "factory_depth": FACTORY_DEPTH,
                    "opponent_go_command": unequal_go,
                    "opponent_budget": unequal_budget,
                    "score_rate": unequal_score,
                    "label": "INFLATED — factory searches deeper",
                },
                "equal": {
                    "factory_depth": 1,
                    "opponent_go_command": "go depth 1",
                    "wins": eq.wins, "draws": eq.draws, "losses": eq.losses,
                    "score_rate": eq.score_rate,
                    "ci_lo": eq.ci_lo, "ci_hi": eq.ci_hi,
                    "perf_elo": eq.perf_elo,
                    "factory_us_median": eq.factory_us_median,
                    "opponent_us_median": eq.opp_us_median,
                    "label": "FAIR — both engines 1 ply",
                },
            }))
        } else {
            None
        }
    } else {
        None
    };

    // --- Emit artifacts. ---
    let artifacts = crate_dir.join("artifacts");
    std::fs::create_dir_all(&artifacts).expect("mkdir artifacts");

    // CSV.
    let mut csv = String::from(
        "opponent,budget,budget_micros,go_command,wins,draws,losses,total,score_rate,ci_lo,ci_hi,perf_elo,factory_us_median,factory_us_p99,opp_us_median,opp_us_p99,moves_sampled\n",
    );
    for r in &results {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.1},{},{},{},{},{}\n",
            r.opponent,
            r.budget,
            r.budget_micros,
            r.go_command,
            r.wins,
            r.draws,
            r.losses,
            r.total,
            r.score_rate,
            r.ci_lo,
            r.ci_hi,
            r.perf_elo,
            r.factory_us_median,
            r.factory_us_p99,
            r.opp_us_median,
            r.opp_us_p99,
            r.moves_sampled,
        ));
    }
    let csv_path = artifacts.join("elo_curve.csv");
    std::fs::write(&csv_path, &csv).expect("write csv");

    // JSON: per-cell points + per-opponent elo_at_100us.
    let points: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "opponent": r.opponent,
                "budget": r.budget,
                "budget_micros": r.budget_micros,
                "wins": r.wins,
                "draws": r.draws,
                "losses": r.losses,
                "total": r.total,
                "score_rate": r.score_rate,
                "ci_lo": r.ci_lo,
                "ci_hi": r.ci_hi,
                "perf_elo": r.perf_elo,
                "go_command": r.go_command,
                "factory_us_median": r.factory_us_median,
                "factory_us_p99": r.factory_us_p99,
                "opponent_us_median": r.opp_us_median,
                "opponent_us_p99": r.opp_us_p99,
                "moves_sampled": r.moves_sampled,
            })
        })
        .collect();

    // elo_at_100us per opponent (the headline tau point), if measured.
    let mut opponents: Vec<String> = results.iter().map(|r| r.opponent.clone()).collect();
    opponents.sort();
    opponents.dedup();
    let elo_at_100us: Vec<serde_json::Value> = opponents
        .iter()
        .map(|o| {
            let pt = results
                .iter()
                .find(|r| &r.opponent == o && r.budget_micros == 100);
            serde_json::json!({
                "opponent": o,
                "elo_at_100us": pt.map(|r| r.perf_elo),
                "score_rate_at_100us": pt.map(|r| r.score_rate),
                "measured": pt.is_some(),
            })
        })
        .collect();

    let curve = serde_json::json!({
        "schema": "bcinr.chess-factory.elo_curve.v1",
        "lines_per_cell": n_lines,
        "points": points,
        "elo_at_100us": elo_at_100us,
    });
    let curve_str = serde_json::to_string_pretty(&curve).unwrap();
    let json_path = artifacts.join("elo_curve.json");
    std::fs::write(&json_path, &curve_str).expect("write json");

    // Per-cell receipt array, hash-chained from genesis. Each cell's
    // `cell_blake3` is distinct (binds opponent+budget+go+result+latency) and
    // links to the previous cell via `prev_hash`.
    let genesis = "0".repeat(64);
    let mut prev = genesis.clone();
    let mut cell_receipts: Vec<serde_json::Value> = Vec::new();
    for r in &results {
        let rc = cell_receipt(r, &prev);
        prev = rc["cell_blake3"].as_str().unwrap().to_string();
        cell_receipts.push(rc);
    }
    let cells_chain_head = prev; // head of the per-cell chain.

    // Provenance: BLAKE3 of every engine binary that could participate, the
    // opening-book hash (over the OPENINGS array), and the captured seed.
    let factory_bin = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sanity_seed = std::env::var("SANITY_SEED")
        .ok()
        .and_then(|s| {
            s.strip_prefix("0x")
                .and_then(|h| u64::from_str_radix(h, 16).ok())
                .or_else(|| s.parse::<u64>().ok())
        })
        .unwrap_or(SANITY_SEED_DEFAULT);
    let engine_binaries = serde_json::json!({
        "stockfish": file_blake3_hex(
            &std::env::var("STOCKFISH_ENGINE").unwrap_or_else(|_| STOCKFISH.to_string())
        ),
        "bcinr_uci": file_blake3_hex(
            &crate_dir.join("../../target/release/bcinr_uci").to_string_lossy()
        ),
        "bcinr_az": file_blake3_hex(
            &crate_dir.join("../../target/release/bcinr_az").to_string_lossy()
        ),
        "bcinr_benchmark_factory": file_blake3_hex(&factory_bin),
    });
    // Opening-book hash: byte-stable fold of the OPENINGS array.
    let book_preimage: String = OPENINGS
        .iter()
        .map(|line| line.join(" "))
        .collect::<Vec<_>>()
        .join("|");
    let opening_book_blake3 = blake3_hex(book_preimage.as_bytes());
    let run_id = blake3_hex(
        format!(
            "{}|{}|{}|{}",
            blake3_hex(matrix_raw.as_bytes()),
            blake3_hex(curve_str.as_bytes()),
            cells_chain_head,
            sanity_seed,
        )
        .as_bytes(),
    );

    // Run receipt: self-certifying input/output hashes + seed + replay pointer +
    // per-cell chain + engine-binary/book provenance.
    let receipt = serde_json::json!({
        "schema": "bcinr.chess-factory.benchmark.receipt.v2",
        "run_id": run_id,
        "input_matrix_blake3": blake3_hex(matrix_raw.as_bytes()),
        "output_curve_blake3": blake3_hex(curve_str.as_bytes()),
        "output_csv_blake3": blake3_hex(csv.as_bytes()),
        "sanity_seed": sanity_seed,
        "sanity_seed_hex": format!("0x{sanity_seed:016X}"),
        "engine_binaries": engine_binaries,
        "opening_book_blake3": opening_book_blake3,
        "opening_book_lines": OPENINGS.len(),
        "lines_per_cell": n_lines,
        "cells_swept": selected.len(),
        "cell_receipts": cell_receipts,
        "cell_chain_head": cells_chain_head,
        "parity_check": parity,
        "replay_pointer": "schema/benchmark_matrix.json",
    });
    let receipt_path = artifacts.join("benchmark.receipt.json");
    std::fs::write(
        &receipt_path,
        serde_json::to_string_pretty(&receipt).unwrap(),
    )
    .expect("write receipt");

    // --- Emit the RESULTS graph (artifacts/results.ttl) for ggen. ---
    // Replaces the deleted tools/emit_results_ttl.py: same self-contained graph,
    // now manufactured in-process (no Python in a Rust pipeline). Full opponent +
    // budget roster from the matrix; one cf:ResultCell per measured cell, each
    // citing its chained blake3 receipt hash. Not-installed opponents and
    // unmeasured cells are NEVER faked — they simply have no cf:ResultCell.
    let (_all_raw, all_cells) = load_matrix(&crate_dir);
    let prov = RunProvenance {
        run_id: &run_id,
        sanity_seed,
        chain_head: &cells_chain_head,
        opening_book_blake3: &opening_book_blake3,
        matrix_raw: &matrix_raw,
    };
    let ttl = emit_results_ttl(&all_cells, &results, &cell_receipts, &prov);
    let ttl_path = artifacts.join("results.ttl");
    std::fs::write(&ttl_path, &ttl).expect("write results.ttl");

    println!("\nArtifacts written:");
    println!("  {}", csv_path.display());
    println!("  {}", json_path.display());
    println!("  {}", receipt_path.display());
    println!("  {}", ttl_path.display());
}

/// Run-level provenance bundled for `emit_results_ttl` (keeps its signature small
/// and self-documenting instead of a long positional argument list).
struct RunProvenance<'a> {
    run_id: &'a str,
    sanity_seed: u64,
    chain_head: &'a str,
    opening_book_blake3: &'a str,
    matrix_raw: &'a str,
}

/// Emit the self-contained RESULTS graph consumed by `ggen sync` (proof_table
/// rule). Carries the full roster (opponents + budgets) plus one cf:ResultCell
/// per measured cell, each citing its chained blake3 receipt hash + run_id.
fn emit_results_ttl(
    all_cells: &[Cell],
    results: &[CellResult],
    cell_receipts: &[serde_json::Value],
    prov: &RunProvenance,
) -> String {
    let RunProvenance {
        run_id,
        sanity_seed,
        chain_head,
        opening_book_blake3,
        matrix_raw,
    } = prov;
    use std::collections::BTreeMap;
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");

    // Unique opponents (id -> (name, tier, installed)) and budgets (id -> (name, micros)).
    let mut opps: BTreeMap<u64, (String, String, bool)> = BTreeMap::new();
    let mut buds: BTreeMap<u64, (String, u64)> = BTreeMap::new();
    for c in all_cells {
        opps.entry(c.opponent_id)
            .or_insert_with(|| (c.opponent.clone(), c.tier.clone(), c.installed));
        buds.entry(c.budget_id)
            .or_insert_with(|| (c.budget.clone(), c.budget_micros));
    }

    // Index results + receipts by (opponent, budget) for rich stats / receipt hash.
    let res_idx: BTreeMap<(String, String), &CellResult> = results
        .iter()
        .map(|r| ((r.opponent.clone(), r.budget.clone()), r))
        .collect();
    let rec_idx: BTreeMap<(String, String), &serde_json::Value> = cell_receipts
        .iter()
        .map(|r| {
            (
                (
                    r["opponent"].as_str().unwrap_or("").to_string(),
                    r["budget"].as_str().unwrap_or("").to_string(),
                ),
                r,
            )
        })
        .collect();

    let mut s = String::new();
    s.push_str("@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    s.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
    s.push_str("@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .\n");
    s.push_str("@prefix cf:   <https://bcinr.dev/chess-factory/ontology#> .\n\n");
    s.push_str("#################################################################\n");
    s.push_str("# bcinr Chess Factory — RESULTS GRAPH (receipt-cited Crown proof table).\n");
    s.push_str("#\n");
    s.push_str("# DO NOT hand-edit: emitted in-process by `bcinr_benchmark` (emit_results_ttl)\n");
    s.push_str("# from the empirical benchmark artifacts. `ggen sync` imports this graph and\n");
    s.push_str("# renders artifacts/proof_table.{md,json}. Every cf:ResultCell carries the\n");
    s.push_str("# blake3 receipt hash that proves it. Not-installed engines and unmeasured\n");
    s.push_str("# cells are NEVER faked — they carry no cf:ResultCell.\n");
    s.push_str("#################################################################\n\n");

    s.push_str("cf:Crown_Run a cf:ResultRun ;\n");
    s.push_str(&format!("    cf:runId           \"{}\" ;\n", esc(run_id)));
    s.push_str("    cf:replayPointer   \"schema/benchmark_matrix.json\" ;\n");
    s.push_str(&format!("    cf:cellChainHead   \"{}\" ;\n", esc(chain_head)));
    s.push_str(&format!("    cf:inputMatrixHash \"{}\" ;\n", esc(&blake3_hex(matrix_raw.as_bytes()))));
    s.push_str(&format!("    cf:openingBookHash \"{}\" ;\n", esc(opening_book_blake3)));
    s.push_str(&format!("    cf:sanitySeedHex   \"0x{sanity_seed:016X}\" ;\n"));
    s.push_str("    rdfs:comment \"Top of the receipt chain for the Crown proof table.\" .\n\n");

    s.push_str("# Opponents (roster axis).\n");
    for (id, (name, tier, installed)) in &opps {
        s.push_str(&format!("cf:R_opp_{name} a cf:ResultOpponent ;\n"));
        s.push_str(&format!("    cf:rOppId    {id} ;\n"));
        s.push_str(&format!("    cf:rOppName  \"{}\" ;\n", esc(name)));
        s.push_str(&format!("    cf:rTier     \"{}\" ;\n", esc(tier)));
        s.push_str(&format!("    cf:rInstalled {} .\n\n", if *installed { "true" } else { "false" }));
    }

    s.push_str("# Budget tiers (tau axis / columns).\n");
    for (id, (name, micros)) in &buds {
        s.push_str(&format!("cf:R_bud_{name} a cf:ResultBudget ;\n"));
        s.push_str(&format!("    cf:rBudId    {id} ;\n"));
        s.push_str(&format!("    cf:rBudName  \"{}\" ;\n", esc(name)));
        s.push_str(&format!("    cf:rBudMicros {micros} .\n\n"));
    }

    s.push_str("# Measured cells (one cf:ResultCell per cell the receipt proves).\n");
    for ((opp, bud), r) in &res_idx {
        let rec = match rec_idx.get(&(opp.clone(), bud.clone())) {
            Some(v) => v,
            None => continue,
        };
        let opp_id = opps
            .iter()
            .find(|(_, v)| &v.0 == opp)
            .map(|(k, _)| *k)
            .unwrap_or(0);
        let bud_id = buds
            .iter()
            .find(|(_, v)| &v.0 == bud)
            .map(|(k, _)| *k)
            .unwrap_or(0);
        let cid = 1000 + opp_id * 100 + bud_id;
        s.push_str(&format!("cf:R_cell_{opp}_{bud} a cf:ResultCell ;\n"));
        s.push_str(&format!("    cf:rCellId      {cid} ;\n"));
        s.push_str(&format!("    cf:rCellOpp     cf:R_opp_{opp} ;\n"));
        s.push_str(&format!("    cf:rCellBud     cf:R_bud_{bud} ;\n"));
        s.push_str(&format!("    cf:rCellOppName \"{}\" ;\n", esc(opp)));
        s.push_str(&format!("    cf:rCellBudName \"{}\" ;\n", esc(bud)));
        s.push_str(&format!("    cf:rScoreRate   {:.4} ;\n", r.score_rate));
        s.push_str(&format!("    cf:rCiLo        {:.4} ;\n", r.ci_lo));
        s.push_str(&format!("    cf:rCiHi        {:.4} ;\n", r.ci_hi));
        s.push_str(&format!("    cf:rPerfElo     {:.1} ;\n", r.perf_elo));
        // Pre-formatted display strings so the proof_table templates stay
        // arithmetic-free (ggen's Tera binds numeric SPARQL literals as strings,
        // so `* 100`/`round` would fail at render time — format here instead).
        s.push_str(&format!(
            "    cf:rScorePct    \"{:.1}\" ;\n",
            r.score_rate * 100.0
        ));
        s.push_str(&format!(
            "    cf:rCiText      \"{:.1}-{:.1}\" ;\n",
            r.ci_lo * 100.0,
            r.ci_hi * 100.0
        ));
        s.push_str(&format!(
            "    cf:rPerfEloR    \"{:.0}\" ;\n",
            r.perf_elo
        ));
        s.push_str(&format!("    cf:rWins        {} ;\n", r.wins));
        s.push_str(&format!("    cf:rDraws       {} ;\n", r.draws));
        s.push_str(&format!("    cf:rLosses      {} ;\n", r.losses));
        s.push_str(&format!("    cf:rFactoryUs   {} ;\n", r.factory_us_median));
        s.push_str(&format!("    cf:rFactoryUsP99 {} ;\n", r.factory_us_p99));
        s.push_str(&format!("    cf:rOppUs       {} ;\n", r.opp_us_median));
        s.push_str(&format!("    cf:rOppUsP99    {} ;\n", r.opp_us_p99));
        s.push_str(&format!("    cf:rGoCommand   \"{}\" ;\n", esc(&r.go_command)));
        s.push_str(&format!("    cf:rReceiptHash \"{}\" ;\n", esc(rec["cell_blake3"].as_str().unwrap_or(""))));
        s.push_str(&format!("    cf:rPrevHash    \"{}\" ;\n", esc(rec["prev_hash"].as_str().unwrap_or(""))));
        s.push_str(&format!("    cf:rRunId       \"{}\" ;\n", esc(run_id)));
        s.push_str("    cf:rReplayPtr   \"schema/benchmark_matrix.json\" .\n\n");
    }
    s
}
